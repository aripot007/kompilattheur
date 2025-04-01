use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{FileType, TargetTriple};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::StructType;
use inkwell::OptimizationLevel;
use tempfile::NamedTempFile;

use std::path::Path;

use crate::ast::nodes::Root;
use crate::common::diagnostic::Diagnostic;

use super::dynamic_linker::get_dynamic_linker;
use super::{llvm_from_root, InternalFuctions};
pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub target_machine: TargetMachine,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
    pub smolpp_types: CodeGenTypedefs<'ctx>,
}

pub struct CodeGenTypedefs<'ctx> {
    pub dynamic_type: StructType<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn create(context: &'ctx Context, target_triple: &TargetTriple) -> Result<Self, String> {
        let config = InitializationConfig {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        };
        Target::initialize_native(&config).map_err(|_| "Failed to initialize native target")?;

        let target = Target::from_triple(target_triple)
            .map_err(|e| format!("Failed to get target from triple: {}", e))?;

        let target_machine = target
            .create_target_machine(
                target_triple,
                &TargetMachine::get_host_cpu_name().to_string(),
                &TargetMachine::get_host_cpu_features().to_string(),
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;

        let module = context.create_module("sum_program");

        let mut codegen = CodeGen {
            context: context,
            module,
            builder: context.create_builder(),
            warnings: Vec::new(),
            errors: Vec::new(),
            target_machine,
            smolpp_types: CodeGenTypedefs {
                dynamic_type: context.opaque_struct_type("dynamic_type_struct"),
            },
        };

        codegen.init_smolpp_types();
        codegen.init_internal_functions();
        codegen.module.set_triple(&target_triple);

        return Ok(codegen);
    }

    fn init_internal_functions(&mut self) {
        //
        // syscalls
        //

        // puts function declaration
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let puts_type = i32_type.fn_type(&[ptr_type.into()], false);
        self.module
            .add_function(InternalFuctions::Puts.into(), puts_type, None);

        // Add main function entry point
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);
        // Create a basic block for the main function
    }

    fn init_smolpp_types(&mut self) {
        self.init_dynamic_type();
    }

    fn init_dynamic_type(&mut self) {
        let context = self.context;

        let var_type_discriminant = context.i8_type();

        let i64_type = context.i64_type();

        let ptr_size = self
            .target_machine
            .get_target_data()
            .get_pointer_byte_size(None)
            * 8;

        // Choose the largest type for the union
        let union_size = i64_type.get_bit_width().max(ptr_size);
        let var_value = context.custom_width_int_type(union_size);

        // Create the struct type
        self.smolpp_types.dynamic_type.set_body(
            &[
                var_type_discriminant.into(), // char type
                var_value.into(),             // Simulated union (either u64 or pointer)
            ],
            false,
        );
    }

    fn get_linker(&self) -> Result<String, String> {
        let dynamic_linker = get_dynamic_linker(&self.target_machine);
        println!("Dynamic linker: {}", dynamic_linker);

        Ok(dynamic_linker)
    }

    pub fn compile(
        &self,
        output_path: &Path,
        filetype: FileType,
        target_machine: &TargetMachine,
    ) -> Result<(), String> {
        self.module
            .set_data_layout(&target_machine.get_target_data().get_data_layout());

        target_machine
            .write_to_file(&self.module, filetype, output_path)
            .map_err(|e| format!("Failed to write object file: {}", e))
    }

    fn try_link_with_command(&self, cmd: &mut std::process::Command) -> Result<(), String> {
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).into_owned())
                }
            }
            Err(e) => Err(format!("Failed to execute linker: {}", e)),
        }
    }

    pub fn link_object_file(
        &self,
        obj_path: &Path,
        exe_path: &Path,
        target_triple: &TargetTriple,
        dynamic_linker: &str,
    ) -> Result<(), String> {
        // Try using ld.lld first with system-specific configuration
        let target_triple = target_triple.to_string();
        if target_triple.contains("apple") {
            // Get the current macOS version
            let macos_version = match std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).trim().to_string()
                    } else {
                        println!("Warning: Failed to get macOS version, using default");
                        "15.0".to_string()
                    }
                }
                Err(e) => {
                    println!(
                        "Warning: Failed to execute sw_vers: {}, using default version",
                        e
                    );
                    "15.0".to_string()
                }
            };
            let arch = target_triple.split('-').next().unwrap_or("arm64");
            println!("Using macOS version: {}", macos_version);

            let mut cmd = std::process::Command::new("ld64.lld");
            cmd.arg("-demangle")
                .arg("-dynamic")
                .arg("-arch")
                .arg(arch)
                .arg("-platform_version")
                .arg("macos")
                .arg(&macos_version)
                .arg(&macos_version)
                .arg("-syslibroot")
                .arg("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk")
                .arg("-o")
                .arg(exe_path)
                .arg("-L/usr/local/lib")
                .arg(obj_path)
                .arg("-lSystem");

            match self.try_link_with_command(&mut cmd) {
                Ok(_) => return Ok(()),
                Err(e) => println!("ld64.lld failed: {}", e),
            }
        } else {
            // Try Linux lld first
            let mut cmd = std::process::Command::new("ld.lld");
            cmd.arg("-o")
                .arg(exe_path)
                .arg(obj_path)
                .arg("/usr/lib/crt1.o")
                .arg("-lc")
                .arg("-L/usr/lib")
                .arg("-dynamic-linker")
                .arg(dynamic_linker);

            match self.try_link_with_command(&mut cmd) {
                Ok(_) => return Ok(()),
                Err(e) => println!("ld.lld failed: {}", e),
            }
        }

        // If lld failed, try clang
        println!("Falling back to clang for linking");
        let mut cmd = std::process::Command::new("clang");
        cmd.arg(obj_path).arg("-o").arg(exe_path);

        match self.try_link_with_command(&mut cmd) {
            Ok(_) => return Ok(()),
            Err(e) => println!("clang failed: {}", e),
        }

        // If clang failed, try gcc as last resort
        println!("Falling back to gcc for linking");
        let mut cmd = std::process::Command::new("gcc");
        cmd.arg(obj_path).arg("-o").arg(exe_path);

        match self.try_link_with_command(&mut cmd) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "All linking attempts failed. Last error (gcc): {}. \
                Tried: ld.lld/ld64.lld, clang, and gcc",
                e
            )),
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        // Verify the module
        if self.module.verify().is_err() {
            eprintln!("Module verification failed!");
            eprintln!("{}", self.module.print_to_string().to_string());
            Err("Module verification failed".into())
        } else {
            println!("Module verification succeeded!");
            Ok(())
        }
    }

    pub fn generate_executable(&self, output_path: &Path, target_triple: &TargetTriple) -> Result<(), String> {
        // Generate the executable
        let dynamic_linker = self.get_linker()?;

        let temp_file = NamedTempFile::new().map_err(|e| format!("Error opening temp file for linking : {}", e))?;
        
        // Compile the object file
        self.compile(temp_file.path(), FileType::Object, &self.target_machine)?;

        // Link the executable
        self.link_object_file(
            temp_file.path(),
            output_path,
            target_triple,
            &dynamic_linker,
        )?;

        temp_file.close().map_err(|e| format!("Error closing temp file : {}", e))?;

        Ok(())
    }

    pub fn generate_llvm(&mut self, root: &Root) {
        let res = llvm_from_root(root, self);

        for w in &self.warnings {
            w.display();
        }

        for e in &self.errors {
            e.display();
        }

        if self.errors.len() > 0 || res.is_err() {
            // Todo : add error handling
            eprintln!("Error during LLVM generation");
        }
    }
}
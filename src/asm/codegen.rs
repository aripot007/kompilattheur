use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::FileType;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::StructType;
use inkwell::values::FunctionValue;
use inkwell::OptimizationLevel;

use std::path::Path;

use crate::ast::nodes::Root;
use crate::common::diagnostic::Diagnostic;

use super::dynamic_linker::get_dynamic_linker;
use super::{llvm_from_root, InternalFuctions};

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    target_machine: TargetMachine, 
    execution_engine: Option<ExecutionEngine<'ctx>>,
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
    pub smolpp_types: CodeGenTypedefs<'ctx>,
}

pub struct CodeGenTypedefs<'ctx> {
    pub dynamic_type: StructType<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn create(jit: bool, context: &'ctx Context) -> Result<Self, String> {

        let config = InitializationConfig {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        };
        Target::initialize_native(&config).map_err(|_| "Failed to initialize native target")?;

        let target_triple = TargetMachine::get_default_triple();
        println!("Target triple: {}", target_triple.to_string());

        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to get target from triple: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                &TargetMachine::get_host_cpu_name().to_string(),
                &TargetMachine::get_host_cpu_features().to_string(),
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;

        let module = context.create_module("sum_program");
        let execution_engine = if jit {
            Some(
                module
                    .create_jit_execution_engine(OptimizationLevel::None)
                    .map_err(|e| format!("Failed to create JIT execution engine: {}", e))?,
            )
        } else {
            None
        };
        
        
        let mut codegen = CodeGen {
            context: context,
            module,
            builder: context.create_builder(),
            execution_engine,
            warnings: Vec::new(),
            errors: Vec::new(),
            target_machine,
            smolpp_types:  CodeGenTypedefs {
                dynamic_type: context.opaque_struct_type("dynamic_type_struct"),
            }
        };

        codegen.init_smolpp_types();
        codegen.init_internal_functions();

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
        self.module.add_function(InternalFuctions::Puts.into(), puts_type, None);

    }

    fn init_smolpp_types(&mut self) {
        self.init_dynamic_type();
    }

    fn init_dynamic_type(&mut self) {

        let context = self.context;
    
        let var_type_discriminant = context.i8_type();
    
        let i64_type = context.i64_type();

        let ptr_size = self.target_machine.get_target_data().get_pointer_byte_size(None) * 8;
    
        // Choose the largest type for the union
        let union_size = i64_type.get_bit_width().max(ptr_size);
        let var_value = context.custom_width_int_type(union_size);
    
        
        // Create the struct type
        self.smolpp_types.dynamic_type.set_body(
            &[
                var_type_discriminant.into(),      // char type
                var_value.into(),   // Simulated union (either u64 or pointer)
            ],
            false,
        );

    }

    fn create_sum_function(&self) -> FunctionValue<'ctx> {
        // ...existing function implementation...
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        // Unwrap parameters safely
        if let (Some(param0), Some(param1), Some(param2)) = (
            function.get_nth_param(0),
            function.get_nth_param(1),
            function.get_nth_param(2),
        ) {
            let x = param0.into_int_value();
            let y = param1.into_int_value();
            let z = param2.into_int_value();

            let sum = self.builder.build_int_add(x, y, "sum").unwrap();
            let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

            self.builder.build_return(Some(&sum)).unwrap();
        }

        function
    }

    fn create_main_function(&self, sum_function: FunctionValue<'ctx>) {
        // ...existing function implementation...
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        // Create printf function declaration - use context.ptr_type instead of i8_type().ptr_type
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create format string
        let format_string = self
            .builder
            .build_global_string_ptr("%lld + %lld + %lld = %lld (%p : %s)\n", "format_str")
            .unwrap();

        // Create format string
        let thestring = self
            .builder
            .build_global_string_ptr("HEUEHURIHEIZRHOOUIZEHORUIHE", "format_str")
            .unwrap();


        // Create constant values for x, y, z
        let x = i64_type.const_int(1, false);
        let y = i64_type.const_int(2, false);
        let z = i64_type.const_int(3, false);

        // Call sum function
        let args = &[x.into(), y.into(), z.into()];
        let result = self
            .builder
            .build_call(sum_function, args, "sum_result")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();

        // Call printf
        let printf_args = &[
            format_string.as_pointer_value().into(),
            x.into(),
            y.into(),
            z.into(),
            result.into(),
            thestring.as_pointer_value().into(),
            thestring.as_pointer_value().into(),
        ];
        self.builder
            .build_call(printf, printf_args, "printf_call")
            .unwrap();

        // Return 0
        let ret_val = i32_type.const_int(0, false);
        self.builder.build_return(Some(&ret_val)).unwrap();
    }

    fn get_linker(&self) -> Result<String, String> {
        
        let dynamic_linker = get_dynamic_linker(&self.target_machine);
        println!("Dynamic linker: {}", dynamic_linker);
        
        Ok(dynamic_linker)
    }

    fn compile(&self, output_path: &Path, filetype: FileType, target_machine: &TargetMachine) -> Result<(), String> {
        let target_triple = TargetMachine::get_default_triple();
        
        self.module
            .set_data_layout(&target_machine.get_target_data().get_data_layout());
        self.module.set_triple(&target_triple);

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
            Err(e) => Err(format!("Failed to execute linker: {}", e))
        }
    }

    fn link_object_file(&self, obj_path: &Path, exe_path: &Path, target_triple: &str, dynamic_linker: &str) -> Result<(), String> {
        // Try using ld.lld first with system-specific configuration
        if target_triple.contains("apple") {
            // Get the current macOS version
            let macos_version = match std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output() {
                Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                } else {
                    println!("Warning: Failed to get macOS version, using default");
                    "15.0".to_string()
                }
                },
                Err(e) => {
                println!("Warning: Failed to execute sw_vers: {}, using default version", e);
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
                Err(e) => println!("ld64.lld failed: {}", e)
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
                Err(e) => println!("ld.lld failed: {}", e)
            }
        }

        // If lld failed, try clang
        println!("Falling back to clang for linking");
        let mut cmd = std::process::Command::new("clang");
        cmd.arg(obj_path)
            .arg("-o")
            .arg(exe_path);

        match self.try_link_with_command(&mut cmd) {
            Ok(_) => return Ok(()),
            Err(e) => println!("clang failed: {}", e)
        }

        // If clang failed, try gcc as last resort
        println!("Falling back to gcc for linking");
        let mut cmd = std::process::Command::new("gcc");
        cmd.arg(obj_path)
            .arg("-o")
            .arg(exe_path);

        match self.try_link_with_command(&mut cmd) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "All linking attempts failed. Last error (gcc): {}. \
                Tried: ld.lld/ld64.lld, clang, and gcc",
                e
            ))
        }
    }
}

type MainFunction = unsafe extern "C" fn();



pub fn example_llvm(jit: bool, ast: Option<&Root>) -> Result<(), String> {
    
    let context = Context::create();
    let mut codegen = CodeGen::create(jit, &context).unwrap();

    if let Some(root) = ast {
        let res = llvm_from_root(root, &mut codegen);

        for w in &codegen.warnings {
            w.display();
        }

        for e in &codegen.errors {
            e.display();
        }

        if codegen.errors.len() > 0 || res.is_err() {
            return Err("Could not generate LLVM from ast".into());
        }

    } else {
        // Create the sum function and main function
        let sum_function = codegen.create_sum_function();
        codegen.create_main_function(sum_function);
    }

    // Verify the module
    if codegen.module.verify().is_err() {
        eprintln!("Module verification failed!");
        eprintln!("{}", codegen.module.print_to_string().to_string());
        return Err("Module verification failed".into());
    } else {
        println!("Module verification succeeded!");
    }

    // Output paths
    let output_dir = Path::new("output");
    std::fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

    if let Some(ref engine) = codegen.execution_engine {
        unsafe {
            let main: JitFunction<MainFunction> = engine
                .get_function("main")
                .map_err(|e| format!("Failed to get JIT function: {}", e))?;

            main.call();
            libc::fflush(std::ptr::null_mut());
            Ok(())
        }
    } else {
        // Generate object file
        let target_machine = &codegen.target_machine;
        let dynamic_linker = codegen.get_linker()?;

        let obj_path = output_dir.join("sum_program.o");
        codegen.compile(&obj_path, FileType::Object, &target_machine)?;
        println!("Object file created at {}", obj_path.display());

        // Also generate assembly for reference
        let asm_path = output_dir.join("sum_program.s");
        codegen.compile(&asm_path, FileType::Assembly, &target_machine)?;
        println!("Assembly file created at {}", asm_path.display());

        // Link the executable using the new fallback system
        let exe_path = output_dir.join("sum_program");
        let target_triple = target_machine.get_triple().to_string();
        
        match codegen.link_object_file(&obj_path, &exe_path, &target_triple, &dynamic_linker) {
            Ok(_) => {
                println!("Executable created at {}", exe_path.display());
                
                // Run the executable to demonstrate it works
                println!("\nRunning the executable:");
                let output = std::process::Command::new(&exe_path).output().map_err(|e| e.to_string())?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            Err(e) => {
                println!("Failed to link the executable: {}", e);
            }
        }

        Ok(())
    }
}

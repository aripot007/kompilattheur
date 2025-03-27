use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::FileType;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::values::FunctionValue;
use inkwell::OptimizationLevel;

use std::error::Error;
use std::path::Path;

use crate::asm::dynamic_linker;

use super::dynamic_linker::get_dynamic_linker;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: Option<ExecutionEngine<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
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
            .build_global_string_ptr("%lld + %lld + %lld = %lld\n\0", "format_str")
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
        ];
        self.builder
            .build_call(printf, printf_args, "printf_call")
            .unwrap();

        // Return 0
        let ret_val = i32_type.const_int(0, false);
        self.builder.build_return(Some(&ret_val)).unwrap();
    }

    fn get_target_machine_and_linker(&self) -> Result<(TargetMachine, String), String> {
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

        let dynamic_linker = get_dynamic_linker(&target_machine);
        println!("Dynamic linker: {}", dynamic_linker);
        
        Ok((target_machine, dynamic_linker))
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
}

type MainFunction = unsafe extern "C" fn();

pub fn example_llvm(jit: bool) -> Result<(), Box<dyn Error>> {
    let context = Context::create();
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

    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    // Create the sum function and main function
    let sum_function = codegen.create_sum_function();
    codegen.create_main_function(sum_function);

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
    std::fs::create_dir_all(output_dir)?;

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
        let (target_machine, dynamic_linker) = codegen.get_target_machine_and_linker()?;

        let obj_path = output_dir.join("sum_program.o");
        codegen
            .compile(&obj_path, FileType::Object, &target_machine)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;
        println!("Object file created at {}", obj_path.display());

        // Also generate assembly for reference
        let asm_path = output_dir.join("sum_program.s");
        codegen
            .compile(&asm_path, FileType::Assembly, &target_machine)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;
        println!("Assembly file created at {}", asm_path.display());

        // Now use LLD to link the executable
        let exe_path = output_dir.join("sum_program");

        // Using lld directly
        let target_triple = target_machine.get_triple().to_string();
        
        // Check if the target is macOS
        let mut lld_cmd = if target_triple.contains("arm64-apple-darwin24.3.0") {
            let mut cmd = std::process::Command::new("/opt/homebrew/bin/ld64.lld");
            cmd
                .arg("-demangle")
                .arg("-dynamic")
                .arg("-arch")
                .arg("arm64")
                .arg("-platform_version")
                .arg("macos")
                .arg("15.0.0")
                .arg("15.2")
                .arg("-syslibroot")
                .arg("/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk")
                .arg("-mllvm")
                .arg("-enable-linkonceodr-outlining")
                .arg("-o")
                .arg(&exe_path)
                .arg("-L/usr/local/lib")
                .arg(obj_path.as_os_str())
                .arg("-lSystem")
                .arg("/Library/Developer/CommandLineTools/usr/lib/clang/16/lib/darwin/libclang_rt.osx.a");
            cmd
        } else {
            let mut cmd = std::process::Command::new("ld.lld");
            // Use /usr/lib to follow systemd standard https://www.freedesktop.org/software/systemd/man/latest/file-hierarchy.html
            cmd.arg("-o")
                .arg(&exe_path)
                .arg("/usr/lib/crt1.o") // C runtime startup file
                .arg(obj_path.as_os_str())
                .arg("-lc")// Link against the C standard library
                .arg("-L/usr/lib")
                .arg("-dynamic-linker")
                .arg(&dynamic_linker);
            cmd
        };
        // Work without this
        // .arg("/usr/lib/crti.o") // C runtime initialization file
        // .arg("/usr/lib/crtn.o") // C runtime termination file 
        // .arg("--hash-style=gnu")
        // .arg("-pie") // Position Independent Executable
        // .arg("/nix/store/7q6fxlj7kc8rfdrd6rgv98567nhqipcs-clang-wrapper-18.1.8/resource-root/lib/linux/clang_rt.crtbegin-x86_64.o")
        //.arg("-L/nix/store/maxa3xhmxggrc5v2vc0c3pjb79hjlkp9-glibc-2.40-66/lib") // Adjust this path as needed
        // Work without this
        // .arg("/nix/store/7q6fxlj7kc8rfdrd6rgv98567nhqipcs-clang-wrapper-18.1.8/resource-root/lib/linux/clang_rt.crtend-x86_64.o")
        // .arg("--init=_init")
        // .arg("--fini=_fini")
        ;

        // Check if LDFLAGS is set in the environment.
        if let Ok(ldflags) = std::env::var("LDFLAGS") {
            // Split LDFLAGS on whitespace and add each flag.
            for flag in ldflags.split_whitespace() {
                lld_cmd.arg(flag);
            }
        }

        lld_cmd.arg("-v"); // Verbose output for debugging

        let lld_output = lld_cmd.output()?;
        if !lld_output.status.success() {
            println!("Failed to link the executable");
            println!("{}", String::from_utf8_lossy(&lld_output.stderr));
        } else {
            println!("Executable created at {}", exe_path.display());

            // Run the executable to demonstrate it works
            println!("\nRunning the executable:");
            let output = std::process::Command::new(exe_path).output()?;

            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }
}

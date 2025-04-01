use std::path::Path;

use inkwell::execution_engine::{ExecutionEngine, JitFunction};

use super::InternalFuctions;

type MainFunction = unsafe extern "C" fn();

pub fn execute_executable(exe_path: &Path) -> Result<(), String> {
    // Execute the generated executable
    let output = std::process::Command::new(exe_path)
        .output()
        .map_err(|e| format!("Failed to execute the program: {}", e))?;

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        Err(format!(
            "Execution failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn execute(engine: ExecutionEngine) {
    unsafe {
        let main_function: JitFunction<MainFunction> = match engine.get_function(InternalFuctions::Main.into()) {
            Ok(func) => func,
            Err(e) => {
                eprintln!("Failed to get main function: {}", e);
                return;
            }
        };
        main_function.call();
        libc::fflush(std::ptr::null_mut());
    };
}

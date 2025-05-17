pub mod codegen;
mod dynamic_linker;
mod internal_functions;
mod llvm;
pub use internal_functions::*;
mod diagnostics;
pub mod execute;
mod internal_global_constants;
use internal_global_constants::*;

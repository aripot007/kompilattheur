pub mod codegen;
mod llvm;
mod dynamic_linker;
mod internal_functions;
pub use internal_functions::*;
mod diagnostics;
pub mod execute;
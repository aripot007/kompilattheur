mod ast;
pub(super) use ast::*;
mod print;
use inkwell::builder::BuilderError;
pub(super) use print::*;
mod panic;
use thiserror::Error;
mod assert_type;
pub use assert_type::*;
pub mod smolvar;

pub mod lists;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LLVMCodegenError {
    #[error("Unimplemented LLVM for {0}")]
    Unimplemented(String),
    #[error("Builder Error : {0:?}")]
    BuilderError(#[from] BuilderError),
    #[error("Invalid Destination expression : {0}")]
    InvalidDestination(String),
    #[error("Invalid Operation : {0}")]
    InvalidOperation(String),
}

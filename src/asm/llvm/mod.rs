mod ast;
pub(super) use ast::*;
mod print;
use inkwell::builder::BuilderError;
pub(super) use print::*;
mod panic;
use thiserror::Error;
pub mod smolvar;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LLVMCodegenError {
    #[error("Unimplemented LLVM for {0}")]
    Unimplemented(String),
    #[error("Builder Error : {0:?}")]
    BuilderError(#[from] BuilderError),
}

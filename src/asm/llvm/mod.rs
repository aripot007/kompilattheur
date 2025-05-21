mod ast;
pub(super) use ast::*;
mod util;
use inkwell::builder::BuilderError;
use thiserror::Error;
pub(crate) use util::*;
pub mod smolvar;
pub(crate) use ast::{user_function_prefix, user_function_prefix_format};

pub mod lists;
pub mod strings;

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

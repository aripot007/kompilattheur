mod ast;
pub(super) use ast::*;
mod print;
use inkwell::builder::BuilderError;
pub(super) use print::*;
mod list_cmp;
pub(super) use list_cmp::*;
pub mod panic;
use thiserror::Error;
mod assert_type;
pub use assert_type::*;
mod cast;
pub mod smolvar;
pub(crate) use ast::{user_function_prefix, user_function_prefix_format};
pub use cast::init_internal_bool_cast_function;

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

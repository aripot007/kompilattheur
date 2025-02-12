mod primitive_types;
mod function;
mod ntuple;

use std::fmt::Display;

pub use primitive_types::*;
pub use function::*;
pub use ntuple::*;

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    NTuple(NTuple),
    Function(Box<Function>),
}


impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(t) => t.fmt(f),
            Type::NTuple(t) => t.fmt(f),
            Type::Function(t) => (*(t.as_ref())).fmt(f),
        }
    }
}
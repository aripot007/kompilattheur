mod function;
mod ntuple;
mod weak;

use std::fmt::Display;

pub use function::*;
pub use ntuple::*;
pub use weak::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    None,
    Bool,
    Int,
    String,
    List,
    Any,
    NTuple(NTuple),
    Function(Box<Function>),
    Weak(Weak),
}

impl Type {
    pub fn get_discriminant(&self) -> u8 {
        match self {
            Type::None => 0,
            Type::Bool => 1,
            Type::Int => 2,
            Type::String => 3,
            Type::List => 4,
            Type::Any
            | Type::NTuple(_)
            | Type::Function(_)
            | Type::Weak(_) => panic!("Cannot get discriminant for type {}", self),
        }
    }
}


impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::None => write!(f, "none"),
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::String => write!(f, "string"),
            Type::List => write!(f, "list"),
            Type::Any => write!(f, "any"),
            Type::NTuple(t) => t.fmt(f),
            Type::Function(t) => (*(t.as_ref())).fmt(f),
            Type::Weak(t) => t.fmt(f), 
        }
    }
}
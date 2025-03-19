mod function;
mod ntuple;
mod weak;

use std::fmt::Display;

pub use function::*;
pub use ntuple::*;
pub use weak::*;

static WORD_SIZE: usize = 8;

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

impl Type {
    pub fn get_decalage(&self) -> usize {
        match self {
            Type::None => 0,
            Type::Bool => WORD_SIZE*2,
            Type::Int => WORD_SIZE*2,
            Type::String => WORD_SIZE*2,
            Type::List => WORD_SIZE*2,
            Type::Any => WORD_SIZE*2,
            Type::NTuple(_) => panic!("You shouldn't ask for the decalage of a NTuple"),
            Type::Function(_) => panic!("You shouldn't ask for the decalage of a Function"),
            Type::Weak(t) => t.get_decalage(),
        }
    }

}
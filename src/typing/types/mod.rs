mod function;
mod ntuple;
mod weak;

use std::{fmt::Display, ops::BitOr};

pub use function::*;
pub use ntuple::*;
pub use weak::*;

static WORD_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Type {
    None,
    Bool,
    Int,
    Float,
    String,
    List,
    Range,
    Any,
    NTuple(NTuple),
    Function(Box<Function>),
    Weak(Weak),
}

impl Type {
    /// Get a bitmask representing this type
    pub fn get_bitmask(&self) -> u8 {
        match self {
            Type::None => 0b00000001,
            Type::Bool => 0b00000010,
            Type::Int => 0b00000100,
            Type::Float => 0b00001000,
            Type::String => 0b00010000,
            Type::List => 0b00100000,
            Type::Range => 0b01000000,
            Type::Any => 0b01111111,
            Type::Weak(w) => w
                .get_possible()
                .iter()
                .map(Type::get_bitmask)
                .reduce(u8::bitor)
                .unwrap(),
            Type::NTuple(_) | Type::Function(_) => {
                panic!("Cannot get discriminant for type {}", self)
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::None => write!(f, "none"),
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::List => write!(f, "list"),
            Type::Range => write!(f, "range"),
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
            Type::Bool => WORD_SIZE + 1,
            Type::Int => WORD_SIZE + 1,
            Type::Float => WORD_SIZE + 1,
            Type::String => WORD_SIZE + 1,
            Type::List => WORD_SIZE + 1,
            Type::Range => WORD_SIZE + 1,
            Type::Any => WORD_SIZE + 1,
            Type::NTuple(_) => panic!("You shouldn't ask for the decalage of a NTuple"),
            Type::Function(_) => panic!("You shouldn't ask for the decalage of a Function"),
            Type::Weak(t) => t.get_decalage(),
        }
    }
}

impl Type {
    pub fn is_compatible(&self, other: Type) -> bool {
        match (self, &other) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Weak(w), t) | (t, Type::Weak(w)) => w.is_compatible(t.clone()),
            (t1, t2) => return t1 == t2,
        }
    }
}

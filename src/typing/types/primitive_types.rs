use std::fmt::Display;


#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    None,
    Bool,
    Int,
    String,
    List,
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::None => write!(f, "none"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Int => write!(f, "int"),
            PrimitiveType::String => write!(f, "string"),
            PrimitiveType::List => write!(f, "list"),
        }
    }
}

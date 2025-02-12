use std::fmt::Display;

use crate::typing::types::PrimitiveType;

use super::Type;

#[derive(Debug, Clone)]
pub struct NTuple {
    vals: Vec<Type>,
}

impl From<Vec<Type>> for NTuple {
    fn from(vals: Vec<Type>) -> Self {
        NTuple {
            vals
        }
    }
}

impl Display for NTuple {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if self.vals.len() == 0 {
            return PrimitiveType::None.fmt(f);
        }

        let strs: Vec<String> = self.vals.iter().map(|t| t.to_string()).collect();
        write!(f, "({})", strs.join(", "))
    }
}

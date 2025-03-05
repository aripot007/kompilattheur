use std::fmt::Display;

use super::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    args: Vec<Type>,
    returns: Type,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if self.args.len() == 0 {
            Type::None.fmt(f)?;
        } else {
            let strs: Vec<String> = self.args.iter().map(|t| t.to_string()).collect();
            write!(f, "({})", strs.join(", "))?;
        }

        write!(f, " -> {}", self.returns)
    }
}

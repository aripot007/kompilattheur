use std::fmt::{Debug, Display, Formatter, Result}; // Add Debug trait import

use crate::common::types::token::Token;

#[derive(Clone)]
pub enum Lexem {
    Terminal(Token),
    NonTerminal(usize),
}

impl Debug for Lexem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.debug())
    }
}

impl Display for Lexem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Lexem::Terminal(token) => write!(f, "{}", token),
            Lexem::NonTerminal(id) => write!(f, "{}", id),   
        }
    }
}

impl Lexem {

    pub fn debug(&self) -> String {
        match self {
            Lexem::Terminal(token) => format!("Terminal({})", token.repr()),
            Lexem::NonTerminal(id) => format!("NonTerminal({})", id),
        }
    }
}
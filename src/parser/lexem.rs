use std::fmt::{Debug, Display, Formatter, Result}; // Add Debug trait import

use crate::{analysis_table::NonTerminal, common::types::Token};

#[derive(Clone, PartialEq, Eq)]
pub enum Lexem {
    Terminal(Token),
    NonTerminal(NonTerminal),
}

impl Debug for Lexem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Lexem::Terminal(token) => write!(f, "Terminal({})", token),
            Lexem::NonTerminal(id) => write!(f, "NonTerminal({})", id),
        }
    }
}

impl Display for Lexem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Lexem::Terminal(token) => write!(f, "{}", token),
            Lexem::NonTerminal(non_terminal) => write!(f, "{}", non_terminal),   
        }
    }
}
use std::fmt::{Debug, Display, Formatter, Result}; // Add Debug trait import

use crate::common::types::token::Token;

use super::analysis_table::get_non_terminal_name;

#[derive(Clone)]
pub enum Lexem {
    Terminal(Token),
    NonTerminal(usize),
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
            Lexem::NonTerminal(id) => write!(f, "{}", get_non_terminal_name(id)),   
        }
    }
}
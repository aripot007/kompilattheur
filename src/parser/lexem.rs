use crate::common::types::token::Token;

pub struct NonTerminal {
    pub name: String,
}

pub enum Lexem {
    Terminal(Token),
    NonTerminal()
}
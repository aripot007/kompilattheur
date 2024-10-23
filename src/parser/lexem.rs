use crate::common::types::token::Token;

pub enum Lexem {
    Terminal(Token),
    NonTerminal(usize),
}

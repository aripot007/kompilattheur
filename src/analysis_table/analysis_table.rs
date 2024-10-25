use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;
use std::collections::HashMap;
use std::mem::{discriminant, Discriminant};

pub struct AnalysisTable {
    pub table: [HashMap<Discriminant<Token>, Vec<Lexem>>; 5],
}

impl AnalysisTable {
    pub fn get(&self, id: &usize, token: &Token) -> Option<&Vec<Lexem>> {
        self.table[*id].get(&discriminant(token))
    }
}

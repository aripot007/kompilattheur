use std::collections::HashMap;
use super::types::token::Token;

struct SymbolTable {
    // structure pour stocker la table des symboles
    symbol_table: HashMap<String, Token>,
    last_assigned_tag: usize,
}

pub fn new() -> SymbolTable {
    // initialisation de la table des symboles
    SymbolTable {
        symbol_table: HashMap::new(),
        last_assigned_tag: 0,
    }
}

impl SymbolTable {
    pub fn insert_keyword(&mut self, name: &str) { 
        // insertion d'un mot clé, sous format KeywordToken
        self.symbol_table.insert(name.to_string(), Token::keyword(self.last_assigned_tag, name.to_string()));
        self.last_assigned_tag += 1;
    }
    
    pub fn get_token(&mut self, name: &str) -> &Token {
        self.symbol_table
            .entry(name.to_string())
            .or_insert_with(|| Token::identifier(self.last_assigned_tag, name.to_string()))
    }
}
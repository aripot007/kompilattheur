use crate::common::types::{Token, IdToken};
use std::collections::HashMap;

pub struct TokenTable {
    /// Stocke les tokens connus
    known_tokens: HashMap<String, Token>,

    /// Stocke les noms des identifieurs déjà connus
    id_names: Vec<String>,
}

impl TokenTable {

    pub fn new() -> TokenTable {
        TokenTable {
            known_tokens: HashMap::new(),
            id_names: Vec::new(),
        }
    }

    pub fn reserve_word(&mut self, name: &str, token: Token) {
        // insertion d'un mot réservé
        self.known_tokens.insert(
            name.to_string(),
            token,
        );
    }

    pub fn get_token(&mut self, name: String) -> Token {
        let token = self
            .known_tokens
            .entry(name.to_string())
            .or_insert_with(|| {
                let token = Token::Identifier(IdToken {id: self.id_names.len()});
                self.id_names.push(name.to_string());
                token
            });
        token.clone()
    }

    pub fn get_ident_name(&self, id: usize) -> &String {
        &self.id_names[id]
    }
}

#[cfg(test)]
mod tests {
    // use std::any::Any;

    // use super::*;

    /*

    #[test]
    fn test_insert_keyword() {
        let mut symbol_table = new();
        symbol_table.insert_keyword("if");
        assert_eq!(symbol_table.symbol_table.get("if").unwrap().get_tag(), 1);
        assert!(match symbol_table.symbol_table.get("if").unwrap() {
            Token::Keyword(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_get_token() {
        let mut symbol_table = new();
        let token = symbol_table.get_token("i");
        assert_eq!(token.get_tag(), 1);
        assert!(match token {
            Token::Identifier(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_automatic_tag_assignment() {
        let mut symbol_table = new();
        symbol_table.insert_keyword("if");
        symbol_table.get_token("j");
        symbol_table.get_token("k");
        assert_eq!(symbol_table.symbol_table.get("if").unwrap().get_tag(), 1);
        assert_eq!(symbol_table.symbol_table.get("j").unwrap().get_tag(), 2);
        assert_eq!(symbol_table.symbol_table.get("k").unwrap().get_tag(), 3);
        assert_eq!(symbol_table.symbol_table.get("j").unwrap().get_tag(), 2);
    }

    */
}

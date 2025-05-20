use crate::common::types::{IdToken, Token};
use std::collections::HashMap;

pub struct TokenTable {
    /// Stocke les tokens connus
    known_tokens: HashMap<String, Token>,

    /// Stocke les noms des identifieurs déjà connus
    id_names: Vec<String>,

    /// Stocke les noms des identifiants des fonctions smollib
    smollib_names: Vec<String>,
    /// as assigned id for smollib function
    last_smollib_ident_id: usize,
}

impl TokenTable {
    pub fn new() -> TokenTable {
        TokenTable {
            known_tokens: HashMap::new(),
            id_names: Vec::new(),
            smollib_names: Vec::new(),
            last_smollib_ident_id: 0,
        }
    }

    pub fn reserve_smollib_name(&mut self, name: &str, token: Token) {
        self.reserve_word(name, token);
        self.smollib_names.push(String::from(name));

        if self.last_smollib_ident_id == 0 {
            self.last_smollib_ident_id = usize::MAX;
        } else {
            self.last_smollib_ident_id -= 1;
        }
    }

    pub fn reserve_word(&mut self, name: &str, token: Token) {
        // insertion d'un mot réservé
        self.known_tokens.insert(name.to_string(), token);
    }

    pub fn get_token(&mut self, name: String) -> Token {
        let token = self
            .known_tokens
            .entry(name.to_string())
            .or_insert_with(|| {
                let token = Token::Identifier(IdToken {
                    id: self.id_names.len(),
                    name: name.clone(),
                });
                self.id_names.push(name.to_string());
                token
            });
        token.clone()
    }

    pub fn get_ident_name(&self, id: usize) -> &String {
        if id < self.id_names.len() {
            &self.id_names[id]
        } else {
            &self.smollib_names[usize::MAX - id]
        }
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

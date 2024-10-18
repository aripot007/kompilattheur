use super::types::token::Token;
use std::collections::HashMap;

struct SymbolTable {
    // structure pour stocker la table des symboles
    symbol_table: HashMap<String, Token>,
    last_assigned_tag: usize,
}

pub fn new() -> SymbolTable {
    // initialisation de la table des symboles
    SymbolTable {
        symbol_table: HashMap::new(),
        last_assigned_tag: 1,
    }
}

impl SymbolTable {
    pub fn insert_keyword(&mut self, name: &str) {
        // insertion d'un mot clé, sous format KeywordToken
        self.symbol_table.insert(
            name.to_string(),
            Token::keyword(self.last_assigned_tag, name.to_string()),
        );
        self.last_assigned_tag += 1;
    }

    pub fn get_token(&mut self, name: &str) -> &Token {
        let token = self
            .symbol_table
            .entry(name.to_string())
            .or_insert_with(|| {
                let token = Token::identifier(self.last_assigned_tag, name.to_string());
                self.last_assigned_tag += 1;
                token
            });
        token
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

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
}

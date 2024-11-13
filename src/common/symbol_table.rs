use std::collections::HashMap;
use super::types::Node;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(),
    Parameter(),
    Function(),
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub table: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn get_symbol(&self, key: &str) -> Option<&Symbol> {
        if self.table.get(key).is_some() {
            return self.table.get(key).clone();
        } else {
            return None;
        }
    }

    pub fn update_symbol(&mut self, key: String, value: Symbol) {
        self.table.insert(key, value);
    }
}
    

#[cfg(test)]
mod tests {
    use std::result;

    use super::*;

    #[test]
    fn test_symbol_table() {
        // TODO
    }
}
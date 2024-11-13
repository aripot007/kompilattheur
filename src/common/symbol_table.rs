use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::common::symbol_table;

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

pub fn init_symbol_table() -> Node<SymbolTable> {
    Node::new(SymbolTable::new())
}

pub fn enter_scope(parent: RefCell<Node<SymbolTable>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let child= Node::new(SymbolTable::new());
    parent.borrow_mut().add_child(&parent, child.clone());
    child.clone()
}

pub fn exit_scope(node: RefCell<Node<SymbolTable>>) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let parent = node.borrow().get_parent();
    parent
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
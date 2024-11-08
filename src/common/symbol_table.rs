use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(),
    Parameter(),
    Function(),
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub table: HashMap<String, Symbol>,
    pub parent: Option<Rc<RefCell<SymbolTable>>>,
    pub childs: Vec<Rc<RefCell<SymbolTable>>>,
}

impl SymbolTable {
    pub fn new(parent: Option<Rc<RefCell<SymbolTable>>>) -> Rc<RefCell<SymbolTable>> {
        Rc::new(RefCell::new(SymbolTable {
            table: HashMap::new(),
            parent,
            childs: Vec::new(),
        }))
    }

    pub fn get_symbol(&mut self, name: String) -> Option<Symbol> {
        if self.table.contains_key(&name) {
            Some(self.table.get(&name).unwrap().clone())
        } else {
            let mut current_parent = self.parent.clone();
            while let Some(parent_rc) = current_parent {
                let parent = parent_rc.borrow();
                if parent.table.contains_key(&name) {
                    return Some(parent.table.get(&name).unwrap().clone());
                }
                current_parent = parent.parent.clone();
            }
            None
        }
    }

    pub fn enter_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        let new_table = SymbolTable::new(Some(Rc::new(RefCell::new(self.clone()))));
        self.childs.push(new_table.clone());
        new_table
    }

    pub fn exit_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        self.parent.as_ref().unwrap().clone()
    }
    
    
    pub fn update_symbol(&mut self, name: String, symbol: Symbol) {
        if self.table.contains_key(&name) {
            self.table.insert(name, symbol);
        } else {
            let mut current_parent = self.parent.clone();
            while let Some(parent_rc) = current_parent {
                let mut parent = parent_rc.borrow_mut();
                if parent.table.contains_key(&name) {
                    parent.table.insert(name, symbol);
                    return;
                }
                current_parent = parent.parent.clone();
            }
        }
    }
}


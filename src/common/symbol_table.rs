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

    pub fn get_symbol(&mut self, name: String, symbol: Symbol) -> Symbol {
        if self.table.contains_key(&name) {
            self.table.get(&name).unwrap().clone()
        }  else {
            while (self.parent.is_some()) {
                let parent = self.parent.as_ref().unwrap().borrow();
                if parent.table.contains_key(&name) {
                    return parent.table.get(&name).unwrap().clone();
                }
            }
            self.table.insert(name.clone(), symbol.clone());
            symbol
        }
    }

    pub fn enter_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        let scope = SymbolTable::new(Some(Rc::clone(self)));
        self.childs.push(Rc::clone(&scope));
        scope
    }

    pub fn exit_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        self.parent.as_ref().unwrap().clone()
    }
}


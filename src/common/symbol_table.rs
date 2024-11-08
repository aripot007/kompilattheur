use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

macro_rules! escape_mermaid {
    ($s: expr) => {
        $s.replace("#", "#35;")
            .replace("<", "#lt;")
            .replace(">", "#gt;")
            .replace("!", "#33;")
            .replace("\"", "#quot;")
            .replace("&", "#amp;")
            .replace("(", "#40;")
            .replace(")", "#41;")
            .replace("*", "#42;")
            .replace("+", "#plus;")
            .replace("-", "#minus;")
            .replace("[", "#91;")
            .replace("\\", "#92;")
            .replace("]", "#93;")
            .replace("^", "#94;")
            .replace("_", "#95;")
            .replace("`", "#96;")
            .replace("|", "#124;")
            .replace("~", "#126;")
    };
}


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

    pub fn enter_scope(&mut self, parent: &SymbolTable) -> Rc<RefCell<SymbolTable>> {
        let new_table = SymbolTable::new(Some(parent));
        self.childs.push(new_table.clone());
        new_table
    }

    pub fn exit_scope(&mut self) -> Rc<RefCell<SymbolTable>> {
        self.parent.as_ref().unwrap().clone()
    }
    
    
    pub fn update(&mut self, name: String, symbol: Symbol) {
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
            self.table.insert(name, symbol);
        }
    }
    
    pub fn generate_mermaid(&self) -> String {
        let mut result = String::new();
        result.push_str("classDiagram\n");
        
        let root = self.clone();
        let mut stack = vec![root];
        let scope = 0;
        while let Some(current) = stack.pop() {
            if current.parent.is_none() {
                result.push_str(&format!("class scope{} {{" , scope));
            } else {
                result.push_str(&format!("scope{}--scope{}\nclass scope{} {{" , scope-1, scope, scope));
            }
            for (name, symbol) in current.table.iter() {
                let symbol_line = match symbol {
                    Symbol::Variable() => "Variable",
                    Symbol::Parameter() => "Parameter",
                    Symbol::Function() => "Function",
                };
                result.push_str(&format!("{} : {}\n", escape_mermaid!(name), symbol_line));
            }
            println!("{:?}", current.childs.len());
            for child in current.childs.iter() {
                stack.push(child.try_borrow().unwrap().clone());
            }
            result.push_str("}\n");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::result;

    use super::*;

    #[test]
    fn test_symbol_table() {
        let root = SymbolTable::new(None);
        
        root.borrow_mut().update("a".to_string(), Symbol::Variable());
        root.borrow_mut().update("b".to_string(), Symbol::Variable());
        
        let child = root.borrow_mut().enter_scope(root.borrow());
        child.borrow_mut().update("a".to_string(), Symbol::Variable());
        child.borrow_mut().update("c".to_string(), Symbol::Variable());
        
        let grand_child = child.borrow_mut().enter_scope();
        grand_child.borrow_mut().update("a".to_string(), Symbol::Variable());
        
        let child = grand_child.borrow_mut().exit_scope();
        let root = child.borrow_mut().exit_scope();
        
        let another_child = root.borrow_mut().enter_scope();
        another_child.borrow_mut().update("d".to_string(), Symbol::Variable());
        
        let root = another_child.borrow_mut().exit_scope();
        
        let result = root.borrow_mut().generate_mermaid();
        println!("{}", result);
    }
}
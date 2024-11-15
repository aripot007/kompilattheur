use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use super::types::Node;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(),
    Parameter(),
    Function(),
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub table: HashMap<usize, (Symbol,)>,
    pub index: usize,
    pub last_given_index: usize,
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display = String::new();
        display.push_str(&format!(
            r#"<table>
    <thead>
        <tr>
            <th>Key</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>"#
        ));

        for (key, value) in self.table.iter() {
            display.push_str(&format!(
                r#"
        <tr>
            <td>{}</td>
            <td>{:?}</td>
        </tr>"#,
                key, value.0
            ));
        }

        display.push_str(&format!(
            r#"
    </tbody>
</table>
"#
        ));

        write!(f, "{}", display)
    }
}

impl SymbolTable {
    pub fn new(index: usize, last_given_index: usize) -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            index,
            last_given_index,
        }
    }

    pub fn get_symbol(&self, key: &usize) -> Option<&(Symbol,)> {
        if self.table.get(key).is_some() {
            return self.table.get(key).clone();
        } else {
            return None;
        }
    }

    pub fn update_symbol(&mut self, key: usize, value: Symbol) {
        self.table.insert(key, (value,));
    }
}

pub fn init_symbol_table() -> Rc<RefCell<Node<SymbolTable>>> {
    Node::new(SymbolTable::new(0,0))
}

pub fn enter_scope(parent: Rc<RefCell<Node<SymbolTable>>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let index = parent.borrow().get_value().index + 1;
    let last_given_index = index.clone();
    let child = Node::new(SymbolTable::new(index, last_given_index));
    parent.borrow_mut().add_child(&parent, child.clone());
    child.clone()
}

pub fn exit_scope(node: Rc<RefCell<Node<SymbolTable>>>) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let parent = node.borrow().get_parent();
    match parent {
        Some(parent) => {
            let last_given_index = node.borrow().get_value().last_given_index;
            parent.borrow_mut().get_value().last_given_index = last_given_index;
            Some(parent)
        }
        None => None,
    }
}

pub fn get_scope(root: Rc<RefCell<Node<SymbolTable>>>, index: usize) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let mut stack = vec![root];
    while !stack.is_empty() {
        let node = stack.pop().unwrap();
        if node.borrow().get_value().index == index {
            return Some(node);
        }
        for child in node.borrow().get_children() {
            stack.push(child);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut symbol_table = SymbolTable::new(0,0);
        symbol_table.update_symbol(1, Symbol::Variable());
        symbol_table.update_symbol(2, Symbol::Parameter());

        print!("{}", symbol_table);
    }
}

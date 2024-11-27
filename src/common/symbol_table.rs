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
            r#"Node index: {}
<table>
    <thead>
        <tr>
            <th>Key</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>"#
        , self.index));

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

    pub fn update_symbol(&mut self, key: usize, value: (Symbol,)) {
        self.table.insert(key, value);
    }
}


/// Create a new symbol table
/// 
/// # Returns
/// A counted reference to a tree that represents the symbol table
pub fn init_symbol_table() -> Rc<RefCell<Node<SymbolTable>>> {
    Node::new(SymbolTable::new(0,0))
}

/// Enter a new scope
/// 
/// # Arguments
/// * `parent` - A counted reference to the parent node
/// 
/// # Returns
/// A counted reference to the child node
/// 
/// # Example
/// ```
/// let root = init_symbol_table();
/// let child = enter_scope(root);
/// ```
pub fn enter_scope(parent: Rc<RefCell<Node<SymbolTable>>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let index = parent.borrow().get_value().index + 1;
    let last_given_index = index.clone();
    let child = Node::new(SymbolTable::new(index, last_given_index));
    parent.borrow_mut().add_child(&parent, child.clone());
    child.clone()
}

/// Exit the current scope
/// 
/// # Arguments
/// * `node` - A counted reference to the current node
/// 
/// # Returns
/// A counted reference to the parent node, if it exists, otherwise the current node
/// 
/// # Example
/// ```
/// let root = init_symbol_table();
/// let child = enter_scope(root);
/// let parent = exit_scope(child);
/// ```
pub fn exit_scope(node: Rc<RefCell<Node<SymbolTable>>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let parent = node.borrow().get_parent();
    match parent {
        Some(parent) => {
            let last_given_index = node.borrow().get_value().last_given_index;
            parent.borrow_mut().get_value().last_given_index = last_given_index;
            parent
        }
        None => node,
    }
}

/// Get the scope of a given index
/// 
/// the index is generated when a new scope is created, it follows a depth-first order, starting from 0
/// 
/// # Arguments
/// * `node` - A counted reference to a node
/// * `index` - The index of the scope
/// 
/// # Returns
/// A counted reference to the node that represents the scope, if it exists, otherwise None
pub fn get_scope(node: Rc<RefCell<Node<SymbolTable>>>, index: usize) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let parent = node.borrow().get_parent();
    let root = match parent {
        Some(parent) => get_scope(parent, index).unwrap_or_else(|| node.clone()),
        None => node.clone(),
    };
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

/// Update the symbol table
/// Adds a new symbol to the current scope
/// 
/// # Arguments
/// * `node` - A counted reference to the current node
/// * `key` - The key of the symbol
/// * `value` - The value of the symbol
/// 
/// # Returns
/// the updated node
pub fn update_tree(node: Rc<RefCell<Node<SymbolTable>>>, key: &usize, value:(Symbol,)) -> Rc<RefCell<Node<SymbolTable>>> {
    node.borrow_mut().get_value().update_symbol(*key, value);
    node
}

/// Get a symbol from the symbol table
/// 
/// # Arguments
/// * `node` - A counted reference to the current node
/// * `key` - The key of the symbol
/// 
/// # Returns
/// The symbol, if it exists, otherwise None
pub fn get_symbol(node: Rc<RefCell<Node<SymbolTable>>>, key: &usize) -> Option<(Symbol,)> {
    if let Some(sym) = node.borrow().get_value().table.get(key) {
        return Some(sym.clone());
    } else {
        match node.borrow().get_parent() {
            Some(parent) => return get_symbol(parent, key),
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut symbol_table = SymbolTable::new(0,0);
        symbol_table.update_symbol(1, (Symbol::Variable(),));
        symbol_table.update_symbol(2, (Symbol::Parameter(),));

        print!("{}", symbol_table);
    }
    
    #[test]
    fn test_symbol_table_tree() {
        let node = init_symbol_table();
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &1, (Symbol::Variable(),));
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &2, (Symbol::Parameter(),));
        print!("{}", node.borrow().get_value());
        let node = enter_scope(node);
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &3, (Symbol::Function(),));
        print!("{}", node.borrow().get_value());
        let node = enter_scope(node);
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &4, (Symbol::Variable(),));
        print!("{}", node.borrow().get_value());
        let node = exit_scope(node);
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &5, (Symbol::Function(),));
        print!("{}", node.borrow().get_value());
        let node = enter_scope(node);
        print!("{}", node.borrow().get_value());
        let node = update_tree(node, &6, (Symbol::Variable(),));
        print!("{}", node.borrow().get_value());
        let node = exit_scope(node);
        print!("{}", node.borrow().get_value());
        let node = exit_scope(node);
        print!("{}", node.borrow().get_value());
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::common::types::Node;

pub type SymbolTableRef = Rc<RefCell<Node<SymbolTable>>>;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(),
    Parameter(),
    Function(),
}

#[derive(Debug, Clone)]
pub struct SymbolTableElement {
    pub symbol: Symbol,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub table: HashMap<usize, SymbolTableElement>,

    pub index: usize,
    last_given_index: usize,
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
            <th>Name</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>"#,
            self.index
        ));

        for (key, value) in self.table.iter() {
            display.push_str(&format!(
                r#"
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{:?}</td>
        </tr>"#,
                key, value.name, value.symbol
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
    fn new(index: usize, last_given_index: usize) -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            index,
            last_given_index,
        }
    }

    fn update_symbol(&mut self, key: usize, value: SymbolTableElement) {
        self.table.insert(key, value);
    }
}

impl Node<SymbolTable> {
    /// # Insert a symbol into the symbol table
    ///
    /// ## Arguments
    /// * `key` - The key of the symbol
    /// * `value` - The value of the symbol
    ///
    /// ## Example
    /// ```
    /// let node = init_symbol_table();
    /// node.borrow_mut().insert_symbol(1, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func") });
    /// ```
    pub fn insert_symbol(&mut self, key: usize, value: SymbolTableElement) {
        self.get_value_ref_mut().table.insert(key, value);
    }

    fn set_last_given_index(&mut self, last_given_index: usize) {
        self.get_value_ref_mut().last_given_index = last_given_index;
    }
}

/// # Create a new symbol table
///
/// ## Returns
/// * `root` - root node of the symbol table
pub fn init_symbol_table() -> SymbolTableRef {
    Node::new(SymbolTable::new(0, 0))
}

/// # Enter a new scope
///
/// ## Arguments
/// * `parent` - A counted reference to the parent node
///
/// ## Returns
/// * `node` - A counted reference to the child node
///
/// ## Example
/// ```
/// let node = init_symbol_table();
/// let node = enter_scope(node);
/// ```
pub fn enter_scope(parent: Rc<RefCell<Node<SymbolTable>>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let index = parent.borrow().get_value().last_given_index + 1;
    let last_given_index = index.clone();
    let child = Node::new(SymbolTable::new(index, last_given_index));
    parent.borrow_mut().add_child(&parent, child.clone());
    child.clone()
}

/// # Exit the current scope
///
/// ## Arguments
/// * `node` - A counted reference to the current node
///
/// ## Returns
/// * `node` - A counted reference to the parent node, if it doesn't have a parent, it returns itself
///
/// ## Example
/// ```
/// let node = init_symbol_table();
/// let node = exit_scope(node); // gives node
///
/// let node = init_symbol_table();
/// let node = enter_scope(node);
/// let node = exit_scope(node); // gives root
/// ```
pub fn exit_scope(node: Rc<RefCell<Node<SymbolTable>>>) -> Rc<RefCell<Node<SymbolTable>>> {
    let parent = node.borrow().get_parent();
    match parent {
        Some(parent) => {
            let last_given_index = node.borrow().get_value().last_given_index;
            parent.borrow_mut().set_last_given_index(last_given_index);
            parent
        }
        _ => node,
    }
}

/// # Get the scope of a given index
/// Each scope has a unique index, defined by the order in which they were created (follows depth-first order), starting from 0
///
/// ## Arguments
/// * `node` - A counted reference to a node
/// * `index` - The index of the scope
///
/// ## Returns
/// * `node` / `None` - A counted reference to the node with the given index, if it exists, otherwise None
///
/// ## Example
/// ```
/// let node = init_symbol_table();
/// let node = enter_scope(node);
/// let node = exit_scope(node);
/// let node = enter_scope(node);
/// let node = exit_scope(node);
/// let node = get_scope(node, 1); // gives node
/// let node = get_scope(node, 3); // gives None
/// ```
pub fn get_scope(
    node: Rc<RefCell<Node<SymbolTable>>>,
    index: usize,
) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let base = node.clone();
    let res = get_scope_rec(node, index);
    match res {
        Some(res) => {
            let last_given_index = base.borrow().get_value().last_given_index;
            res.borrow_mut().set_last_given_index(last_given_index);
            Some(res)
        }
        _ => None,
    }
}

fn get_scope_rec(
    node: Rc<RefCell<Node<SymbolTable>>>,
    index: usize,
) -> Option<Rc<RefCell<Node<SymbolTable>>>> {
    let parent = node.borrow().get_parent();
    let root = match parent {
        Some(parent) => get_scope(parent, index).unwrap_or_else(|| node.clone()),
        _ => node.clone(),
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

/// # Get a symbol from the symbol table
///
/// ## Arguments
/// * `node` - A counted reference to the current node
/// * `key` - The key of the symbol
///
/// ## Returns
/// * `base` - the node given as an argument
/// * `symbol` / `None` - the symbol if it exists, otherwise None
pub fn get_symbol(
    node: Rc<RefCell<Node<SymbolTable>>>,
    key: &usize,
) -> (Rc<RefCell<Node<SymbolTable>>>, Option<SymbolTableElement>) {
    let base = node.clone();
    let symbol = get_symbol_rec(node, key);
    (base, symbol)
}

fn get_symbol_rec(node: Rc<RefCell<Node<SymbolTable>>>, key: &usize) -> Option<SymbolTableElement> {
    if let Some(sym) = node.borrow().get_value().table.get(key) {
        return Some(sym.clone());
    } else {
        match node.borrow().get_parent() {
            Some(parent) => return get_symbol_rec(parent, key),
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut symbol_table = SymbolTable::new(0, 0);
        symbol_table.update_symbol(1, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var1") });
        symbol_table.update_symbol(2, SymbolTableElement { symbol: Symbol::Parameter(), name: String::from("param1") });

        print!("{}", symbol_table);
    }

    #[test]
    fn test_symbol_table_tree() {
        let root = init_symbol_table();
        let node = root.clone();
        node.borrow_mut().insert_symbol(1, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func1") });

        let node = enter_scope(node);

        let node = exit_scope(node);

        let node = enter_scope(node);

        let node = exit_scope(node);

        let res = root.borrow().generate_unsafe_mermaid();

        let expected = r#"%%{ init: { 'flowchart': { 'curve': 'linear' } } }%%
flowchart TD
0["Node index: 0
<table>
    <thead>
        <tr>
            <th>Key</th>
            <th>Name</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>1</td>
            <td>func1</td>
            <td>Function</td>
        </tr>
    </tbody>
</table>
"]
1["Node index: 1
<table>
    <thead>
        <tr>
            <th>Key</th>
            <th>Name</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>
    </tbody>
</table>
"]
0 --> 1
2["Node index: 2
<table>
    <thead>
        <tr>
            <th>Key</th>
            <th>Name</th>
            <th>Type</th>
        </tr>
    </thead>
    <tbody>
    </tbody>
</table>
"]
0 --> 2
"#;
        assert_eq!(res, expected);
        assert_eq!(
            node.borrow().get_value().index,
            root.borrow().get_value().index
        );
    }

    #[test]
    fn test_get_scope() {
        let root = init_symbol_table();
        let node = root.clone();

        let node = enter_scope(node);
        let node = exit_scope(node);

        let node = enter_scope(node);
        let node = exit_scope(node);

        let node = get_scope(node, 1).unwrap();
        assert_eq!(node.borrow().get_value().index, 1);

        let node = get_scope(node, 0).unwrap();
        assert_eq!(
            node.borrow().get_value().index,
            root.borrow().get_value().index
        );

        let node = get_scope(node, 3);
        assert!(node.is_none());
    }

    #[test]
    fn test_get_symbol() {
        let node = init_symbol_table();
        node.borrow_mut().insert_symbol(1, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func2") });
        node.borrow_mut().insert_symbol(2, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var2") });

        let node = enter_scope(node);
        node.borrow_mut().insert_symbol(3, SymbolTableElement { symbol: Symbol::Parameter(), name: String::from("param2") });

        let (node, symbol) = get_symbol(node, &1);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Function(), name: \"func2\" })");

        let (node, symbol) = get_symbol(node, &2);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Variable(), name: \"var2\" })");

        let (node, symbol) = get_symbol(node, &3);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Parameter(), name: \"param2\" })");

        let node = exit_scope(node);
        let (_node, symbol) = get_symbol(node, &3);
        assert!(symbol.is_none());
    }

    #[test]
    fn bigger_tree_and_all_functions() {
        let root = init_symbol_table();
        let node = root.clone();
        node.borrow_mut().insert_symbol(1, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func1") });
        node.borrow_mut().insert_symbol(2, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var1") });

        let node = enter_scope(node);
        node.borrow_mut().insert_symbol(3, SymbolTableElement { symbol: Symbol::Parameter(), name: String::from("param1") });
        node.borrow_mut().insert_symbol(4, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var2") });
        node.borrow_mut().insert_symbol(5, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func2") });

        let node = enter_scope(node);
        node.borrow_mut().insert_symbol(6, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func3") });
        node.borrow_mut().insert_symbol(7, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var3") });

        let node = exit_scope(node);
        node.borrow_mut().insert_symbol(8, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func4") });

        let node = enter_scope(node);

        let node = get_scope(node, 0).unwrap();
        node.borrow_mut().insert_symbol(9, SymbolTableElement { symbol: Symbol::Function(), name: String::from("func5") });

        let node = enter_scope(node);
        node.borrow_mut().insert_symbol(10, SymbolTableElement { symbol: Symbol::Variable(), name: String::from("var4") });

        let node = exit_scope(node);

        let node = enter_scope(node);
        let node = exit_scope(node);

        assert_eq!(
            node.borrow().get_value().index,
            root.borrow().get_value().index
        );

        let (node, symbol) = get_symbol(node, &1);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Function, name: \"func1\" })");

        let (node, symbol) = get_symbol(node, &7);
        assert!(symbol.is_none());

        let node = get_scope(node, 2).unwrap();
        let (node, symbol) = get_symbol(node, &7);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Variable, name: \"var3\" })");
        let (node, symbol) = get_symbol(node, &5);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Function, name: \"func2\" })");
        let (node, symbol) = get_symbol(node, &2);
        let res = format!("{:?}", symbol);
        assert_eq!(res, "Some(SymbolTableElement { symbol: Variable, name: \"var1\" })");
        let (_node, symbol) = get_symbol(node, &10);
        assert!(symbol.is_none());

        let res = root.borrow().generate_unsafe_mermaid();
        println!("{}", res);
    }
}

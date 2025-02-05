use super::{init_symbol_table, SymbolTableRef};
use crate::{ast::nodes, common::types::Node};

pub fn generate(root: nodes::Root) -> (nodes::Root, SymbolTableRef) {
    let table = init_symbol_table();

    /*
    TODO
    Recursive
    1. switch type AST Node of node
    2. call the corresponding function
    3. return the new node and the table


    function def -> create child table and enter scope
    function param -> insert symbol in current table
    ...
     */

    return (root, table);
}

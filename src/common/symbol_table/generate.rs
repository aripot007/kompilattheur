use super::{enter_scope, exit_scope, init_symbol_table, Symbol, SymbolTableElement, SymbolTableRef};
use crate::{ast::nodes, typing::{Type, Weak}};

pub fn generate(root: nodes::Root) -> (nodes::Root, SymbolTableRef) {
    let table = init_symbol_table();
    let (root, _) = generate_from_node_root(root, table.clone());

    return (root, table);
}

fn generate_from_node_root(root: nodes::Root, table: SymbolTableRef) -> (nodes::Root, SymbolTableRef) {
    let mut table = table;
    
    // Process all function definitions
    for def in &root.defs.defs {
        table = generate_from_def(def, table);
    }
    
    // Process the main block
    table = generate_from_block(&root.block, table);
    
    (root, table)
}

fn generate_from_def(def: &nodes::Def, table: SymbolTableRef) -> SymbolTableRef {
    let func_id = def.identifier.element.id;
    let name = def.identifier.element.name.clone();
    let symbol_table_element = SymbolTableElement { 
        symbol: Symbol::Function(), 
        name: name,
        symbol_type: Type::Any, // TODO : Function typing
    };
    table.borrow_mut().insert_symbol(func_id, symbol_table_element);
    
    // Enter a new scope for this function
    let function_table = enter_scope(table.clone());

    for param in &def.params {
        let param_id = param.identifier.element.id;
        let param_name = param.identifier.element.name.clone();
        let param_element = SymbolTableElement {
            symbol: Symbol::Parameter(),
            name: param_name,
            symbol_type: Type::Weak(Weak::new())
        };
        function_table.borrow_mut().insert_symbol(param_id, param_element);
    }
    
    let _ = generate_from_block(&def.block, function_table.clone());
    
    exit_scope(function_table)
}

fn generate_from_block(block: &nodes::Block, table: SymbolTableRef) -> SymbolTableRef {
    let mut table = table;
    
    for statement in &block.statements {
        match statement {
            nodes::Statement::Assign(assign) => {
                // Extract variable identifier from the left side of the assignment
                if let Some((id, name)) = extract_identifier_from_expression(&assign.destination) {
                    let var_element = SymbolTableElement {
                        symbol: Symbol::Variable(),
                        name,
                        symbol_type: Type::Any, // TODO: type assign expressions
                    };
                    table.borrow_mut().insert_symbol(id, var_element);
                }
            },
            nodes::Statement::For(for_loop) => {
                let var_id = for_loop.var.element.id;
                let var_name = for_loop.var.element.name.clone();
                
                let loop_table = enter_scope(table.clone());
                
                let var_element = SymbolTableElement {
                    symbol: Symbol::Variable(),
                    name: var_name,
                    symbol_type: Type::Any
                };
                loop_table.borrow_mut().insert_symbol(var_id, var_element);

                let _ = generate_from_block(&for_loop.block, loop_table.clone());

                table = exit_scope(loop_table);
            },
            nodes::Statement::Conditional(cond) => {
                let if_table = enter_scope(table.clone());
                
                let _ = generate_from_block(&cond.if_block, if_table.clone());

                table = exit_scope(if_table);
                
                if let Some(else_block) = &cond.else_block {
                    let else_table = enter_scope(table.clone());

                    let _ = generate_from_block(else_block, else_table.clone());

                    table = exit_scope(else_table);
                }
            },

            _ => {},
        }
    }
    
    table
}

fn extract_identifier_from_expression(expr: &nodes::Expression) -> Option<(usize, String)> {
    match expr {
        nodes::Expression::Factor(factor) => get_identifier_from_factor(factor),
        nodes::Expression::BINOP(_, _, _) => None, 
        nodes::Expression::UNOP(_, _) => None,    
        nodes::Expression::NotImplemented => None,
    }
}

fn get_identifier_from_factor(factor: &nodes::Factor) -> Option<(usize, String)> {
    match factor {
        nodes::Factor::Identifier(id_file_element) => {
            Some((id_file_element.element.id, id_file_element.element.name.clone()))
        },
        nodes::Factor::Call { identifier, .. } => {
            Some((identifier.id, identifier.name.clone()))
        },
        nodes::Factor::Expr(expr) => {
            extract_identifier_from_expression(expr)
        },
        
        _ => None,
    }
}

use super::{enter_scope, exit_scope, init_symbol_table, Symbol, SymbolTableRef};
use crate::ast::nodes;

pub fn generate(root: nodes::Root) -> (nodes::Root, SymbolTableRef) {
    let table = init_symbol_table();
    let (root, _) = generate_from_node_root(root, table.clone());

    return (root, table);
}

fn generate_from_node_root(root: nodes::Root, table: SymbolTableRef) -> (nodes::Root, SymbolTableRef) {
    // Process definitions first - they should be at the top level
    let mut current_table = table.clone();
    
    // Process all function definitions
    for def in &root.defs.defs {
        current_table = generate_from_def(def, current_table);
    }
    
    // Then process the main block
    current_table = generate_from_block(&root.block, current_table);
    
    (root, current_table)
}

fn generate_from_def(def: &nodes::Def, table: SymbolTableRef) -> SymbolTableRef {
    // Get the identifier from the function definition
    let func_id = def.identifier.element.id;
    
    // Insert function symbol in the current table
    table.borrow_mut().insert_symbol(func_id, (Symbol::Function(),));
    
    // Enter a new scope for this function
    let function_table = enter_scope(table.clone());
    
    // Process parameters
    for param in &def.params {
        let param_id = param.identifier.element.id;
        function_table.borrow_mut().insert_symbol(param_id, (Symbol::Parameter(),));
    }
    
    // Process function body block
    let _ = generate_from_block(&def.block, function_table.clone());
    
    // Exit scope and return original table
    exit_scope(function_table)
}

fn generate_from_block(block: &nodes::Block, table: SymbolTableRef) -> SymbolTableRef {
    let mut current_table = table.clone();
    
    // Process each statement in the block
    for statement in &block.statements {
        match statement {
            nodes::Statement::Assign(assign) => {
                // Extract variable identifier from the left side of the assignment
                if let Some(id) = extract_identifier_from_expression(&assign.destination) {
                    current_table.borrow_mut().insert_symbol(id, (Symbol::Variable(),));
                }
            },
            nodes::Statement::For(for_loop) => {
                // Extract the loop variable id
                let var_id = for_loop.var.element.id;
                
                // Enter a new scope for the loop
                let loop_table = enter_scope(current_table.clone());
                
                // Add the loop variable to the scope
                loop_table.borrow_mut().insert_symbol(var_id, (Symbol::Variable(),));
                
                // Process loop body
                let _ = generate_from_block(&for_loop.block, loop_table.clone());
                
                // Exit loop scope
                current_table = exit_scope(loop_table);
            },
            nodes::Statement::Conditional(cond) => {
                // Enter a new scope for the if block
                let if_table = enter_scope(current_table.clone());
                
                // Process if block
                let _ = generate_from_block(&cond.if_block, if_table.clone());
                
                // Exit if scope
                current_table = exit_scope(if_table);
                
                // Handle else block if present
                if let Some(else_block) = &cond.else_block {
                    // Enter a new scope for the else block
                    let else_table = enter_scope(current_table.clone());
                    
                    // Process else block
                    let _ = generate_from_block(else_block, else_table.clone());
                    
                    // Exit else scope
                    current_table = exit_scope(else_table);
                }
            },
            // Add handling for other statement types
            nodes::Statement::Print(_) => {
                // No symbols to add for Print statements
            },
            nodes::Statement::Return(_) => {
                // No symbols to add for Return statements
            },
            nodes::Statement::Expr(_) => {
                // No symbols to add for simple expression statements
            },
            nodes::Statement::NotImplemented => {
                // No symbols to add for not implemented statements
            },
        }
    }
    
    current_table
}

// Extract an identifier from an Expression
fn extract_identifier_from_expression(expr: &nodes::Expression) -> Option<usize> {
    match expr {
        nodes::Expression::Factor(factor) => extract_identifier_from_factor(factor),
        nodes::Expression::BINOP(_, _, _) => None, // Binary operations don't directly represent identifiers
        nodes::Expression::UNOP(_, _) => None,     // Unary operations don't directly represent identifiers
        nodes::Expression::NotImplemented => None,
    }
}

// Extract an identifier from a Factor
fn extract_identifier_from_factor(factor: &nodes::Factor) -> Option<usize> {
    match factor {
        nodes::Factor::Identifier(id_file_element) => {
            // Extract the hash of the identifier
            Some(id_file_element.element.id)
        },
        nodes::Factor::Call { identifier, .. } => {
            // Extract the hash of the function identifier
            Some(identifier.id)
        },
        nodes::Factor::Expr(expr) => {
            // If it's a wrapped expression, try to extract from it
            extract_identifier_from_expression(expr)
        },
        // Other factor types don't represent identifiers
        nodes::Factor::Integer(_) => None,
        nodes::Factor::String(_) => None,
        nodes::Factor::True(_) => None,
        nodes::Factor::False(_) => None,
        nodes::Factor::None(_) => None,
        nodes::Factor::List(_) => None,
    }
}

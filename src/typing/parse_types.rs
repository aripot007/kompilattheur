use std::fs::read;

use crate::common::diagnostic::{Diagnostic, DiagnosticGravity};
use crate::common::symbol_table::{
    enter_scope, exit_scope, init_symbol_table, Symbol, SymbolTableElement, SymbolTableRef,
};
use crate::{
    ast::nodes::{self, Factor, Statement},
    typing::{Function, Type, Typeable, TypingContext, Weak},
};

pub fn parse_types(root: nodes::Root) -> (nodes::Root, SymbolTableRef, TypingContext) {
    let table = init_symbol_table();

    let mut context: TypingContext = TypingContext {
        symbol_table: table.clone(),
        warnings: Vec::new(),
        errors: Vec::new(),
        func_id: None,
    };

    let (root, _) = generate_from_node_root(root, table.clone(), &mut context);

    return (root, table, context);
}

fn generate_from_node_root(
    root: nodes::Root,
    table: SymbolTableRef,
    context: &mut TypingContext,
) -> (nodes::Root, SymbolTableRef) {
    let mut table = table;

    // Process all function definitions
    for def in &root.defs.defs {
        table = generate_from_def(def, table, context);
    }

    // Process the main block
    table = generate_from_block(&root.block, table, context);

    (root, table)
}

fn generate_from_def(
    def: &nodes::Def,
    table: SymbolTableRef,
    context: &mut TypingContext,
) -> SymbolTableRef {
    let func_id = def.identifier.element.id;
    context.func_id = Some(def.identifier.element.clone());

    let name = def.identifier.element.name.clone();

    let function_type = Function {
        args: Vec::new(),
        returns: Type::Any,
    };

    let symbol_table_element = SymbolTableElement {
        symbol: Symbol::Function(),
        name,
        symbol_type: Type::Function(Box::from(function_type)),
    };
    table
        .borrow_mut()
        .insert_symbol(func_id, symbol_table_element);

    // Enter a new scope for this function
    let function_table = enter_scope(table.clone());
    context.symbol_table = function_table.clone();

    for param in &def.params {
        let param_id = param.identifier.element.id;
        let param_name = param.identifier.element.name.clone();
        let param_element = SymbolTableElement {
            symbol: Symbol::Parameter(),
            name: param_name,
            symbol_type: Type::Weak(Weak::new()),
        };
        function_table
            .borrow_mut()
            .insert_symbol(param_id, param_element);
    }

    generate_from_block(&def.block, function_table.clone(), context);

    context.symbol_table = table.clone();
    context.func_id = None;
    exit_scope(function_table)
}

fn generate_from_block(
    block: &nodes::Block,
    table: SymbolTableRef,
    context: &mut TypingContext,
) -> SymbolTableRef {
    let mut table = table;

    for statement in &block.statements {
        match statement {
            Statement::Assign(assign) => {
                let value_type: Type = match assign.value.parse_type(context) {
                    Ok(t) => t,
                    Err(()) => continue, // No use in typing the value if the destination cannot be typed
                };

                // If the destination is a single identifier, check or set the type with the value
                if let nodes::Expression::Factor(Factor::Identifier(id)) = &assign.destination {
                    // TODO : check if types are compatible
                    match context.get_symbol_type(&id.element) {
                        Some(_) => (),
                        None => {
                            // Symbol does not exist, insert it
                            context.add_symbol(&id.element, Symbol::Variable(), value_type);
                        }
                    }
                }
            }
            Statement::For(for_loop) => {
                let var_id = for_loop.var.element.id;
                let var_name = for_loop.var.element.name.clone();

                let loop_table = enter_scope(table.clone());
                context.symbol_table = loop_table.clone();

                let var_element = SymbolTableElement {
                    symbol: Symbol::Variable(),
                    name: var_name,
                    symbol_type: Type::Any,
                };
                loop_table.borrow_mut().insert_symbol(var_id, var_element);

                generate_from_block(&for_loop.block, loop_table.clone(), context);

                table = exit_scope(loop_table);
                context.symbol_table = table.clone();
            }
            Statement::Conditional(cond) => {
                let if_table = enter_scope(table.clone());
                context.symbol_table = if_table.clone();

                generate_from_block(&cond.if_block, if_table.clone(), context);

                table = exit_scope(if_table);
                context.symbol_table = table.clone();

                if let Some(else_block) = &cond.else_block {
                    let else_table = enter_scope(table.clone());
                    context.symbol_table = else_table.clone();

                    generate_from_block(else_block, else_table.clone(), context);

                    table = exit_scope(else_table);
                    context.symbol_table = table.clone();
                }
            }
            Statement::Expr(expr) => {
                let _ = expr.parse_type(context);
            }
            Statement::Return(expr) => {
                let symbol_type = match expr.parse_type(context) {
                    Ok(symbol_type) => symbol_type,
                    Err(_) => continue,
                };
                if let Some(func_id) = context.func_id.clone() {
                    context.update_function_return(&func_id, symbol_type);
                } else {
                    context.errors.push(Diagnostic::from_localizable_ref(
                        block,
                        DiagnosticGravity::Error,
                        String::from("SyntaxError"),
                        format!("'return' outside of function"),
                    ));
                }
            }
            _ => {}
        }
    }

    table
}

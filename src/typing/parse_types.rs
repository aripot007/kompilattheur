
use crate::ast::nodes::{ExpressionKind, FactorKind};
use crate::common::diagnostic::Diagnostic;
use crate::common::symbol_table::{enter_scope, exit_scope, init_symbol_table, Symbol, SymbolTableElement, SymbolTableRef};
use crate::{ast::nodes::{self, Factor, Statement}, typing::{Function, Type, Typeable, TypingContext, Weak}};

pub fn parse_types(root: nodes::Root) -> (nodes::Root, SymbolTableRef, TypingContext) {
    let table = init_symbol_table();

    let mut root = root;

    let mut context: TypingContext = TypingContext {
        symbol_table: table.clone(),
        warnings: Vec::new(),
        errors: Vec::new(),
        func_id: None,
    };

    let _ = generate_from_node_root(&mut root, table.clone(), &mut context);

    return (root, table, context);
}

fn generate_from_node_root(root: &mut nodes::Root, table: SymbolTableRef, context: &mut TypingContext) -> SymbolTableRef {
    let mut table = table;

    // Process all function definitions
    for mut def in &mut root.defs.defs {
        table = generate_from_def(&mut def, table, context);
    }

    // Process the main block
    table = generate_from_block(&mut root.block, table, context);
    
    table
}

fn generate_from_def(def: &mut nodes::Def, table: SymbolTableRef, context: &mut TypingContext) -> SymbolTableRef {
    let func_id = def.identifier.element.id;
    context.func_id = Some(def.identifier.element.clone());

    let name = def.identifier.element.name.clone();

    let function_type = Function {
        args: Vec::new(),
        returns: Type::Weak(Weak::new_with_possible(&[])),
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

    generate_from_block(&mut def.block, function_table.clone(), context);

    // TODO(Romain): check return type if empty type Weak None

    context.symbol_table = table.clone();
    context.func_id = None;
    exit_scope(function_table)
}

fn generate_from_block(block: &mut nodes::Block, table: SymbolTableRef, context: &mut TypingContext) -> SymbolTableRef {
    let mut table = table;

    for statement in &mut block.statements {
        match statement {
            Statement::Assign(ref mut assign) => {
                        let value_type: Type = match assign.value.parse_type(context) {
                            Ok(t) => t,
                            Err(()) => continue, // No use in typing the value if the destination cannot be typed
                        };

                        // If the destination is a single identifier, check or set the type with the value
                        if let ExpressionKind::Factor(Factor {factor_type: _, kind: FactorKind::Identifier(id)}) = &assign.destination.kind {
                            // TODO : check if types are compatible
                            match context.get_symbol_type(&id.element) {
                                Some(_) => (),
                                None => {
                                    // Symbol does not exist, insert it
                                    context.add_symbol(&id.element, Symbol::Variable(), value_type);
                                }
                            }
                        }

            },
            Statement::For(ref mut for_loop) => {
                        let var_id = for_loop.var.element.id;
                        let var_name = for_loop.var.element.name.clone();
                
                        let loop_table = enter_scope(table.clone());
                        context.symbol_table = loop_table.clone();
                
                        let var_element = SymbolTableElement {
                            symbol: Symbol::Variable(),
                            name: var_name,
                            symbol_type: Type::Any
                        };
                        loop_table.borrow_mut().insert_symbol(var_id, var_element);

                        let _ = generate_from_block(&mut for_loop.block, loop_table.clone(), context);

                        table = exit_scope(loop_table);
                        context.symbol_table = table.clone();
            },
            Statement::Conditional(ref mut cond) => {
                
                        // Parse condition expression type
                        if let Ok(t) = cond.condition.parse_type(context) {
                            // TODO: Correct comparison with weak
                            if t != Type::Bool {
                                context.errors.push(
                                    Diagnostic::incompatible_type(&cond.condition, &t, &[Type::Bool])
                                );
                                continue;
                            }
                        } else {

                        }
                
                        let if_table = enter_scope(table.clone());
                        context.symbol_table = if_table.clone();
   
                        let _ = generate_from_block(&mut cond.if_block, if_table.clone(), context);

                        table = exit_scope(if_table);
                        context.symbol_table = table.clone();
                
                        if let Some(else_block) = &mut cond.else_block {
                            let else_table = enter_scope(table.clone());
                            context.symbol_table = else_table.clone();

                            let _ = generate_from_block(else_block, else_table.clone(), context);

                            table = exit_scope(else_table);
                            context.symbol_table = table.clone();
                        }
            },
            Statement::Expr(ref mut expr) => {
                let _ = expr.parse_type(context);
            },
            Statement::Print(ref mut expr) => {
                let _ = expr.parse_type(context);
            },
            Statement::Return(ref mut expr) => {
                let _ = expr.parse_type(context);
            },
            Statement::NotImplemented => todo!(),
        }
    }

    table
}

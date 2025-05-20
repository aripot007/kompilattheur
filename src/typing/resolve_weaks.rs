use crate::{
    ast::nodes::{Ast, Def, Expression, ExpressionKind, Factor, FactorKind, Statement},
    common::symbol_table::{get_symbol, SymbolTableRef},
};

use super::Type;

/// Resolve a single type to its final type
fn resolve_type(typ: &Type) -> Type {
    match typ {
        Type::Weak(weak) => {
            let possible = weak.get_possible();
            match possible.len() {
                1 => possible[0].clone(),
                0 => panic!("Unresolvable weak in symbol table"),
                _ => typ.clone(),
            }
        }
        _ => typ.clone(),
    }
}

fn resolve_type_opt(typ: &Option<Type>) -> Option<Type> {
    match &typ {
        Some(t) => Some(resolve_type(t)),
        None => None,
    }
}

pub fn resolve_weaks(node: Ast, symbol_table: &SymbolTableRef) -> Ast {
    match node {
        Ast::Expression(mut expr) => {
            expr.expr_type = resolve_type_opt(&expr.expr_type);
            expr.kind = match expr.kind {
                ExpressionKind::BINOP(e1, op, e2) => {
                    let e1: Expression = resolve_weaks(Ast::Expression(*e1), symbol_table).into();
                    let e2: Expression = resolve_weaks(Ast::Expression(*e2), symbol_table).into();
                    ExpressionKind::BINOP(Box::from(e1), op, Box::from(e2))
                }
                ExpressionKind::UNOP(op, e) => {
                    let e: Expression = resolve_weaks(Ast::Expression(*e), symbol_table).into();
                    ExpressionKind::UNOP(op, Box::from(e))
                }
                ExpressionKind::Factor(f) => {
                    let f: Factor = resolve_weaks(Ast::Factor(f), symbol_table).into();
                    ExpressionKind::Factor(f)
                }
                ExpressionKind::NotImplemented => ExpressionKind::NotImplemented,
            };
            Ast::Expression(expr)
        }
        Ast::Assign(mut assign) => {
            assign.destination =
                resolve_weaks(Ast::Expression(assign.destination), symbol_table).into();
            assign.value = resolve_weaks(Ast::Expression(assign.value), symbol_table).into();
            Ast::Assign(assign)
        }
        Ast::Block(mut block) => {
            // Resolve symbol table
            let block_table = block.symbol_table.clone().unwrap();
            resolve_symbol_table_weaks(&block_table);

            let mut statements: Vec<Statement> = Vec::with_capacity(block.statements.len());
            for statement in block.statements {
                statements.push(resolve_weaks(Ast::Statement(statement), &block_table).into());
            }
            block.statements = statements;
            Ast::Block(block)
        }
        Ast::Conditional(mut cdt) => {
            cdt.condition = resolve_weaks(Ast::Expression(cdt.condition), symbol_table).into();
            cdt.if_block = resolve_weaks(Ast::Block(cdt.if_block), symbol_table).into();
            if let Some(else_block) = cdt.else_block {
                cdt.else_block = Some(resolve_weaks(Ast::Block(else_block), symbol_table).into());
            }
            Ast::Conditional(cdt)
        }
        Ast::Def(mut def) => {
            def.block = resolve_weaks(Ast::Block(def.block), symbol_table).into();
            Ast::Def(def)
        }
        Ast::Defs(mut defs) => {
            let mut new_defs: Vec<Def> = Vec::with_capacity(defs.defs.len());
            for def in defs.defs {
                new_defs.push(resolve_weaks(Ast::Def(def), symbol_table).into());
            }
            defs.defs = new_defs;
            Ast::Defs(defs)
        }
        Ast::Factor(mut factor) => {
            factor.factor_type = resolve_type_opt(&factor.factor_type);
            factor.kind = match factor.kind {
                FactorKind::Identifier(fe) => {
                    if let Some(symbol) = get_symbol(&symbol_table.clone(), &fe.element.id) {
                        factor.factor_type = Some(resolve_type(&symbol.symbol_type));
                    };
                    FactorKind::Identifier(fe)
                }
                FactorKind::Expr(expr) => {
                    let expr: Expression =
                        resolve_weaks(Ast::Expression(*expr), symbol_table).into();
                    FactorKind::Expr(Box::from(expr))
                }
                FactorKind::Call {
                    identifier,
                    args,
                    localization,
                } => {
                    let mut new_args: Vec<Expression> = Vec::with_capacity(args.len());
                    for arg in args {
                        new_args.push(resolve_weaks(Ast::Expression(arg), symbol_table).into());
                    }
                    FactorKind::Call {
                        identifier,
                        args: new_args,
                        localization,
                    }
                }
                k => k,
            };
            Ast::Factor(factor)
        }
        Ast::For(mut for_loop) => {
            for_loop.iterator =
                resolve_weaks(Ast::Expression(for_loop.iterator), symbol_table).into();
            for_loop.block = resolve_weaks(Ast::Block(for_loop.block), symbol_table).into();
            Ast::For(for_loop)
        }
        Ast::Param(_) => node,
        Ast::Root(mut root) => {
            root.block = resolve_weaks(Ast::Block(root.block), symbol_table).into();
            root.defs = resolve_weaks(Ast::Defs(root.defs), symbol_table).into();
            Ast::Root(root)
        }
        Ast::Statement(stmt) => {
            let stmt = match stmt {
                Statement::Print(expr) => {
                    Statement::Print(resolve_weaks(Ast::Expression(expr), symbol_table).into())
                }
                Statement::Println(expr) => {
                    Statement::Println(resolve_weaks(Ast::Expression(expr), symbol_table).into())
                }
                Statement::Expr(expr) => {
                    Statement::Expr(resolve_weaks(Ast::Expression(expr), symbol_table).into())
                }
                Statement::Return(expr) => {
                    Statement::Return(resolve_weaks(Ast::Expression(expr), symbol_table).into())
                }
                Statement::For(for_loop) => {
                    Statement::For(resolve_weaks(Ast::For(for_loop), symbol_table).into())
                }
                Statement::Conditional(conditional) => Statement::Conditional(
                    resolve_weaks(Ast::Conditional(conditional), symbol_table).into(),
                ),
                Statement::Assign(assign) => {
                    Statement::Assign(resolve_weaks(Ast::Assign(assign), symbol_table).into())
                }
                Statement::NotImplemented => Statement::NotImplemented,
            };
            Ast::Statement(stmt)
        }
    }
}

fn resolve_symbol_table_weaks(table_ref: &SymbolTableRef) {
    // Use a block for the mutable borrow
    {
        let mut table_mut_ref = table_ref.borrow_mut();
        let table = table_mut_ref.get_value_ref_mut();

        // Resolve current table
        for symbol in table.table.values_mut() {
            symbol.symbol_type = resolve_type(&symbol.symbol_type);
        }
    }

    // Resolve children
    for child in table_ref.borrow().get_children() {
        resolve_symbol_table_weaks(&child);
    }
}

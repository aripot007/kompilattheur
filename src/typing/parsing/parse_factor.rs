use std::iter::{self, zip};

use crate::{
    ast::nodes::{Factor, FactorKind},
    common::{
        diagnostic::Diagnostic,
        localizable::{localization_info, Localizable, LocalizationInfo},
    },
    typing::{Type, Typeable, TypingContext},
};

impl Typeable for Factor {
    fn parse_type(&mut self, context: &mut TypingContext) -> Result<Type, ()> {
        let localization = localization_info!(self);

        let res = match &mut self.kind {
            FactorKind::Integer(_) => Ok(Type::Int),
            FactorKind::Float(_) => Ok(Type::Float),
            FactorKind::String(_) => Ok(Type::String),
            FactorKind::True(_) | FactorKind::False(_) => Ok(Type::Bool),
            FactorKind::None(_) => Ok(Type::None),
            FactorKind::Identifier(id) => match context.get_symbol_type(&id.element.clone()) {
                Some(t) => Ok(t),
                None => {
                    context
                        .errors
                        .push(Diagnostic::unknown_symbol(&localization, &id.element.name));
                    return Err(());
                }
            },
            FactorKind::List(values) => {
                for ref mut e in values {
                    let _ = e.parse_type(context);
                }
                Ok(Type::List)
            }
            FactorKind::Expr(ref mut expr) => expr.as_mut().parse_type(context),
            FactorKind::Call {
                identifier,
                args,
                localization: _,
            } => {
                let func_type_res = context.get_symbol_type(identifier);

                // Unknown function, we still type the parameters without checking compatibility
                if func_type_res.is_none() {
                    context
                        .errors
                        .push(Diagnostic::unknown_symbol(&localization, &identifier.name));

                    for arg in args {
                        let _ = arg.parse_type(context);
                    }
                    return Err(());
                }

                // Function exists, we check parameters
                let mut err = false;

                let func_type = match &func_type_res {
                    Some(Type::Function(f)) => f,
                    _ => panic!("Function type is not a function : {:?}", func_type_res),
                };

                if func_type.args.len() != args.len() {
                    context.errors.push(Diagnostic::invalid_arg_count(
                        &localization,
                        func_type.args.len(),
                        args.len(),
                    ));
                }

                let expected_args = func_type.args.iter().map(Some).chain(iter::repeat(None));

                // Parse arguments types
                for (expected_opt, arg) in zip(expected_args, args) {
                    let arg_type = match arg.parse_type(context) {
                        Ok(t) => t,
                        Err(_) => {
                            err = true;
                            continue;
                        }
                    };

                    if let Some(expected) = expected_opt {
                        let allowed_types = match expected.clone() {
                            Type::Weak(weak) => weak.get_possible(),
                            t => vec![t],
                        };

                        if !arg_type.is_compatible(expected.clone()) {
                            context.errors.push(Diagnostic::incompatible_type(
                                arg,
                                &arg_type,
                                &allowed_types,
                            ));
                            err = true;
                            continue;
                        }

                        // Merge arg if its a weak
                        if let Type::Weak(weak) = arg_type {
                            weak.restrict(&allowed_types).expect(
                                "Restriction should not fail because compatibility was checked",
                            );
                        }
                    }
                }

                if err {
                    return Err(());
                }

                // self.factor_type = Some(func_type.returns.clone());
                Ok(func_type.returns.clone())
            }
        };
        if let Ok(t) = &res {
            self.set_type(t.clone());
        }
        return res;
    }

    fn get_type(&self) -> &Type {
        self.factor_type.as_ref().unwrap()
    }

    fn set_type(&mut self, t: Type) {
        self.factor_type = Some(t);
    }
}

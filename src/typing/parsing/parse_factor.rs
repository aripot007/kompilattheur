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
            FactorKind::String(_) => Ok(Type::String),
            FactorKind::True(_) | FactorKind::False(_) => Ok(Type::Bool),
            FactorKind::None(_) => Ok(Type::None),
            FactorKind::Identifier(id) => Ok(context.get_type_or_create(&id.element)), // Get or add to tds
            FactorKind::List(_) => Ok(Type::List),
            FactorKind::Expr(ref mut expr) => expr.as_mut().parse_type(context),
            FactorKind::Call {
                identifier,
                args,
                localization: _,
            } => {
                let func_type_res = context.get_symbol_type(identifier);

                let mut err = match func_type_res {
                    Some(_) => false,
                    None => {
                        context
                            .errors
                            .push(Diagnostic::unknown_symbol(&localization, &identifier.name));
                        true
                    }
                };

                // Parse arguments types, do nothing with it yet since function typing is not done
                for arg in args {
                    match arg.parse_type(context) {
                        Ok(_) => (),
                        Err(_) => err = true,
                    }
                }

                if err {
                    return Err(());
                }

                return match func_type_res {
                    Some(Type::Function(func_type)) => Ok((*func_type).returns),
                    _ => Err(()),
                };
            }
        };
        if let Ok(t) = &res {
            self.set_type(t.clone());
        }
        return res;
    }

    fn is_typed(&self) -> bool {
        self.factor_type.is_some()
    }

    fn get_type(&self) -> &Type {
        self.factor_type.as_ref().unwrap()
    }

    fn get_type_opt(&self) -> Option<&Type> {
        match &self.factor_type {
            Some(t) => Some(t),
            None => None,
        }
    }

    fn set_type(&mut self, t: Type) {
        self.factor_type = Some(t);
    }
}

use crate::{ast::nodes::Factor, common::diagnostic::Diagnostic, typing::{Type, Typeable, TypingContext}};


impl Typeable for Factor {
    fn parse_type(&self, context: &mut TypingContext) -> Result<Type, ()> {
        return match self {
            Factor::Integer(_) => Ok(Type::Int),
            Factor::String(_) => Ok(Type::String),
            Factor::True(_)
            | Factor::False(_) => Ok(Type::Bool),
            Factor::None(_) => Ok(Type::None),
            Factor::Identifier(id) => Ok(context.get_type_or_create(&id.element)), // Get or add to tds
            Factor::List(_) => Ok(Type::List),
            Factor::Expr(expr) => expr.as_ref().parse_type(context),
            Factor::Call { identifier, args, localization: _ } => {
                let func_type_res = context.get_symbol_type(identifier, &self);

                let mut err = match func_type_res {
                    Some(_) => false,
                    None => {
                        context.errors.push(Diagnostic::unknown_symbol(&self, &identifier.name));
                        true
                    },
                };

                // Parse arguments types, do nothing with it yet since function typing is not done
                for arg in args {
                    match arg.parse_type(context) {
                        Ok(_) => (),
                        Err(_) => err = true,
                    } 
                }

                if err {
                    return Err(())
                }

                return match func_type_res {
                    Some(Type::Function(func_type)) => Ok((*func_type).returns),
                    _ => Err(()),
                };
            }
        };
    }
}

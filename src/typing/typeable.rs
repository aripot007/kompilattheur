use super::{Type, TypingContext};

pub trait Typeable {
    fn parse_type(&self, context: &mut TypingContext) -> Result<Type, ()>;
}
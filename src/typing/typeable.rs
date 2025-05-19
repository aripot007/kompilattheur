use super::{Type, TypingContext};

pub trait Typeable {
    /// Parse the type of this object
    fn parse_type(&mut self, context: &mut TypingContext) -> Result<Type, ()>;

    /// Return the type or panics if the it wasn't typed before
    fn get_type(&self) -> &Type;

    /// Set the type of this object
    fn set_type(&mut self, t: Type);
}

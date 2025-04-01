use super::{Type, TypingContext};

pub trait Typeable {
    /// Parse the type of this object
    fn parse_type(&mut self, context: &mut TypingContext) -> Result<Type, ()>;

    /// Returns true if the object was typed before
    fn is_typed(&self) -> bool;
    
    /// Return the type or panics if the it wasn't typed before
    fn get_type(&self) -> &Type;

    /// Return an Option containing the type if it was typed, or else None
    fn get_type_opt(&self) -> Option<&Type>;

    /// Set the type of this object
    fn set_type(&mut self, t: Type);
}
use std::{fmt::Display, sync::atomic::{AtomicUsize, Ordering}};

use super::Type;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

/// Denotes a type that is not yet known or cannot be known at compile time
#[derive(Debug, Clone)]
pub struct Weak {
    /// Each weak type has an id, and weak types of the same id denote the same type
    id: usize,
    possible: Vec<Type>,
}

impl Weak {
    pub fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        return Weak {id, possible: vec![Type::None, Type::Bool, Type::Int, Type::String, Type::List]};
    }
    
    pub fn new_with_possible(possible_types: &[Type]) -> Self {
        let mut w = Weak::new();
        w.possible = Vec::from(possible_types);
        return w;
    }
}

impl PartialEq for Weak {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Display for Weak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strs: Vec<String> = self.possible.iter().map(|t| t.to_string()).collect();
        write!(f, "weak{}({})", self.id, strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::Weak;


    #[test]
    fn test_incrementing_id() {
        let w1 = Weak::new();
        let w2 = Weak::new();
        assert!(w1.id < w2.id);
    }

    #[test]
    fn test_equality() {

        let w1 = Weak::new();
        let w2 = Weak::new();

        assert_eq!(w1, w1);
        assert_ne!(w1, w2);

    }

}
use once_cell::sync::Lazy;

use crate::typing::Type;
use std::{
    collections::HashSet,
    fmt::Display,
    sync::{Arc, Mutex},
};

use super::UnionFind;

/// Denotes a type that is not yet known or cannot be known at compile time
#[derive(Debug, Clone, Hash)]
pub struct Weak {
    id: usize,
}

static WEAK_TYPES: Lazy<Arc<Mutex<UnionFind>>> =
    Lazy::new(|| Arc::new(Mutex::new(UnionFind::new())));

impl Weak {
    pub fn new() -> Self {
        return Self::new_with_possible(&[
            Type::None,
            Type::Bool,
            Type::Int,
            Type::String,
            Type::List,
        ]);
    }

    pub fn new_with_possible(possible_types: &[Type]) -> Self {
        let possible = HashSet::from_iter(possible_types.to_owned());
        let id = WEAK_TYPES.lock().unwrap().add(possible);
        return Self { id };
    }

    pub fn get_decalage(&self) -> usize {
        let mut types = WEAK_TYPES.lock().unwrap();
        let possible = types.get_elt(self.id);
        possible
            .iter()
            .map(|t| t.get_decalage())
            .max()
            .expect("Error: Weak type has no decalage computed!")
    }

    pub fn get_possible(&self) -> Vec<Type> {
        let mut types = WEAK_TYPES.lock().unwrap();
        let possible = types.get_elt(self.id);
        possible.iter().cloned().collect()
    }
}

impl PartialEq for Weak {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Weak {}

impl Display for Weak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strs: Vec<String> = self.get_possible().iter().map(|t| t.to_string()).collect();
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

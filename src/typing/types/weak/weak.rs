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

    /// Merge two weak types
    pub fn union(&mut self, other: &Self) {
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id);
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let r_types = weak_types.find_elt(other.id);

        let new_types = l_types.union(r_types).cloned().collect();

        let mut weak_types = WEAK_TYPES.lock().unwrap();
        weak_types.union(self.id, other.id);
        weak_types.set_elt(self.id, new_types);
    }

    /// Add a possible type to a weak
    pub fn add_type(&mut self, typ: Type) {
        match typ {
            Type::Weak(other) => return self.union(&other),
            Type::None | Type::Bool | Type::Int | Type::String | Type::List => {
                let mut weak_types = WEAK_TYPES.lock().unwrap();
                let current_types = weak_types.find_elt_mut(self.id);
                current_types.insert(typ);
            }
            _ => {
                panic!("Cannot add type {} to weak type", typ);
            }
        };
    }

    /// Intersect two weak types
    pub fn intersection(&mut self, other: &Self) {
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id);

        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let r_types = weak_types.find_elt(other.id);

        let new_types = l_types.intersection(r_types).cloned().collect();

        let mut weak_types = WEAK_TYPES.lock().unwrap();
        weak_types.union(self.id, other.id);
        weak_types.set_elt(self.id, new_types);
    }

    /// Intersect restrict possible types for a weak
    pub fn restrict(&mut self, others: &[Type]) {
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id);

        let others: HashSet<Type> = others
            .iter()
            .filter(|t| match t {
                Type::Bool | Type::Int | Type::List | Type::String | Type::None => true,
                _ => panic!("Cannot restrict weak with type {}", t),
            })
            .cloned()
            .collect();
        let new_types = l_types.intersection(&others).cloned().collect();

        let mut weak_types = WEAK_TYPES.lock().unwrap();
        weak_types.set_elt(self.id, new_types);
    }

    pub fn is_compatible(&self, other: Type) -> bool {
        if let Type::Weak(weak) = other {
            return *self == weak;
        };
        let mut types = WEAK_TYPES.lock().unwrap();
        let possible = types.get_elt(self.id);
        return possible.contains(&other);
    }
}

impl PartialEq for Weak {
    fn eq(&self, other: &Self) -> bool {
        let self_id = WEAK_TYPES.lock().unwrap().find(self.id);
        let other_id = WEAK_TYPES.lock().unwrap().find(other.id);
        return self_id == other_id;
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

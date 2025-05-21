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
    locked: bool,
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
            Type::Range,
        ]);
    }

    pub fn new_with_possible(possible_types: &[Type]) -> Self {
        let possible = HashSet::from_iter(possible_types.to_owned());
        let id = WEAK_TYPES.lock().unwrap().add(possible);
        return Self { id, locked: false };
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
    pub fn union(&self, other: &Self) {
        if self.locked || other.locked {
            return;
        }
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id).clone();
        let r_types = weak_types.find_elt(other.id).clone();

        let new_types = l_types.union(&r_types).cloned().collect();
        weak_types.union(self.id, other.id);
        weak_types.set_elt(self.id, new_types);
    }

    /// Add a possible type to a weak
    pub fn add_type(&self, typ: Type) {
        if self.locked {
            return;
        }
        match typ {
            Type::Weak(other) => return self.union(&other),
            Type::None
            | Type::Bool
            | Type::Int
            | Type::String
            | Type::List
            | Type::Range
            | Type::Any => {
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
    pub fn intersection(&self, other: &Self) {
        if self.locked || other.locked {
            return;
        }
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id).clone();
        let r_types = weak_types.find_elt(other.id).clone();

        let new_types = l_types.intersection(&r_types).cloned().collect();

        weak_types.union(self.id, other.id);
        weak_types.set_elt(self.id, new_types);
    }

    /// Intersect restrict possible types for a weak
    pub fn restrict(&self, others: &[Type]) -> Result<Type, ()> {
        if self.locked {
            return Ok(Type::Weak(self.clone()));
        }
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt(self.id).clone();

        let others: HashSet<Type> = others
            .iter()
            .filter(|t| match t {
                Type::Bool | Type::Int | Type::List | Type::String | Type::None | Type::Range => {
                    true
                }
                _ => panic!("Cannot restrict weak with type {}", t),
            })
            .cloned()
            .collect();

        let new_types: HashSet<Type> = l_types.intersection(&others).cloned().collect();

        if new_types.len() == 0 {
            return Err(());
        }

        weak_types.set_elt(self.id, new_types.clone());

        match new_types.len() {
            1 => {
                let vals: Vec<&Type> = new_types.iter().collect();
                Ok(vals[0].clone())
            }
            _ => Ok(Type::Weak(self.clone())),
        }
    }

    /// Remove an allowed type for a weak
    pub fn remove(&self, typ: Type) -> Result<Type, ()> {
        if self.locked {
            return Ok(Type::Weak(self.clone()));
        }
        let mut weak_types = WEAK_TYPES.lock().unwrap();
        let l_types = weak_types.find_elt_mut(self.id);

        l_types.remove(&typ);

        match l_types.len() {
            0 => Err(()),
            1 => {
                let vals: Vec<&Type> = l_types.iter().collect();
                Ok(vals[0].clone())
            }
            _ => Ok(Type::Weak(self.clone())),
        }
    }

    pub fn is_compatible(&self, other: Type) -> bool {
        if let Type::Weak(weak) = other.clone() {
            let same = *self == weak;
            if same {
                return true;
            }

            let mut types = WEAK_TYPES.lock().unwrap();
            let self_possible = types.get_elt(self.id).clone();
            let other_possible = types.get_elt(weak.id).clone();
            let intersection_count = self_possible.intersection(&other_possible).count();
            return intersection_count > 0;
        };
        let mut types = WEAK_TYPES.lock().unwrap();
        let possible = types.get_elt(self.id);
        let compatible = possible.contains(&other);
        return compatible;
    }

    /// Get this weak's id
    pub fn get_id(&self) -> usize {
        return WEAK_TYPES.lock().unwrap().find(self.id);
    }

    /// Returns a locked version of ths weak, that cannot be updated
    pub fn locked(&self) -> Self {
        Weak {
            id: self.id,
            locked: true,
        }
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
        let id = WEAK_TYPES.lock().unwrap().find(self.id);
        let strs: Vec<String> = self.get_possible().iter().map(|t| t.to_string()).collect();
        write!(f, "weak{}({})", id, strs.join(", "))
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

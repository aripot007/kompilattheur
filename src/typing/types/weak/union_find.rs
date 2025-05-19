use std::collections::HashSet;

use crate::typing::Type;

pub struct UnionFind {
    parents: Vec<Option<usize>>,
    elements: Vec<HashSet<Type>>,
    rangs: Vec<usize>,
}

impl UnionFind {
    pub fn new() -> Self {
        UnionFind {
            parents: Vec::new(),
            elements: Vec::new(),
            rangs: Vec::new(),
        }
    }

    /// Adds a new element and returns its id
    pub fn add(&mut self, types: HashSet<Type>) -> usize {
        self.parents.push(None);
        self.elements.push(types);
        self.rangs.push(0);
        return self.elements.len() - 1;
    }

    /// Find an element by id
    pub fn find(&mut self, id: usize) -> usize {
        if let Some(parent_id) = self.parents[id] {
            let real_parent = self.find(parent_id);
            self.parents[id] = Some(real_parent);
            return real_parent;
        } else {
            return id;
        }
    }

    /// Find an element by id
    pub fn find_elt(&mut self, id: usize) -> &HashSet<Type> {
        let id = self.find(id);
        return &self.elements[id];
    }

    /// Find an element by id
    pub fn find_elt_mut(&mut self, id: usize) -> &mut HashSet<Type> {
        let id = self.find(id);
        return &mut self.elements[id];
    }

    pub fn set_elt(&mut self, id: usize, elt: HashSet<Type>) {
        let id = self.find(id);
        self.elements[id] = elt;
    }

    pub fn get_elt(&mut self, id: usize) -> &HashSet<Type> {
        let id = self.find(id);
        return &self.elements[id];
    }

    /// Union two elements
    pub fn union(&mut self, x: usize, y: usize) {
        let x_root = self.find(x);
        let y_root = self.find(y);

        if x_root != y_root {
            if self.rangs[x_root] < self.rangs[y_root] {
                self.parents[x_root] = Some(y_root);
            } else {
                self.parents[y_root] = Some(x_root);
                if self.rangs[x_root] == self.rangs[y_root] {
                    self.rangs[x_root] += 1;
                }
            }
        }
    }
}

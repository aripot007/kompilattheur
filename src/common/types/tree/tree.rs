use std::cell::RefCell;
use std::fmt::Display;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub value: T,
    parent: Option<Weak<RefCell<Node<T>>>>,
    childs: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T: Display + ToString> Node<T> {
    pub fn new(value: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            value,
            parent: None,
            childs: Vec::new(),
        }))
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node<T>>>> {
        self.childs.clone()
    }

    pub fn set_children(
        &mut self,
        parent: &Rc<RefCell<Node<T>>>,
        children: Vec<Rc<RefCell<Node<T>>>>,
    ) {
        for child in children.iter() {
            child.borrow_mut().parent = Some(Rc::downgrade(parent));
        }
        self.childs = children;
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<Node<T>>>> {
        match &self.parent {
            Some(parent) => parent.upgrade(),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn add_child(&mut self, parent: &Rc<RefCell<Node<T>>>, child: Rc<RefCell<Node<T>>>) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        self.childs.push(child);
    }

    pub fn insert_child(
        &mut self,
        parent: &Rc<RefCell<Node<T>>>,
        index: usize,
        child: Rc<RefCell<Node<T>>>,
    ) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        self.childs.insert(index, child);
    }

    #[allow(dead_code)]
    pub fn remove_child(&mut self, n: usize) -> Vec<Rc<RefCell<Node<T>>>> {
        let mut result = Vec::new();
        if n < self.childs.len() {
            result = self.childs.remove(n).borrow().get_children();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn test_generate_mermaid() {
        let root = Node::new("root");
        let child1 = Node::new("child1");
        let child11 = Node::new("child11");
        child1.borrow_mut().add_child(&child1, child11);
        root.borrow_mut().add_child(&root, child1);
        let child2 = Node::new("child2");
        root.borrow_mut().add_child(&root, child2);

        let result = root.borrow().generate_mermaid();
        let expected = concat!(
            "flowchart TD\n",
            "0[\"root\"]\n",
            "1[\"child1\"]\n",
            "0 --> 1\n",
            "2[\"child11\"]\n",
            "1 --> 2\n",
            "3[\"child2\"]\n",
            "0 --> 3\n"
        );

        println!("{}", result);

        assert!(expected == result);
    }

    #[test]
    fn test_remove_child() {
        let root = Node::new("root");
        let child1 = Node::new("child1");
        let child11 = Node::new("child11");
        child1.borrow_mut().add_child(&child1, child11);
        root.borrow_mut().add_child(&root, child1);
        let child2 = Node::new("child2");
        root.borrow_mut().add_child(&root, child2);

        let children = root.borrow_mut().remove_child(0);

        println!("{:?}", children[0].borrow().value);

        assert!(children.len() == 1);
        assert!(children[0].borrow().value == "child11");
    }
}

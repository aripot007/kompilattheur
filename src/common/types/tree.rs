use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub childs: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T: std::fmt::Display> Node<T> {
    pub fn new(value: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            value,
            childs: Vec::new(),
        }))
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node<T>>>> {
        self.childs.clone()
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Node<T>>>) {
        self.childs.push(child);
    }

    pub fn remove_child(&mut self, n: usize) -> Vec<Rc<RefCell<Node<T>>>> {
        let mut result = Vec::new();
        if n < self.childs.len() {
            result = self.childs.remove(n).borrow().get_children();
        }
        result
    }

    pub fn generate_mermaid(&self) -> String {
        let mut result = String::new();
        result.push_str("flowchart TD\n");

        let mut counter: usize = 0;
        result.push_str(&format!("{}[\"{}\"]\n", counter, self.value));

        fn generate_child<T: std::fmt::Display>(node: &Node<T>, counter: &mut usize) -> String {
            let mut result = String::new();
            let nb = *counter;
            for child in node.get_children() {
                let child_borrowed = &*child.borrow();
                *counter += 1;
                result.push_str(&format!("{}[\"{}\"]\n", counter, child.borrow().value));
                result.push_str(&format!("{} --> {}\n", nb, counter));
                result.push_str(&generate_child(child_borrowed, counter));
            }
            result
        }

        result.push_str(&generate_child(self, &mut counter));

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
        child1.borrow_mut().add_child(child11);
        root.borrow_mut().add_child(child1);
        let child2 = Node::new("child2");
        root.borrow_mut().add_child(child2);

        let result = root.borrow().generate_mermaid();
        let expected = concat!(
            "flowchart TD\n",
            "0[root]\n",
            "1[child1]\n",
            "0 --> 1\n",
            "2[child11]\n",
            "1 --> 2\n",
            "3[child2]\n",
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
        child1.borrow_mut().add_child(child11);
        root.borrow_mut().add_child(child1);
        let child2 = Node::new("child2");
        root.borrow_mut().add_child(child2);

        let children = root.borrow_mut().remove_child(0);

        println!("{:?}", children[0].borrow().value);

        assert!(children.len() == 1);
        assert!(children[0].borrow().value == "child11");
    }
}

pub struct Node<T> {
    value: T,
    childs: Option<Vec<Node<T>>>,
}

pub fn new<T>(value: T) -> Node<T> {
    Node {
        value,
        childs: None,
    }
}

impl<T: std::fmt::Display> Node<T> {
    pub fn get_children(&self) -> Option<&Vec<Node<T>>> {
        self.childs.as_ref()
    }

    pub fn add_child(&mut self, child: Node<T>) {
        match &mut self.childs {
            Some(children) => children.push(child),
            None => self.childs = Some(vec![child]),
        }
    }

    pub fn generate_mermaid(&self) -> String {
        let mut result = String::new();
        result.push_str("flowchart TD\n");

        let mut counter: usize = 0;
        result.push_str(&format!("{}[{}]\n", counter, self.value));

        fn generate_child<T: std::fmt::Display>(node: &Node<T>, counter: &mut usize) -> String {
            let mut result = String::new();
            let nb = *counter;
            if let Some(children) = node.get_children() {
                for child in children {
                    *counter += 1;
                    result.push_str(&format!("{}[{}]\n", counter, child.value));
                    result.push_str(&format!("{} --> {}\n", nb, counter));
                    result.push_str(&generate_child(child, counter));
                }
            }
            result
        }

        result.push_str(&generate_child(self, &mut counter));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_generate_mermaid() {
        let mut root = new("root");
        let mut child1 = new("child1");
        let child11 = new("child11");
        child1.add_child(child11);
        root.add_child(child1);
        let child2 = new("child2");
        root.add_child(child2);

        let result = root.generate_mermaid();
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

        print!("{}", result);

        assert!(expected == result);
    }
}

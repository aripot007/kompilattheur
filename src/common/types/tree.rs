struct Node<T> {
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

        result.push_str("flowchart TD;\n");

        let mut counter: usize = 0;
        result.push_str(&format!("{}[{}]\n", counter, self.value));

        fn generate_child<T: std::fmt::Display>(node: &Node<T>, counter: &mut usize) -> String {
            let mut result = String::new();
            let nb = *counter;
            if let Some(children) = node.get_children() {
                for child in children {
                    *counter += 1;
                    result.push_str(&format!("{}[{}]\n", counter, node.value));
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

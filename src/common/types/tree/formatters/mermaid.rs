use std::fmt::Display;

use super::super::Node;

macro_rules! escape_mermaid {
    ($s: expr) => {
        $s.replace("#", "#35;")
            .replace("<", "#lt;")
            .replace(">", "#gt;")
            .replace("!", "#33;")
            .replace("\"", "#quot;")
            .replace("&", "#amp;")
            .replace("(", "#40;")
            .replace(")", "#41;")
            .replace("*", "#42;")
            .replace("+", "#plus;")
            .replace("-", "#minus;")
            .replace("[", "#91;")
            .replace("\\", "#92;")
            .replace("]", "#93;")
            .replace("^", "#94;")
            .replace("_", "#95;")
            .replace("`", "#96;")
            .replace("|", "#124;")
            .replace("~", "#126;")
    };
}

impl<T: Display + ToString> Node<T> {
    pub fn generate_mermaid(&self) -> String {
        let mut result = String::new();
        result.push_str("flowchart TD\n");

        let mut counter: usize = 0;
        result.push_str(&format!(
            "{}[\"{}\"]\n",
            counter,
            escape_mermaid!(self.value.to_string())
        ));

        fn generate_child<T: Display>(node: &Node<T>, counter: &mut usize) -> String {
            let mut result = String::new();
            let nb = *counter;
            for child in node.get_children() {
                let child_borrowed = &*child.borrow();
                *counter += 1;
                result.push_str(&format!(
                    "{}[\"{}\"]\n",
                    counter,
                    escape_mermaid!(child.borrow().value.to_string())
                ));
                result.push_str(&format!("{} --> {}\n", nb, counter));
                result.push_str(&generate_child(child_borrowed, counter));
            }
            result
        }

        result.push_str(&generate_child(self, &mut counter));

        result
    }

    pub fn generate_unsafe_mermaid(&self) -> String {
        let mut result = String::new();
        result.push_str("flowchart TD\n");

        let mut counter: usize = 0;
        result.push_str(&format!("{}[\"{}\"]\n", counter, self.value.to_string()));

        fn generate_child<T: Display>(node: &Node<T>, counter: &mut usize) -> String {
            let mut result = String::new();
            let nb = *counter;
            for child in node.get_children() {
                let child_borrowed = &*child.borrow();
                *counter += 1;
                result.push_str(&format!(
                    "{}[\"{}\"]\n",
                    counter,
                    child.borrow().value.to_string()
                ));
                result.push_str(&format!("{} --> {}\n", nb, counter));
                result.push_str(&generate_child(child_borrowed, counter));
            }
            result
        }

        result.push_str(&generate_child(self, &mut counter));

        result
    }
}

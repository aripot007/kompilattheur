use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub value: T,
    pub childs: Vec<Rc<RefCell<Node<T>>>>,
}

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
    pub fn new(value: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            value,
            childs: Vec::new(),
        }))
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node<T>>>> {
        self.childs.clone()
    }

    pub fn set_children(&mut self, children: Vec<Rc<RefCell<Node<T>>>>) {
        self.childs = children;
    }

    #[allow(dead_code)]
    pub fn add_child(&mut self, child: Rc<RefCell<Node<T>>>) {
        self.childs.push(child);
    }

    pub fn insert_child(&mut self, index: usize, child: Rc<RefCell<Node<T>>>) {
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

    pub fn generate_html(&self) -> String {
        let html_content = format!(
            r#"
    <!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Zoomable & Draggable Mermaid Diagram</title>

    <script type="module">
        import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
        mermaid.initialize({{ startOnLoad: true }});
    </script>

    <style>
        html, body, .mermaid-container {{
            width: 100vw;
            height: 100vh;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            background: #f0f0f0;
            overflow: hidden;
            position: relative;
        }}
        .mermaid {{
            transform-origin: center;
            cursor: grab;
            position: absolute;
        }}
        .mermaid.dragging {{
            cursor: grabbing;
        }}
        .zoom-controls {{
            position: absolute;
            top: 10px;
            right: 10px;
            display: flex;
            gap: 10px;
            z-index: 10; /* Ensure buttons are on top */
        }}
        .zoom-controls button {{
            padding: 8px 12px;
            font-size: 16px;
            cursor: pointer;
        }}
    </style>
</head>

<body>
    <div class="mermaid-container">
        <div class="zoom-controls">
            <button onclick="zoomIn()">Zoom In</button>
            <button onclick="zoomOut()">Zoom Out</button>
            <button onclick="resetZoom()">Reset</button>
        </div>
        <div class="mermaid" id="mermaid">
            {}
        </div>
    </div>

    <script>
        let scale = 1;
        let isDragging = false;
        let startX, startY;
        let translateX = 0;
        let translateY = 0;
        let requestId = null;
        const mermaidDiv = document.getElementById('mermaid');

        function zoomIn() {{
            scale += 0.5;
            updateTransform();
        }}

        function zoomOut() {{
            scale = Math.max(0.5, scale - 0.5); // Prevent scaling below 0.5
            updateTransform();
        }}

        function resetZoom() {{
            scale = 2.5;
            translateX = 0;
            translateY = 0;
            updateTransform();
        }}

        function updateTransform() {{
            mermaidDiv.style.transform = `translate(${{translateX}}px, ${{translateY}}px) scale(${{scale}})`;
        }}

        function onMove(e) {{
            if (isDragging) {{
                translateX = e.clientX - startX;
                translateY = e.clientY - startY;

                if (!requestId) {{
                    requestId = requestAnimationFrame(() => {{
                        updateTransform();
                        requestId = null;
                    }});
                }}
            }}
        }}

        mermaidDiv.addEventListener('mousedown', (e) => {{
            isDragging = true;
            startX = e.clientX - translateX;
            startY = e.clientY - translateY;
            mermaidDiv.classList.add('dragging');
        }});

        document.addEventListener('mousemove', onMove);

        document.addEventListener('mouseup', () => {{
            isDragging = false;
            mermaidDiv.classList.remove('dragging');
            if (requestId) {{
                cancelAnimationFrame(requestId);
                requestId = null;
            }}
        }});
        mermaidDiv.addEventListener('wheel', (e) => {{
            e.preventDefault(); // Prevent page scroll

            // Zoom in or out based on the scroll direction
            if (e.deltaY < 0) {{
                // Scroll up, zoom in
                scale += 0.5
            }} else {{
                // Scroll down, zoom out, ensuring minimum scale
                scale = Math.max(0.1, scale - 0.5);
            }}

            updateTransform();
        }});
    </script>
</body>
</html>
    "#,
            self.generate_mermaid()
        );
        html_content
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

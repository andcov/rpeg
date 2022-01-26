pub struct HuffmanTree {
    nodes: Vec<Node>,
}

type NodeIndex = usize;

impl HuffmanTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn build(&mut self, lengths: &Vec<u8>, vals: &Vec<u8>) {
        self.nodes = Vec::new();

        let root = Node::new(None, None, None, 0);
        self.nodes.push(root);

        self.insert_children(0);

        let mut leftmost_node_index: NodeIndex = 1;

        let mut current_value_index = 0;

        for l in lengths {
            let mut current_node_index = leftmost_node_index;
            for _ in 0..*l {
                self.nodes[current_node_index].code = vals[current_value_index];
                current_value_index += 1;
                current_node_index = self
                    .right_node_level(current_node_index)
                    .expect("[E] - length exceeded current's level space");
            }

            let leftmost_parent_index = current_node_index;

            while let Some(next_node_index) = self.right_node_level(current_node_index) {
                self.insert_children(current_node_index);
                current_node_index = next_node_index;
            }
            self.insert_children(current_node_index);

            leftmost_node_index = self.nodes[leftmost_parent_index].left_child.unwrap();
        }
    }

    pub fn print(&self) {
        let mut stack = Vec::new();

        stack.push((0, String::from("")));

        while let Some((node, node_code)) = stack.pop() {
            if self.nodes[node].is_leaf() {
                println!("{} = {}", self.nodes[node].code, node_code);
            }
            if let Some(left_node) = self.nodes[node].left_child {
                stack.push((left_node, format!("{}0", node_code)));
            }
            if let Some(right_node) = self.nodes[node].right_child {
                stack.push((right_node, format!("{}1", node_code)));
            }
        }
    }

    fn insert_children(&mut self, parent_node: NodeIndex) {
        let left_child = Node::new(Some(parent_node), None, None, 0);
        self.nodes.push(left_child);
        self.nodes[parent_node].left_child = Some(self.nodes.len() - 1);

        let right_child = Node::new(Some(parent_node), None, None, 0);
        self.nodes.push(right_child);
        self.nodes[parent_node].right_child = Some(self.nodes.len() - 1);
    }

    fn right_node_level(&self, node: NodeIndex) -> Option<NodeIndex> {
        let parent = self.nodes[node].parent_index;

        let parent = match parent {
            Some(parent) => parent,
            None => return None,
        };

        let right_sibling = self.nodes[parent].right_child.unwrap();

        if right_sibling == node {
            match self.right_node_level(parent) {
                Some(parent_right_sibling) => {
                    println!("{} {}", node, parent_right_sibling);
                    Some(self.nodes[parent_right_sibling].left_child.unwrap())
                }
                None => None,
            }
        } else {
            Some(right_sibling)
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Node {
    parent_index: Option<NodeIndex>,

    left_child: Option<NodeIndex>,
    right_child: Option<NodeIndex>,
    code: u8,
}

impl Node {
    fn new(
        parent_index: Option<NodeIndex>,
        left_child: Option<NodeIndex>,
        right_child: Option<NodeIndex>,
        code: u8,
    ) -> Self {
        Self {
            parent_index,
            left_child,
            right_child,
            code,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left_child.is_none() && self.right_child.is_none()
    }
}

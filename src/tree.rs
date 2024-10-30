use std::cmp::max;

use crate::file::{Content, File};

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Root(),
    Pattern(String),
}
impl Pattern {
    pub fn to_string(&self) -> String {
        match self {
            Pattern::Root() => String::from("."),
            Pattern::Pattern(p) => p.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryTreeNode {
    pub name: String,
    pub path: String,
    pub children: Vec<DirectoryTreeNode>,
}
impl DirectoryTreeNode {
    pub fn new(f: File) -> Self {
        let mut tree = DirectoryTreeNode {
            name: ".".to_string(),
            path: "".to_string(),
            children: Vec::new(),
        };
        for line in f.content.iter() {
            match &line.content {
                Content::Comment(_) => continue,
                Content::Pattern(p) => {
                    if !p.contains("/") {
                        continue;
                    }
                    let mut current_path = "".to_string();
                    for part in p.split('/') {
                        current_path = format!("{}/{}", current_path, part);
                        let node = DirectoryTreeNode {
                            name: part.to_string(),
                            path: current_path.clone(),
                            children: Vec::new(),
                        };
                        tree.add_node(node);
                    }
                }
            }
        }
        tree
    }
    fn is_parent(&self, node: &DirectoryTreeNode) -> bool {
        self.path == &node.path[..node.path.rfind('/').unwrap_or(node.path.len())]
    }
    fn add_node(&mut self, node: DirectoryTreeNode) {
        if self.children.iter().any(|c| c.name == node.name) {
            return;
        }
        if self.is_parent(&node) {
            self.children.push(node);
        } else {
            for child in self.children.iter_mut() {
                child.add_node(node.clone());
            }
        }
    }
}
impl std::fmt::Display for DirectoryTreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn fmt_node(node: &DirectoryTreeNode, f: &mut std::fmt::Formatter, depth: isize) {
            writeln!(
                f,
                "{:indent$}{}{}",
                "",
                (if depth == 0 { "" } else { "├─" }),
                node.name,
                indent = max(depth - 2, 0) as usize
            )
            .unwrap();
            for child in node.children.iter() {
                fmt_node(child, f, depth + 2);
            }
        }
        fmt_node(self, f, 0);
        Ok(())
    }
}

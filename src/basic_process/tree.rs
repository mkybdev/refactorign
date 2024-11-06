use fs_tree::FsTree;

use crate::basic_process::pattern::{Kind, Pattern};

#[derive(Debug, Clone)]
pub struct DirectoryTree {
    pub root: FsTree,
    pub re_included_nodes: Vec<FsTree>,
}
impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            root: FsTree::new_dir(),
            re_included_nodes: Vec::new(),
        }
    }
    pub fn add(&mut self, pattern: Pattern) {
        match pattern.kind {
            Kind::Global => {
                return;
            }
            Kind::Normal => {
                let mut current = String::new();
                for part in pattern.path.split('/') {
                    current = format!(
                        "{}{}{}",
                        current.clone(),
                        if current.is_empty() { "" } else { "/" },
                        part
                    );
                    if let None = self.root.get(&current) {
                        self.root.insert(current.clone(), FsTree::new_dir());
                    }
                }
            }
            Kind::Negation(k) => {
                if let Kind::Normal = *k {
                    if let Some(node) = self.root.get(pattern.path) {
                        self.re_included_nodes.push(node.clone());
                    }
                }
            }
        }
    }
}

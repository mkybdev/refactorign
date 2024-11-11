use std::{collections::HashMap, path::PathBuf};

use fs_tree::FsTree;

use super::pattern::{Kind, Pattern};

use super::file::Line;

#[derive(Debug, Clone)]
pub struct DirectoryTree {
    pub root: FsTree,
    pub globals: Vec<PathBuf>,
    pub re_included_nodes: Vec<FsTree>,
    pub node_line_map: HashMap<PathBuf, Line>,
}
impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            root: FsTree::new_dir(),
            globals: Vec::new(),
            re_included_nodes: Vec::new(),
            node_line_map: HashMap::new(),
        }
    }
    pub fn add(&mut self, pattern: Pattern, line: Line) {
        match pattern.kind {
            Kind::Global | Kind::Wildcard => {
                self.globals.push(PathBuf::from(pattern.path));
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
                self.node_line_map.insert(PathBuf::from(current), line);
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

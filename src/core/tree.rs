use std::{collections::HashMap, path::PathBuf};

use fs_tree::FsTree;

use super::pattern::{Kind, Pattern};

use super::file::Line;

#[derive(Debug, Clone)]
pub struct DirectoryTree {
    pub root: FsTree,
    pub globals: HashMap<String, Kind>,
    pub re_included: HashMap<String, Kind>,
    pub node_line_map: HashMap<PathBuf, Line>,
}
impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            root: FsTree::new_dir(),
            globals: HashMap::new(),
            re_included: HashMap::new(),
            node_line_map: HashMap::new(),
        }
    }
    pub fn add(&mut self, pattern:  Pattern, line: Line) {
        match pattern.kind {
            Kind::Global | Kind::Wildcard => {
                self.globals.insert(pattern.path, pattern.kind);
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
                if let Kind::Wildcard = *k {
                    panic!("Negation of wildcard is not allowed");
                }
                self.re_included.insert(pattern.path, *k);
            }
        }
    }
}

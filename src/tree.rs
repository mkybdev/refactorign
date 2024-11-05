use fs_tree::FsTree;

use crate::pattern::{Kind, Pattern};

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
            Kind::Negation => {
                if let Some(node) = self.root.get(pattern.path) {
                    self.re_included_nodes.push(node.clone());
                }
            }
        }
    }
}

// use std::cmp::max;

// use crate::file::{Content, File};

// #[derive(Debug, Clone)]
// pub struct DirectoryTreeNode {
//     pub name: String,
//     pub path: String,
//     pub line_number: usize,
//     pub children_lines: usize,
//     pub children: Vec<DirectoryTreeNode>,
// }
// impl DirectoryTreeNode {
//     pub fn new(f: File) -> Self {
//         // f.print();
//         let mut tree = DirectoryTreeNode {
//             name: ".".to_string(),
//             path: "".to_string(),
//             line_number: 0,
//             children_lines: 0,
//             children: Vec::new(),
//         };
//         for line in f.content.iter() {
//             // match &line.content {
//             //     Content::Comment(_) => continue,
//             //     Content::Pattern(p) => match p {
//             //         PatternType::Global(_) => continue,
//             //         PatternType::Normal(path) => {
//             //             let mut current_path = "".to_string();
//             //             for part in path.split('/') {
//             //                 current_path = format!("{}/{}", current_path, part);
//             //                 let node = DirectoryTreeNode {
//             //                     name: part.to_string(),
//             //                     path: current_path.clone(),
//             //                     line_number: line.line_number,
//             //                     children_lines: 0,
//             //                     children: Vec::new(),
//             //                 };
//             //                 tree.add_node(node);
//             //             }
//             //         }
//             //         PatternType::Negation(path) => {
//             //             let node = DirectoryTreeNode {
//             //                 name: path.split('/').last().unwrap().to_string(),
//             //                 path: format!("/{}", path.to_string()),
//             //                 line_number: line.line_number,
//             //                 children_lines: 0,
//             //                 children: Vec::new(),
//             //             };
//             //             tree.remove_node(node);
//             //         }
//             //     },
//             // }
//         }
//         tree
//     }
//     fn is_parent(&self, node: &DirectoryTreeNode) -> bool {
//         self.path == &node.path[..node.path.rfind('/').unwrap_or(node.path.len())]
//     }
//     fn add_node(&mut self, node: DirectoryTreeNode) {
//         // println!("Adding {} to {}", node.path, self.path);
//         if self.children.iter().any(|c| c.path == node.path) {
//             return;
//         }
//         if self.is_parent(&node) {
//             // println!("Added {} to {}", node.path, self.path);
//             self.children.push(node);
//         } else {
//             for child in self.children.iter_mut() {
//                 child.add_node(node.clone());
//             }
//         }
//     }
//     fn remove_node(&mut self, node: DirectoryTreeNode) {
//         println!("Removing {} from {}", node.path, self.path);
//         if self.children.iter().any(|c| c.path == node.path) {
//             println!("Removed {} from {}", node.path, self.path);
//             self.children.retain(|c| c.path != node.path);
//         } else {
//             for child in self.children.iter_mut() {
//                 child.remove_node(node.clone());
//             }
//         }
//     }
// }
// impl std::fmt::Display for DirectoryTreeNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         fn fmt_node(node: &DirectoryTreeNode, f: &mut std::fmt::Formatter, depth: isize) {
//             writeln!(
//                 f,
//                 "{:indent$}{}{} ({})",
//                 "",
//                 (if depth == 0 { "" } else { "├─" }),
//                 node.name,
//                 node.line_number,
//                 indent = max(depth - 2, 0) as usize
//             )
//             .unwrap();
//             for child in node.children.iter() {
//                 fmt_node(child, f, depth + 2);
//             }
//         }
//         fmt_node(self, f, 0);
//         Ok(())
//     }
// }

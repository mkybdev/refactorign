use std::path::Path;

use crate::{
    core::{
        file::{Content, File},
        pattern::Pattern,
        tree::DirectoryTree,
    },
    Refactor,
};

fn remove_dupl(f: &mut File) {
    let mut i = 0;
    while i < f.content.len() {
        let mut j = i + 1;
        while j < f.content.len() {
            if f.get(i).content == f.get(j).content {
                f.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

fn build_tree_from_file(f: &File) -> DirectoryTree {
    let mut tree = DirectoryTree::new();
    for line in f.content.iter() {
        if let Content::Pattern(pat) = &line.content {
            let pattern = Pattern::new(pat.to_string());
            // println!("adding {:?}", pattern.path);
            tree.add(pattern);
        }
    }
    tree
}

impl Refactor {
    pub fn basic_process(&mut self) -> (File, DirectoryTree) {
        remove_dupl(&mut self.state.file);
        (self.state.file.clone(), build_tree_from_file(&self.state.file))
    }
}

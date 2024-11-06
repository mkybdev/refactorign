use std::path::Path;

use crate::basic_process::{
    file::{Content, File},
    pattern::Pattern,
    tree::DirectoryTree,
};

use super::file;

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

pub fn run(path: &Path) -> DirectoryTree {
    let f = &mut file::File::new(path);
    remove_dupl(f);
    build_tree_from_file(f)
}

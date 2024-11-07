use crate::{
    core::{
        file::{Content, File},
        pattern::Pattern,
        tree::DirectoryTree,
    },
    Refactor,
};

fn build_tree_from_file(f: &File) -> DirectoryTree {
    let mut tree = DirectoryTree::new();
    for line in f.content.iter() {
        if let Content::Pattern(pat) = &line.content {
            let pattern = Pattern::new(pat.to_string());
            tree.add(pattern, line.clone());
        }
    }
    tree
}

impl Refactor {
    pub fn basic_process(&mut self) -> &mut Self {
        self.file_mut().remove_dupl();
        let tree = build_tree_from_file(&(self.file()));
        self.state.tree = tree;
        self
    }
}

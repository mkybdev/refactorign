use crate::{tree::DirectoryTree, Refactor};

impl Refactor {
    pub fn basic_process(&mut self) -> &mut Self {
        self.file_mut().remove_dupl();
        let tree = DirectoryTree::build_tree_from_file(&(self.file()));
        self.state.tree = tree;
        self
    }
}

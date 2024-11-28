#[allow(unused_imports)]
use crate::{printv, tree::DirectoryTree, Refactor};

impl Refactor {
    pub fn basic_process(&mut self) -> &mut Self {
        let line_num = self.file().content.len();
        self.file_mut().remove_dupl();
        self.write_report(vec![format!(
            "Lines reduced by basic process (removing duplication): {}",
            line_num - self.file().content.len()
        )]);
        let tree = DirectoryTree::build_tree_from_file(&(self.file()));
        self.state.tree = tree;
        let (_, (verbose, root, tree, _)) = self.get_borrows();
        if verbose {
            printv!(root, tree);
        }
        self
    }
}

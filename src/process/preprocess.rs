#[allow(unused_imports)]
use crate::{printv, tree::DirectoryTree, Refactor};

impl Refactor {
    pub fn preprocess(&mut self) -> &mut Self {
        let tree = DirectoryTree::build_tree_from_file(&(self.file()));
        self.state.tree = tree;
        let (verbose, root, tree, file) = self.get_borrows();
        if verbose == 2 {
            printv!(root, tree, file);
        }
        let line_num = self.file().content.len();
        self.file_mut().remove_dupl();
        self.finish(false, "preprocess", line_num);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{process::test, show_result};
    #[test]
    fn test_preprocess() {
        for level in 1..=1 {
            for path in test::get_input_paths("preprocess") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, 2);
                let result = refactor.preprocess().postprocess();
                show_result!(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }
}

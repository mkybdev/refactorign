#[allow(unused_imports)]
use crate::{printv, tree::DirectoryTree, Refactor};

impl Refactor {
    pub fn preprocess(&mut self) -> &mut Self {
        let line_num = self.file().content.len();
        self.file_mut().remove_dupl();
        self.write_report(vec![format!(
            "Lines reduced by preprocess (removing duplication): {}",
            line_num - self.file().content.len()
        )]);
        let tree = DirectoryTree::build_tree_from_file(&(self.file()));
        self.state.tree = tree;
        let (prev, (verbose, root, tree, file)) = self.get_borrows();
        if verbose {
            printv!(root, tree, file);
        }
        if prev.violate {
            if self.state.lines_diff() > prev.state.unwrap().lines_diff() {
                self.update(false);
            } else {
                self.back();
            }
        } else {
            self.update(false);
        }
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
                let refactor = &mut Refactor::new(&path, level, true);
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

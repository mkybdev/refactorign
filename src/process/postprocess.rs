#[allow(unused_imports)]
use crate::{printv, tree::DirectoryTree, Refactor};

impl Refactor {
    pub fn postprocess(&mut self) -> &mut Self {
        let (verbose, root, tree, file) = self.get_borrows();
        if verbose {
            printv!(root, tree, file);
        }
        let line_num = self.file().content.len();
        self.file_mut().remove_dupl();
        self.write_report(vec![format!(
            "Lines reduced by postprocess process: {}",
            line_num - self.file().content.len()
        )]);
        if let Some(pended) = self.pended() {
            self.state = pended;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{process::test, show_result};
    #[test]
    fn test_postprocess() {
        for level in 1..=1 {
            for path in test::get_input_paths("postprocess") {
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

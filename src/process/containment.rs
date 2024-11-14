use std::path::PathBuf;

use crate::printv;

use super::refactor::Refactor;

impl Refactor {
    pub fn containment(&mut self) -> &mut Self {
        let verbose = self.verbose().clone();
        let root = self.root().clone();
        let tree = self.tree().clone();
        // let file = self.file_mut();
        if verbose {
            printv!(root, tree);
        }
        // directory-structural containment
        for parent in tree.root.paths().skip(1) {
            let children = tree.root.get(&parent).unwrap().children().unwrap();
            // global containment (wildcard / global)
            if self.is_globally_ignored(&parent) {
                if let Some(line) = tree.node_line_map.get(&parent) {
                    let file = self.file_mut();
                    file.remove_line_with_path(PathBuf::from(line.content.unwrap()), verbose);
                }
            }
            // normal containment
            if self.is_normally_ignored(&parent) {
                for child in children.keys() {
                    if let Some(line) = tree.node_line_map.get(&parent.join(child.clone())) {
                        let file = self.file_mut();
                        file.remove_line_with_path(PathBuf::from(line.content.unwrap()), verbose);
                    }
                }
            }
        }
        // update state
        self.update();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::test;
    #[test]
    fn test_containment() {
        for level in 1..=1 {
            for path in test::get_input_paths("containment") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().containment();
                test::show_result(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }
}

use std::{collections::BTreeMap, path::PathBuf};

use crate::pattern::{does_match, ToString};
#[allow(unused_imports)]
use crate::printv;

use super::refactor::Refactor;

impl Refactor {
    pub fn containment(&mut self) -> &mut Self {
        let (verbose, root, tree, file) = self.get_borrows();
        if verbose == 2 {
            printv!(root, tree, file);
        }

        let line_num = file.content.len();
        for node in tree.root.paths().min_depth(1) {
            // global containment (wildcard / global)
            if self.is_globally_ignored(&node) {
                if let Some(line) = tree.node_line_map.get(&node) {
                    let file = self.file_mut();
                    file.remove_line_with_path(PathBuf::from(line.content.unwrap()), verbose);
                }
            }
            // normal containment (directory-structure)
            if self.is_normally_ignored(&node) {
                // let children = tree.root.get(&parent).unwrap().children().unwrap();
                let childrens = tree
                    .root
                    .paths()
                    .filter(|path| does_match(path, &node.to_string()))
                    .map(|path| {
                        (
                            path.clone(),
                            tree.root.get(path).unwrap().children().unwrap().clone(),
                        )
                    })
                    .collect::<Vec<(PathBuf, BTreeMap<_, _>)>>();
                // let map = tree.node_line_map.clone();
                // printv!(node, childrens, map);
                for (parent, children) in childrens {
                    for child in children.keys() {
                        // if let Some(line) =
                        //     map.get(&node.join(child.clone().strip_prefix("/").unwrap_or(child)))
                        // {
                        //     let file = self.file_mut();
                        //     file.remove_line_with_path(
                        //         PathBuf::from(line.content.unwrap()),
                        //         verbose,
                        //     );
                        // }
                        let file = self.file_mut();
                        file.remove_line_with_path(PathBuf::from(parent.join(child)), verbose);
                    }
                }
            }
        }
        self.finish(false, "containment", line_num);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{process::test, show_result};
    #[test]
    fn test_containment() {
        for level in 1..=1 {
            for path in test::get_input_paths("containment") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, 2);
                let result = refactor.preprocess().containment().postprocess();
                show_result!(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }
}

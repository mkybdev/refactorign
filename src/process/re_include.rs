use std::{fs, path::PathBuf};

use fs_tree::FsTree;

use super::refactor::Refactor;
use crate::printv;

fn get_children(
    globals: Vec<PathBuf>,
    root_path: PathBuf,
    parent_path: PathBuf,
    gign_path: PathBuf,
) -> Vec<PathBuf> {
    // get children (except globally ignored ones)
    if let Ok(rd) = fs::read_dir(root_path.join(parent_path)) {
        rd.filter_map(|entry| {
            let path = entry.unwrap().path();
            if !globals.contains(&PathBuf::from(path.file_name().unwrap())) && path != gign_path {
                Some(path.strip_prefix(root_path.clone()).unwrap().to_path_buf())
            } else {
                None
            }
        })
        .collect()
    } else {
        Vec::new()
    }
}

impl Refactor {
    pub fn re_include(&mut self) -> &mut Self {
        let verbose = self.verbose().clone();
        let root = self.root().clone();
        let tree = self.tree().clone();
        let file = self.file_mut();
        if verbose {
            printv!(root, tree);
        }
        // iterate over nodes (parent nodes)
        // parent nodes should not be ignored for re-including children
        for parent_path in FsTree::read_at(&root).unwrap().paths() {
            if let Some(parent) = tree.root.get(parent_path.clone()) {
                // check if parent is not ignored
                if tree.node_line_map.get(&parent_path).is_none() {
                    if let Some(ign_children_map) = parent.children() {
                        // ignored children (should be in node_line_map)
                        let ign_children = ign_children_map
                            .keys()
                            .filter(|&path| {
                                tree.node_line_map.contains_key(&parent_path.join(path))
                            })
                            .map(|key| parent_path.join(key))
                            .collect::<Vec<PathBuf>>();
                        let ign_children_num = ign_children.len();
                        // all children (except globally ignored ones)
                        let children = get_children(
                            tree.globals.clone(),
                            root.clone(),
                            parent_path.clone(),
                            file.path.clone(),
                        );
                        let children_num = children.len();
                        if verbose {
                            printv!(parent_path, ign_children, children);
                        }
                        if ign_children_num > (children_num + 1) / 2 {
                            // remove lines
                            for child_path in ign_children.clone().into_iter() {
                                file.remove_line_with_path(child_path, verbose);
                            }
                            // ignore parent
                            file.add_line(
                                parent_path.join("*").to_str().unwrap().to_string(),
                                verbose,
                            );
                            // re-include child(ren) not ignored
                            for child_path in children {
                                if !ign_children.contains(&child_path) {
                                    let new_line = format!("!{}", child_path.to_str().unwrap());
                                    file.add_line(new_line, verbose);
                                }
                            }
                        }
                    }
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::test;
    #[test]
    fn test_re_include() {
        for path in test::get_input_paths("re_include") {
            for level in 1..=1 {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().re_include();
                test::show_result(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path)
                ));
            }
        }
    }
}

use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::PathBuf,
};

use fs_tree::FsTree;

use super::refactor::Refactor;
use crate::{
    pattern::{expand_range, Kind},
    printv,
};

fn get_children(
    globals: HashMap<String, Kind>,
    re_included: HashMap<String, Kind>,
    root_path: PathBuf,
    parent_path: PathBuf,
    gign_path: PathBuf,
) -> Vec<PathBuf> {
    // get children (except globally ignored ones)
    if let Ok(rd) = fs::read_dir(root_path.join(parent_path.clone())) {
        rd.filter_map(|entry| {
            let path = entry.unwrap().path();
            let path_file = path.file_name().unwrap();
            let find_from_map = |map: HashMap<String, Kind>| -> bool {
                map.iter()
                    .find(|(ref s_raw, ref k)| {
                        expand_range(s_raw.to_string())
                            .into_iter()
                            .find(|s| match k {
                                // if path is global, compare with file name
                                // if normal, compare with parent path + file name
                                // if wildcard, compare with file name without the first character
                                Kind::Global => *s == path_file.to_str().unwrap(),
                                Kind::Normal => *s == parent_path.join(path_file).to_str().unwrap(),
                                Kind::Wildcard => s[1..] == path_file.to_str().unwrap()[1..],
                                _ => panic!("Invalid Kind"),
                            })
                            .is_some()
                    })
                    .is_some()
            };
            if (!find_from_map(globals.clone())
                || (find_from_map(globals.clone()) && find_from_map(re_included.clone())))
                && path != gign_path
            {
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

fn get_ign_children(
    paths: Vec<&PathBuf>,
    node_line_map_keys: Vec<&PathBuf>,
    globals: HashMap<String, Kind>,
    re_included: HashMap<String, Kind>,
    parent_path: PathBuf,
    gign_path: PathBuf,
) -> (BTreeSet<PathBuf>, usize, BTreeSet<PathBuf>, usize) {
    // get ignored children (except globally ignored ones, should be in node_line_map)
    let find_from_map = |map: HashMap<String, Kind>, path_file: &str| -> bool {
        map.iter()
            .find(|(ref s_raw, ref k)| {
                expand_range(s_raw.to_string())
                    .into_iter()
                    .find(|s| match k {
                        // if path is global, compare with file name
                        // if normal, compare with parent path + file name
                        // if wildcard, compare with file name without the first character
                        Kind::Global => *s == path_file,
                        Kind::Normal => *s == parent_path.join(path_file).to_str().unwrap(),
                        Kind::Wildcard => s[1..] == path_file[1..],
                        _ => panic!("Invalid Kind"),
                    })
                    .is_some()
            })
            .is_some()
    };
    let ign_children_lines = paths
        .into_iter()
        .filter_map(|path| {
            let path_file = path.file_name().unwrap().to_str().unwrap();
            if (!find_from_map(globals.clone(), path_file)
                || (find_from_map(globals.clone(), path_file)
                    && find_from_map(re_included.clone(), path_file)))
                && *path != gign_path
            {
                Some(path)
            } else {
                None
            }
        })
        .filter(|path| node_line_map_keys.contains(&&parent_path.join(path)))
        .map(|key| parent_path.join(key))
        .collect::<BTreeSet<PathBuf>>();
    let ign_children = ign_children_lines
        .iter()
        .flat_map(|path| expand_range(path.to_str().unwrap().to_string()))
        .map(|s| PathBuf::from(s))
        .filter_map(|path| {
            let path_file = path.file_name().unwrap().to_str().unwrap();
            if (!find_from_map(globals.clone(), path_file)
                || (find_from_map(globals.clone(), path_file)
                    && find_from_map(re_included.clone(), path_file)))
                && *path != gign_path
            {
                Some(path)
            } else {
                None
            }
        })
        .collect::<BTreeSet<PathBuf>>();
    let ign_children_num = ign_children.len();
    let ign_children_lines_num = ign_children_lines.len();
    (
        ign_children,
        ign_children_num,
        ign_children_lines,
        ign_children_lines_num,
    )
}

impl Refactor {
    pub fn re_include(&mut self) -> &mut Self {
        let (end, params) = self.get_borrows();
        if end {
            return self;
        }
        let (verbose, root, tree, file) = params;
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
                        let (
                            ign_children,
                            ign_children_num,
                            ign_children_lines,
                            ign_children_lines_num,
                        ) = get_ign_children(
                            ign_children_map.keys().collect::<Vec<&PathBuf>>(),
                            tree.node_line_map.keys().collect::<Vec<&PathBuf>>(),
                            tree.globals.clone(),
                            tree.re_included.clone(),
                            parent_path.clone(),
                            file.path.clone(),
                        );
                        // all children (except globally ignored ones)
                        let children = get_children(
                            tree.globals.clone(),
                            tree.re_included.clone(),
                            root.clone(),
                            parent_path.clone(),
                            file.path.clone(),
                        );
                        let children_num = children.len();
                        if verbose {
                            printv!(parent_path, ign_children, ign_children_lines, children);
                        }
                        if 1 + children_num - ign_children_num < ign_children_lines_num {
                            let file = self.file_mut();
                            // remove lines
                            for child_path in ign_children_lines.clone().into_iter() {
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
                            self.halt();
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
    use crate::{process::test, show_result};
    #[test]
    fn test_re_include() {
        for level in 1..=1 {
            for path in test::get_input_paths("re_include") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().re_include();
                show_result!(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }
}

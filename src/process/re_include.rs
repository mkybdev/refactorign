use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::PathBuf,
};

use fs_tree::FsTree;

use super::refactor::Refactor;

use crate::pattern::does_match;
#[allow(unused_imports)]
use crate::{
    pattern::{expand_range, Kind},
    printv,
};

#[allow(unused_variables)]
fn get_children(
    globals: HashMap<String, Kind>,
    re_included: HashMap<String, Kind>,
    root_path: PathBuf,
    parent_path: PathBuf,
    gign_path: PathBuf,
) -> Vec<PathBuf> {
    // get children (except globally ignored ones)
    if let Ok(rd) = fs::read_dir(
        root_path.join(
            parent_path
                .clone()
                .strip_prefix("/")
                .unwrap_or(&parent_path),
        ),
    ) {
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
                                Kind::Normal => {
                                    *s == parent_path
                                        .join(
                                            path_file
                                                .to_str()
                                                .unwrap()
                                                .strip_prefix("/")
                                                .unwrap_or(path_file.to_str().unwrap()),
                                        )
                                        .to_str()
                                        .unwrap()
                                }
                                Kind::Wildcard => {
                                    s.chars().skip(1).collect::<String>()
                                        == path_file
                                            .to_str()
                                            .unwrap()
                                            .chars()
                                            .skip(1)
                                            .collect::<String>()
                                }
                                _ => panic!("Invalid Kind"),
                            })
                            .is_some()
                    })
                    .is_some()
            };
            // if (!find_from_map(globals.clone())
            //     || (find_from_map(globals.clone()) && find_from_map(re_included.clone())))
            //     && path != gign_path
            if !find_from_map(globals.clone())
                || (find_from_map(globals.clone()) && find_from_map(re_included.clone()))
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

#[allow(unused_variables)]
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
                        Kind::Normal => {
                            *s == parent_path
                                .join(path_file.strip_prefix("/").unwrap_or(path_file))
                                .to_str()
                                .unwrap()
                        }
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
            // if (!find_from_map(globals.clone(), path_file)
            //     || (find_from_map(globals.clone(), path_file)
            //         && find_from_map(re_included.clone(), path_file)))
            //     && *path != gign_path
            if !find_from_map(globals.clone(), path_file)
                || (find_from_map(globals.clone(), path_file)
                    && find_from_map(re_included.clone(), path_file))
            {
                Some(path)
            } else {
                None
            }
        })
        .filter(|path| {
            node_line_map_keys.contains(&&parent_path.join(path.strip_prefix("/").unwrap_or(path)))
        })
        .map(|key| parent_path.join(key))
        .collect::<BTreeSet<PathBuf>>();
    let ign_children = ign_children_lines
        .iter()
        .flat_map(|path| expand_range(path.to_str().unwrap().to_string()))
        .map(|s| PathBuf::from(s))
        .filter_map(|path| {
            let path_file = path.file_name().unwrap().to_str().unwrap();
            // if (!find_from_map(globals.clone(), path_file)
            //     || (find_from_map(globals.clone(), path_file)
            //         && find_from_map(re_included.clone(), path_file)))
            //     && *path != gign_path
            if !find_from_map(globals.clone(), path_file)
                || (find_from_map(globals.clone(), path_file)
                    && find_from_map(re_included.clone(), path_file))
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
        let (prev, params) = self.get_borrows();
        // if end {
        //     self.write_report(vec![format!("Lines reduced by re-include process: 0")]);
        //     return self;
        // }
        let (verbose, root, tree, file) = params;
        if verbose {
            printv!(root, tree, file);
        }

        let line_num = file.content.len();
        // iterate over nodes (parent nodes)
        // parent nodes should not be ignored for re-including children
        if let Ok(parent_tree) = FsTree::read_at(&root) {
            for parent_path in parent_tree.paths() {
                if let Some(parent) = tree.root.get(parent_path.clone()) {
                    // check if parent is not ignored
                    if tree.node_line_map.get(&parent_path).is_none() {
                        if let Some(ign_children_map) = parent.children().as_mut() {
                            // println!("{:?}", ign_children_map);
                            // all children (except globally ignored ones)
                            let children = get_children(
                                tree.globals.clone(),
                                tree.re_included.clone(),
                                root.clone(),
                                parent_path.clone(),
                                file.path.clone(),
                            );
                            let children_num = children.len();
                            let retained_children: BTreeMap<_, _> = ign_children_map
                                .iter()
                                .filter(|(path, _)| {
                                    children.iter().any(|child| {
                                        does_match(
                                            child,
                                            &parent_path
                                                .join(path.strip_prefix("/").unwrap_or(path))
                                                .to_str()
                                                .unwrap()
                                                .to_string(),
                                        )
                                    })
                                })
                                .map(|(path, value)| (path.clone(), value.clone()))
                                .collect();
                            *ign_children_map = &retained_children;
                            // println!("{:?}", ign_children_map);
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
                            if verbose {
                                printv!(parent_path, ign_children, ign_children_lines, children);
                            }
                            if 1 + children_num - ign_children_num < ign_children_lines_num {
                                let file = self.file_mut();
                                // remove lines
                                for child_path in ign_children_lines.clone().into_iter() {
                                    // println!("Removing: {:?}", child_path);
                                    file.remove_line_with_path(
                                        if child_path.to_str().unwrap().contains("/") {
                                            child_path
                                        } else {
                                            PathBuf::from(format!(
                                                "/{}",
                                                child_path.to_str().unwrap()
                                            ))
                                        },
                                        verbose,
                                    );
                                }
                                // ignore parent
                                file.add_line(
                                    if parent_path.as_os_str() == "" {
                                        String::from("/*")
                                    } else {
                                        parent_path.join("*").to_str().unwrap().to_string()
                                    },
                                    verbose,
                                );
                                // re-include child(ren) not ignored
                                for child_path in children {
                                    if !ign_children.contains(&child_path) {
                                        let new_line = format!("!{}", child_path.to_str().unwrap());
                                        file.add_line(new_line, verbose);
                                    }
                                }
                                // self.halt();
                            }
                        }
                    }
                }
            }
        } else {
            println!("{:?}", &root);
            eprintln!("Failed to read directory tree. Aborting.");
            std::process::exit(1);
        }
        if prev.violate {
            if self.state.lines_diff() > prev.state.unwrap().lines_diff() {
                self.update(true);
            } else {
                self.back();
            }
        } else {
            self.update(true);
        }
        self.write_report(vec![format!(
            "Lines reduced by re-include process: {}",
            line_num - self.file().content.len()
        )]);
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
                let result = refactor.basic_process().re_include().finish();
                show_result!(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }
}

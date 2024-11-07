use std::{fs, path::PathBuf};

use walkdir::WalkDir;

use super::refactor::Refactor;

fn get_children(globals: Vec<PathBuf>, parent_path: PathBuf) -> Vec<PathBuf> {
    // get children (except globally ignored ones)
    fs::read_dir(parent_path)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if !globals.contains(&path) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

impl Refactor {
    pub fn re_include(&mut self) -> &mut Self {
        let tree = self.tree().clone();
        let file = self.file_mut();
        // iterate over nodes (parent nodes)
        // parent nodes should not be ignored for re-including children
        for parent_path in WalkDir::new(file.path.clone())
            .sort_by_key(|entry| entry.depth())
            .into_iter()
            .filter_entry(|entry| !tree.globals.contains(&entry.path().to_path_buf()))
            .map(|entry| entry.unwrap().path().to_path_buf())
        {
            if let Some(parent) = tree.root.get(parent_path.clone()) {
                // check if parent is not ignored
                if tree.node_line_map.get(parent).is_none() {
                    if let Some(ign_children) = parent.children() {
                        let ign_children_num = ign_children.len(); // number of ignored children
                        let children = get_children(tree.globals.clone(), parent_path.clone());
                        let children_num = children.len(); // number of children (except globally ignored ones)
                        if ign_children_num > (children_num + 1) / 2 {
                            // remove lines
                            for child in ign_children.values() {
                                file.remove_line(
                                    tree.node_line_map.get(child).unwrap().line_number,
                                );
                            }
                            // ignore parent
                            file.add_line(format!("{}/*", parent_path.to_str().unwrap()));
                            // re-include child(ren) not ignored
                            for child in children {
                                if ign_children.keys().find(|&path| path == &child).is_none() {
                                    file.add_line(format!("!{}", child.to_str().unwrap()));
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

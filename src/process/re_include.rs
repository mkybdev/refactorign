use std::{
    fs::{self, ReadDir},
    path::PathBuf,
};

use fs_tree::FsTree;
use walkdir::WalkDir;

use super::refactor::Refactor;

fn get_children(globals: Vec<PathBuf>, parent_path: PathBuf) -> Vec<PathBuf> {
    // get children (except globally ignored ones)
    if let Ok(rd) = fs::read_dir(parent_path) {
        rd.filter_map(|entry| {
            let path = entry.unwrap().path();
            if !globals.contains(&path) {
                Some(path)
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
        // iterate over nodes (parent nodes)
        // parent nodes should not be ignored for re-including children
        for parent_path in FsTree::read_at(&root).unwrap().paths().skip(0) {
            if let Some(parent) = tree.root.get(parent_path.clone()) {
                // check if parent is not ignored
                if tree.node_line_map.get(&parent_path).is_none() {
                    if let Some(ign_children) = parent.children() {
                        let ign_children_num = ign_children.len(); // number of ignored children
                        let children =
                            get_children(tree.globals.clone(), root.clone().join(&parent_path));
                        let children_num = children.len(); // number of children (except globally ignored ones)
                        if ign_children_num > (children_num + 1) / 2 {
                            // remove lines
                            for child_path in
                                ign_children.keys().map(|child| parent_path.join(child))
                            {
                                if verbose {
                                    println!("Removing line: {:?}", child_path);
                                }
                                file.remove_line_with_path(child_path);
                            }
                            // ignore parent
                            file.add_line(parent_path.join("*").to_str().unwrap().to_string());
                            // re-include child(ren) not ignored
                            for child in children {
                                let child_path = child.strip_prefix(&root).unwrap();
                                if ign_children
                                    .keys()
                                    .find(|&path| parent_path.join(path) == child_path)
                                    .is_none()
                                {
                                    file.add_line(format!("!{}", child_path.to_str().unwrap()));
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

use crate::core::{file::File, tree::DirectoryTree};
use std::cell::{Ref, RefCell};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct State {
    verbose: bool,
    file: RefCell<File>,
    pub root: PathBuf,
    pub level: u8,
    pub tree: DirectoryTree,
}

struct StateFileValue<'a> {
    value: Ref<'a, File>,
}

impl StateFileValue<'_> {
    pub fn get(&self) -> &File {
        &self.value
    }
}

impl State {
    pub fn new(path: &Path, level: u8, verbose: bool) -> Self {
        let mut root = path.to_path_buf();
        root.pop();
        State {
            verbose,
            file: RefCell::new(File::new(path.to_path_buf())),
            root,
            level,
            tree: DirectoryTree::new(),
        }
    }
}

#[derive(Clone)]
pub struct Refactor {
    pub state: State,
}
impl Refactor {
    pub fn new(path: &Path, level: u8, verbose: bool) -> Self {
        Refactor {
            state: State::new(path, level, verbose),
        }
    }
    pub fn verbose(&self) -> bool {
        self.state.verbose
    }
    pub fn level(&self) -> u8 {
        self.state.level
    }
    pub fn file(&self) -> File {
        let value = StateFileValue {
            value: self.state.file.borrow(),
        };
        value.get().clone()
    }
    pub fn file_mut(&mut self) -> &mut File {
        self.state.file.get_mut()
    }
    pub fn root(&self) -> &PathBuf {
        &self.state.root
    }
    pub fn tree(&self) -> &DirectoryTree {
        &self.state.tree
    }
    pub fn tree_mut(&mut self) -> &mut DirectoryTree {
        &mut self.state.tree
    }
    pub fn rebuild_tree(&mut self) {
        self.state.tree = DirectoryTree::build_tree_from_file(&self.file());
    }
    pub fn update(&mut self) {
        self.rebuild_tree();
    }
    pub fn is_normally_ignored(&self, path: &Path) -> bool {
        self.tree().node_line_map.get(path).is_some()
    }
    pub fn is_globally_ignored(&self, path: &Path) -> bool {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        self.tree()
            .globals
            .keys()
            .find(|&x| x == file_name || x[1..] == file_name[1..])
            .is_some()
    }
    pub fn is_ignored(&self, path: &Path) -> bool {
        self.is_normally_ignored(path) || self.is_globally_ignored(path)
    }
    pub fn run(path: &Path, level: u8) -> Refactor {
        let refactor = &mut Refactor::new(path, level, false);
        refactor
            .basic_process()
            .re_include()
            .containment()
            .merge()
            .clone()
    }
    pub fn run_verbose(path: &Path, level: u8) -> Refactor {
        let refactor = &mut Refactor::new(path, level, true);
        refactor
            .basic_process()
            .re_include()
            .containment()
            .merge()
            .clone()
    }
}

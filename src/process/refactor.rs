use crate::core::{file::File, tree::DirectoryTree};
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::path::Path;

#[derive(Clone)]
pub struct State {
    file: RefCell<File>,
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

// impl<'b> Deref for StateFileValue<'b> {
//     type Target = File;
//     fn deref(&self) -> &File {
//         &self.value
//     }
// }

impl State {
    pub fn new(path: &Path, level: u8) -> Self {
        State {
            file: RefCell::new(File::new(path.to_path_buf())),
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
    pub fn new(path: &Path, level: u8) -> Self {
        Refactor {
            state: State::new(path, level),
        }
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
    pub fn tree(&self) -> &DirectoryTree {
        &self.state.tree
    }
    pub fn tree_mut(&mut self) -> &mut DirectoryTree {
        &mut self.state.tree
    }
    pub fn run(path: &Path, level: u8) -> Refactor {
        let refactor = &mut Refactor::new(path, level);
        refactor
            .basic_process()
            .re_include()
            .containment()
            .merge()
            .clone()
    }
}

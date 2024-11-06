use crate::core;
use std::path::Path;

pub struct State {
    pub file: core::file::File,
    pub level: u8,
    pub tree: core::tree::DirectoryTree,
}
impl State {
    pub fn new(path: &Path) -> Self {
        State {
            file: core::file::File::new(path.to_path_buf()),
            level: 0,
            tree: core::tree::DirectoryTree::new(),
        }
    }
}

pub struct Refactor {
    pub state: State,
}
impl Refactor {
    pub fn new(path: &Path) -> Self {
        Refactor {
            state: State::new(path),
        }
    }
    pub fn run(path: &Path, level: u8) {
        let refactor = &mut Refactor::new(path);
        let (file, tree) = refactor.basic_process();
        let refactor = Refactor {
            state: State { file, level, tree },
        };
        let result = refactor.re_include().containment().merge();
        result.state.file.print();
    }
}
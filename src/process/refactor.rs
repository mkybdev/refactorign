use crate::core::{file::File, tree::DirectoryTree};
use std::cell::{Ref, RefCell};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct State {
    verbose: bool,
    orig_file: File,
    file: RefCell<File>,
    pub root: PathBuf,
    pub level: u8,
    pub tree: DirectoryTree,
    pub end: bool,
    pub report: Vec<String>,
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
            orig_file: File::new(path.to_path_buf()),
            file: RefCell::new(File::new(path.to_path_buf())),
            root,
            level,
            tree: DirectoryTree::new(),
            end: false,
            report: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
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
    pub fn orig_file(&self) -> &File {
        &self.state.orig_file
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
    pub fn halt(&mut self) {
        self.state.end = true;
    }
    pub fn end(&self) -> bool {
        self.state.end
    }
    pub fn write_report(&mut self, lines: Vec<String>) {
        self.state.report.extend(lines.into_iter());
    }
    pub fn get_borrows(&self) -> (bool, (bool, PathBuf, DirectoryTree, File)) {
        let end = self.end().clone();
        let verbose = self.verbose().clone();
        let root = self.root().clone();
        let tree = self.tree().clone();
        let file = self.file().clone();
        (end, (verbose, root, tree, file))
    }
    pub fn update(&mut self) {
        self.rebuild_tree();
    }
    pub fn is_normally_ignored(&self, path: &Path) -> bool {
        self.tree().node_line_map.get(path).is_some()
    }
    pub fn is_globally_ignored(&self, path: &Path) -> bool {
        // println!("path: {:?}", path);
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
    fn run_inner(path: &Path, level: u8, verbose: bool) -> Refactor {
        let refactor = &mut Refactor::new(path, level, verbose);
        refactor
            .basic_process()
            .containment()
            .re_include()
            .merge()
            .clone()
    }
    pub fn run(path: &Path, level: u8) -> Refactor {
        Self::run_inner(path, level, false)
    }
    pub fn run_verbose(path: &Path, level: u8) -> Refactor {
        Self::run_inner(path, level, true)
    }
    pub fn save(&self, path: PathBuf) {
        let f = fs::File::create(path.clone()).expect(&format!(
            "Failed to save result to: {}",
            path.clone().display()
        ));
        let mut file = std::io::BufWriter::new(f);
        for line in self.file().content.iter() {
            if let Err(_) = writeln!(file, "{}", line.content.unwrap()) {
                fs::remove_file(path.clone()).expect(&format!(
                    "Failed to remove file: {}",
                    path.clone().display()
                ));
                eprintln!("Error occurred when writing to file: {}", path.display());
                std::process::exit(1);
            }
        }
    }
    pub fn save_orig(&self, path: &Path) {
        let f =
            fs::File::create(path).expect(&format!("Failed to save result to: {}", path.display()));
        let mut file = std::io::BufWriter::new(f);
        for line in self.orig_file().content.iter() {
            if let Err(_) = writeln!(file, "{}", line.content.unwrap()) {
                fs::remove_file(path).expect(&format!("Failed to remove file: {}", path.display()));
                eprintln!("Error occurred when writing to file: {}", path.display());
                std::process::exit(1);
            }
        }
    }
    pub fn save_report(&self, path: &Path, result_path: PathBuf) {
        let f =
            fs::File::create(path).expect(&format!("Failed to save result to: {}", path.display()));
        let mut file = std::io::BufWriter::new(f);
        let report_content = [
            "Refactorign Report".to_string(),
            "==================".to_string(),
            format!("Refactoring level: {}", self.level()),
            format!("Original file: {}", self.orig_file().path.display()),
            format!("Refactored file: {}", result_path.display()),
            "==================".to_string(),
            format!("Lines (Original file): {}", self.orig_file().content.len()),
            format!("Lines (Refactored file): {}", self.file().content.len()),
            format!(
                "Reduced lines: {}",
                self.orig_file().content.len() - self.file().content.len()
            ),
            "==================".to_string(),
        ];
        for line in report_content.iter().chain(self.state.report.iter()) {
            if let Err(_) = writeln!(file, "{}", line) {
                fs::remove_file(path).expect(&format!("Failed to remove file: {}", path.display()));
                eprintln!("Error occurred when writing to file: {}", path.display());
                std::process::exit(1);
            }
        }
    }
}

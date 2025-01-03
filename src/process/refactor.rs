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
    pub prev: Option<Box<State>>,
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
            prev: None,
        }
    }
    pub fn lines_diff(&self) -> usize {
        self.orig_file.content.len() - self.file.borrow().content.len()
    }
}

#[derive(Debug, Clone)]
pub struct Refactor {
    pub state: State,
    pub pended: Option<State>,
    report: Vec<String>,
}
impl Refactor {
    pub fn new(path: &Path, level: u8, verbose: bool) -> Self {
        Refactor {
            state: State::new(path, level, verbose),
            pended: None,
            report: Vec::new(),
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
    pub fn pended(&self) -> Option<State> {
        self.pended.clone()
    }
    pub fn skip_report(&mut self) {
        let last = self.report.pop().unwrap();
        self.report.push(format!(
            "{}{}",
            last,
            if last.ends_with(" 0") {
                ""
            } else {
                " (Skipped)"
            }
        ));
    }
    pub fn write_report(&mut self, lines: Vec<String>) {
        self.report.extend(lines.into_iter());
    }
    pub fn get_borrows(&self) -> (bool, PathBuf, DirectoryTree, File) {
        let state = self.state.clone();
        (
            state.verbose,
            state.root,
            state.tree,
            StateFileValue {
                value: self.state.file.borrow(),
            }
            .get()
            .clone(),
        )
    }
    fn update(&mut self, violate: bool) {
        if !violate {
            self.pended = None;
        } else {
            self.pended = Some(self.state.clone());
            if let Some(prev) = self.state.prev.clone() {
                self.state = *prev;
            }
        }
    }
    fn back(&mut self) {
        self.state = self.pended().unwrap();
        self.pended = None;
    }
    pub fn finish(&mut self, violate: bool, process: &str, line_num: usize) {
        if let Some(pended) = self.pended() {
            if self.state.lines_diff() >= pended.lines_diff() {
                if pended.lines_diff() > 0 {
                    self.skip_report();
                }
                self.write_report(vec![format!(
                    "Lines reduced by {} process: {}",
                    process,
                    line_num - self.file().content.len()
                )]);
                self.update(violate);
            } else {
                let dec = line_num - self.file().content.len();
                self.write_report(vec![format!(
                    "Lines reduced by {} process: {}{}",
                    process,
                    dec,
                    if dec > 0 { " (Skipped)" } else { "" }
                )]);
                self.back();
            }
        } else {
            self.write_report(vec![format!(
                "Lines reduced by {} process: {}",
                process,
                line_num - self.file().content.len()
            )]);
            self.update(violate);
        }
        if !violate {
            self.state.prev = Some(Box::new(self.state.clone()));
        }
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
            .find(|&x| x == file_name || (&x[..1] == "*" && x[1..] == file_name[1..]))
            .is_some()
    }
    pub fn is_ignored(&self, path: &Path) -> bool {
        self.is_normally_ignored(path) || self.is_globally_ignored(path)
    }
    fn run_inner(path: &Path, level: u8, verbose: bool) -> Refactor {
        let refactor = &mut Refactor::new(path, level, verbose);
        refactor
            .preprocess()
            .containment()
            .re_include()
            .merge()
            .postprocess()
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
        for line in report_content.iter().chain(self.report.iter()) {
            if let Err(_) = writeln!(file, "{}", line) {
                fs::remove_file(path).expect(&format!("Failed to remove file: {}", path.display()));
                eprintln!("Error occurred when writing to file: {}", path.display());
                std::process::exit(1);
            }
        }
    }
}

use std::{fs, path::PathBuf};

use walkdir::WalkDir;

use crate::file::*;

#[allow(dead_code)]
pub fn get_input_paths(process: &str) -> Vec<PathBuf> {
    WalkDir::new(format!("tests/data/{}/input", process))
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.ends_with("gitignore"))
        .collect()
}

#[allow(dead_code)]
pub fn get_expected_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(
        path.iter()
            .take(5)
            .collect::<PathBuf>()
            .to_str()
            .unwrap()
            .replace("input", "expected"),
    )
}

#[allow(dead_code)]
pub fn file_cmp(result: File, expected: PathBuf) -> bool {
    let expected_content = fs::read_to_string(expected.clone())
        .unwrap()
        .lines()
        .enumerate()
        .map(|(i, l)| Line {
            content: match l.chars().next() {
                Some('#') => Content::Comment(l.to_string()),
                _ => Content::Pattern(l.to_string()),
            },
            line_number: i + 1,
        })
        .collect::<Vec<Line>>();
    result.content == expected_content
}

#[allow(dead_code)]
pub fn show_title(path: &PathBuf, level: u8) {
    println!("\r\n{:?} (level {}):\r\n", path, level);
}

#[allow(dead_code)]
pub fn show_result(file: &File) {
    println!("\r\nResult: --------------------------------------\r\n");
    file.print();
    println!("\r\n----------------------------------------------\r\n");
}
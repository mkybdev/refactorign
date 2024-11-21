use std::{collections::BTreeSet, fs, path::PathBuf};

use walkdir::WalkDir;

use crate::file::*;

#[allow(dead_code)]
pub fn get_input_paths(process: &str) -> BTreeSet<PathBuf> {
    WalkDir::new(format!("tests/data/{}/input", process))
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.ends_with("gitignore"))
        .collect::<BTreeSet<_>>()
}

#[allow(dead_code)]
pub fn get_expected_path(path: &PathBuf, level: u8) -> PathBuf {
    PathBuf::from(
        path.iter()
            .take(4)
            .collect::<PathBuf>()
            .to_str()
            .unwrap()
            .replace("input", "expected"),
    )
    .join(level.to_string())
    .join(path.iter().nth(4).unwrap())
}

#[allow(dead_code)]
pub fn file_cmp(result: File, expected: PathBuf) -> bool {
    let expected_content_raw = fs::read_to_string(expected.clone()).unwrap();
    let mut expected_content = expected_content_raw.lines().collect::<Vec<&str>>();
    let mut result_content = result
        .content
        .iter()
        .map(|l| match &l.content {
            Content::Pattern(p) => p.as_str(),
            Content::Blank() => "",
            Content::Comment(c) => c.as_str(),
        })
        .collect::<Vec<&str>>();
    expected_content.sort();
    result_content.sort();
    expected_content == result_content
}

#[allow(dead_code)]
pub fn show_title(path: &PathBuf, level: u8) {
    println!("\r\n{:?} (level {}):\r\n", path, level);
}

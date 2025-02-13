extern crate refactorign;
// use tokio::process::Command;
use std::process::Command;

use refactorign::{process::test, show_result, Refactor};
use walkdir::WalkDir;

#[test]
fn test_integration() {
    for level in 1..=1 {
        for path in test::get_input_paths("integration") {
            test::show_title(&path, level);
            let result = Refactor::run_verbose(&path, level, 1);
            show_result!(&result.file());
            assert!(test::file_cmp(
                result.file(),
                test::get_expected_path(&path, level)
            ));
        }
    }
}

#[test]
fn test_real() {
    Command::new("rm")
        .arg("-rf")
        .arg("tests/data/real/tmp")
        .status()
        .expect("Failed to remove tmp folder");
    for level in 1..=1 {
        for case in WalkDir::new("tests/data/real")
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.into_path())
            .skip(1)
        {
            Command::new("unzip")
                .arg("-qq")
                .arg(case.clone())
                .arg("-d")
                .arg("tests/data/real/tmp")
                .status()
                .expect("Failed to unzip file");
            if let Some(path) = WalkDir::new("tests/data/real/tmp")
                .into_iter()
                .filter_map(Result::ok)
                .map(|entry| entry.into_path())
                .find(|path| path.file_name().unwrap() == ".gitignore")
            {
                test::show_title(&path, level);
                let result = Refactor::run_verbose(&path, level, 1);
                show_result!(&result.file());
            }
            Command::new("rm")
                .arg("-rf")
                .arg("tests/data/real/tmp")
                .status()
                .expect("Failed to remove tmp folder");
        }
    }
}

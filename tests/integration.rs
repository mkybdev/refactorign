use std::fs;

extern crate refactorign;

#[test]
fn test() {
    for entry in fs::read_dir("tests/data").unwrap() {
        let path = entry.unwrap().path();
        if path.ends_with("gitignore") {
            for level in 1..=3 {
                refactorign::Refactor::run(&path, level);
            }
        }
    }
}

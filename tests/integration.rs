use walkdir::WalkDir;

extern crate refactorign;

#[test]
fn integration_test() {
    for entry in WalkDir::new("tests/data")
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.ends_with("gitignore") {
            for level in 1..=3 {
                let result = refactorign::Refactor::run(&path, level);
                result.file().print();
            }
        }
    }
}

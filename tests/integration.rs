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
            for level in 1..=1 {
                println!("\r\n{:?}: ------------------------\r\n", path);
                let result = refactorign::Refactor::run_verbose(&path, level);
                println!("\r\nResult: ------------------------\r\n");
                result.file().print();
                println!("\r\n------------------------\r\n");
            }
        }
    }
}

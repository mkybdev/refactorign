extern crate refactorign;
use refactorign::{process::test, show_result};

#[test]
fn test_integration() {
    for level in 1..=1 {
        for path in test::get_input_paths("integration") {
            test::show_title(&path, level);
            let refactor = &mut refactorign::Refactor::new(&path, level, true);
            let result = refactor
                .preprocess()
                .containment()
                .re_include()
                .merge()
                .postprocess();
            show_result!(&result.file());
            assert!(test::file_cmp(
                result.file(),
                test::get_expected_path(&path, level)
            ));
        }
    }
}

peg::parser! {
    grammar pattern_parser() for str {
        pub rule pattern() = negation() / path()

        pub rule negation() = "!" path()

        pub rule path() = global() / normal()

        pub rule global() = wildcard() / string()!"/"

        pub rule normal() = ((string() "/")+ / ("/" (string() "/")*)) string()?

        pub rule wildcard() = "*" string()

        #[cache_left_rec]
        pub rule string() = (range_notation() / char()) string()?

        pub rule range_notation() = "[" char()+ "]"

        // #[cache_left_rec]
        // pub rule ranges() = (range() / char()+) ranges()?

        // pub rule range() = char() "-" char()

        pub rule char() = [^ ('#'|'!'|'/'|'*'|'['|']'|'^'|'$'|'+'|'|'|'('|')'|'\\'|'?')]
    }
}

pub fn check(l: &str) -> bool {
    pattern_parser::pattern(l).is_ok()
}

#[test]
fn test() {
    let ok_pat = [
        "a",
        "*.txt",
        "[a-z]",
        "[abc]",
        "[a-zABC]",
        "*.py[cod]",
        "/",
        "a/",
        "/a",
        "a/b",
        "a/b/c",
        "a[1-3]/b",
        "a[1-3A-C]/b/c",
        "!a",
        "!*.txt",
        "![a-z]",
        "![abc]",
        "![a-zABC]",
        "!*.py[cod]",
        "!a/b",
        "!a/b/c",
        "!a[1-3]/b",
        "!a[1-3A-C]/b/c",
    ];
    for p in ok_pat.iter() {
        assert!(check(p), "{}", p);
    }
    let ng_pat = ["", "*", "**", "*a/b", "a/*", "[a-z", "a//", "!!a"];
    for p in ng_pat.iter() {
        assert!(!check(p), "{}", p);
    }
}

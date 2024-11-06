use crate::basic_process::pattern::Kind;

peg::parser! {
    grammar pattern_parser() for str {
        pub rule pattern() -> Kind = negation() / path()

        pub rule negation() -> Kind = "!" p:(path()) { Kind::Negation(Box::new(p)) }

        pub rule path() -> Kind = global() / normal()

        pub rule global() -> Kind = (wildcard() / string()!"/") "/"? { Kind::Global }

        pub rule normal() -> Kind = ((string() "/")+ / ("/" (string() "/")*)) (string() / "*")? { Kind::Normal }

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

pub fn parse(l: &str) -> Option<Kind> {
    match pattern_parser::pattern(l) {
        Ok(k) => Some(k),
        Err(_) => None,
    }
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
        "!a/b/c/*",
        "a/*",
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
        assert!(parse(p).is_some(), "{}", p);
    }
    let ng_pat = ["", "*", "**", "*a/b", "[a-z", "a//", "!!a", "a/b.*"];
    for p in ng_pat.iter() {
        assert!(parse(p).is_none(), "{}", p);
    }
}

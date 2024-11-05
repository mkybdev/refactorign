#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub kind: Kind,
    pub path: String, // leading '/', '!', or both are stripped
}
impl Pattern {
    pub fn new(l: String) -> Self {
        if pattern_valid_check(&l) {
            if &l[..1] == "!" {
                Self {
                    kind: Kind::Negation,
                    path: remove_leading_slash(&l[1..]),
                }
            } else if !l.contains("/") {
                Self {
                    kind: Kind::Global,
                    path: l,
                }
            } else {
                Self {
                    kind: Kind::Normal,
                    path: remove_leading_slash(&l),
                }
            }
        } else {
            panic!("Invalid pattern: {}", l);
        }
    }
}

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

fn pattern_valid_check(l: &str) -> bool {
    pattern_parser::pattern(l).is_ok()
}

fn remove_leading_slash(l: &str) -> String {
    if &l[..1] == "/" {
        l[1..].to_string()
    } else {
        l.to_string()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Negation,
    Global,
    Normal,
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
        assert!(pattern_valid_check(p), "{}", p);
    }
    let ng_pat = ["", "*", "**", "*a/b", "a/*", "[a-z", "a//", "!!a"];
    for p in ng_pat.iter() {
        assert!(!pattern_valid_check(p), "{}", p);
    }
}

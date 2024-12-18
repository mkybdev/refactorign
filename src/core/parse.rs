use super::pattern::Kind;

peg::parser! {
    grammar pattern_parser() for str {
        pub rule pattern() -> Kind = negation() / path()

        // pub rule negation() -> Kind = "!" p:(path()) { Kind::Negation(Box::new(p)) }
        pub rule negation() -> Kind = "!" k:(global() / normal()) { Kind::Negation(Box::new(k)) }

        // pub rule path() -> Kind = global() / normal()
        pub rule path() -> Kind = special() / normal()

        pub rule special() -> Kind = wildcard() / global()

        pub rule wildcard() -> Kind = "*" string() { Kind::Wildcard }

        // pub rule global() -> Kind = (wildcard() / string()!"/") "/"? { Kind::Global }
        pub rule global() -> Kind = string()!"/" { Kind::Global }

        // pub rule normal() -> Kind = ((string() "/")+ / ("/" (string() "/")*)) (string() / "*")? { Kind::Normal }
        pub rule normal() -> Kind = "/"? (string() ** "/") { Kind::Normal }

        #[cache_left_rec]
        pub rule string() = (range_notation() / char()) string()?

        pub rule range_notation() = "[" ranges()+ "]"

        #[cache_left_rec]
        pub rule ranges() = (range() / char()+) ranges()?

        pub rule range() = char() "-" char()

        pub rule char() = [^ ('#'|'!'|'/'|'*'|'['|']'|'^'|'$'|'+'|'|'|'('|')'|'\\'|'?')]
    }
}

#[allow(unused_variables)]
pub fn parse(l: &str) -> Option<Kind> {
    let stripped = l.strip_suffix("/").unwrap_or(l);
    if stripped.is_empty() {
        return Some(Kind::Normal);
    }
    // if stripped.contains("..") {
    //     return None;
    // }
    match pattern_parser::pattern(stripped) {
        Ok(k) => Some(k),
        Err(e) => {
            // eprintln!("Parse Error: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let ok_pat = [
            ("a", Kind::Global),
            ("*.txt", Kind::Wildcard),
            ("[a-z]", Kind::Global),
            ("[abc]", Kind::Global),
            ("[a-zABC]", Kind::Global),
            ("*.py[cod]", Kind::Wildcard),
            ("/", Kind::Normal),
            ("a/", Kind::Global),
            ("/a", Kind::Normal),
            ("a/b", Kind::Normal),
            ("a/b/c", Kind::Normal),
            ("a[1-3]/b", Kind::Normal),
            ("a[1-3A-C]/b/c", Kind::Normal),
            ("!a", Kind::Negation(Box::new(Kind::Global))),
            ("![a-z]", Kind::Negation(Box::new(Kind::Global))),
            ("![abc]", Kind::Negation(Box::new(Kind::Global))),
            ("![a-zABC]", Kind::Negation(Box::new(Kind::Global))),
            ("!a/b", Kind::Negation(Box::new(Kind::Normal))),
            ("!a/b/c", Kind::Negation(Box::new(Kind::Normal))),
            ("!a[1-3]/b", Kind::Negation(Box::new(Kind::Normal))),
            ("!a[1-3A-C]/b/c", Kind::Negation(Box::new(Kind::Normal))),
            ("", Kind::Normal),
            ("..", Kind::Global),
            ("a/../b", Kind::Normal),
        ];
        for (p, k) in ok_pat.into_iter() {
            assert_eq!(parse(p), Some(k), "Failed: {:?}", p);
        }
        let ng_pat = [
            "*",
            "**",
            "*a/b",
            "[a-z",
            "a//",
            "!!a",
            "a/b.*",
            "!*.txt",
            "!a/b/c/*",
            "a/*",
            "!*.py[cod]",
        ];
        for p in ng_pat.iter() {
            assert!(parse(p).is_none(), "Failed: {:?}", p);
        }
    }
}

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
        pub rule global() -> Kind = string()!"/" "/"? { Kind::Global }

        // pub rule normal() -> Kind = ((string() "/")+ / ("/" (string() "/")*)) (string() / "*")? { Kind::Normal }
        pub rule normal() -> Kind = ((string() "/")+ / ("/" (string() "/")*)) string()? { Kind::Normal }

        #[cache_left_rec]
        pub rule string() = (range_notation() / char()) string()?

        // pub rule range_notation() = "[" char()+ "]"
        pub rule range_notation() = "[" ranges()+ "]"

        #[cache_left_rec]
        pub rule ranges() = (range() / char()+) ranges()?

        pub rule range() = char() "-" char()

        pub rule char() = [^ ('#'|'!'|'/'|'*'|'['|']'|'^'|'$'|'+'|'|'|'('|')'|'\\'|'?')]
    }
}

pub fn parse(l: &str) -> Option<Kind> {
    match pattern_parser::pattern(l) {
        Ok(k) => Some(k),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
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
            // "!*.txt",
            // "!a/b/c/*",
            // "a/*",
            "![a-z]",
            "![abc]",
            "![a-zABC]",
            // "!*.py[cod]",
            "!a/b",
            "!a/b/c",
            "!a[1-3]/b",
            "!a[1-3A-C]/b/c",
        ];
        for p in ok_pat.iter() {
            assert!(parse(p).is_some(), "{}", p);
        }
        let ng_pat = [
            "",
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
            assert!(parse(p).is_none(), "{}", p);
        }
    }
}

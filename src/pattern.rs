use crate::pattern_check::check;

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub kind: Kind,
    pub path: String, // leading '/', '!', or both are stripped
}
impl Pattern {
    pub fn new(l: String) -> Self {
        if check(&l) {
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

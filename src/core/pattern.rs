use crate::core::parse;

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub kind: Kind,
    pub path: String, // leading '/', '!', or both + trailing "/" are stripped
}
impl Pattern {
    pub fn new(l: String) -> Self {
        match parse::parse(&l) {
            Some(k) => Self {
                kind: k,
                path: remove_slash(&l),
            },
            None => panic!("Invalid pattern: {}", l),
        }
    }
}

fn remove_slash(l: &str) -> String {
    let mut l = l.to_string();
    if l.starts_with('!') {
        l.remove(0);
    }
    if l.starts_with('/') {
        l.remove(0);
    }
    if l.ends_with('/') {
        l.pop();
    }
    l
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Negation(Box<Kind>),
    Global,
    Normal,
}

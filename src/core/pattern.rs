use std::{collections::HashSet, path::PathBuf};

use super::parse;

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
    Wildcard,
    Global,
    Normal,
}

pub fn expand_range(pat: String) -> Vec<String> {
    if !(pat.find('[').is_some() && pat.find(']').is_some()) {
        return vec![pat];
    }

    fn cat(l: &str, rg: &str, r: &str) -> HashSet<String> {
        let mut res = HashSet::new();
        let mut chars = Vec::new();
        let mut rg_chars: Vec<char> = rg.chars().collect();
        let mut i = 0;

        while i < rg_chars.len() {
            if rg_chars[i] == '-' {
                let start = rg_chars[i - 1] as u8;
                let end = rg_chars[i + 1] as u8;
                chars.extend((start..=end).map(|x| x as char));
                rg_chars.drain(i - 1..=i + 1);
                if i < 2 {
                    break;
                }
                i -= 2;
            }
            i += 1;
        }
        chars.extend(rg_chars);
        let chars_set: HashSet<_> = chars.into_iter().collect();

        for c in chars_set {
            res.insert(format!("{}{}{}", l, c, r));
        }
        res
    }

    let mut res = cat(
        &pat[..pat.find('[').unwrap()],
        &pat[pat.find('[').unwrap() + 1..pat.find(']').unwrap()],
        &pat[pat.find(']').unwrap() + 1..],
    );

    loop {
        let mut tmp = HashSet::new();
        for p in res.clone() {
            if let (Some(start), Some(end)) = (p.find('['), p.find(']')) {
                res.remove(&p);
                let expanded = cat(&p[..start], &p[start + 1..end], &p[end + 1..]);
                tmp.extend(expanded);
            }
        }
        if tmp.is_empty() {
            break;
        }
        res.extend(tmp);
    }
    res.into_iter().collect()
}

pub fn does_match(path: &PathBuf, pat: &String) -> bool {
    let mut path_it = path.to_str().unwrap().chars();
    let mut pat_it = pat.chars();
    'outer: loop {
        match (path_it.next(), pat_it.next()) {
            (Some(_), Some('*')) => {
                if let Some(pat_next) = pat_it.clone().next() {
                    while let Some(c) = path_it.next() {
                        if c == pat_next {
                            pat_it.next();
                            continue 'outer;
                        }
                    }
                    return false;
                } else {
                    return true;
                }
            }
            (Some(p), Some('[')) => {
                let mut rg = "[".to_string();
                while let Some(c) = pat_it.next() {
                    rg.push(c);
                    if c == ']' {
                        break;
                    }
                }
                let expanded = expand_range(rg);
                if expanded.iter().any(|e| e.starts_with(p)) {
                    continue 'outer;
                } else {
                    return false;
                }
            }
            (Some(p), Some(p2)) => {
                if p != p2 {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_expand_range() {
        let cases = vec![
            ("[123a-d]", vec!["1", "2", "3", "a", "b", "c", "d"]),
            ("a[abc]d", vec!["abd", "acd", "aad"]),
            ("a[abc-ef]d", vec!["aad", "abd", "acd", "add", "aed", "afd"]),
            (
                "a[abc-ef][123]",
                vec![
                    "aa1", "aa2", "aa3", "ab1", "ab2", "ab3", "ac1", "ac2", "ac3", "ad1", "ad2",
                    "ad3", "ae1", "ae2", "ae3", "af1", "af2", "af3",
                ],
            ),
        ];
        for (pat, mut expected) in cases {
            let mut expanded = expand_range(pat.to_string());
            expanded.sort();
            expected.sort();
            assert_eq!(expanded, expected);
        }
    }

    #[test]
    fn test_does_match() {
        let cases = vec![
            ("a/b", "a/b", true),
            ("a/*.txt", "a/b.txt", true),
            ("a/*.txt", "a/abc.txt", true),
            ("a/*.txt", "a/.txt", false),
            ("a/[a-d].txt", "a/b.txt", true),
            ("a/[1-3a-d].txt", "a/e.txt", false),
            ("a/*.py[cod]", "a/test.pyd", true),
            ("a/*.py[cod]", "a/test.pyw", false),
        ];
        for (pat, path, expected) in cases {
            assert_eq!(does_match(&PathBuf::from(path), &pat.to_string()), expected);
        }
    }
}

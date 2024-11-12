use std::collections::HashSet;

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_expand_range() {
        let mut expanded = expand_range("a[abc-ef][123]".to_string());
        expanded.sort();
        assert_eq!(
            expanded,
            vec![
                "aa1", "aa2", "aa3", "ab1", "ab2", "ab3", "ac1", "ac2", "ac3", "ad1", "ad2", "ad3",
                "ae1", "ae2", "ae3", "af1", "af2", "af3"
            ]
        );
    }
}

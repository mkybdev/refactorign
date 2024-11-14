use std::path::{Path, PathBuf};

use crate::{file::Content, printv};

use super::refactor::Refactor;
use fs_tree::FsTree;
use itertools::Itertools;

fn one_char_diff(a: &str, b: &str, level: u8) -> Option<Vec<usize>> {
    let mut diff = 0;
    let mut index = Vec::new();
    for (i, (c_a, c_b)) in a.chars().zip(b.chars()).enumerate() {
        if c_a != c_b {
            diff += 1;
            index.push(i);
        }
        if diff
            > (if level < 3 {
                level as usize
            } else {
                a.chars().count()
            })
        {
            return None;
        }
    }
    if diff == 0 {
        return None;
    }
    Some(index)
}

fn to_range(chars: Vec<char>) -> String {
    let mut range = String::new();
    let mut start = chars[0];
    let mut end = chars[0];
    for c in chars.iter().skip(1) {
        if *c as u8 == end as u8 + 1 {
            end = *c;
        } else {
            if start == end {
                range.push(start);
            } else {
                range.push(start);
                range.push('-');
                range.push(end);
            }
            start = *c;
            end = *c;
        }
    }
    if start == end {
        range.push(start);
    } else {
        range.push(start);
        range.push('-');
        range.push(end);
    }
    range
}

impl Refactor {
    pub fn merge(&mut self) -> &mut Self {
        // iterate over all of the sets of lines, from largest to smallest
        let verbose = self.verbose().clone();
        let root = self.root().clone();
        let tree = self.tree().clone();
        let level = self.level().clone();
        let file = self.file();
        let mut size = file.content.len();
        'outer: loop {
            for set in file
                .content
                .iter()
                .filter(|line| matches!(line.content, Content::Pattern(_)))
                .map(|line| line.content.unwrap().to_string())
                .filter(|line_str| self.is_normally_ignored(Path::new(line_str)))
                .combinations(size)
            {
                // level 1: merge lines with one character difference (only once)
                // level 2: merge lines with one character difference (up to twice)
                // level 3: merge lines with one character difference (as many as possible)
                let mut diff_chars_index = set
                    .iter()
                    .combinations(2)
                    .map(|x| one_char_diff(x[0], x[1], level));
                if diff_chars_index.clone().all(|x| x.is_some())
                    && diff_chars_index.clone().all_equal()
                {
                    let indices = diff_chars_index.next().unwrap().unwrap();
                    if verbose {
                        printv!(indices);
                    }
                    let mut lines = set.clone();
                    let mut offset = 0;
                    for (step, index) in indices.iter().enumerate() {
                        // replace each line in the set with a new line with wildcard / range notation
                        let diff_chars = lines
                            .iter()
                            .map(|line_str| line_str.chars().nth(index + offset).unwrap())
                            .collect::<Vec<char>>();
                        let old_line = PathBuf::from(lines[0].clone());
                        let parent = old_line.parent().unwrap();
                        let mut new_line: String = String::new();
                        let parent_tree = FsTree::read_at(&root.join(parent)).unwrap();
                        let not_ignored_children = parent_tree
                            .children()
                            .unwrap()
                            .keys()
                            .filter(|path| !self.is_ignored(&parent.join(path)));
                        let can_wildcard = not_ignored_children.clone().count() == 0
                            || not_ignored_children
                                .map(|path| {
                                    one_char_diff(
                                        &set[0],
                                        parent.join(path).to_str().unwrap(),
                                        if step < 3 { (step + 1) as u8 } else { 3 },
                                    )
                                })
                                .any(|x| x.is_none());
                        for (i, line) in lines.clone().iter().enumerate() {
                            new_line = if can_wildcard {
                                // merge with wildcard
                                format!(
                                    "{}*{}",
                                    line.chars().take(index + offset).collect::<String>(),
                                    line.chars().skip(index + 1 + offset).collect::<String>()
                                )
                            } else {
                                // merge with range notation
                                format!(
                                    "{}[{}]{}",
                                    line.chars().take(index + offset).collect::<String>(),
                                    to_range(diff_chars.clone()),
                                    line.chars().skip(index + 1 + offset).collect::<String>()
                                )
                            };
                            let file = self.file_mut();
                            file.replace_line(line.to_string(), new_line.to_string(), verbose);
                            lines[i] = new_line.clone();
                        }
                        offset += new_line.len() - old_line.to_str().unwrap().len();
                    }
                    let file = self.file_mut();
                    file.remove_dupl();
                    break 'outer;
                }
            }
            size -= 1;
            if size == 1 {
                break;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::test;
    #[test]
    fn test_merge() {
        for level in 1..=3 {
            for path in test::get_input_paths("merge") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().merge();
                test::show_result(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }

    #[test]
    fn test_to_range() {
        assert_eq!(to_range(vec!['a', 'b', 'c', 'd']), "a-d".to_string());
        assert_eq!(to_range(vec!['a', 'b', 'c', 'e']), "a-ce".to_string());
        assert_eq!(to_range(vec!['a', 'b', 'd', 'e']), "a-bd-e".to_string());
        assert_eq!(to_range(vec!['a', 'c', 'd', 'e']), "ac-e".to_string());
        assert_eq!(to_range(vec!['a', 'b', 'c']), "a-c".to_string());
        assert_eq!(to_range(vec!['a', 'b']), "a-b".to_string());
        assert_eq!(to_range(vec!['a']), "a".to_string());
    }
}

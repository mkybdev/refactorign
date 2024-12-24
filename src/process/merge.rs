use std::{iter, ops::Range, path::PathBuf};

#[allow(unused_imports)]
use crate::{file::Content, pattern::does_match, printv};

use super::refactor::Refactor;
use fs_tree::FsTree;
use itertools::Itertools;
use regex::Regex;

// get wildcard-able line diff
// consider only the last part of the line
// prefix or suffix only
// return ([prefix ranges], [suffix ranges])
fn line_diff_string(
    set_raw: Vec<String>,
) -> Option<(Vec<Option<Range<usize>>>, Vec<Option<Range<usize>>>)> {
    let set = set_raw
        .iter()
        .map(|line| line.split('/').last().unwrap().chars());

    let shortest_list = set.clone().min_by_key(|list| list.clone().count()).unwrap();

    let mut longest_subsequence = vec![];

    for start in 0..shortest_list.clone().count() {
        for end in start + 1..=shortest_list.clone().count() {
            let candidate = &shortest_list.clone().collect::<Vec<char>>()[start..end];
            if set.clone().all(|chars| {
                chars
                    .collect::<Vec<char>>()
                    .windows(candidate.len())
                    .position(|window| window == candidate)
                    .map(|start| (start, start + candidate.len() - 1))
                    .is_some()
            }) {
                if candidate.len() > longest_subsequence.len() {
                    longest_subsequence = candidate.to_vec();
                }
            }
        }
    }

    if longest_subsequence.len() == 0
        || !(longest_subsequence.starts_with(&['.']) || longest_subsequence.ends_with(&['.']))
    {
        return None;
    }

    let result_indices = set
        .clone()
        .map(|chars| {
            chars
                .collect::<Vec<char>>()
                .windows(longest_subsequence.len())
                .position(|window| window == &longest_subsequence)
                .map(|start| (start..(start + &longest_subsequence.len())))
                .unwrap()
        })
        // .enumerate()
        // .filter(|(i, start)| {
        //     let chars = set.clone().nth(*i).unwrap();
        //     start.start == 0
        //         || (start.start + longest_subsequence.len() < chars.clone().count()
        //             && chars
        //                 .clone()
        //                 .nth(start.start + longest_subsequence.len())
        //                 .unwrap()
        //                 == '.')
        // })
        // .map(|(_, range)| range)
        .collect::<Vec<Range<usize>>>();

    if !set.clone().all(|line| line.clone().contains(&'.')) {
        return None;
    }

    if result_indices.len() == 0 {
        None
    } else {
        let mut prefix_ranges = vec![];
        let mut suffix_ranges = vec![];
        for (range, chars) in result_indices.iter().zip(set_raw) {
            let mut tmp = chars.split("/").collect::<Vec<_>>();
            tmp.pop();
            let offset = tmp.join("/").len() + (if tmp.len() > 0 { 1 } else { 0 });
            let left = offset + range.start;
            let right = offset + range.end;
            prefix_ranges.push(if left <= offset {
                None
            } else {
                Some(offset..left)
            });
            suffix_ranges.push(if right >= chars.len() {
                None
            } else {
                Some(right..chars.len())
            })
        }
        Some((prefix_ranges, suffix_ranges))
    }
}

// get the difference between all of the lines (character-wise)
fn line_diff_char(set: Vec<String>) -> Option<Vec<usize>> {
    if !set.iter().map(|line| line.len()).all_equal() || set.len() < 2 {
        return None;
    }
    let mut flag = false;
    let mut diff: Vec<usize> = Vec::new();
    let mut index = 0;
    loop {
        let chars = set.iter().map(|line| line.chars().nth(index).unwrap());
        if !chars.clone().all_equal() {
            diff.push(index);
        } else {
            if chars.clone().next().unwrap() != '/' {
                flag = true;
            }
        }
        index += 1;
        if index == set[0].len() {
            break;
        }
    }
    if diff.len() == 0 || !flag {
        return None;
    }
    Some(diff)
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

fn replace_ranges_with_wildcard(orig: &str, ranges: Vec<&(usize, String)>) -> String {
    let mut new_line = orig.to_string();
    for (index, range) in ranges.iter() {
        let len = 2 + range.len();
        let start = new_line.chars().take(*index).collect::<String>();
        let wildcards = iter::repeat('*').take(len).collect::<String>();
        let end = new_line.chars().skip(*index + len).collect::<String>();
        new_line = format!("{}{}{}", start, wildcards, end);
    }
    Regex::new(r"\*+")
        .unwrap()
        .replace_all(&new_line, "*")
        .to_string()
}

#[allow(dead_code)]
fn binomial_coefficient(n: usize, k: usize) -> Option<usize> {
    if k > n {
        return Some(0);
    }

    let mut result: usize = 1;
    let k = k.min(n - k);

    for i in 0..k {
        result = match result.checked_mul(n - i).and_then(|r| r.checked_div(i + 1)) {
            Some(val) => val,
            None => return None,
        };
    }

    Some(result)
}

impl Refactor {
    pub fn merge(&mut self) -> &mut Self {
        // iterate over all of the sets of lines, from largest to smallest
        let (prev, params) = self.get_borrows();
        // if end {
        //     self.write_report(vec![format!("Lines reduced by merge process: 0")]);
        //     return self;
        // }
        let (verbose, root, tree, file) = params;
        if verbose {
            printv!(root, tree, file);
        }

        let line_num = file.content.len();
        'outer: loop {
            let file = self.file().clone();
            // if verbose {
            //     printv!(file.content);
            // }
            for size in (2..=file.content.len()).rev() {
                // println!("size: {}", size);
                let filtered_lines = file
                    .content
                    .iter()
                    .filter(|line| matches!(line.content, Content::Pattern(_)))
                    .map(|line| PathBuf::from(line.content.unwrap()))
                    // .filter(|line_str| self.is_normally_ignored(Path::new(line_str)))
                    .filter(|line_str| {
                        tree.node_line_map
                            .keys()
                            .any(|pat| does_match(pat, &line_str.to_str().unwrap().to_string()))
                    });
                // let sets_size = binomial_coefficient(filtered_lines.clone().count(), size);
                // match sets_size {
                //     Some(s) => {
                //         if s > 100000 {
                //             continue;
                //         }
                //     }
                //     None => continue,
                // }
                let sets = if filtered_lines.clone().count() >= size {
                    filtered_lines.combinations(size)
                } else {
                    continue;
                };
                if sets.clone().nth(10000).is_some() {
                    continue;
                }
                for set in sets {
                    // check if all of the lines in the set are siblings (skip if not)
                    let tmp = set[0].clone();
                    let parent = tmp.parent().unwrap();
                    if !set
                        .iter()
                        .map(|line| line.parent().unwrap())
                        .all(|x| x == parent)
                    {
                        continue;
                    }

                    let parent_tree =
                        FsTree::read_at(root.join(parent.strip_prefix("/").unwrap_or(parent)))
                            .expect(&format!(
                                "Failed to read tree at: {:?}",
                                root.join(parent.strip_prefix("/").unwrap_or(parent))
                            ));
                    let not_ignored_children = parent_tree
                        .children()
                        .unwrap()
                        .keys()
                        .map(|path| parent.join(path.strip_prefix("/").unwrap_or(path)))
                        .filter(|path| !self.is_ignored(path))
                        .collect::<Vec<_>>();
                    let mut set_str = set
                        .iter()
                        .map(|x| x.to_str().unwrap().to_string())
                        .collect::<Vec<String>>();

                    let can_range;
                    let diff_indices;

                    // can be merged with range notation
                    if let Some(indices) = line_diff_char(set_str.clone()) {
                        diff_indices = Some(indices);
                        can_range = true;
                    } else {
                        diff_indices = None;
                        can_range = false;
                    }

                    if let Some(ranges) = line_diff_string(set_str.clone()) {
                        // can be merged with wildcard
                        // check all replace patterns: (prefix, suffix), (prefix, None), (None, suffix)
                        for (pre, suf) in [(true, true), (true, false), (false, true)] {
                            let orig = set_str[0].clone();
                            let mut new_line = orig.clone();
                            let mut offset = 0;
                            if pre {
                                if let Some(pre_range) = &ranges.0[0] {
                                    new_line = format!(
                                        "{}*{}",
                                        new_line.chars().take(pre_range.start).collect::<String>(),
                                        new_line.chars().skip(pre_range.end).collect::<String>()
                                    );
                                    offset += pre_range.end - pre_range.start - 1;
                                } else {
                                    continue;
                                }
                            }
                            if suf {
                                if let Some(suf_range) = &ranges.1[0] {
                                    new_line = format!(
                                        "{}*{}",
                                        new_line
                                            .chars()
                                            .take(suf_range.start - offset)
                                            .collect::<String>(),
                                        new_line
                                            .chars()
                                            .skip(suf_range.end - offset)
                                            .collect::<String>()
                                    );
                                } else {
                                    continue;
                                }
                            }
                            if not_ignored_children
                                .iter()
                                .all(|child| !does_match(child, &new_line))
                            {
                                if verbose {
                                    println!("Merging with wildcard:\r\n");
                                    // printv!(new_line);
                                }
                                let file = self.file_mut();
                                for (i, line) in set_str.iter().enumerate() {
                                    let mut new_line = line.clone();
                                    if pre {
                                        if let Some(pre_range) = &ranges.0[i] {
                                            new_line = format!(
                                                "{}*{}",
                                                new_line
                                                    .chars()
                                                    .take(pre_range.start)
                                                    .collect::<String>(),
                                                new_line
                                                    .chars()
                                                    .skip(pre_range.end)
                                                    .collect::<String>()
                                            );
                                            offset += pre_range.end - pre_range.start - 1;
                                        }
                                    }
                                    if suf {
                                        if let Some(suf_range) = &ranges.1[i] {
                                            new_line = format!(
                                                "{}*{}",
                                                new_line
                                                    .chars()
                                                    .take(suf_range.start - offset)
                                                    .collect::<String>(),
                                                new_line
                                                    .chars()
                                                    .skip(suf_range.end - offset)
                                                    .collect::<String>()
                                            );
                                        }
                                    }
                                    file.replace_line(line.to_string(), new_line.clone(), verbose);
                                }
                                file.remove_dupl();
                                // self.halt();
                                continue 'outer;
                            }
                        }
                    }

                    if can_range {
                        let file = self.file_mut();
                        if let Some(diff_indices) = diff_indices {
                            if verbose {
                                println!("Merging with range:\r\n");
                                printv!(diff_indices);
                            }
                            let mut offset = 0;
                            let mut ranges = Vec::new();
                            for index in diff_indices.iter() {
                                // replace each line in the set with a new line with range notation at the index
                                let diff_chars = set_str
                                    .iter()
                                    .map(|line_str| line_str.chars().nth(*index + offset).unwrap())
                                    .collect::<Vec<char>>();
                                let range_str = to_range(diff_chars.clone());
                                for i in 0..set_str.len() {
                                    let new_line = format!(
                                        "{}[{}]{}",
                                        set_str[i]
                                            .chars()
                                            .take(*index + offset)
                                            .collect::<String>(),
                                        range_str.clone(),
                                        set_str[i]
                                            .chars()
                                            .skip(*index + 1 + offset)
                                            .collect::<String>()
                                    );
                                    file.replace_line(
                                        set_str[i].clone(),
                                        new_line.clone(),
                                        verbose,
                                    );
                                    set_str[i] = new_line;
                                }
                                ranges.push((*index + offset, range_str));
                                offset += 2 + to_range(diff_chars.clone()).len() - 1;
                            }
                            file.remove_dupl();

                            // check if any of the range notations can be replaced with a wildcard
                            let orig = set_str[0].clone();
                            'wildcard: for size_ranges in (1..=ranges.len()).rev() {
                                if ranges.len() < size_ranges {
                                    continue;
                                }
                                // let sets_size = binomial_coefficient(ranges.len(), size_ranges);
                                // match sets_size {
                                //     Some(s) => {
                                //         if s > 100000 {
                                //             continue;
                                //         }
                                //     }
                                //     None => continue,
                                // }
                                let sets_ranges = ranges.iter().combinations(size_ranges);
                                if sets_ranges.clone().nth(10000).is_some() {
                                    continue;
                                }
                                for set_ranges in sets_ranges {
                                    let new_line = replace_ranges_with_wildcard(&orig, set_ranges);
                                    if not_ignored_children
                                        .iter()
                                        .all(|child| !does_match(child, &new_line))
                                    {
                                        file.replace_line(orig, new_line.clone(), verbose);
                                        break 'wildcard;
                                    }
                                }
                            }
                            file.remove_dupl();
                        }
                        continue 'outer;
                    }
                }
            }
            break;
        }
        if prev.violate {
            if self.state.lines_diff() > prev.state.unwrap().lines_diff() {
                self.update(true);
            } else {
                self.back();
            }
        } else {
            self.update(true);
        }
        self.write_report(vec![format!(
            "Lines reduced by merge process: {}",
            line_num - self.file().content.len()
        )]);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{process::test, show_result};
    #[test]
    fn test_merge() {
        for level in 1..=1 {
            for path in test::get_input_paths("merge") {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().merge().finish();
                show_result!(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path, level)
                ));
            }
        }
    }

    #[test]
    fn test_line_diff_string() {
        assert_eq!(
            line_diff_string(vec![
                "a/a123.txt".to_string(),
                "a/a456.txt".to_string(),
                "a/a789.txt".to_string()
            ]),
            Some((
                vec![Some(2..6), Some(2..6), Some(2..6)],
                vec![None, None, None]
            ))
        );
        assert_eq!(
            line_diff_string(vec![
                "a/a123.txt".to_string(),
                "b/b456.txt".to_string(),
                "c/c789.txt".to_string()
            ]),
            Some((
                vec![Some(2..6), Some(2..6), Some(2..6)],
                vec![None, None, None]
            ))
        );
        assert_eq!(
            line_diff_string(vec![
                "a/a123.pyo".to_string(),
                "a/a456.pyd".to_string(),
                "a/a789.pyc".to_string()
            ]),
            Some((
                vec![Some(2..6), Some(2..6), Some(2..6)],
                vec![Some(9..10), Some(9..10), Some(9..10)]
            ))
        );
        assert_eq!(
            line_diff_string(vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string()
            ]),
            None
        );
    }

    #[test]
    fn test_line_diff_char() {
        assert_eq!(
            line_diff_char(vec![
                "a/a123.txt".to_string(),
                "a/a456.txt".to_string(),
                "a/a789.txt".to_string()
            ]),
            Some(vec![3, 4, 5])
        );
        assert_eq!(
            line_diff_char(vec![
                "a/a123.txt".to_string(),
                "b/b456.txt".to_string(),
                "c/c789.txt".to_string()
            ]),
            Some(vec![0, 2, 3, 4, 5])
        );
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

    #[test]
    fn test_replace_ranges_with_wildcard() {
        assert_eq!(
            replace_ranges_with_wildcard("a[1-3]b[4-6]c", vec![&(1, "1-3".to_string())]),
            "a*b[4-6]c".to_string()
        );
        assert_eq!(
            replace_ranges_with_wildcard(
                "a[1-3]b[4-6]c",
                vec![&(1, "1-3".to_string()), &(7, "4-6".to_string())]
            ),
            "a*b*c".to_string()
        );
        assert_eq!(
            replace_ranges_with_wildcard("a[1-3][4-6]c", vec![&(6, "4-6".to_string())]),
            "a[1-3]*c".to_string()
        );
        assert_eq!(
            replace_ranges_with_wildcard(
                "a[1-3][4-6]c",
                vec![&(1, "1-3".to_string()), &(6, "4-6".to_string())]
            ),
            "a*c".to_string()
        );
    }
}

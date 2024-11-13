use crate::file::Content;

use super::refactor::Refactor;
use itertools::Itertools;

fn one_char_diff(a: &str, b: &str) -> Option<usize> {
    let mut diff = 0;
    let mut index = 0;
    for (i, (a, b)) in a.chars().zip(b.chars()).enumerate() {
        if a != b {
            diff += 1;
            index = i;
        }
        if diff > 1 {
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
        let file = self.file();
        let mut size = file.content.len();
        'outer: loop {
            for set in file
                .content
                .iter()
                .filter(|line| matches!(line.content, Content::Pattern(_)))
                .map(|line| line.content.unwrap().to_string())
                .combinations(size)
            {
                let mut diff_chars_index = set
                    .iter()
                    .combinations(2)
                    .map(|x| one_char_diff(x[0], x[1]));
                if diff_chars_index.clone().all(|x| x.is_some())
                    && diff_chars_index.clone().all_equal()
                {
                    let index = diff_chars_index.next().unwrap().unwrap();
                    let diff_chars = set
                        .iter()
                        .map(|line_str| line_str.chars().nth(index).unwrap())
                        .collect::<Vec<char>>();
                    let file = self.file_mut();
                    for line in set.clone() {
                        file.remove_line(line, verbose);
                    }
                    file.add_line(
                        format!(
                            "{}[{}]{}",
                            set.iter()
                                .next()
                                .unwrap()
                                .chars()
                                .take(index)
                                .collect::<String>(),
                            to_range(diff_chars),
                            set.iter()
                                .next()
                                .unwrap()
                                .chars()
                                .skip(index + 1)
                                .collect::<String>()
                        ),
                        verbose,
                    );
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
        for path in test::get_input_paths("merge") {
            for level in 1..=1 {
                test::show_title(&path, level);
                let refactor = &mut Refactor::new(&path, level, true);
                let result = refactor.basic_process().merge();
                test::show_result(&result.file());
                assert!(test::file_cmp(
                    result.file(),
                    test::get_expected_path(&path)
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

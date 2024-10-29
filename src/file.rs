use std::path::Path;

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub content: Vec<Line>,
}
impl File {
    pub fn new(f: &Path) -> Self {
        let name = f.file_name().unwrap().to_str().unwrap().to_string();
        let content = std::fs::read_to_string(f)
            .unwrap()
            .lines()
            .enumerate()
            .map(|(i, l)| Line {
                content: match l.chars().next() {
                    Some('#') => Content::Comment(l.to_string()),
                    _ => Content::Pattern(l.to_string()),
                },
                line_number: i + 1,
            })
            .collect();
        Self { name, content }
    }
    pub fn get(&self, i: usize) -> &Line {
        &self.content[i]
    }
    pub fn remove(&mut self, i: usize) {
        self.content.remove(i);
        self.content.iter_mut().for_each(|l| {
            if l.line_number > i {
                l.line_number -= 1;
            }
        });
    }
}

#[derive(Debug, PartialEq)]
pub struct Line {
    pub content: Content,
    pub line_number: usize,
}

#[derive(Debug, PartialEq)]
pub enum Content {
    Comment(String),
    Pattern(String),
}

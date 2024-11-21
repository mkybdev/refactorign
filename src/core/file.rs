use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub content: Vec<Line>,
}
impl File {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let content = std::fs::read_to_string(path.clone())
            .unwrap()
            .lines()
            .enumerate()
            .map(|(i, l)| Line {
                content: if l.trim().is_empty() {
                    Content::Blank()
                } else {
                    match l.chars().next() {
                        Some('#') => Content::Comment(l.to_string()),
                        _ => Content::Pattern(l.to_string()),
                    }
                },
                line_number: i + 1,
            })
            .collect::<Vec<Line>>();
        Self {
            name,
            path,
            content,
        }
    }
    pub fn get_line(&self, i: usize) -> &Line {
        &self.content[i]
    }
    pub fn add_line(&mut self, l: String, verbose: bool) {
        self.content.push(Line {
            content: Content::Pattern(l.clone()),
            line_number: self.content.len() + 1,
        });
        if verbose {
            println!("Added: {}\r\n", l);
        }
    }
    pub fn remove_line(&mut self, l: String, verbose: bool) {
        let i = self
            .content
            .iter()
            .position(|line| match &line.content {
                Content::Pattern(p) => *p == l,
                _ => false,
            })
            .unwrap();
        self.remove_line_with_index(i, verbose);
    }
    pub fn remove_line_with_index(&mut self, i: usize, verbose: bool) {
        let removed = self.content.remove(i);
        self.content.iter_mut().for_each(|l| {
            if l.line_number > i {
                l.line_number -= 1;
            }
        });
        if verbose {
            println!("Removed: {}\r\n", removed.content.unwrap());
        }
    }
    pub fn remove_line_with_path(&mut self, path: PathBuf, verbose: bool) {
        let i = self
            .content
            .iter()
            .position(|l| match &l.content {
                Content::Pattern(p) => p.strip_prefix("/").unwrap_or(p) == path.to_str().unwrap(),
                _ => false,
            })
            .unwrap();
        self.content.remove(i);
        self.content.iter_mut().for_each(|l| {
            if l.line_number > i {
                l.line_number -= 1;
            }
        });
        if verbose {
            println!("Removed: {:?}\r\n", path);
        }
    }
    pub fn remove_dupl(&mut self) {
        let mut i = 0;
        while i < self.content.len() {
            let target = self.get_line(i).content.clone();
            if let Content::Pattern(_) = target {
                let mut j = i + 1;
                while j < self.content.len() {
                    if target == self.get_line(j).content {
                        self.remove_line_with_index(j, false);
                    } else {
                        j += 1;
                    }
                }
            }
            i += 1;
        }
    }
    pub fn replace_line_with_index(&mut self, i: usize, l: String, verbose: bool) {
        let old = self.content[i].content.clone();
        self.content[i] = Line {
            content: Content::Pattern(l.clone()),
            line_number: i + 1,
        };
        if verbose {
            println!("Replaced: {} -> {}\r\n", old.unwrap(), l);
        }
    }
    pub fn replace_line(&mut self, from: String, to: String, verbose: bool) {
        let i = self
            .content
            .iter()
            .position(|line| match &line.content {
                Content::Pattern(p) => *p == from,
                _ => false,
            })
            .unwrap();
        self.replace_line_with_index(i, to, verbose);
    }
    pub fn print_dbg(&self) {
        for line in self.content.iter() {
            println!("{:?}", line);
        }
    }
    pub fn print(&self) {
        for line in self.content.iter() {
            println!("{}", line.content.unwrap());
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Line {
    pub content: Content,
    pub line_number: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Content {
    Comment(String),
    Blank(),
    Pattern(String),
}
impl Content {
    pub fn unwrap(&self) -> &str {
        match self {
            Content::Comment(c) => c,
            Content::Blank() => "",
            Content::Pattern(p) => p,
        }
    }
}

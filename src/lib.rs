pub use core::{file, parse, pattern, tree};
pub use process::refactor::Refactor;

pub mod core {
    pub mod file;
    pub mod parse;
    pub mod pattern;
    pub mod tree;
}

pub mod process {
    pub mod basic_process;
    pub mod containment;
    pub mod merge;
    pub mod re_include;
    pub mod refactor;
    mod test;
}

#[macro_use]
mod macros;

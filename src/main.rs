extern crate refactorign;

use clap::Parser;
use std::path::Path;
use std::path::PathBuf;

pub use refactorign::core;
pub use refactorign::Refactor;

#[allow(unused_imports)]
use refactorign::{show_input, show_result};

/// .gitignore file refactoring tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the .gitignore file to refactor
    #[arg(
        short,
        long,
        help = "Path to the .gitignore file to refactor (If not provided, the tool will look for a .gitignore file in the current directory)"
    )]
    path: Option<String>,

    /// Destination path to the directory to place the refactored .gitignore file
    #[arg(
        short,
        long,
        help = "Destination path to the directory to place the refactored .gitignore file (If not provided, the same directory as the original .gitignore file will be used)"
    )]
    destination: Option<String>,

    /// Whether to overwrite the original .gitignore file
    #[arg(
        short,
        long,
        help = "Whether to overwrite the original .gitignore file",
        default_value_t = false
    )]
    overwrite: bool,

    /// Whether to generate a detailed report on refactoring
    #[arg(
        short,
        long,
        help = "Whether to generate a detailed report on refactoring",
        default_value_t = false
    )]
    report: bool,

    /// Refactoring level (1-3)
    #[arg(
        short,
        long,
        allow_hyphen_values = true,
        default_value_t = 2,
        help = "Refactoring level (1 - 3, Higher level means more aggressive refactoring)"
    )]
    level: isize,
}

fn validate_args(args: &Args) -> (&Path, PathBuf, bool, u8, bool) {
    let path = Path::new(args.path.as_deref().unwrap_or("./.gitignore"));
    if let Some(_) = &args.path {
        if !path.exists() {
            eprintln!("Error: The provided path does not exist.");
            std::process::exit(1);
        }
    } else if !path.exists() {
        eprintln!("Error: No .gitignore file found in the current directory.");
        std::process::exit(1);
    }

    let destination_default = path.parent().unwrap();
    let destination = if let Some(d) = args.destination.as_deref() {
        PathBuf::from(d)
    } else {
        destination_default.to_path_buf()
    };
    if let Some(_) = &args.destination {
        if !destination.exists() {
            eprintln!("Error: The provided destination path does not exist.");
            std::process::exit(1);
        }
    }

    if args.level > 3 || args.level <= 0 {
        eprintln!(
            "Error: Invalid refactoring level. The refactoring level must be between 1 and 3."
        );
        std::process::exit(1);
    }

    (
        path,
        destination.to_path_buf(),
        args.overwrite,
        args.level as u8,
        args.report,
    )
}

const TEST: bool = true;
const VERBOSE: bool = false;

fn main() {
    let args = Args::parse();
    let (path, destination, overwrite, level, report) = validate_args(&args);
    let result = if VERBOSE {
        Refactor::run_verbose(path, level)
    } else {
        Refactor::run(path, level)
    };
    let result_path = if overwrite {
        path.to_path_buf()
    } else {
        destination.join("refactored.gitignore")
    };
    result.save(result_path.clone());
    if overwrite {
        println!("Overwritten: {}", path.display());
    } else {
        println!("Saved: {}", destination.display());
    }
    if report {
        result.save_report(destination.join("refactorign_report").as_path(), result_path);
    }
    if TEST {
        result.save_orig(destination.join("original.gitignore").as_path());
    }
}

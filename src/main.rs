use clap::Parser;
use std::path::Path;

mod basic_process;
mod file;

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

    /// Refactoring level (1-3)
    #[arg(short, long, allow_hyphen_values = true, default_value_t = 2)]
    level: isize,
}

fn validate_args(args: &Args) -> (&Path, u8) {
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

    if args.level > 3 || args.level <= 0 {
        eprintln!(
            "Error: Invalid refactoring level. The refactoring level must be between 1 and 3."
        );
        std::process::exit(1);
    }

    (path, args.level as u8)
}

fn main() {
    let args = Args::parse();
    let (path, level) = validate_args(&args);

    let mut f = file::File::new(path);
    basic_process::remove_dupl(&mut f);
    println!("{:?}", f.content);
}

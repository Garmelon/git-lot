use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    /// Path to a git repository.
    #[arg(default_value = ".")]
    repo: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("Args: {args:#?}");
}

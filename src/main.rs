use std::fs;

// use ast::Prgm;
use clap::Parser;

mod ast;
mod eval;

use ast::Prgm;
use eval::{Context, Eval};

/// The slpy programming language.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The file to run
    #[clap(value_parser)]
    file: String,
}

/// Run the source file.
///
/// # Errors
/// If parsing or evaluation fails.
///
pub fn run(source: String) -> Result<(), &'static str> {
    let contents = fs::read_to_string(source).expect("Should have been able to read the file");
    let mut prgm: Prgm = contents.parse().map_err(|e| {
        dbg!(e);
        "parsing failed"
    })?;
    prgm.eval(&mut Context::default())?;
    Ok(())
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();
    run(args.file)
}

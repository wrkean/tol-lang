use std::path::PathBuf;

use clap::Parser;

fn main() {
    let args = Args::parse();

    let result = tol_lang::driver::run_file(args.input);

    if !result.output.is_empty() {
        println!("{}", result.output);
    }

    for diagnostic in result.diagnostics {
        eprintln!("{diagnostic}");
    }
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

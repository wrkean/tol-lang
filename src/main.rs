#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

use crate::compiler::Compiler;

mod compiler;
mod parse;
mod prelude;
mod tol;

fn main() {
    let args = Args::parse();

    let mut compiler = Compiler::new(args);
    compiler.compile_entry_point();
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

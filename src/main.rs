#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

use crate::global_ctx::GlobalContext;

mod global_ctx;
mod parse;
mod prelude;
mod tol;

fn main() {
    let args = Args::parse();

    let mut compiler = GlobalContext::new(args);
    compiler.compile_entry_point();
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

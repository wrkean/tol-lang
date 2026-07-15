#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

use crate::global_ctx::GlobalContext;

mod analyze;
mod codegen;
mod driver;
mod global_ctx;
mod module;
mod parse;
mod prelude;
mod tol;
mod vm;

fn main() {
    let args = Args::parse();

    let mut global_context = GlobalContext::new(args);
    driver::compile_entry_point(&mut global_context);
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

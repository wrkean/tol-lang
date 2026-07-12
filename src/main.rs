#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

mod compiler;

fn main() {
    let args = Args::parse();
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

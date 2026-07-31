#![allow(unused)]

use std::path::PathBuf;

use clap::Parser;

use crate::{global_ctx::GlobalContext, tol::diagnostic::miette_diagnostic::MietteDiagnostic};

mod analyze;
mod builtins;
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

    let stdlib_path = option_env!("TOL_STDLIB");
    let stdlib_path = match stdlib_path {
        Some(s) => PathBuf::from(s),
        None => {
            eprintln!(
                "{}",
                miette::miette!(
                    "hindi nakaset ang environment variable na TOL_STDLIB, dapat ay ma-iset ito kung saan nakalagay ang stdlib ng tol"
                )
            );
            return;
        }
    };

    let mut global_context = GlobalContext::new(args, stdlib_path);
    if let Err(diag) = driver::compile_entry_point(&mut global_context) {
        eprintln!("{}", miette::Report::new(MietteDiagnostic::from(*diag)));
        return;
    }
    driver::run(0, global_context);
}

#[derive(Parser)]
pub struct Args {
    input: PathBuf,
}

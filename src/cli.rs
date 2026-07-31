use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(help_template = "\
{about}

Gamit:
    {usage}

Mga commands:
{subcommands}

Mga opsyon:
{options}
")]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "takbo")]
    #[command(about("i-compile at patakbuhin ang input file"))]
    Run(RunArgs),
}

#[derive(Args)]
pub struct RunArgs {
    pub input: PathBuf,
}

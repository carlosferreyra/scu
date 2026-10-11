//! Clap command model for `scu`.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// `scu` command-line interface.
#[derive(Debug, Parser)]
#[command(name = "scu", version, about)]
pub struct Cli {
    /// Command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run a WASI P3 component implementing the SCU smoke contract.
    Run {
        /// Component artifact to execute.
        component: PathBuf,
    },
}

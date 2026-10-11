//! Executable entrypoint for `scu`.

use clap::Parser;
use scu_cli::{Cli, Command};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command {
        Command::Run { component } => scu_wasi_host::run_smoke(component).await,
    };
    match result {
        Ok(output) => {
            println!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

//! Executable entrypoint for `scu`.

use clap::Parser;
use scu_cli::{Cli, Command};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let action = match cli.command {
        Command::Init(args) => format!(
            "initializing project at {}; config={}; local tools={:?}; providers={:?}; mode={}",
            args.path.display(),
            args.path.join(&cli.config).display(),
            args.local_tool,
            args.provider,
            if args.non_interactive {
                "non-interactive"
            } else {
                "interactive selection"
            },
        ),
        Command::Check(args) => format!(
            "checking SDK declarations, config, and {} backend compatibility; config={}",
            args.name(),
            cli.config.display(),
        ),
        Command::Plan(args) => format!(
            "check -> previewing infrastructure changes for {} env; config={}",
            args.name(),
            cli.config.display(),
        ),
        Command::Up(args) => format!(
            "check -> plan -> {}; config={}",
            if args.prod {
                "deploying infra to prod using Pulumi"
            } else {
                "running dev env locally"
            },
            cli.config.display(),
        ),
        Command::Down(args) => format!(
            "{}; config={}",
            if args.prod {
                "destroying prod infra using Pulumi"
            } else {
                "stopping dev env and retaining persistent data"
            },
            cli.config.display(),
        ),
        Command::Status(args) => format!(
            "showing {} env status; config={}",
            args.name(),
            cli.config.display(),
        ),
        Command::Run { component } => return run_smoke(component).await,
    };

    println!("CLI skeleton (not implemented): {action}");
    std::process::ExitCode::SUCCESS
}

async fn run_smoke(component: std::path::PathBuf) -> std::process::ExitCode {
    let result = scu_wasi_host::run_smoke(component).await;
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

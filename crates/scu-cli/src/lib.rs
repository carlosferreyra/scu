//! Clap command model for `scu`.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// `scu` command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "scu",
    version,
    about = "Compose infrastructure from SDK declarations and scu.toml",
    after_help = "Project commands are placeholders: they do not create files, validate, or deploy resources."
)]
pub struct Cli {
    /// Project configuration path (not read by the skeleton).
    #[arg(long, global = true, default_value = "scu.toml")]
    pub config: PathBuf,

    /// Command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Available commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scaffold scu.toml and an SDK quickstart (placeholder).
    Init(InitArgs),
    /// Check SDK declarations, configuration, and backend compatibility (placeholder).
    Check(EnvironmentArgs),
    /// Check and preview infrastructure changes (placeholder).
    Plan(EnvironmentArgs),
    /// Check, plan, then start dev or deploy prod infrastructure (placeholder).
    Up(EnvironmentArgs),
    /// Stop dev or destroy prod infrastructure (placeholder).
    Down(EnvironmentArgs),
    /// Show the selected environment's state (placeholder).
    Status(EnvironmentArgs),
    /// Run a WASI P3 component implementing the SCU smoke contract.
    #[command(hide = true)]
    Run {
        /// Component artifact to execute.
        component: PathBuf,
    },
}

/// Project initialization options.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Project directory to initialize.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Local tool to select; repeat to select multiple tools.
    #[arg(long, value_name = "TOOL")]
    pub local_tool: Vec<String>,

    /// Provider to configure; repeat to select multiple providers.
    #[arg(long, value_name = "PROVIDER")]
    pub provider: Vec<String>,

    /// Use supplied selections and defaults without prompting.
    #[arg(long)]
    pub non_interactive: bool,
}

/// Environment selection shared by project lifecycle commands.
#[derive(Debug, Args)]
#[group(id = "environment", multiple = false)]
pub struct EnvironmentArgs {
    /// Select the local development environment (default).
    #[arg(long)]
    pub dev: bool,

    /// Select the production environment using Pulumi.
    #[arg(long)]
    pub prod: bool,
}

impl EnvironmentArgs {
    /// Selected environment, defaulting to local development.
    pub fn name(&self) -> &'static str {
        if self.prod { "prod" } else { "dev" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn lifecycle_commands_default_to_dev_and_reject_conflicting_environments() {
        for command in ["check", "plan", "up", "down", "status"] {
            for selection in [None, Some("--dev"), Some("--prod")] {
                let mut arguments = vec!["scu", command];
                if let Some(flag) = selection {
                    arguments.push(flag);
                }
                let cli = Cli::try_parse_from(arguments).unwrap();
                let (Command::Check(args)
                | Command::Plan(args)
                | Command::Up(args)
                | Command::Down(args)
                | Command::Status(args)) = cli.command
                else {
                    panic!("expected lifecycle command")
                };
                assert_eq!(
                    args.name(),
                    if selection == Some("--prod") {
                        "prod"
                    } else {
                        "dev"
                    }
                );
                assert_eq!(cli.config, PathBuf::from("scu.toml"));
            }
            let error = Cli::try_parse_from(["scu", command, "--dev", "--prod"]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
        }
    }

    #[test]
    fn config_override_works_before_or_after_the_command() {
        for argv in [
            ["scu", "--config", "custom.toml", "check"],
            ["scu", "check", "--config", "custom.toml"],
        ] {
            assert_eq!(
                Cli::try_parse_from(argv).unwrap().config,
                PathBuf::from("custom.toml")
            );
        }
    }

    #[test]
    fn init_accepts_repeated_selections_and_non_interactive_mode() {
        let cli = Cli::try_parse_from([
            "scu",
            "init",
            "example",
            "--local-tool",
            "wasmtime",
            "--local-tool",
            "postgres",
            "--provider",
            "aws",
            "--provider",
            "cloudflare",
            "--non-interactive",
        ])
        .unwrap();
        let Command::Init(args) = cli.command else {
            panic!("expected init")
        };
        assert_eq!(args.path, PathBuf::from("example"));
        assert_eq!(args.local_tool, ["wasmtime", "postgres"]);
        assert_eq!(args.provider, ["aws", "cloudflare"]);
        assert!(args.non_interactive);

        let cli = Cli::try_parse_from(["scu", "init"]).unwrap();
        let Command::Init(args) = cli.command else {
            panic!("expected init")
        };
        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.local_tool, Vec::<String>::new());
        assert_eq!(args.provider, Vec::<String>::new());
        assert!(!args.non_interactive);
    }

    #[test]
    fn smoke_runner_remains_available() {
        let cli = Cli::try_parse_from(["scu", "run", "smoke.wasm"]).unwrap();
        let Command::Run { component } = cli.command else {
            panic!("expected run")
        };
        assert_eq!(component, PathBuf::from("smoke.wasm"));
    }
}

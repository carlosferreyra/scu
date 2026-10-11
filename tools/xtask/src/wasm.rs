use std::path::PathBuf;
use std::process::Command;

use crate::Result;
use crate::cli::WasmCommand;
use crate::process;
use crate::workspace::{Workspace, read_document};

// Native --workspace operations must explicitly exclude the target-only guest.
pub(crate) const GUEST_PACKAGE: &str = "scu-p3-smoke";

struct Settings {
    toolchain: String,
    target: String,
    wasm_tools: String,
}

impl Settings {
    fn load(workspace: &Workspace) -> Result<Self> {
        let document = read_document(&workspace.path("wasm-toolchain.toml"))?;
        let value = |table: &str, key: &str| {
            document[table][key]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("missing {table}.{key} in wasm-toolchain.toml"))
        };
        let settings = Self {
            toolchain: value("p3", "toolchain")?,
            target: value("p3", "target")?,
            wasm_tools: value("tools", "wasm-tools")?,
        };
        if settings.target != "wasm32-wasip3" || value("p3", "package")? != GUEST_PACKAGE {
            return Err("P3 verification requires wasm32-wasip3 and scu-p3-smoke".into());
        }
        Ok(settings)
    }

    fn artifact(&self, workspace: &Workspace) -> PathBuf {
        workspace.path(format!("target/p3/{}/debug/scu_p3_smoke.wasm", self.target))
    }
}

pub(crate) fn run(workspace: &Workspace, command: &WasmCommand) -> Result {
    let settings = Settings::load(workspace)?;
    match command {
        WasmCommand::Setup => setup(workspace, &settings),
        WasmCommand::Doctor => doctor(workspace, &settings),
        WasmCommand::Build => build(workspace, &settings),
        WasmCommand::Check => check(workspace, &settings),
        WasmCommand::Test => {
            check(workspace, &settings)?;
            let status = Command::new("cargo")
                .current_dir(workspace.root())
                .args([
                    "test",
                    "--locked",
                    "-p",
                    "scu-wasi-host",
                    "--test",
                    "p3_component",
                    "--",
                    "--ignored",
                ])
                .env("SCU_P3_COMPONENT", settings.artifact(workspace))
                .status()
                .map_err(|error| format!("failed to run P3 host tests: {error}"))?;
            if status.success() {
                Ok(())
            } else {
                Err("P3 host integration tests failed".into())
            }
        }
    }
}

fn setup(workspace: &Workspace, settings: &Settings) -> Result {
    process::run(
        workspace.root(),
        "rustup",
        [
            "toolchain",
            "install",
            &settings.toolchain,
            "--profile",
            "minimal",
            "--component",
            "clippy,rustfmt",
            "--target",
            &settings.target,
        ],
    )?;
    process::run(
        workspace.root(),
        "cargo",
        [
            "install",
            "wasm-tools",
            "--version",
            &settings.wasm_tools,
            "--locked",
            "--root",
            ".xtask/tools",
            "--no-default-features",
            "--features",
            "validate,component",
        ],
    )
}

fn tool(workspace: &Workspace) -> PathBuf {
    workspace.path(format!(
        ".xtask/tools/bin/wasm-tools{}",
        std::env::consts::EXE_SUFFIX
    ))
}

fn doctor(workspace: &Workspace, settings: &Settings) -> Result {
    let targets = Command::new("rustup")
        .args([
            "target",
            "list",
            "--installed",
            "--toolchain",
            &settings.toolchain,
        ])
        .output()
        .map_err(|error| format!("failed to inspect P3 target: {error}"))?;
    if !targets.status.success()
        || !String::from_utf8_lossy(&targets.stdout)
            .lines()
            .any(|line| line == settings.target)
    {
        return Err("pinned P3 compiler/target missing; run `cargo xtask wasm setup`".into());
    }
    println!("{}: {} installed", settings.toolchain, settings.target);
    let output = Command::new(tool(workspace))
        .arg("--version")
        .output()
        .map_err(|error| format!("wasm-tools missing ({error}); run `cargo xtask wasm setup`"))?;
    let version = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || version.trim() != format!("wasm-tools {}", settings.wasm_tools) {
        return Err("wasm-tools version mismatch; run `cargo xtask wasm setup`".into());
    }
    println!("{}", version.trim());
    Ok(())
}

fn guest_cargo(workspace: &Workspace, settings: &Settings, command: &str, lint: bool) -> Result {
    let toolchain = format!("+{}", settings.toolchain);
    let mut args = vec![
        toolchain.as_str(),
        command,
        "--locked",
        "-p",
        GUEST_PACKAGE,
        "--target",
        &settings.target,
        "--target-dir",
        "target/p3",
    ];
    if lint {
        args.extend(["--", "-D", "warnings"]);
    }
    process::run(workspace.root(), "cargo", args)
}

fn build(workspace: &Workspace, settings: &Settings) -> Result {
    guest_cargo(workspace, settings, "build", false)
}

fn check(workspace: &Workspace, settings: &Settings) -> Result {
    doctor(workspace, settings)?;
    build(workspace, settings)?;
    guest_cargo(workspace, settings, "clippy", true)?;
    let artifact = settings.artifact(workspace);
    process::run(
        workspace.root(),
        tool(workspace),
        [
            "validate",
            "--features",
            "all",
            artifact.to_str().ok_or("non-UTF-8 component path")?,
        ],
    )?;
    let output = Command::new(tool(workspace))
        .current_dir(workspace.root())
        .args(["component", "wit"])
        .arg(&artifact)
        .output()
        .map_err(|error| format!("failed to inspect component interfaces: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "component inspection failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let wit = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    if wit.contains("@0.2.")
        || wit.contains("wasi_snapshot_preview1")
        || !wit.contains("scu:runtime/host@0.1.0")
        || !wit.contains("run: async func() -> string")
    {
        return Err("component does not match the strict P3 smoke contract".into());
    }
    let path = workspace.path("target/p3/component.wit");
    std::fs::write(&path, wit).map_err(|error| error.to_string())?;
    println!("P3 component validated; interfaces: {}", path.display());
    Ok(())
}

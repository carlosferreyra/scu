# SCU — System Composition Utility

> System Composition Utility (SCU): provider-agnostic infrastructure composition.

SCU starts a fresh design for a native Rust infrastructure composition tool.
The library and CLI will be implemented in Rust, with Python bindings considered
later. The workspace now includes a native Wasmtime host, a CLI, and a strict
WASI P3 component fixture. Infrastructure composition remains to be implemented.

The primary library remains dependency-free. Native runtime dependencies live
in `scu-wasi-host`, and guest code lives in `examples/p3-smoke`.
See [WASI P3 development](docs/wasm.md) for the layout and pinned toolchain.

`AGENTS.md` guides coding agents through the workspace CLI, scaffolding, and
verification commands. It is included in every generated project.

## Project CLI skeleton

```sh
cargo run -p scu-cli -- init my-app --local-tool wasmtime --provider aws --non-interactive
cargo run -p scu-cli -- check
cargo run -p scu-cli -- plan --prod
cargo run -p scu-cli -- up --dev
cargo run -p scu-cli -- up --prod
cargo run -p scu-cli -- down --dev
cargo run -p scu-cli -- status --prod --config custom.toml
cargo run -p scu-cli -- --help
```

These six project commands currently print placeholder messages. They do not
create files, load configuration, validate SDK code, or provision resources.
Dev is the default; `--dev` and `--prod` are mutually exclusive. `--config`
defaults to `scu.toml`. `init` accepts repeated local-tool/provider selections.
Tool and provider names are illustrative, not implemented integrations.
See the [implementation plan](docs/implementation-plan.md) for the SDK,
configuration, local lifecycle, and Pulumi delivery sequence.

## Develop

```sh
cargo xtask check
cargo xtask test
cargo xtask build
cargo xtask --help
```

```sh
cargo xtask wasm setup
cargo xtask wasm test
cargo run -p scu-cli -- run target/p3/wasm32-wasip3/debug/scu_p3_smoke.wasm
```

## Grow the workspace

Use project-prefixed semantic crate names, inspired by uv's ownership model:

```sh
cargo xtask scaffold crate workspace
cargo xtask scaffold crate resolver
cargo xtask scaffold crate client --private
```

Those examples are illustrative. Choose a noun from your own domain; do not
copy uv's Python-specific crate names.

Avoid generic buckets such as `core`, `common`, `shared`, `utils`, and `misc`.
A CLI command or a Cargo feature is not automatically a crate either. Keep
behavior inside the primary crate first, then extract a crate when it owns a
coherent responsibility, gives callers a smaller interface than its
implementation, and improves locality for changes and tests.

Use `types` only for a shared vocabulary or trait that prevents a real
dependency cycle. Use `dispatch` only when multiple domain modules need a
dedicated orchestration module. One adapter is a hypothetical seam; introduce
an adapter seam when behavior actually varies.

An application can eventually grow toward this shape without pre-generating it:

```text
scu                 # entry/composition package
├── scu-cli         # command grammar and parsing
├── scu-workspace   # workspace model
├── scu-resolver    # domain behavior
└── scu-client      # external adapter
```

The entry/composition package may depend on many product crates. Domain crates
must not depend on the entrypoint or CLI. When one product crate depends on
another, declare it once under root `[workspace.dependencies]` and use
`{ workspace = true }` from the member manifest. External dependency versions
are also declared at the workspace root; `xtask` and optional CLI crates inherit
them. The primary library remains independent of Clap.

## Add a CLI

```sh
# Keep a public library independent of Clap. This is the default.
cargo xtask scaffold cli --entrypoint companion

# Use the primary crate as an uv-like application composition package.
cargo xtask scaffold cli --entrypoint primary
```

Both modes create `scu-cli` for the command model. The companion
mode owns the executable in that private package; the primary mode adds a thin
binary wrapper to the primary package and makes the CLI crate publishable.
Publish the CLI crate before the primary package when releasing to crates.io.

## More capabilities

```sh
cargo xtask scaffold ci
cargo xtask scaffold docs
cargo xtask scaffold agents --claude
cargo xtask scaffold crate <semantic-name> --dry-run
```

## Update development tools

This project was generated with current stable dependency releases and GitHub
action tags. Its `Cargo.toml`, `Cargo.lock`, and bundled action registry record
that generation's choices. Adding CI later uses those recorded action tags.
`cargo xtask scaffold ci --preset full` adds weekly Dependabot checks for Cargo
dependencies and GitHub Actions. Major dependency upgrades can require code changes.

```sh
cargo xtask tools sync all
```

Sync updates Rust's `stable` toolchain, Clippy, and rustfmt, then installs the
latest stable releases of the selected tools under `.xtask/tools`. Use `test`,
`coverage`, `ci`, or `release` instead of `all` to update one group. Existing
working local or global tools remain usable between syncs; other commands do
not install or upgrade tools implicitly. Sync requires network access and may
change tool behavior as new releases become available.

Scaffolds are explicit, idempotent, and refuse to overwrite unmarked
user-owned files. Generated projects are snapshots: template updates are not
applied automatically.

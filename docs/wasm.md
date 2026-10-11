# WASI P3 development

SCU uses Wasmtime for local Component Model execution. The initial runnable
slice is an async smoke contract, not infrastructure provisioning. Pulumi,
service lifecycle management, database engines, and Wasmer/WASIX integration
remain future work.

## Workspace boundaries

| Package | Compiles for | Responsibility |
| --- | --- | --- |
| `scu` | Native and `wasm32-wasip3` | Portable project behavior; no runtime dependencies |
| `scu-wasi-host` | Native | Wasmtime engine, P3 WASI context, async host bindings |
| `scu-cli` | Native | `scu run <component>` for the smoke world |
| `scu-p3-smoke` | `wasm32-wasip3` only | Component fixture using the portable library and async WIT bindings |
| `xtask` | Native | Setup, builds, validation, and tests |

Native adapters are intentionally native: "Wasm compatible workspace" means
portable guest code and explicit build boundaries, not compiling Wasmtime or
repository automation into the guest. The guest has a compile-time target
guard. Every native xtask workspace operation explicitly excludes it;
`default-members` also makes ordinary `cargo check` select native members.
For raw workspace commands, pass `--exclude scu-p3-smoke` yourself.

## Pinned toolchain

`rust-toolchain.toml` pins native Rust 1.99.0. `wasm-toolchain.toml` is custom
configuration read by xtask, not a rustup override file. It pins:

- Rust `nightly-2026-10-09`, targeting literal `wasm32-wasip3`.
- Wasm tools 1.261.0, installed under ignored `.xtask/tools`.

The selected nightly distributes the P3 standard library and component linker.
Pure Rust guests therefore need neither a separate WASI SDK nor `build-std`.
The stable compiler used for native code does not provide this target's
prebuilt standard library. Do not substitute `wasm32-wasip2` when setup fails.

Root workspace dependencies pin Wasmtime and wasmtime-wasi to 49.0.2 and
wit-bindgen to 0.62.0; `Cargo.lock` records the complete dependency graph.
The WASI adapter enables P3 only. The host enables component async and its
stackful task builtins, which the pinned Rust standard library uses. It does
not enable the separate core Wasm stack-switching proposal: Wasmtime 49
cannot enable that proposal on this Apple Silicon configuration.

## Commands

```sh
# Explicit installation; network access required. No shared stable update.
cargo xtask wasm setup
cargo xtask wasm doctor

# Build the fixture, including the portable SCU library.
cargo xtask wasm build

# Build, lint, validate the binary, and inspect its WIT contract.
cargo xtask wasm check

# Check, then run the async component through the real native host.
cargo xtask wasm test

# Native formatting, compilation, Clippy, tests, and documentation.
cargo xtask ci

cargo run -p scu-cli -- run target/p3/wasm32-wasip3/debug/scu_p3_smoke.wasm
```

Guest output uses `target/p3` to avoid collisions with native Cargo artifacts.
`target/p3/component.wit` contains the inspected artifact interfaces.
Verification rejects P1/P2 interfaces and requires the SCU async smoke export.
The fixture calls an async host import that suspends before returning. It also
checks that the guest receives no host environment variables or preopened
directories. Native tests reject malformed artifacts, missing files, and
unsupported imports. The artifact test is explicitly ignored during native
tests and enabled by `cargo xtask wasm test`, which supplies its freshly built
artifact; a missing artifact is an error, never a silently skipped test.

The CI workflow has separate native and P3 jobs. Remote CI still needs to run
on GitHub; local verification is not proof of the Ubuntu job's result.

## Contract and capability policy

`wit/scu/world.wit` owns `scu:runtime/smoke@0.1.0`, with async `host.echo` and
`run` functions. Host and guest bindings derive from the same file. Standard
WASI dependencies are supplied by the pinned compiler/runtime; there is no
hand-copied `wit/deps` tree to maintain yet.

The initial host uses Wasmtime's default WASI context: no inherited environment,
arguments, filesystem directories, or permitted network addresses. Clocks and
randomness use WASI defaults. Unsupported imports fail linking; no stub
implementations or P2 adapters are registered.

This is a development fixture host. It does not yet impose production resource
budgets or support arbitrary WASI command worlds. The integration timeout
detects an async hang; it is not a CPU or memory budget for untrusted guests.

Keep WASI/P3 upgrades deliberate: update pins together, run both verification
lanes, inspect emitted imports, and investigate ABI failures. C/C++ dependencies
may require a matching SDK even though this pure Rust fixture does not.

Add Wasmer/WASIX only for a concrete compatibility workload that fails the P3
path. Keep those dependencies and build tooling in a separate adapter boundary.
Running PostgreSQL in Wasm requires a verified port and fidelity tests; adding
Wasmer alone does not establish PostgreSQL compatibility.

# SDK and configuration implementation plan

## Direction and ownership

SDK code declares resources, dependencies, and application requirements.
`scu.toml` selects environments, providers, local implementations, and settings.
These are the authored inputs. SCU generates a resolved infrastructure graph
and backend programs; generated files are not another source to maintain.

The SDK belongs in the portable `scu` library until extraction has a concrete
ownership benefit. Clap stays in `scu-cli`; Wasmtime stays in `scu-wasi-host`.
Use strict WASI P3 for infrastructure declaration components. Keep declaration
execution separate from application startup and resource provisioning.

Proposed user project layout, to be created by a future `init` implementation:

```text
scu.toml
infra/                         # SDK declaration program
src/                           # Application
.scu/generated/dev/infra.yml    # Resolved SCU graph
.scu/generated/prod/Pulumi.yaml # Lowered Pulumi program
```

The SCU graph has its own versioned schema. It is not directly a Pulumi YAML
program. Generated artifacts must reference the exact SDK build, configuration,
environment, and dependency pins used to construct them.

## Delivery sequence and verification

1. **CLI skeleton (this change).** Parse the six project commands and their
   parameters; print explicitly marked placeholders. Preserve the existing
   smoke runner as a hidden development command. Verify command help, defaults,
   conflicting environment flags, repeated initialization selections, and
   absence of filesystem changes. No SDK, config loading, prompts, or deployment
   is implemented by this step.
2. **Project configuration and initialization.** Define a minimal versioned TOML
   schema, discover project configuration, and resolve relative paths against
   its directory. Implement `init` without overwriting existing user files.
   Detect tools and suggest selections interactively; support reproducible
   non-interactive initialization. Record selections; do not silently change
   backends based on installed tools. Use credential references rather than
   committed secrets. Verify a fresh quickstart, repeat-init safety, invalid
   configuration, and equivalent interactive/non-interactive selections.
3. **SDK and resource graph.** Implement one useful resource declaration and
   its connections in `scu`. Define a declaration WIT contract separate from
   the smoke world. Compile and execute the declaration component through
   Wasmtime to obtain a deterministic graph without provisioning capabilities.
   Verify typed connections, graph validation, deterministic output, and the
   strict P3 host/guest boundary.
4. **Check and plan.** Compile declarations, validate TOML and backend support,
   then generate backend inputs and preview their changes against state.
   Compile-time guarantees cover typed SDK relationships; configuration,
   credentials, quotas, and provider failures require additional checks.
   A failed check cancels planning and deployment. Verify invalid graphs,
   unsupported backend capabilities, and rejection of stale generated plans.
5. **Local lifecycle.** Implement one concrete local resource plus component
   execution before broadening resource support. `up` checks, plans, and starts
   the environment; `down` stops it while retaining persistent data; `status`
   reports actual state. Verify repeated `up`, startup failures, shutdown, and
   persistence. Wasmtime executes components; service provisioning still needs
   concrete local implementations. Add Wasmer/WASIX only for a verified need.
6. **Pulumi lifecycle.** Lower the validated graph to a Pulumi YAML program and
   invoke the Pulumi CLI for preview, deployment, destruction, and status.
   Separate provider authentication from selection. Apply the same inputs
   that passed validation; source/config changes invalidate a saved plan.
   Verify generated programs, adapter error propagation, failed-check
   cancellation, stale-plan rejection, and an explicitly authorized test
   deployment before claiming end-to-end support.

Pulumi's official Automation API currently lists TypeScript/JavaScript, Python,
Go, .NET, and Java, and requires the Pulumi CLI underneath. The initial Rust
integration therefore uses the CLI rather than introducing an unsupported
library dependency. See [Automation API](https://www.pulumi.com/docs/iac/concepts/automation-api/)
and [its CLI requirement](https://www.pulumi.com/docs/iac/guides/building-extending/automation-api/).

## Initial CLI contract

All project handlers currently print placeholder text and return success for
valid arguments. This means successful argument dispatch, not a successful
check, plan, initialization, or deployment. No files are read or written, no
prompts are displayed, and no providers are contacted.

| Command | Parameters | Intended behavior |
| --- | --- | --- |
| `init [PATH]` | Repeatable `--local-tool TOOL`, repeatable `--provider PROVIDER`, `--non-interactive` | Create TOML and SDK quickstart; select tools/providers |
| `check` | `--dev` or `--prod` | Compile and validate declarations, config, and backend compatibility |
| `plan` | `--dev` or `--prod` | Check, then preview changes |
| `up` | `--dev` or `--prod` | Check, plan, then start local dev or deploy prod through Pulumi |
| `down` | `--dev` or `--prod` | Stop local dev, or destroy the prod deployment |
| `status` | `--dev` or `--prod` | Report environment state |

`--config PATH` is global and defaults to `scu.toml`. For `init`, a relative
config path is under the selected project directory. The skeleton only displays
the path. Initialization defaults to `PATH=.` and interactive selection;
lifecycle commands default to dev. `--dev` and `--prod` are mutually exclusive.
Tool/provider names are currently free-form selections, not verified integrations.
Clap supplies `--help` and top-level `--version`; malformed arguments fail.

The existing `run <COMPONENT>` command still executes the fixed P3 smoke
contract and is hidden from top-level help. It does not use project configuration.

Do not add a duplicate `apply` command initially. Consider separate `dev` and
`deploy` commands when foreground watching/rebuilding and persistent deployment
actually have different implemented lifecycles. Named environments, structured
JSON output, and saved-plan application can be designed when their behavior is
implemented rather than adding inactive flags now.

## Repository verification

For the CLI skeleton, run `cargo xtask test`, inspect all command help/output,
and finish with `git diff --check`. After future host, guest, WIT, or toolchain
changes, also run `cargo xtask wasm test` as required by `AGENTS.md`.

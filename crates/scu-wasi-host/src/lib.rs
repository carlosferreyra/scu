//! Native host for the experimental SCU WASI P3 smoke contract.

use std::path::Path;
use wasmtime::component::{Accessor, Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

#[allow(missing_docs, clippy::all, clippy::pedantic)]
mod bindings {
    wasmtime::component::bindgen!({
        path: "../../wit/scu",
        world: "smoke",
    });
}

#[derive(Default)]
struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

impl bindings::scu::runtime::host::Host for State {}

impl bindings::scu::runtime::host::HostWithStore<State> for HasSelf<State> {
    async fn echo(_store: &Accessor<State, Self>, value: String) -> String {
        // Force suspension so the fixture verifies the P3 async boundary.
        tokio::task::yield_now().await;
        value
    }
}

/// Run a component implementing `scu:runtime/smoke@0.1.0`.
///
/// The host grants no filesystem directories, environment, arguments, or
/// network access. This is the initial fixture host, not a deployment engine.
pub async fn run_smoke(path: impl AsRef<Path>) -> wasmtime::Result<String> {
    let engine = engine()?;
    let component = Component::from_file(&engine, path)?;
    let linker = linker(&engine)?;
    let mut store = Store::new(&engine, State::default());
    let smoke = bindings::Smoke::instantiate_async(&mut store, &component, &linker).await?;
    store
        .run_concurrent(async move |accessor| smoke.call_run(accessor).await)
        .await?
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model_async(true);
    // The pinned Rust P3 standard library uses cooperative task builtins.
    config.wasm_component_model_async_stackful(true);
    Engine::new(&config)
}

fn linker(engine: &Engine) -> wasmtime::Result<Linker<State>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p3::add_to_linker(&mut linker)?;
    bindings::Smoke::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;
    Ok(linker)
}

#[cfg(test)]
mod tests {
    #[test]
    fn unsupported_import_is_rejected() {
        let engine = super::engine().expect("engine");
        let bytes = wat::parse_str(
            r#"(component
            (type $function (func))
            (import "unknown-import" (func (type $function))))"#,
        )
        .expect("valid component WAT");
        let component = super::Component::from_binary(&engine, &bytes).expect("valid component");
        let linker = super::linker(&engine).expect("linker");
        let error = linker
            .instantiate_pre(&component)
            .err()
            .expect("unknown import must fail");
        assert!(error.to_string().contains("unknown-import"), "{error:#}");
    }
}

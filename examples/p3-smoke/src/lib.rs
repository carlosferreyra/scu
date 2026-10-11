//! Strict WASI P3 fixture for the SCU async host contract.

#[cfg(not(all(target_os = "wasi", target_env = "p3")))]
compile_error!("build this fixture with `cargo xtask wasm build` (wasm32-wasip3)");

// Generated ABI glue contains unsafe code; application code does not.
#[allow(unsafe_code, missing_docs, clippy::all, clippy::pedantic)]
mod bindings {
    use super::Smoke;
    wit_bindgen::generate!({ path: "../../wit/scu", world: "smoke" });
    export!(Smoke with_types_in self);
}

struct Smoke;

impl bindings::Guest for Smoke {
    async fn run() -> String {
        // Exercise the P3 standard library without granting host capabilities.
        assert_eq!(std::env::vars().count(), 0);
        assert!(std::fs::read_dir(".").is_err());
        bindings::scu::runtime::host::echo(format!("{} from WASI P3", scu::hello())).await
    }
}

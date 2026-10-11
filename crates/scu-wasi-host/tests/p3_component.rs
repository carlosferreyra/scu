//! Artifact tests are explicit: `cargo xtask wasm test` builds their guest first.

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the strict P3 artifact; run cargo xtask wasm test"]
async fn async_component_round_trip() {
    let path = std::env::var_os("SCU_P3_COMPONENT")
        .expect("SCU_P3_COMPONENT must be supplied by cargo xtask wasm test");
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        scu_wasi_host::run_smoke(path),
    )
    .await
    .expect("async guest did not complete")
    .expect("P3 guest failed");
    assert_eq!(output, "Hello from scu! from WASI P3");
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_component_is_rejected() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let result = scu_wasi_host::run_smoke(path).await;
    assert!(
        result.is_err(),
        "a TOML manifest must not be accepted as a component"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn missing_component_is_rejected() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("absent-component.wasm");
    assert!(!path.exists());
    assert!(scu_wasi_host::run_smoke(path).await.is_err());
}

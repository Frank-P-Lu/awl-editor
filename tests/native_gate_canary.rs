//! The native-gate integration canary. `scripts/native-gate.sh` runs this
//! target before the unfiltered suites so excluding `tests/` cannot look like a
//! successful full native gate.

#[test]
fn integration_target_is_discovered_before_the_native_suite() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("native_gate_canary.rs");
    assert!(
        source.is_file(),
        "native-gate canary must remain an integration target under tests/"
    );
}

//! Structural laws for the ReplaySession decomposition.

use std::path::{Path, PathBuf};

fn replay_sources() -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main");
    let mut paths = vec![root.join("run.rs"), root.join("replay_effects.rs")];
    paths.extend(
        std::fs::read_dir(root.join("run"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs")),
    );
    paths.sort();
    assert!(
        paths.len() >= 8,
        "replay source enrolment unexpectedly collapsed: {paths:?}"
    );
    paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).unwrap();
            (path, source)
        })
        .collect()
}

fn owners(sources: &[(PathBuf, String)], needle: &str) -> Vec<PathBuf> {
    sources
        .iter()
        .filter_map(|(path, source)| source.contains(needle).then_some(path.clone()))
        .collect()
}

#[test]
fn replay_resolution_effect_order_buffer_switching_and_trace_each_have_one_owner() {
    let sources = replay_sources();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main/run");
    let exact = |needle, file| {
        assert_eq!(
            owners(&sources, needle),
            vec![root.join(file)],
            "`{needle}` must have one replay owner"
        );
    };

    exact("EffectWorklist::root(", "chord.rs");
    exact("actions::apply_transition(", "chord.rs");
    exact("self.registry.park(", "buffers.rs");
    exact("self.registry.take(", "buffers.rs");
    exact("crate::replay::classify_for(", "trace.rs");
}

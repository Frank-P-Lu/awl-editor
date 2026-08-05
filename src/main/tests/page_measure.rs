use super::super::*;
use super::{keyspec, replay_keys};
use crate::testscratch::ScratchDir;

#[test]
fn replay_keys_page_reset_restores_default_measure() {
    let _pg = crate::testlock::serial();
    crate::page::set_measure(40);
    let mut buffer = Buffer::scratch();
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-j").unwrap();
    let mut km = crate::keymap::KeymapState::with_overrides_and_convention(
        &[("reset_page_width".into(), vec!["C-j".into()])],
        crate::convention::Convention::Mac,
    );
    let _ = super::super::replay_keys_mode(
        crate::replay::Mode::Permissive,
        crate::replay::FilesystemCapability::None,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
        &mut km,
    )
    .unwrap();
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE,
        "PageReset snaps the measure back to the built-in default"
    );
    crate::page::set_measure(crate::page::DEFAULT_MEASURE); // leave as found
}

#[test]
fn replay_keys_page_reset_restores_the_code_default_for_a_code_buffer() {
    // The prose/code page-width split: PageReset on a CODE buffer (a `.rs`
    // path) must snap to DEFAULT_MEASURE_CODE (100), never the prose default
    // (70) — `Action::PageReset` resolves via `ctx.buffer.page_class()` on
    // the shared `apply_transition` seam, so this is byte-identical to the live
    // App's own reset.
    let _pg = crate::testlock::serial();
    crate::page::set_measure(40);
    let mut buffer = Buffer::from_str("fn main() {}\n");
    buffer.set_path(PathBuf::from("/tmp/main.rs"));
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-j").unwrap();
    let mut km = crate::keymap::KeymapState::with_overrides_and_convention(
        &[("reset_page_width".into(), vec!["C-j".into()])],
        crate::convention::Convention::Mac,
    );
    let _ = super::super::replay_keys_mode(
        crate::replay::Mode::Permissive,
        crate::replay::FilesystemCapability::None,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
        &mut km,
    )
    .unwrap();
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE_CODE,
        "PageReset on a code buffer snaps to the CODE default, not the prose one"
    );
    crate::page::set_measure(crate::page::DEFAULT_MEASURE); // leave as found
}

#[test]
fn replay_keys_goto_switch_reapplies_measure_per_buffer_kind() {
    let _fs = crate::testlock::serial();
    let _pg = crate::testlock::serial();
    let measure0 = crate::page::measure();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-mb-measure-{}", std::process::id())),
    );
    std::fs::write(dir.join("a.md"), "# hello\n").unwrap();
    std::fs::write(dir.join("b.rs"), "fn main() {}\n").unwrap();
    let cfg = Config {
        page_width_prose: Some(55),
        page_width_code: Some(120),
        ..Config::empty()
    };
    let mut buffer = Buffer::scratch();
    let corpus = vec!["a.md".to_string(), "b.rs".to_string()];
    crate::page::set_measure(1); // deliberately wrong, so the switch below can't coincide

    let keys_to_b = keyspec::parse_keys("s-o b . r s RET").unwrap();
    let _ = replay_keys(&mut buffer, &keys_to_b, &corpus, &dir, None, &cfg, None);
    assert_eq!(
        crate::page::measure(),
        120,
        "b.rs (code) picks up the configured code measure"
    );

    let keys_to_a = keyspec::parse_keys("s-o a . m d RET").unwrap();
    let _ = replay_keys(&mut buffer, &keys_to_a, &corpus, &dir, None, &cfg, None);
    assert_eq!(
        crate::page::measure(),
        55,
        "back to a.md (prose) picks up the configured prose measure"
    );

    crate::page::set_measure(measure0);
}

use super::super::*;
use super::{keyspec, replay_keys, replay_keys_mode, replay_keys_mode_isolated};
use crate::testscratch::ScratchDir;

#[test]
fn strict_replay_aborts_on_an_unsupported_effect_naming_action_and_effect() {
    // Cmd-Q's `Effect::Quit` is classified Unsupported (live exits the
    // event loop; a replay would keep applying keys past it) — the strict
    // door must refuse it, naming the exact action AND effect.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-q").unwrap();
    let root = PathBuf::from("/tmp");
    let err = replay_keys_mode(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    )
    .err()
    .expect("strict replay aborts on an unsupported effect")
    .to_string();
    assert!(err.contains("`quit`"), "effect named: {err}");
    assert!(err.contains("Quit"), "action named: {err}");
    assert!(err.starts_with("strict replay:"), "{err}");
}

#[test]
fn strict_replay_records_intercepted_handoffs_without_aborting() {
    // C-c C-o on a link produces `Effect::FollowLink(url)` — an EXTERNAL
    // handoff the replay observes and records but never performs. Strict
    // must PASS it (that's the intercept contract, not a violation) and the
    // recorded intercept must carry the observed URL — the phase-5 trace seam.
    let mut buffer = Buffer::from_str("[a](https://awl.example/doc) tail");
    buffer.set_cursor(1); // inside the link
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-c C-o").unwrap();
    let res = replay_keys_mode(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    )
    .expect("intercepted handoffs are legal under strict");
    assert_eq!(
        res.intercepts,
        vec![crate::replay::Intercept {
            effect: "follow_link",
            detail: "https://awl.example/doc".into()
        }]
    );
    assert!(
        res.warnings.is_empty(),
        "strict records silently, never warns"
    );
}

#[test]
fn permissive_replay_never_aborts_and_warns_on_both_non_applied_seams() {
    let mut buffer = Buffer::from_str("[a](https://awl.example/x) tail");
    buffer.set_cursor(1);
    let keys = keyspec::parse_keys("s-q C-c C-o s-Down").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.warnings.len(),
        2,
        "one warning per crossing: {:?}",
        res.warnings
    );
    assert!(
        res.warnings[0].contains("skipped unsupported effect `quit`"),
        "{}",
        res.warnings[0]
    );
    assert!(
        res.warnings[1].contains("intercepted `follow_link`")
            && res.warnings[1].contains("https://awl.example/x"),
        "{}",
        res.warnings[1]
    );
    assert_eq!(
        res.intercepts.len(),
        1,
        "the handoff is recorded permissively too"
    );
    let (line, col) = buffer.cursor_line_col();
    assert!(
        line > 0 || col > 0,
        "the key after Quit still applied (BufferEnd moved)"
    );
}

#[test]
fn a_fully_applied_replay_stays_warning_and_intercept_free() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("a b c C-a C-e").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.warnings.is_empty(), "{:?}", res.warnings);
    assert!(res.intercepts.is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn hermetic_scenario_save_lands_in_the_sandbox_never_on_real_disk() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-hermetic-save-{}", std::process::id())),
    );
    let input = dir.join("doc.md");
    std::fs::write(&input, "alpha\n").unwrap();
    {
        let _restore = crate::fs::FsGuard::capture();
        crate::scenario::install_hermetic_fs(Some(&input), None, Some(&dir), None);
        let mut buffer = load_buffer(&Some(input.clone()));
        assert_eq!(
            buffer.text(),
            "alpha\n",
            "the sandbox seeded the real input's bytes"
        );
        let keys = keyspec::parse_keys("X s-s").unwrap();
        let res = replay_keys_mode_isolated(
            crate::replay::Mode::Strict,
            &mut buffer,
            &keys,
            &[],
            &dir,
            None,
            &Config::empty(),
            None,
        )
        .expect("an edit + save crosses no unsupported seam");
        assert!(res.intercepts.is_empty());
        assert_eq!(
            crate::fs::active().read_to_string(&input).unwrap(),
            "Xalpha\n",
            "the replayed save landed in the sandbox"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&input).unwrap(),
        "alpha\n",
        "the REAL file keeps every byte a hermetic scenario 'saved'"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn hermetic_scenario_witnesses_the_url_handoff_as_an_intercept() {
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-hermetic-link-{}", std::process::id())),
    );
    let input = dir.join("linked.md");
    let body = "[a](https://awl.example/doc) tail\n";
    std::fs::write(&input, body).unwrap();
    {
        let _restore = crate::fs::FsGuard::capture();
        crate::scenario::install_hermetic_fs(Some(&input), None, Some(&dir), None);
        let mut buffer = load_buffer(&Some(input.clone()));
        let keys = keyspec::parse_keys("Right C-c C-o").unwrap();
        let res = replay_keys_mode(
            crate::replay::Mode::Strict,
            &mut buffer,
            &keys,
            &[],
            &dir,
            None,
            &Config::empty(),
            None,
        )
        .expect("an intercepted handoff is legal under strict");
        assert_eq!(
            res.intercepts,
            vec![crate::replay::Intercept {
                effect: "follow_link",
                detail: "https://awl.example/doc".into()
            }],
            "the handoff was observed and recorded, not performed"
        );
        assert_eq!(
            crate::fs::active().read_to_string(&input).unwrap(),
            body,
            "the sandbox copy is untouched (following a link edits nothing)"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&input).unwrap(),
        body,
        "the real file too"
    );
}

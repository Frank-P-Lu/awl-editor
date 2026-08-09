use super::super::*;
use super::{keyspec, replay_keys};
// The isolated-filesystem helper is native-only, and so are the four tests here
// that call it; the import carries the same gate or wasm32 fails to resolve it.
#[cfg(not(target_arch = "wasm32"))]
use super::replay_keys_mode_isolated;

/// The settings trio's own hermetic proof, mirroring
/// `hermetic_scenario_save_lands_in_the_sandbox_never_on_real_disk`'s shape
/// for `save`: a STRICT replay (which owns Isolated filesystem authority, per
/// `ReplayPolicy::isolated`) opens the Settings picker with the real chord,
/// filters to a real row, and presses Enter — crossing no Unsupported seam —
/// then the isolated config.toml is read back to prove the write landed for
/// real, not just that the classifier says it would.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn isolated_strict_replay_toggles_a_settings_row_and_persists_it_for_real() {
    use crate::fs::{FileSystem, InMemoryFs};
    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new().with_dir("/cfg");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    let before = crate::markdown::wysiwyg_on();
    // The real Settings-menu chord (`s-,` → `Action::OpenSettingsMenu`,
    // `keymap::tests` pins this exact binding), then typing "wysiwyg" — which
    // both filters the rail to the row AND hands focus to it, the workspace's
    // own type-to-search gesture (`actions/workspace_nav.rs`) — then Enter,
    // the row's own toggle key.
    let keys = keyspec::parse_keys("s-, w y s i w y g Enter").unwrap();
    let root = PathBuf::from("/proj");
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let res = replay_keys_mode_isolated(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &cfg,
        None,
    )
    .expect("opening Settings and toggling a row crosses no unsupported seam under Isolated");
    assert!(
        res.replay_skips.is_empty(),
        "a genuinely Applied toggle must not record a skip: {:?}",
        res.replay_skips
    );
    assert_ne!(
        crate::markdown::wysiwyg_on(),
        before,
        "the picker door must flip the SAME live global its command twin flips"
    );
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .expect("the isolated config file exists");
    let expected = format!("wysiwyg = {}", !before);
    assert!(
        written.lines().any(|l| l.trim() == expected),
        "the isolated config must carry {expected:?} — config was:\n{written}"
    );
    crate::markdown::set_wysiwyg_on(before);
}

#[test]
fn ordinary_replay_save_on_scratch_is_nonmutating_and_records_the_skip() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    assert!(buffer.path().is_none() && !buffer.is_unnamed_fresh());
    let keys = keyspec::parse_keys("m e a d o w s-s").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        buffer.path().is_none(),
        "ordinary replay must not promote scratch into a real document"
    );
    assert!(
        !mem.exists(&root),
        "ordinary replay must not create a file under the active root"
    );
    assert_eq!(res.replay_skips.len(), 1);
    assert_eq!(res.replay_skips[0].effect, "save");
}

/// The Unsupported half: ORDINARY `--keys` (no capability) still
/// cannot witness a settings change — the same real chord as the isolated
/// proof above, replayed PERMISSIVE/`FilesystemCapability::None`, must skip
/// and warn rather than silently pretend the value moved.
#[test]
fn ordinary_replay_settings_toggle_is_nonmutating_and_records_the_skip() {
    use crate::fs::{FileSystem, InMemoryFs};
    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new().with_dir("/cfg");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    let before = crate::markdown::wysiwyg_on();
    let keys = keyspec::parse_keys("s-, w y s i w y g Enter").unwrap();
    let root = PathBuf::from("/proj");
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &cfg, None);
    assert_eq!(
        crate::markdown::wysiwyg_on(),
        before,
        "ordinary replay owns no filesystem capability; the global must not move"
    );
    assert!(
        mem.read_to_string(std::path::Path::new("/cfg/config.toml"))
            .is_err(),
        "ordinary replay must not write the config file at all"
    );
    assert_eq!(res.replay_skips.len(), 1);
    assert_eq!(res.replay_skips[0].effect, "setting_toggle");
}

/// The CONFIG-ONLY toggle branch (`autosave`/`history`/
/// `session_restore` have no live process global — the disk write IS the
/// whole effect, per `App::setting_toggle`'s own doc). A companion to the
/// `wysiwyg` proof above, which only exercises the global-backed branch.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn isolated_strict_replay_toggles_a_config_only_settings_row_and_persists_it() {
    use crate::fs::{FileSystem, InMemoryFs};
    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new().with_dir("/cfg");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-, a u t o s a v e Enter").unwrap();
    let root = PathBuf::from("/proj");
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let res = replay_keys_mode_isolated(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &cfg,
        None,
    )
    .expect("toggling a config-only row crosses no unsupported seam under Isolated");
    assert!(res.replay_skips.is_empty(), "{:?}", res.replay_skips);
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .expect("the isolated config file exists");
    assert!(
        written.lines().any(|l| l.trim() == "autosave = false"),
        "the isolated config must carry the negated value — config was:\n{written}"
    );
}

/// `SettingValueCommit`'s own hermetic proof: the Zoom row's exact
/// numeric entry (Enter opens it seeded with the current readout — retyping
/// over it is the point, per `settings::SettingsValues`'s own doc — so the
/// field is cleared with Backspaces first, mirroring the path-picker sweep).
/// A Strict/Isolated replay commits it for real: the process-global zoom AND
/// the persisted config key both move, onto the SAME authored step grid the
/// live `App::setting_value_commit` clamps onto.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn isolated_strict_replay_commits_a_range_rows_exact_value_and_persists_it() {
    use crate::fs::{FileSystem, InMemoryFs};
    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new().with_dir("/cfg");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys(
        "s-, z o o m Enter Backspace Backspace Backspace Backspace 1 2 5 Enter",
    )
    .unwrap();
    let root = PathBuf::from("/proj");
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let res = replay_keys_mode_isolated(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &cfg,
        None,
    )
    .expect("committing an exact zoom value crosses no unsupported seam under Isolated");
    assert!(res.replay_skips.is_empty(), "{:?}", res.replay_skips);
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .expect("the isolated config file exists");
    assert!(
        written.lines().any(|l| l.trim().starts_with("zoom = ")),
        "the isolated config must carry a zoom line — config was:\n{written}"
    );
}

/// `SettingPathPick`'s own hermetic proof, over its simplest key
/// (`default_folder`: a plain persisted path, no further live re-scope). The
/// `.` row at the top of the real folder navigator accepts the level you are
/// standing in — the exact interaction `sweep_path` drives live.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn isolated_strict_replay_picks_a_settings_path_and_persists_it() {
    use crate::fs::{FileSystem, InMemoryFs};
    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new()
        .with_dir("/cfg")
        .with_dir("/ws")
        .with_dir("/ws/proj");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    // Filter the rail to "Default folder", Enter opens the real folder
    // navigator standing at the WORKSPACE (the project root's parent — the
    // same level `sweep_path` picks from); `Up` reaches its
    // synthetic `.` row (the level you are standing in), `Enter` accepts it.
    let keys = keyspec::parse_keys("s-, d e f a u l t Space f o l d e r Enter Up Enter").unwrap();
    let root = PathBuf::from("/ws/proj");
    let cfg = Config {
        path: PathBuf::from("/cfg/config.toml"),
        ..Config::empty()
    };
    let res = replay_keys_mode_isolated(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &cfg,
        None,
    )
    .expect("picking a default-folder path crosses no unsupported seam under Isolated");
    assert!(res.replay_skips.is_empty(), "{:?}", res.replay_skips);
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .expect("the isolated config file exists");
    assert!(
        written
            .lines()
            .any(|l| l.trim() == "default_folder = \"/ws\""),
        "the isolated config must carry the picked workspace folder — config was:\n{written}"
    );
}

#[test]
fn ordinary_replay_paste_never_reads_or_writes_an_image_and_uses_text_fallback() {
    use crate::fs::{FileSystem, InMemoryFs};

    let _guard = crate::testlock::serial();
    let mem = InMemoryFs::new()
        .with_dir("/proj")
        .with_file("/proj/a.md", "before\n");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::from_file(std::path::Path::new("/proj/a.md"));
    buffer.set_kill("fallback");
    let keys = keyspec::parse_keys("s-v").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &[],
        std::path::Path::new("/proj"),
        None,
        &Config::empty(),
        None,
    );
    assert_eq!(buffer.text(), "fallbackbefore\n");
    assert!(
        !mem.exists(std::path::Path::new("/proj/assets")),
        "ordinary replay has no image-filesystem authority"
    );
    assert!(
        res.intercepts
            .iter()
            .any(|effect| effect.effect == "clipboard_paste_image"),
        "the external image probe is recorded, never performed"
    );
}

#[test]
fn ordinary_replay_save_on_pathed_buffer_never_writes() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new()
        .with_dir("/proj")
        .with_file("/proj/a.md", "before\n");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::from_file(std::path::Path::new("/proj/a.md"));
    let keys = keyspec::parse_keys("h i s-s").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(buffer.path(), Some(std::path::Path::new("/proj/a.md")));
    assert_eq!(
        mem.read_to_string(std::path::Path::new("/proj/a.md"))
            .unwrap(),
        "before\n",
        "the in-session edit must never escape through ordinary replay"
    );
    assert_eq!(res.replay_skips[0].effect, "save");
}

#[test]
fn ordinary_replay_finish_never_writes_notifies_or_switches() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new()
        .with_dir("/proj")
        .with_file("/proj/a.md", "before\n");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::from_file(std::path::Path::new("/proj/a.md"));
    let root = PathBuf::from("/proj");
    let keys = keyspec::parse_keys("X C-j").unwrap();
    let mut km = crate::keymap::KeymapState::with_overrides_and_convention(
        &[("finish_file".into(), vec!["C-j".into()])],
        crate::convention::Convention::Mac,
    );
    let res = super::super::replay_keys_mode(
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
        mem.read_to_string(std::path::Path::new("/proj/a.md"))
            .unwrap(),
        "before\n"
    );
    assert_eq!(buffer.text(), "Xbefore\n", "the in-session edit remains");
    assert_eq!(
        res.intercepts,
        vec![crate::replay::Intercept {
            effect: "daemon_notify_finished",
            detail: String::new(),
        }],
        "the daemon request is observed, never performed"
    );
    let skipped: Vec<_> = res.replay_skips.iter().map(|s| s.effect).collect();
    assert_eq!(skipped, ["finish_save", "finish_buffer"]);
}

#[test]
fn ordinary_replay_open_settings_never_materializes_an_absent_config() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new().with_dir("/cfg");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut config = Config::empty();
    config.path = PathBuf::from("/cfg/config.toml");
    let mut buffer = Buffer::from_str("stay");
    let root = PathBuf::from("/proj");
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = ReplaySession::new(
        ReplayPolicy::ordinary(),
        &mut buffer,
        &[],
        &root,
        None,
        &config,
        None,
        &mut km,
    );
    assert!(session.interpret_headless_effect(&actions::Effect::Buffer(
        actions::BufferEffect::OpenSettings
    )));
    assert_eq!(session.buffer().text(), "stay");
    assert!(
        !mem.exists(std::path::Path::new("/cfg/config.toml")),
        "ordinary replay owns no authority to create config.toml"
    );
}

use super::super::*;
use crate::testscratch::ScratchDir;

#[test]
fn resolve_root_explicit_flag_wins_over_file() {
    let flag = PathBuf::from("/flag/root");
    let file = PathBuf::from("/some/file.txt");
    assert_eq!(resolve_root(&Some(flag.clone()), &Some(file)), flag);
}

#[test]
fn resolve_root_file_argument_resolves_from_its_own_directory() {
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-resolve-root-{}", std::process::id())),
    );
    let file = dir.join("note.txt");
    std::fs::write(&file, "hi").unwrap();
    assert_eq!(resolve_root(&None, &Some(file)), dir.to_path_buf());
}

#[test]
fn resolve_root_bare_falls_to_cwd() {
    // `resolve_root` alone (the explicit-only half) never consults a
    // remembered folder or a default — that's `resolve_launch_context`'s
    // job. Its own bare fallback stays cwd, unchanged.
    //
    // TWO reads of the process-CWD global (ours + `resolve_root`'s own), so
    // the guard is what makes them the SAME cwd — a `CwdGuard` landing
    // between them would otherwise compare two different directories
    // unless the guard holds.
    let _tg = crate::testlock::serial();
    let cwd = crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    assert_eq!(resolve_root(&None, &None), cwd);
}

#[test]
fn resolve_launch_context_explicit_root_wins_over_remembered_and_file() {
    let flag = PathBuf::from("/flag/root");
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    let file = PathBuf::from("/some/file.txt");
    assert_eq!(
        resolve_launch_context(
            &Some(flag.clone()),
            &Some(file),
            Some(&remembered),
            &default_folder,
            true,
        ),
        flag
    );
}

#[test]
fn resolve_launch_context_file_argument_wins_over_remembered() {
    let _tg = crate::testlock::serial();
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-launch-ctx-file-{}", std::process::id())),
    );
    let file = dir.join("note.txt");
    std::fs::write(&file, "hi").unwrap();
    assert_eq!(
        resolve_launch_context(&None, &Some(file), Some(&remembered), &default_folder, true),
        dir.to_path_buf()
    );
}

#[test]
fn resolve_launch_context_dir_argument_awl_dot_is_explicit_not_remembered() {
    // `awl .` — a DIR argument — is door 1 (explicit), the crisp
    // bare-vs-dot distinction: it must win over whatever is remembered,
    // exactly like a file argument does.
    //
    // THE VICTIM: `resolve_root` decides "is this
    // argument a directory?" through `fs::active().is_dir(f)`. Without
    // this guard the probe could land on a sibling test's `InMemoryFs`,
    // which knows nothing of this real temp dir — `is_dir` came back
    // false, the dir argument decayed to its PARENT (`/tmp`), and the
    // assertion below failed under parallel load.
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-launch-ctx-dot-{}", std::process::id())),
    );
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(
            &None,
            &Some(dir.to_path_buf()),
            Some(&remembered),
            &default_folder,
            true,
        ),
        dir.to_path_buf()
    );
}

#[test]
fn resolve_launch_context_bare_launch_restores_remembered() {
    let remembered = PathBuf::from("/home/me/work/repo-a");
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(&None, &None, Some(&remembered), &default_folder, true),
        remembered
    );
}

#[test]
fn resolve_launch_context_first_run_uses_only_an_explicit_default_folder() {
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(&None, &None, None, &default_folder, true),
        default_folder
    );
    assert_eq!(
        resolve_launch_context(&None, &None, None, &default_folder, false),
        crate::fs::data_root(),
        "an implicit ~/notes fallback is never a first-launch destination"
    );
    let _tg = crate::testlock::serial();
    let cwd = crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    assert_ne!(default_folder, cwd);
}

#[test]
fn capture_mode_bare_invocation_never_restores_a_remembered_folder() {
    // The CAPTURE-GATE LAW: headless capture is structurally free of
    // session state (the old sticky `config.project_root` is folded
    // into the live-App-only session) — a bare `--screenshot` (no file,
    // no --root) always falls to cwd via the explicit-only `resolve_root`,
    // never a remembered/default folder, regardless of what the config
    // carries. Reads the REAL disk (Project::resolve / build_index walk
    // it) -> hold the fs TEST_LOCK like the other real-fs test in this
    // module.
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-capture-bare-{}", std::process::id())),
    );
    let _cwd_guard = crate::fs::CwdGuard::enter(&dir);
    let cwd = crate::fs::current_dir().unwrap();
    let config = Config {
        default_folder: Some(PathBuf::from("/should/never/be/read")),
        ..Config::empty()
    };
    let out = dir.join("cap.png");
    let default_folder = dir.join("notes");
    let result = capture_screenshot(
        out.clone(),
        None, // no file argument: a bare capture
        CaptureOpts::default(),
        Vec::new(),
        crate::keymap::KeymapState::new(),
        None, // no explicit --root
        None,
        default_folder,
        config,
        false, // permissive (the legacy default)
    );
    result.expect("capture succeeds");
    let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["project"]["root"].as_str().unwrap(),
        // Through the sidecar's own home redaction, so this stays a law about
        // WHICH root was resolved on a host whose temp dir sits under `$HOME`
        // too, rather than a second, accidental assertion about spelling.
        crate::capture::redact::redact(&cwd.to_string_lossy()),
        "sidecar project.root reflects cwd, never the config default_folder"
    );
}

/// THE LEAK LAW (wired, real door, nothing seeded): a capture taken with no
/// `--root`, no `--workspace` and the DEFAULT default-folder writes a sidecar
/// that contains this machine's home path NOWHERE — and still reports the folder
/// it resolved, spelled `~/…`.
///
/// The leak this closes is not a typo anyone can be careful about: a lane does
/// not have to WRITE a path to publish one, it only has to capture, and the repo
/// is public. `capture::redact` is the mechanism; this is the law that fails if
/// the mechanism is removed. The absence half sweeps the WHOLE artifact rather
/// than a named block, so any future block that leaks fails here too.
///
/// The subject of the presence half is `project.default_folder` deliberately: it
/// falls back to `$HOME/notes` whether or not anything is configured, so it is
/// the one home path present in EVERY unseeded capture on every developer
/// machine — independent of where this checkout happens to sit, which the root
/// and workspace are not. The field-by-field sweep is
/// `capture::tests::redact_law::every_path_bearing_sidecar_field_is_home_relative`.
#[test]
fn a_capture_taken_under_a_non_seeded_root_leaks_no_home_path() {
    let home = crate::fs::home_dir();
    let Some(home) = home.filter(|h| crate::capture::redact::is_redactable(h)) else {
        // A configuration this law cannot enrol in, named rather than passed
        // silently: `$HOME` unset, or a single-component root like `/root` that
        // cannot be told from an ordinary path.
        eprintln!(
            "skipping a_capture_taken_under_a_non_seeded_root_leaks_no_home_path: \
             $HOME is unset or too generic to strip ({:?})",
            crate::fs::home_dir()
        );
        return;
    };
    let _fs = crate::testlock::serial();
    // The artifact is written OUTSIDE the tree the capture reports on, so
    // nothing the capture emits lands where the capture is looking.
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-402-unseeded-{}", std::process::id())),
    );
    let _cwd_guard = crate::fs::CwdGuard::enter(&dir);
    let out = dir.join("cap.png");
    // Nothing seeded: no `--root`, no `--workspace`, and the default folder
    // resolved by the SAME owner a flagless launch uses.
    let default_folder = crate::args::resolve_default_folder(&None);
    assert!(
        default_folder.starts_with(&home),
        "premise: an unseeded default folder is under $HOME (got {})",
        default_folder.display()
    );
    capture_screenshot(
        out.clone(),
        None, // no file argument: a bare capture
        CaptureOpts::default(),
        Vec::new(),
        crate::keymap::KeymapState::new(),
        None, // no --root
        None, // no --workspace
        default_folder.clone(),
        Config::empty(),
        false,
    )
    .expect("capture succeeds");

    let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let home_str = home.to_string_lossy().to_string();
    assert!(
        !json.contains(&home_str),
        "a sidecar from a non-seeded capture must carry no absolute home path — \
         found {home_str:?} in {}",
        out.with_extension("json").display()
    );

    // PRESENCE, beside the absence: the field is still there and still names the
    // folder. Without this half the assertion above is satisfied by a serializer
    // that simply drops every path it cannot sanitise.
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let reported = v["project"]["default_folder"]
        .as_str()
        .expect("project.default_folder is still reported, not dropped to null");
    assert_eq!(
        reported,
        crate::capture::redact::redact(&default_folder.to_string_lossy()),
        "the sidecar must still say WHICH folder it resolved, home-relative"
    );
    assert!(
        reported.starts_with("~/"),
        "an under-home path is reported under ~/ (got {reported:?})"
    );
}

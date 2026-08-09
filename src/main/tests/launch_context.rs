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
        cwd.to_string_lossy(),
        "sidecar project.root reflects cwd, never the config default_folder"
    );
}

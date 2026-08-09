use super::super::*;

/// SAME-PARENT EDGE: switching
/// between two projects that share a parent must still rebuild `corpus` from
/// the NEW root — a fix that only re-derives `workspace` when the parent
/// visibly changes would leave this case green by coincidence. Calls
/// `resync_project_location` directly (this test lives in `run::tests`, a
/// descendant module, so the private fields are readable) to isolate the
/// re-scoping mechanism from chord resolution, which the law above already
/// covers end to end.
#[test]
fn resync_project_location_same_parent_switch_still_rebuilds_the_corpus() {
    use std::sync::Arc;
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/ws/proj-a/keep.md", "keep")
            .with_file("/ws/proj-b/target.md", "target"),
    );
    crate::fs::with_fs(mem, || {
        let mut buffer = Buffer::scratch();
        let root = PathBuf::from("/ws/proj-a");
        let config = Config::empty();
        let mut km =
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
        let mut session = ReplaySession::new(
            ReplayPolicy::ordinary(),
            &mut buffer,
            &["keep.md".to_string()],
            &root,
            None,
            &config,
            None,
            &mut km,
        );
        assert_eq!(session.workspace, PathBuf::from("/ws"));
        session.resync_project_location(PathBuf::from("/ws/proj-b"));
        assert_eq!(session.root, PathBuf::from("/ws/proj-b"));
        assert_eq!(
            session.workspace,
            PathBuf::from("/ws"),
            "the parent is unchanged, but freshly re-derived, not merely untouched"
        );
        assert_eq!(
            session.corpus,
            vec!["target.md".to_string()],
            "a same-parent switch must still rebuild the corpus from the NEW root; \
             the stale ['keep.md'] here would be a half-fix that only watches workspace"
        );
    });
}

/// `SettingPathPick{key: "project_root"}` re-scopes root/workspace/
/// corpus through the SAME `resync_project_location` owner a Project-picker
/// accept already uses — a white-box companion to the black-box
/// hermetic proofs above, exercising the interpreter directly (mirroring
/// `resync_project_location_same_parent_switch_still_rebuilds_the_corpus`'s
/// own construction) since `ReplayResult` carries no root/workspace/corpus
/// field to assert against through the public `--keys` door.
#[test]
fn setting_path_pick_project_root_resyncs_root_workspace_and_corpus() {
    use std::sync::Arc;
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/ws/proj-a/keep.md", "keep")
            .with_file("/ws/proj-b/target.md", "target"),
    );
    crate::fs::with_fs(mem, || {
        let mut buffer = Buffer::scratch();
        let root = PathBuf::from("/ws/proj-a");
        let config = Config::empty();
        let mut km =
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
        let mut session = ReplaySession::new(
            ReplayPolicy::isolated(),
            &mut buffer,
            &["keep.md".to_string()],
            &root,
            None,
            &config,
            None,
            &mut km,
        );
        session.interpret_setting_path_pick("project_root", "/ws/proj-b");
        assert_eq!(session.root, PathBuf::from("/ws/proj-b"));
        assert_eq!(session.workspace, PathBuf::from("/ws"));
        assert_eq!(
            session.corpus,
            vec!["target.md".to_string()],
            "the corpus must be rebuilt from the NEW root, exactly like a Project accept"
        );
    });
}

/// `SettingPathPick{key: "workspace"}` persists the picked folder
/// AND re-derives this session's own `workspace` (root unchanged) so a chord
/// applied afterward reads the new scope — the one observable slice of live
/// `App::reload_config`'s work for this key.
#[test]
fn setting_path_pick_workspace_persists_and_resyncs_the_workspace_field() {
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_file("/ws/proj-a/keep.md", "keep")
            .with_dir("/elsewhere"),
    );
    crate::fs::with_fs(mem.clone(), || {
        let mut buffer = Buffer::scratch();
        let root = PathBuf::from("/ws/proj-a");
        let config = Config {
            path: PathBuf::from("/cfg/config.toml"),
            ..Config::empty()
        };
        let mut km =
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
        let mut session = ReplaySession::new(
            ReplayPolicy::isolated(),
            &mut buffer,
            &["keep.md".to_string()],
            &root,
            None,
            &config,
            None,
            &mut km,
        );
        assert_eq!(session.workspace, PathBuf::from("/ws"));
        session.interpret_setting_path_pick("workspace", "/elsewhere");
        assert_eq!(
            session.workspace,
            PathBuf::from("/elsewhere"),
            "the session's own workspace scope must move to the picked folder"
        );
        assert_eq!(
            session.root,
            PathBuf::from("/ws/proj-a"),
            "picking the workspace folder must not re-scope the active project root"
        );
    });
    let written = mem
        .read_to_string(std::path::Path::new("/cfg/config.toml"))
        .expect("the isolated config file exists");
    assert!(
        written
            .lines()
            .any(|l| l.trim() == "workspace = \"/elsewhere\""),
        "the isolated config must carry the picked workspace — config was:\n{written}"
    );
}

/// NO-PARENT / FILESYSTEM-ROOT EDGE: `Path::parent()` returns `None` only for
/// a root component itself, so
/// switching TO the filesystem root is the one case that exercises
/// `location::resolve_workspace`'s fallback-to-self arm inside a re-scope,
/// not just at the free-function level (`workspace_falls_back_to_root_when_
/// no_parent` above). A half-fix that leaves the OLD workspace in place when
/// the new root has no parent would pass every other test and fail only here.
#[test]
fn resync_project_location_no_parent_root_falls_back_to_itself_not_the_old_workspace() {
    use std::sync::Arc;
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_file("/ws/proj-a/keep.md", "keep")
            .with_file("/marker.md", "marker"),
    );
    crate::fs::with_fs(mem, || {
        let mut buffer = Buffer::scratch();
        let root = PathBuf::from("/ws/proj-a");
        let config = Config::empty();
        let mut km =
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
        let mut session = ReplaySession::new(
            ReplayPolicy::ordinary(),
            &mut buffer,
            &["keep.md".to_string()],
            &root,
            None,
            &config,
            None,
            &mut km,
        );
        assert_eq!(session.workspace, PathBuf::from("/ws"));
        session.resync_project_location(PathBuf::from("/"));
        assert_eq!(session.root, PathBuf::from("/"));
        assert_eq!(
            session.workspace,
            PathBuf::from("/"),
            "no parent: the workspace must fall back to the root itself, not stay \
             stuck at the old `/ws`"
        );
        assert!(
            session.corpus.contains(&"marker.md".to_string()),
            "corpus rebuilt from the new (rootless-parent) root: {:?}",
            session.corpus
        );
    });
}

#[test]
fn permissive_skip_sweeps_the_four_known_live_only_effects() {
    let cases = [
        (
            Action::Newline,
            actions::Effect::OverlayAccept(crate::overlay::OverlayKind::MoveDest, "archive".into()),
            "overlay_accept",
        ),
        (
            Action::Newline,
            actions::Effect::RenameNoteCommit {
                new_name: "renamed.md".into(),
            },
            "rename_note_commit",
        ),
        (
            Action::DuplicateNote,
            actions::Effect::DuplicateNote,
            "duplicate_note",
        ),
        (
            Action::Newline,
            actions::Effect::SettingPathPick {
                key: "default_folder".into(),
                path: "/notes".into(),
            },
            "setting_path_pick",
        ),
    ];
    for (action, effect, expected) in cases {
        let skip = crate::replay::permissive_skip(&action, &crate::replay::classify(&effect))
            .expect("known live-only effect records a skip");
        assert_eq!(skip.effect, expected);
        assert_eq!(skip.action, format!("{action:?}"));
    }
    assert!(
        crate::replay::permissive_skip(
            &Action::InsertChar('x'),
            &crate::replay::classify(&actions::Effect::None),
        )
        .is_none()
    );
}

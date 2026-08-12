use super::*;

#[test]
fn switch_marks_git_children() {
    // Project picker: repo-alpha/repo-beta are git, plain-notes is not.
    let corpus = vec![
        "plain-notes".to_string(),
        "repo-alpha".to_string(),
        "repo-beta".to_string(),
    ];
    let git = vec![false, true, true];
    let is_dir = vec![true, true, true];
    let mut ov = OverlayState::new_marked(
        OverlayKind::Project,
        corpus,
        git,
        is_dir,
        vec![],
        vec![],
        None,
    );
    let items = ov.item_strings();
    // The NAME column carries no git marker any more — a clean folder name (+ `/`).
    assert!(
        items.iter().all(|s| !s.contains('•')),
        "no bullet in names: {items:?}"
    );
    // Git repos carry a `"git"` SECONDARY-column tag, parallel to `items`; a plain
    // folder's slot is empty.
    let tags = ov.item_git_tags();
    assert_eq!(tags.len(), items.len(), "git tags parallel to rows");
    let tag_of = |name: &str| {
        let pos = items.iter().position(|s| s.contains(name)).unwrap();
        tags[pos].as_str()
    };
    assert_eq!(tag_of("repo-alpha"), "git");
    assert_eq!(tag_of("repo-beta"), "git");
    assert_eq!(tag_of("plain-notes"), "", "plain folder carries no git tag");
    // plain-notes is a plain folder: trailing slash, no marker.
    let pn = items.iter().find(|s| s.contains("plain-notes")).unwrap();
    assert!(pn.ends_with('/'));
    // The accept value is always the RAW name (no marker).
    assert_eq!(
        ov.rows[ov.selected_corpus_index().unwrap()].accept,
        "plain-notes"
    );

    // PRESERVATION LAW (git/dir survive reorder): a typed query that
    // RE-RANKS the rows must never let a git/dir marker follow the wrong row —
    // identity now travels as ONE `OverlayRow`, not a separate array read by a
    // (potentially shuffled) index. "repo" out-ranks "plain-notes" for both git
    // folders, reordering the items; every reordered repo row must still carry
    // its own git tag and directory-trailing slash.
    ov.push('r');
    ov.push('e');
    ov.push('p');
    ov.push('o');
    let items2 = ov.item_strings();
    let tags2 = ov.item_git_tags();
    assert_eq!(
        tags2.len(),
        items2.len(),
        "git tags stay parallel after reorder"
    );
    assert!(
        items2.iter().any(|s| s.starts_with("repo")),
        "the query genuinely reordered toward the repo rows: {items2:?}"
    );
    for (i, s) in items2.iter().enumerate() {
        assert!(
            s.ends_with('/'),
            "every surviving row is still a directory: {s:?}"
        );
        if s.contains("repo") {
            assert_eq!(
                tags2[i], "git",
                "reordered {s:?} keeps its OWN git tag, not a neighbor's"
            );
        }
    }
}

#[test]
fn new_project_pins_accept_row_and_marks_git() {
    // Folders for the explorer level: a plain folder + two git repos.
    let folders = vec![
        ("plain-notes".to_string(), false),
        ("repo-alpha".to_string(), true),
        ("repo-beta".to_string(), true),
    ];
    let ov = OverlayState::new_project("/ws".to_string(), folders, &[]);
    assert_eq!(ov.kind.as_str(), "switch");
    // The synthetic "." accept-this-folder row is pinned at the TOP.
    let items = ov.item_strings();
    assert_eq!(items[0], ".");
    // browse_dir carries the ABSOLUTE dir for path navigation.
    assert_eq!(ov.browse_dir.as_deref(), Some("/ws"));
    // Default selection skips "." and lands on the first REAL folder so
    // Enter descends into it immediately (the "." select row is Up).
    assert_eq!(ov.selected_value(), Some("plain-notes"));
    assert!(ov.selected_is_dir(), "first folder is a directory");
    // Git children carry the `"git"` SECONDARY tag (not a name bullet); "." is
    // neither git nor a dir, and no name carries a bullet.
    assert!(
        items.iter().all(|s| !s.contains('•')),
        "no name bullet: {items:?}"
    );
    let tags = ov.item_git_tags();
    let alpha = items.iter().position(|s| s.contains("repo-alpha")).unwrap();
    assert_eq!(tags[alpha], "git");
    assert_eq!(tags[0], "", "the '.' accept row is never git-tagged");
    assert!(!items[0].ends_with('/'));
}

#[test]
fn project_hides_dotfolders_but_keeps_accept_row_and_env() {
    // A workspace level with dotfolders (.git/.claude), an .env, and plain folders.
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let folders = vec![
        (".git".to_string(), false),
        (".claude".to_string(), false),
        (".env".to_string(), false),
        ("src".to_string(), false),
        ("repo".to_string(), true),
    ];
    let mut ov = OverlayState::new_project("/ws".to_string(), folders, &[]);
    // Project now HIDES dotfolders by default (the Batch dotfile filter extended to
    // it), while the synthetic "." accept-this-folder row and `.env` (the earned
    // exception) stay visible.
    assert!(ov.kind.hides_dotfiles(), "Project hides dotfiles now");
    let shown = ov.item_strings();
    assert!(
        shown.iter().any(|s| s == "."),
        "the '.' accept row survives: {shown:?}"
    );
    assert!(
        !shown.iter().any(|s| s.starts_with(".git")),
        ".git hidden: {shown:?}"
    );
    assert!(
        !shown.iter().any(|s| s.starts_with(".claude")),
        ".claude hidden: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s.starts_with(".env")),
        ".env stays visible: {shown:?}"
    );
    assert!(shown.iter().any(|s| s.starts_with("src")));
    assert!(shown.iter().any(|s| s.starts_with("repo")));
    // The `.env` folder is not git, so its secondary tag is empty; the repo carries
    // the "git" tag — and no dotfolder-tag leaks (they are filtered out entirely).
    let tags = ov.item_git_tags();
    let repo_i = shown.iter().position(|s| s.starts_with("repo")).unwrap();
    assert_eq!(tags[repo_i], "git");
    // File visibility: All reveals the dotfolders for Project too.
    crate::file_visibility::set_all_on(true);
    ov.refilter();
    let revealed = ov.item_strings();
    assert!(
        revealed.iter().any(|s| s.starts_with(".git")),
        "revealed: {revealed:?}"
    );
    assert!(
        revealed.iter().any(|s| s.starts_with(".claude")),
        "revealed: {revealed:?}"
    );
    assert!(
        revealed.iter().any(|s| s == "."),
        "'.' still present after reveal"
    );
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn project_picker_has_an_all_recent_strip_and_lands_on_all() {
    // The switch-project navigator now FACETS: All (the flat workspace-folder
    // listing, the home) · Recent (the recent-projects MRU). It LANDS on All.
    let folders = vec![("proj-a".to_string(), true), ("proj-b".to_string(), false)];
    let ov = OverlayState::new_project("/ws".to_string(), folders, &[]);
    assert!(ov.is_faceting(), "Project facets now");
    let strip: Vec<String> = ov.lens_strip().into_iter().map(|(l, _)| l).collect();
    assert_eq!(strip, vec!["All".to_string(), "Recent".to_string()]);
    // HOME LAW: All is FIRST and the picker lands on it (the flat list).
    assert_eq!(ov.active_facet_id(), Some("all"));
    assert_eq!(
        ov.lens_strip().first().map(|(_, a)| *a),
        Some(true),
        "All is active on open"
    );
    // The synthetic "." select-this-folder row survives under All (flat home).
    assert!(
        ov.item_strings().iter().any(|s| s == "."),
        "'.' survives under All"
    );
}

#[test]
fn project_recent_lens_shows_only_mru_projects_in_mru_order() {
    // The workspace level lists three folders; the recent-PROJECTS MRU (absolute
    // paths, most-recent first) names two of them, out of listing order.
    let folders = vec![
        ("proj-a".to_string(), false), // corpus 1 — in the MRU (2nd most recent)
        ("proj-b".to_string(), false), // corpus 2 — NOT in the MRU
        ("proj-c".to_string(), false), // corpus 3 — in the MRU (most recent)
    ];
    // MRU: proj-c is most recent, then proj-a. A stale root elsewhere opts out.
    let recent = vec![
        "/ws/proj-c".to_string(),
        "/ws/proj-a".to_string(),
        "/elsewhere/gone".to_string(),
    ];
    let mut ov = OverlayState::new_project("/ws".to_string(), folders, &recent);
    // Switch to the Recent lens (strip index 1).
    ov.set_facet_lens(1);
    assert_eq!(ov.active_facet_id(), Some("recent"));
    // ONLY the two MRU folders show, in MRU order (proj-c before proj-a) — the
    // "." row and the non-MRU proj-b opt out.
    // (Folder rows render with a trailing "/" in the display strings.)
    assert_eq!(
        ov.item_strings(),
        vec!["proj-c/".to_string(), "proj-a/".to_string()]
    );
    // Every surviving row sits under the single "Recent" section header.
    assert!(ov.item_sections().iter().all(|s| s == "Recent"));
}

#[test]
fn project_recent_lens_is_empty_on_a_fresh_session() {
    // Nothing switched-to yet → empty MRU → the Recent lens lists NOTHING (shows
    // its "no recent projects yet" empty state), never the whole workspace.
    let folders = vec![("proj-a".to_string(), false), ("proj-b".to_string(), false)];
    let mut ov = OverlayState::new_project("/ws".to_string(), folders, &[]);
    ov.set_facet_lens(1);
    assert_eq!(ov.active_facet_id(), Some("recent"));
    assert!(
        ov.item_strings().is_empty(),
        "Recent is empty with no recent projects: {:?}",
        ov.item_strings()
    );
    // The warm empty-lens wording invites rather than reports.
    assert_eq!(
        OverlayKind::Project.empty_lens_message("recent"),
        Some("no recent projects yet")
    );
}

// ─── Roster derivation: one directory-level read ────────────────────────────
//
// `new_project` above is fed a hand-written `folders` list, so it cannot by
// itself prove the roster stays flat — that guarantee lives one seam lower, in
// `crate::overlay::browse_level` / `crate::index::list_dir_level`, which is
// what actually walks the disk. These sweep the directory-SHAPE axis: ordinary
// child, git child, grandchild (must be excluded), an empty child, a
// workspace with no children at all, and — on a real disk, where links exist
// — a link to a folder, a link to a file, a broken link, a link loop and a
// link back at an ancestor. Each cell names what enrolled (or didn't) in its
// own failure message.

#[test]
fn project_roster_sweeps_ordinary_git_grandchild_and_empty_children() {
    let ws = std::path::PathBuf::from("/ws");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(ws.join("ordinary"))
        .with_dir(ws.join("gitrepo"))
        .with_dir(ws.join("gitrepo/.git")) // marks gitrepo a git repo
        .with_dir(ws.join("ordinary/grandchild")) // one level below a direct child
        .with_dir(ws.join("empty")); // a direct child with nothing inside it
    let _guard = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let ov = crate::overlay::browse_level(
        OverlayKind::Project,
        None,
        std::path::Path::new("/unused"),
        Some(&ws),
        &[],
    )
    .expect("a configured workspace always builds a Project overlay");
    let shown = ov.item_strings();
    for name in ["ordinary", "gitrepo", "empty"] {
        assert!(
            shown.iter().any(|s| s.starts_with(name)),
            "direct child {name:?} must enrol: {shown:?}"
        );
    }
    assert!(
        !shown.iter().any(|s| s.contains("grandchild")),
        "a grandchild must never enrol, however deep it lives: {shown:?}"
    );
    // The git marker stays informational, never an enrolment gate — an
    // ordinary child and a git child both enrol, only their secondary tag
    // differs.
    let tags = ov.item_git_tags();
    let git_i = shown.iter().position(|s| s.starts_with("gitrepo")).unwrap();
    assert_eq!(
        tags[git_i], "git",
        "gitrepo is tagged, not filtered on: {shown:?}"
    );
    let plain_i = shown
        .iter()
        .position(|s| s.starts_with("ordinary"))
        .unwrap();
    assert_eq!(
        tags[plain_i], "",
        "ordinary is untagged but still enrolled: {shown:?}"
    );
    let empty_i = shown.iter().position(|s| s.starts_with("empty")).unwrap();
    assert_eq!(
        tags[empty_i], "",
        "an empty child enrols on equal footing: {shown:?}"
    );
}

#[test]
fn project_roster_on_a_childless_workspace_is_just_the_accept_row() {
    let ws = std::path::PathBuf::from("/empty-ws");
    let mem = crate::fs::InMemoryFs::new().with_dir(&ws);
    let _guard = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let ov = crate::overlay::browse_level(
        OverlayKind::Project,
        None,
        std::path::Path::new("/unused"),
        Some(&ws),
        &[],
    )
    .expect("a configured workspace with zero children still builds a picker");
    assert_eq!(
        ov.item_strings(),
        vec![".".to_string()],
        "a childless workspace shows only the accept-this-folder row, not an \
         empty picker or a build failure"
    );
}

/// DELIBERATELY INVERTED. This law used to be
/// `project_roster_excludes_a_symlinked_child_folder`, and it was not wrong:
/// it MEASURED that `std::fs::DirEntry::file_type()` reports a link's OWN type
/// without following it, so `NativeFs::read_dir` called a symlinked folder
/// neither dir nor file and `list_dir_level`'s filter dropped it before the
/// picker saw it — and it recorded that as the roster's true behavior rather
/// than claiming it was a requirement, precisely so a change to that
/// classification would have a law to answer to. This is that answer. A
/// symlink is now classified by its TARGET, and the law is turned over in
/// place, same fixture shape and same guard-owned cleanup, so the inversion
/// reads as one decision rather than as a deleted law and an unrelated new
/// one.
///
/// `InMemoryFs` has no symlink concept, so this axis cell needs a real
/// instrument rather than an assumed one: a real scratch directory. The whole
/// fixture is unix-gated because `std::os::unix::fs::symlink` is the
/// instrument — the CONDITION is on the API's existence, not on a
/// `cfg!(target_os)` value read at runtime.
#[cfg(unix)]
#[test]
fn project_roster_includes_a_symlinked_child_folder() {
    let _guard = crate::fs::FsGuard::capture();
    // The guard's Drop owns the cleanup: an end-of-function remove runs only on
    // the happy path, and this fixture has assertions that can panic before it.
    let base = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-project-symlink-roster-{}", std::process::id())),
    );
    let ws = base.join("ws");
    let real_target = base.join("real-target");
    let real_file = base.join("real-file.md");
    // The shape axis, all on ONE real disk so the classification is proved
    // against the backend that actually does the following.
    std::fs::create_dir_all(ws.join("ordinary/grandchild")).unwrap();
    std::fs::create_dir_all(ws.join("gitrepo/.git")).unwrap();
    std::fs::create_dir_all(ws.join("empty")).unwrap();
    std::fs::create_dir_all(&real_target).unwrap();
    std::fs::write(&real_file, b"body").unwrap();
    let link = |target: &std::path::Path, name: &str| {
        std::os::unix::fs::symlink(target, ws.join(name)).unwrap()
    };
    link(&real_target, "linked-dir"); // → a directory
    link(&real_file, "linked-file"); // → a file
    link(&base.join("does-not-exist"), "linked-broken"); // → nothing
    link(&ws.join("loop-b"), "loop-a"); // ─┐ a two-hop cycle:
    link(&ws.join("loop-a"), "loop-b"); // ─┘ the stat answers ELOOP
    link(&ws, "linked-ancestor"); // → the workspace it lives in

    let ov = crate::overlay::browse_level(
        OverlayKind::Project,
        None,
        std::path::Path::new("/unused"),
        Some(&ws),
        &[],
    )
    .expect("a configured workspace always builds a Project overlay");
    let shown = ov.item_strings();
    let has = |n: &str| shown.iter().any(|s| s.starts_with(n));

    // The pre-existing shape cells, re-proved on the real backend.
    for name in ["ordinary", "gitrepo", "empty"] {
        assert!(has(name), "direct child {name:?} must enrol: {shown:?}");
    }
    assert!(
        !shown.iter().any(|s| s.contains("grandchild")),
        "a grandchild must never enrol, however deep it lives: {shown:?}"
    );

    // THE INVERSION. A link to a folder is a folder: `NativeFs::read_dir`
    // follows the link with a `metadata` stat on the entry's own path.
    assert!(
        has("linked-dir"),
        "a symlinked folder enrols as the folder it points to: {shown:?}"
    );
    // A link to an ANCESTOR is still a real directory and still enrols — the
    // cycle it would create belongs to the recursive walk, not to one level
    // read (`index::tests::go_to_index_does_not_descend_a_symlinked_dir`).
    assert!(
        has("linked-ancestor"),
        "a link back at the workspace is a directory like any other: {shown:?}"
    );
    // A link to a FILE is a file, so the folders-only project roster excludes
    // it for the reason an ordinary file is excluded — not for being a link.
    assert!(
        !has("linked-file"),
        "a symlinked FILE is a file, and the roster names folders: {shown:?}"
    );
    // Three links whose target cannot be stat'd: nothing to open, nothing to
    // descend, so nothing shown. A name that errors on Enter is worse than an
    // absent one.
    for name in ["linked-broken", "loop-a", "loop-b"] {
        assert!(
            !has(name),
            "an unresolvable link ({name:?}) is neither dir nor file: {shown:?}"
        );
    }

    // One seam lower, so "excluded from the ROSTER" is not mistaken for
    // "invisible everywhere": the level read that Browse and the Settings
    // folder-value navigator share classifies the file link AS A FILE, and
    // shows it.
    let level = crate::index::list_dir_level(&ws, None);
    let file_link = level
        .iter()
        .find(|e| e.name == "linked-file")
        .unwrap_or_else(|| {
            panic!(
                "a symlinked file is present at the level read: {:?}",
                level.iter().map(|e| &e.name).collect::<Vec<_>>()
            )
        });
    assert!(
        !file_link.is_dir,
        "a link to a file classifies as a file, not a folder"
    );
    let dir_link = level.iter().find(|e| e.name == "linked-dir").unwrap();
    assert!(
        dir_link.is_dir,
        "a link to a folder classifies as a folder at the same seam"
    );
}

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
    // The synthetic accept-this-folder row is pinned at the TOP, and it READS
    // as plain language naming the level's own directory — never the bare `.`
    // its corpus carries.
    let items = ov.item_strings();
    assert_eq!(items[0], "use this folder — ws");
    // …while its ACCEPT string is unchanged, because the path math, the
    // dotfile exemption and the selection skip all key off it.
    assert_eq!(ov.rows[0].accept, ".");
    // browse_dir carries the ABSOLUTE dir for path navigation.
    assert_eq!(ov.browse_dir.as_deref(), Some("/ws"));
    // Default selection skips the accept row and lands on the first REAL folder
    // so Enter descends into it immediately (the accept row is Up).
    assert_eq!(ov.selected_value(), Some("plain-notes"));
    assert!(ov.selected_is_dir(), "first folder is a directory");
    // Git children carry the `"git"` SECONDARY tag (not a name bullet); the
    // accept row is neither git nor a dir, and no name carries a bullet.
    assert!(
        items.iter().all(|s| !s.contains('•')),
        "no name bullet: {items:?}"
    );
    let tags = ov.item_git_tags();
    let alpha = items.iter().position(|s| s.contains("repo-alpha")).unwrap();
    assert_eq!(tags[alpha], "git");
    assert_eq!(
        tags[0], "",
        "the accept-this-folder row is never git-tagged"
    );
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
    // it), while the synthetic accept-this-folder row and `.env` (the earned
    // exception) stay visible.
    assert!(ov.kind.hides_dotfiles(), "Project hides dotfiles now");
    let shown = ov.item_strings();
    assert!(
        shown.iter().any(|s| s == "use this folder — ws"),
        "the accept-this-folder row survives: {shown:?}"
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
        revealed.iter().any(|s| s == "use this folder — ws"),
        "the accept-this-folder row is still present after reveal"
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
    // The synthetic accept-this-folder row survives under All (flat home).
    assert!(
        ov.item_strings()
            .iter()
            .any(|s| s == "use this folder — ws"),
        "the accept-this-folder row survives under All"
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
    // MRU: proj-c is most recent, then proj-a — in the shape `recent::resolve`
    // hands over (the root, its git flag). A stale root elsewhere never gets
    // here at all: `resolve` drops a root that no longer names a directory.
    let recent = vec![
        ("/ws/proj-c".to_string(), false),
        ("/ws/proj-a".to_string(), false),
    ];
    let mut ov = OverlayState::new_project("/ws".to_string(), folders, &recent);
    // Switch to the Recent lens (strip index 1).
    ov.set_facet_lens(1);
    assert_eq!(ov.active_facet_id(), Some("recent"));
    // ONLY the two MRU folders show, in MRU order (proj-c before proj-a) — the
    // accept-this-folder row and the non-MRU proj-b opt out.
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

// ─── The Recent lens's OTHER route: remembered roots ────────────────────────
//
// The lens shipped able to mark only what the level read already listed, which
// made it structurally blind to the one case it exists for: a project nested
// below a direct workspace child. These sweep the remembered route's whole
// verdict table — where a root enrols, where it is only a mark, where it enrols
// nowhere at all, how it reads, and where it sits among the rows the flat picker
// already had.

/// THE REPORTED CASE, in the user's own configuration: `workspace = ~`, with the
/// MRU holding `~/code2026/awl-next` (a GRANDCHILD, which no directory-level
/// read can ever list) and `~/notes` (a direct child, which it always does).
/// Both must be in Recent, in MRU order, and the nested one must read as the
/// path that tells the two apart.
///
/// The ORDER assertion is the second half, and it pins a rule that was already
/// shipped but invisible while the MRU could only ever be empty: a remembered
/// row rides `Tier::Recent`, so the projects you actually use open the card
/// ABOVE the accept-this-folder row, and the door stays last regardless because
/// it is terminal. Stated as one equality so a change to any of the three has to
/// be a decision.
#[test]
fn a_remembered_root_below_the_workspace_enrols_where_the_level_read_cannot() {
    let ws = std::path::PathBuf::from("/home/me");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(ws.join("code2026/awl-next"))
        .with_dir(ws.join("notes"))
        .with_dir(ws.join("pictures"));
    let _guard = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let mru = vec![
        "/home/me/code2026/awl-next".to_string(), // most recent — a GRANDCHILD
        "/home/me/notes".to_string(),             // next — a direct child
    ];
    let mut ov = crate::overlay::browse_level(
        OverlayKind::Project,
        None,
        std::path::Path::new("/unused"),
        Some(&ws),
        &mru,
    )
    .expect("a configured workspace always builds the flat picker");
    ov.attach_browse_door();

    // ALL — the flat home, in the order a reader meets it.
    assert_eq!(
        ov.item_strings(),
        vec![
            "code2026/awl-next/".to_string(),
            "notes/".to_string(),
            "use this folder — me".to_string(),
            "code2026/".to_string(),
            "pictures/".to_string(),
            OverlayKind::BROWSE_DOOR_LABEL.to_string(),
        ],
        "the remembered projects open the card, the accept row follows the \
         rows it did not know about, and the door is terminal"
    );
    // The direct child was MARKED, never duplicated: `notes` appears once, and
    // never as a second whole-path row beside its own name.
    assert!(
        !ov.rows.iter().any(|r| r.accept == "/home/me/notes"),
        "a remembered root that IS a listed child marks that child instead of \
         enrolling twice: {:?}",
        ov.rows.iter().map(|r| &r.accept).collect::<Vec<_>>()
    );

    // RECENT — only the two, still in MRU order, each under the one section.
    ov.set_facet_lens(1);
    assert_eq!(ov.active_facet_id(), Some("recent"));
    assert_eq!(
        ov.item_strings(),
        vec!["code2026/awl-next/".to_string(), "notes/".to_string()],
        "the lens the split was decided for finally holds the project it was \
         decided for"
    );
    assert!(ov.item_sections().iter().all(|s| s == "Recent"));
}

/// THE VERDICT TABLE for a root that cannot become a project, each cell named in
/// its own failure message. The reasoning is one sentence and it is the same one
/// the roster already reached for a broken symlink: a row whose Enter can only
/// fail is worse than no row, because the absent row costs a reader nothing and
/// the failing one costs them a switch, a notice and a guess.
///
/// The cells are chosen so no two share a mechanism: a directory that is GONE, a
/// path that resolves to a FILE, a RELATIVE root (which would otherwise resolve
/// against a working directory that is not a project), and the level's OWN
/// directory (already answered by the accept-this-folder row, so a second row
/// would be a duplicate rather than an error). A live root is carried alongside
/// so the law cannot be satisfied by an enrolment that stopped working.
#[test]
fn a_remembered_root_that_cannot_be_switched_to_never_enrols() {
    let ws = std::path::PathBuf::from("/ws");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(ws.join("live/proj"))
        .with_file("/ws/not-a-folder.md", "body");
    let _guard = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let mru = vec![
        "/ws/live/proj".to_string(),       // the control: a real nested directory
        "/ws/deleted-since".to_string(),   // gone
        "/ws/not-a-folder.md".to_string(), // a file
        "relative/proj".to_string(),       // not absolute
        "/ws".to_string(),                 // the level itself
    ];
    let mut ov = crate::overlay::browse_level(
        OverlayKind::Project,
        None,
        std::path::Path::new("/unused"),
        Some(&ws),
        &mru,
    )
    .expect("a configured workspace always builds the flat picker");
    let remembered: Vec<&String> = ov
        .rows
        .iter()
        .map(|r| &r.accept)
        .filter(|a| crate::overlay::is_remembered_root(a))
        .collect();
    assert_eq!(
        remembered,
        vec!["/ws/live/proj"],
        "exactly the root that still names a directory enrols"
    );
    for (root, why) in [
        (
            "/ws/deleted-since",
            "a deleted project offers a row Enter can only fail on",
        ),
        ("/ws/not-a-folder.md", "a file is not a project root"),
        ("relative/proj", "a relative root names no fixed place"),
    ] {
        assert!(
            !ov.rows.iter().any(|r| r.accept == root),
            "{root} must not enrol: {why}"
        );
    }
    // The level itself is refused as a ROW, not as an answer: the
    // accept-this-folder row is still the way to pick it, and there is exactly
    // one of it.
    assert_eq!(
        ov.rows
            .iter()
            .filter(|r| r.accept == crate::overlay::HERE_ACCEPT)
            .count(),
        1,
    );
    assert!(
        ov.item_strings()
            .iter()
            .any(|s| s == "use this folder — ws"),
        "the level keeps its one row, the accept row: {:?}",
        ov.item_strings()
    );
    // And Recent is not left empty by the refusals — the live root is there.
    ov.set_facet_lens(1);
    assert_eq!(ov.item_strings(), vec!["live/proj/".to_string()]);
}

/// HOW A REMEMBERED ROOT READS, swept over the whole decision rather than the
/// one branch this machine happens to take — `home` is injected, so both the
/// level-relative and the home-relative arms are asserted here and neither is a
/// property of who is running the test.
///
/// The last two cells are the ones a byte-prefix implementation passes and a
/// component-wise one survives: a sibling whose name merely EXTENDS the level's,
/// and a root that strips to nothing at all.
#[test]
fn a_remembered_root_reads_level_relative_then_home_relative_then_absolute() {
    use crate::overlay::build::recent::label_with_home;
    let home = std::path::Path::new("/home/me");
    let level = Some("/home/me/ws");
    for (root, expect, why) in [
        (
            "/home/me/ws/code/awl-next",
            "code/awl-next",
            "under the level: relative to the level, parents first",
        ),
        (
            "/home/me/elsewhere/proj",
            "~/elsewhere/proj",
            "outside the level but under home: relative to home",
        ),
        (
            "/opt/shared/proj",
            "/opt/shared/proj",
            "outside both: its own path, unshortened",
        ),
        (
            "/home/me/ws-archive/proj",
            "~/ws-archive/proj",
            "a sibling that EXTENDS the level's name is not under the level",
        ),
        (
            "/home/me/ws",
            "~/ws",
            "a root that strips to NOTHING against the level falls through to \
             the next form rather than becoming an empty row (unreachable in \
             the product — `resolve` refuses the level — and this is what the \
             fall-through does if it ever is)",
        ),
    ] {
        assert_eq!(label_with_home(root, level, Some(home)), expect, "{why}");
    }
    // With no home to compare against, the home arm simply does not apply.
    assert_eq!(
        label_with_home("/home/me/elsewhere/proj", level, None),
        "/home/me/elsewhere/proj",
        "no home, no shortening — never a half-applied one"
    );
}

/// A remembered root under a DOTTED parent survives the hidden-entry filter.
/// That filter governs what a directory READ puts in front of you; a remembered
/// root is not an entry of any directory being read but a project already
/// chosen, so a setting about listings must not be able to delete it from
/// Recent. Swept over both settings, because a law that ran only under
/// `all_on` would be blind to the one configuration where the filter exists.
#[test]
fn a_remembered_root_under_a_dotted_parent_stays_in_recent_under_both_visibilities() {
    let ws = std::path::PathBuf::from("/ws");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(ws.join(".hidden/proj"))
        .with_dir(ws.join("plain"));
    let _guard = crate::fs::FsGuard::install(std::sync::Arc::new(mem));
    let saved = crate::file_visibility::all_on();
    let mru = vec!["/ws/.hidden/proj".to_string()];
    for all_on in [false, true] {
        crate::file_visibility::set_all_on(all_on);
        let mut ov = crate::overlay::browse_level(
            OverlayKind::Project,
            None,
            std::path::Path::new("/unused"),
            Some(&ws),
            &mru,
        )
        .expect("a configured workspace always builds the flat picker");
        ov.set_facet_lens(1);
        assert_eq!(
            ov.item_strings(),
            vec![".hidden/proj/".to_string()],
            "the remembered project stays in Recent with all_on {all_on}"
        );
        // NON-VACUITY: the filter is genuinely doing its job on the same card —
        // the dotted DIRECTORY ENTRY is hidden when the setting is off and shows
        // when it is on, so the exemption above is an exemption and not a filter
        // that stopped running.
        ov.set_facet_lens(0);
        assert_eq!(
            ov.item_strings().iter().any(|s| s == ".hidden/"),
            all_on,
            "the dotted child follows the setting under All (all_on {all_on}): {:?}",
            ov.item_strings()
        );
    }
    crate::file_visibility::set_all_on(saved);
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
    // ONE row, and it READS — the label names both what the row does and the
    // folder it would do it to, so the oracle is the exact copy a user sees,
    // not merely "one row survived". A label that stopped naming the folder
    // (or went back to shell notation) fails here by inequality, which is what
    // keeps this law about the row rather than about the count.
    assert_eq!(
        ov.item_strings(),
        vec!["use this folder — empty-ws".to_string()],
        "a childless workspace shows only the accept-this-folder row, reading \
         as its own plain-language label, not an empty picker or a build failure"
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

// ─── The accept-this-folder row's LABEL ─────────────────────────────────────
//
// The row's corpus string is `.` and its label is not. These pin the two
// halves of that split: the label NAMES the level it stands on (so the card
// answers "where am I" at all), and the accept string stays `.` (so the path
// math, the dotfile exemption and the selection skip keep working).

#[test]
fn the_accept_row_names_the_level_it_stands_on_and_follows_it() {
    // TWO LEVELS, one builder. The flat switch-project picker cannot browse
    // today, so its `browse_dir` is always the configured workspace — but the
    // SAME card is also the Settings folder-VALUE navigator, which descends,
    // and a browse door would descend it too. So the axis this sweeps is not
    // "which picker" but "which directory", asked of two of them.
    let ws = OverlayState::new_project(
        "/home/me/notes".to_string(),
        vec![("journal".to_string(), false)],
        &[],
    );
    let deeper = OverlayState::new_project(
        "/home/me/notes/journal".to_string(),
        vec![("2026".to_string(), false)],
        &[],
    );
    let label = |ov: &OverlayState| ov.item_strings()[0].clone();
    assert_eq!(label(&ws), "use this folder — notes");
    assert_eq!(label(&deeper), "use this folder — journal");
    // NON-VACUITY, stated as an assertion rather than left to the reader: a
    // label that ignored `browse_dir` — the state before this row said
    // anything at all — would be EQUAL on the two levels.
    assert_ne!(
        label(&ws),
        label(&deeper),
        "the accept row must follow the level; a constant label answers \
         'where am I' with the same word everywhere"
    );
    // The LEAF, not the path: the parents are not spelled out, so the row can
    // never be mistaken for a path readout that `elide_path` would then chew
    // from the wrong end.
    for ov in [&ws, &deeper] {
        assert!(
            !label(ov).contains('/'),
            "the row names a folder, not a path: {:?}",
            label(ov)
        );
    }
    // And the ACCEPT string is untouched on both — the label is a display, and
    // everything that keys off `.` still finds it.
    assert_eq!(ws.rows[0].accept, crate::overlay::HERE_ACCEPT);
    assert_eq!(deeper.rows[0].accept, crate::overlay::HERE_ACCEPT);
    // The selection skip reads the ACCEPT, so it still lands on the child.
    assert_eq!(ws.selected_value(), Some("journal"));
    assert_eq!(deeper.selected_value(), Some("2026"));
}

#[test]
fn no_switch_project_row_ever_reads_as_shell_notation() {
    // THE SWEEP: every state the flat picker's roster can be in — both file-
    // visibility settings, both lenses, a level with children and one without,
    // and a query that filters the roster down — asserting on what a reader
    // SEES. `.` is shell notation; awl's audience is not required to know it.
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    let folders = vec![
        (".git".to_string(), false),
        ("journal".to_string(), false),
        ("code".to_string(), true),
    ];
    let mut checked = 0usize;
    for all_on in [false, true] {
        crate::file_visibility::set_all_on(all_on);
        for (dir, roster) in [
            ("/home/me/notes", folders.clone()),
            ("/home/me/notes/journal", Vec::new()),
        ] {
            for lens in [0usize, 1] {
                let mut ov = OverlayState::new_project(dir.to_string(), roster.clone(), &[]);
                ov.set_facet_lens(lens);
                let shown = ov.item_strings();
                for s in &shown {
                    assert_ne!(s, ".", "a bare `.` reached the roster: {shown:?}");
                    assert!(
                        !s.starts_with("./"),
                        "shell path notation reached the roster: {shown:?}"
                    );
                    checked += 1;
                }
                // The here row, found by its ACCEPT rather than by position, so
                // the enrolment cannot quietly stop matching: when it is
                // visible under this lens it reads as its label, and its
                // display is never its accept string.
                if let Some(pos) = ov
                    .items
                    .iter()
                    .position(|&ci| ov.rows[ci].accept == crate::overlay::HERE_ACCEPT)
                {
                    assert!(
                        shown[pos].starts_with(crate::overlay::HERE_LABEL),
                        "the accept row reads as its label under lens {lens} / \
                         all_on {all_on}: {shown:?}"
                    );
                    assert_ne!(shown[pos], ov.rows[ov.items[pos]].accept);
                }
            }
        }
    }
    crate::file_visibility::set_all_on(saved);
    assert!(
        checked >= 8,
        "the sweep enrolled almost nothing ({checked} rows) — it would pass \
         over an empty roster"
    );
}

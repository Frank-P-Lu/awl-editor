use super::*;

#[test]
fn elide_keeps_filename_and_extension_with_one_ellipsis() {
    // A deep path, narrow budget: the filename + ext survive, the DIR is elided.
    let out = elide_path("src/app/render/chrome.rs", 16);
    assert!(
        out.ends_with("chrome.rs"),
        "filename+ext must survive: {out}"
    );
    assert_eq!(out.matches('…').count(), 1, "exactly one ellipsis: {out}");
    assert!(out.chars().count() <= 16, "fits the budget: {out}");
    // The split point is the last '/': dir prefix (muted) vs filename (content).
    let split = row_split(&out);
    assert!(out[..split].ends_with('/'));
    assert_eq!(&out[split..], "chrome.rs");
    // A row that already fits is returned WHOLE (no ellipsis, no change).
    assert_eq!(elide_path("src/main.rs", 40), "src/main.rs");
    assert_eq!(row_split("src/main.rs"), 4); // "src/"
}

#[test]
fn elide_middle_truncates_the_filename_when_it_alone_overflows() {
    // Filename longer than the whole budget → the filename ITSELF is middle-elided,
    // the directory dropped, extension end kept, still a single ellipsis.
    let out = elide_path("deep/dir/averyveryverylongfilename.rs", 12);
    assert_eq!(out.matches('…').count(), 1, "one ellipsis: {out}");
    assert!(out.chars().count() <= 12, "fits: {out}");
    assert!(out.ends_with(".rs"), "extension survives: {out}");
    assert!(
        !out.contains('/'),
        "dir dropped when the filename alone overflows: {out}"
    );
    assert_eq!(row_split(&out), 0, "no '/', so all content ink");
    // A bare filename with no directory elides the same way.
    let bare = elide_path("supercalifragilistic.md", 10);
    assert!(bare.ends_with(".md") && bare.matches('…').count() == 1);
}

#[test]
fn directory_elision_keeps_path_identity_and_the_final_folder() {
    let cases = [
        ("/Users/writer/Documents/notes", 18),
        (
            "/Users/writer/a-single-enormously-descriptive-folder-name-with-no-parents-to-drop",
            18,
        ),
        ("/Users/writer/仕事/原稿/長編小説の作業中の草稿", 12),
    ];
    for (path, allowance) in cases {
        let out = elide_directory_path(path, allowance);
        assert_eq!(
            out.chars().count(),
            allowance,
            "{path:?}: the shortened readout spends its authored allowance"
        );
        assert!(
            out.contains('/'),
            "{path:?}: path identity survives in {out:?}"
        );
        let leaf = path.rsplit('/').next().unwrap();
        let recognizable_leaf: String = leaf
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        assert!(
            out.ends_with(&recognizable_leaf),
            "{path:?}: the final folder remains recognizable in {out:?}"
        );
        assert!(
            out.contains('…'),
            "{path:?}: elision is disclosed in {out:?}"
        );
    }

    assert_eq!(
        elide_directory_path("/tmp/notes", 40),
        "/tmp/notes",
        "a directory that fits is not touched"
    );
    assert_eq!(
        elide_path("deep/dir/averyveryverylongfilename.rs", 12),
        "avery…ame.rs",
        "the file-row owner remains extension-biased and is not routed through directory elision"
    );
}

#[test]
fn browse_dir_flags_directories() {
    // One level: a folder (docs) and a file (README.md).
    let corpus = vec!["docs".to_string(), "README.md".to_string()];
    let git = vec![false, false];
    let is_dir = vec![true, false];
    let mut ov = OverlayState::new_marked(
        OverlayKind::Browse,
        corpus,
        git,
        is_dir,
        vec![],
        vec![],
        None,
    );
    // docs selected -> a directory; README.md selected -> a file.
    assert!(ov.selected_is_dir());
    ov.move_sel(1);
    assert!(!ov.selected_is_dir());
    assert_eq!(ov.selected_value(), Some("README.md"));
}

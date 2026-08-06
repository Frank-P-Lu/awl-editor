use super::super::*;
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

use super::*;

#[test]
fn empty_query_shows_all() {
    let ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    assert_eq!(ov.items.len(), 4);
}

#[test]
fn typing_filters() {
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    ov.push('e');
    ov.push('n');
    ov.push('v');
    // ".env" should be the top match.
    assert_eq!(ov.selected_value(), Some(".env"));
}

#[test]
fn goto_hides_dotfiles_until_revealed() {
    // A go-to corpus with a hidden dotfile, a hidden dir entry, an `.env` (the
    // earned exception), and ordinary files.
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let corpus = vec![
        ".gitignore".to_string(),
        ".env".to_string(),
        "src/.hidden/x.rs".to_string(),
        "README.md".to_string(),
        "src/main.rs".to_string(),
    ];
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![]);
    // Default (Text): dotfiles hidden, `.env` and ordinary files visible.
    let shown = ov.item_strings();
    assert!(
        !shown.iter().any(|s| s == ".gitignore"),
        "dotfile hidden: {shown:?}"
    );
    assert!(
        !shown.iter().any(|s| s == "src/.hidden/x.rs"),
        "nested dot dir hidden: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s == ".env"),
        ".env stays visible: {shown:?}"
    );
    assert!(shown.iter().any(|s| s == "README.md"));
    assert!(shown.iter().any(|s| s == "src/main.rs"));
    // Flip to All -> dotfiles now revealed alongside everything.
    crate::file_visibility::set_all_on(true);
    ov.refilter();
    let shown = ov.item_strings();
    assert!(
        shown.iter().any(|s| s == ".gitignore"),
        "dotfile revealed: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s == "src/.hidden/x.rs"),
        "nested dot dir revealed: {shown:?}"
    );
    assert!(shown.iter().any(|s| s == ".env"));
    // Flip back to Text -> hidden again.
    crate::file_visibility::set_all_on(false);
    ov.refilter();
    assert!(!ov.item_strings().iter().any(|s| s == ".gitignore"));
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn browse_hides_dot_leaves_until_revealed() {
    // Browse lists one directory LEVEL: bare leaf names.
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let corpus = vec![
        ".config".to_string(),
        "notes.md".to_string(),
        ".env".to_string(),
    ];
    let git = vec![false; 3];
    let is_dir = vec![true, false, false];
    let mut ov = OverlayState::new_marked(
        OverlayKind::Browse,
        corpus,
        git,
        is_dir,
        vec![],
        vec![],
        None,
    );
    let shown = ov.item_strings();
    assert!(
        !shown.iter().any(|s| s.starts_with(".config")),
        "dot dir hidden: {shown:?}"
    );
    assert!(shown.iter().any(|s| s == "notes.md"));
    assert!(
        shown.iter().any(|s| s == ".env"),
        ".env visible in browse too"
    );
    crate::file_visibility::set_all_on(true);
    ov.refilter();
    assert!(
        ov.item_strings().iter().any(|s| s.starts_with(".config")),
        "dot dir revealed"
    );
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn browse_listing_hides_unsupported_files_in_text_and_labels_them_in_all() {
    // ITEM 77: the REAL `overlay::build::browse_level` (not the hand-built
    // `new_marked` fixture the other tests above use), over a seeded
    // InMemoryFs directory, so this exercises the actual per-file
    // `crate::openable::classify` wiring — a supported unusual-extension
    // file, a binary file, and a folder.
    use crate::fs::{FileSystem, InMemoryFs};
    use std::sync::Arc;
    let _g_lock = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    let mem = InMemoryFs::new();
    mem.write(
        std::path::Path::new("/proj/notes.xyzzy"),
        b"real prose, odd extension\n",
    )
    .unwrap();
    mem.write(
        std::path::Path::new("/proj/logo.png"),
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00],
    )
    .unwrap();
    mem.create_dir_all(std::path::Path::new("/proj/sub"))
        .unwrap();
    let _g_fs = crate::fs::FsGuard::install(Arc::new(mem));

    crate::file_visibility::set_all_on(false);
    let ov = crate::overlay::browse_level(
        OverlayKind::Browse,
        None,
        std::path::Path::new("/proj"),
        None,
        &[],
    )
    .expect("a browse level always builds");
    let shown = ov.item_strings();
    assert!(
        shown.iter().any(|s| s == "notes.xyzzy"),
        "unusual-extension TEXT stays listed: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s == "sub/"),
        "folders always list: {shown:?}"
    );
    assert!(
        !shown.iter().any(|s| s == "logo.png"),
        "Text mode hides the unsupported file: {shown:?}"
    );

    crate::file_visibility::set_all_on(true);
    let ov = crate::overlay::browse_level(
        OverlayKind::Browse,
        None,
        std::path::Path::new("/proj"),
        None,
        &[],
    )
    .expect("a browse level always builds");
    let shown = ov.item_strings();
    let secs = ov.item_bindings(); // the secondary column, parallel to item_strings
    let i = shown
        .iter()
        .position(|s| s == "logo.png")
        .expect("All mode reveals the unsupported file");
    assert_eq!(secs[i], "PNG", "the row carries a concise type label");

    crate::file_visibility::set_all_on(saved);
}

#[test]
fn non_file_picker_ignores_file_visibility() {
    // A theme/command picker never hides dotfiles regardless of the global.
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    let mut ov = OverlayState::new_command(
        vec!["Save".into(), ".secret command".into()],
        vec!["C-x C-s".into(), String::new()],
        vec![false, false],
    );
    assert!(!ov.kind.hides_dotfiles());
    let before = ov.item_strings();
    crate::file_visibility::set_all_on(!crate::file_visibility::all_on());
    ov.refilter();
    assert_eq!(
        ov.item_strings(),
        before,
        "listing unchanged for a non-file picker"
    );
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn move_clamps() {
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    ov.move_sel(-1);
    assert_eq!(ov.selected, 0);
    ov.move_sel(100);
    assert_eq!(ov.selected, ov.items.len() - 1);
}

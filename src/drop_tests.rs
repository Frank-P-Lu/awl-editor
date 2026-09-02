use super::*;
use std::path::Path;

#[test]
fn classify_drop_routes_every_roster_extension_to_image() {
    // Non-vacuity: the sweep below is meaningless if the roster is empty.
    assert!(!crate::assets::IMAGE_EXTS.is_empty());
    // Sweep the SAME roster the asset-cleaner scan uses — not a hand-picked
    // subset — so a future addition to `IMAGE_EXTS` is covered here for free.
    for ext in crate::assets::IMAGE_EXTS {
        let path = Path::new(&format!("/tmp/photo.{ext}"));
        assert_eq!(
            classify_drop(path),
            DropRoute::Image,
            "extension {ext:?} should route to the image door"
        );
        // Case-insensitivity: the OS/Finder can hand back any casing.
        let upper = Path::new(&format!("/tmp/photo.{}", ext.to_ascii_uppercase()));
        assert_eq!(
            classify_drop(upper),
            DropRoute::Image,
            "uppercased extension {ext:?} should still route to the image door"
        );
    }
}

#[test]
fn classify_drop_routes_text_and_markdown_to_open() {
    for name in ["notes.md", "readme.txt", "main.rs", "config.toml", "noext"] {
        assert_eq!(
            classify_drop(Path::new(name)),
            DropRoute::Open,
            "{name:?} should route through the open-file door"
        );
    }
}

#[test]
fn classify_drop_falls_back_to_open_for_a_stripped_or_unfamiliar_extension() {
    // No extension at all, and an extension outside the image roster: both
    // fall to Open, where `openable::classify` decides by CONTENT rather than
    // this module guessing from a bare name.
    assert_eq!(classify_drop(Path::new("/tmp/IMG_1234")), DropRoute::Open);
    assert_eq!(classify_drop(Path::new("/tmp/archive.zip")), DropRoute::Open);
}

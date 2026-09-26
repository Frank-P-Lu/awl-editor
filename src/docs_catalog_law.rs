//! User-facing documentation agrees with live settings and filesystem behavior.

#[test]
fn every_settings_row_the_docs_cite_still_exists() {
    let _g = crate::testlock::serial();
    let live = crate::settings::names();
    assert!(!live.is_empty());
    for row in ["Format popover", "Edit config as text"] {
        assert!(
            live.iter().any(|name| name == row),
            "unknown settings row {row:?}"
        );
        assert!(
            crate::embedded_docs::GUIDE_MD.contains(row),
            "uncited settings row {row:?}"
        );
    }
}

#[test]
fn first_launch_docs_never_assign_an_implicit_notes_folder() {
    let _g = crate::testlock::serial();
    for (name, text) in [
        ("PHILOSOPHY.md", crate::embedded_docs::PHILOSOPHY_MD),
        ("GUIDE.md", crate::embedded_docs::GUIDE_MD),
        ("samples/welcome.md", crate::embedded_docs::WELCOME_MD),
    ] {
        assert!(
            !text.contains("active folder starts out as")
                && !text.contains("writes `welcome.md` into the active folder"),
            "{name} assigns an implicit user folder"
        );
    }
    assert!(crate::embedded_docs::GUIDE_MD.contains("does not create `~/notes`"));
}

#[test]
fn sibling_site_pages_carry_no_chord_glyphs() {
    let _g = crate::testlock::serial();
    for (name, text) in [
        ("site/credits.html", include_str!("../site/credits.html")),
        ("site/index.html", include_str!("../site/index.html")),
        ("site/check.html", include_str!("../site/check.html")),
    ] {
        assert!(
            !text.chars().any(|c| "⌘⌥⇧⌃".contains(c)) && !text.contains("Ctrl+"),
            "{name} carries unchecked chord labels; use the generated reference"
        );
    }
}

use super::*;

#[test]
fn goto_headings_lens_folds_in_the_docs_headings() {
    // THE FOLD: a Go-to overlay with the doc's headings attached lists them mixed
    // with the files under the flat `All` home, and
    // ONLY the headings under the dedicated Headings lens — which is where the
    // retired standalone Outline picker now lives as an explicit refinement.
    let corpus = vec!["README.md".to_string(), "src/main.rs".to_string()];
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![]);
    ov.attach_headings(vec![
        ("Introduction".to_string(), 3),
        ("  Details".to_string(), 7),
    ]);
    // Strip carries the Headings lens, parked last — "By type" was CUT.
    let strip: Vec<String> = ov.lens_strip().into_iter().map(|(l, _)| l).collect();
    assert_eq!(strip, vec!["All", "Recent", "This folder", "Headings"]);
    // ALL home: files AND headings, mixed in the same fuzzy-ranked list. A heading
    // row carries the `❡ ` KIND-HINT marker (the rowlayout PRIMARY-cell
    // disambiguator); a file row never does.
    const H: &str = OverlayKind::HEADING_MARKER_PREFIX;
    assert_eq!(ov.active_facet_id(), Some("all"));
    let all = ov.item_strings();
    assert!(all.iter().any(|s| s == "README.md") && all.iter().any(|s| s == "src/main.rs"));
    assert!(
        all.iter().any(|s| s == &format!("{H}Introduction")),
        "headings mixed into All, marked: {all:?}"
    );
    assert!(
        all.iter().any(|s| s == &format!("{H}  Details")),
        "headings mixed into All, marked: {all:?}"
    );
    assert_eq!(all.len(), 4);
    // Headings lens (strip index 3): ONLY the headings, and each row IS a heading
    // whose accept is its line number, not a file open.
    ov.focus_facet_id("headings");
    assert_eq!(ov.active_facet_id(), Some("headings"));
    assert_eq!(
        ov.item_strings(),
        vec![format!("{H}Introduction"), format!("{H}  Details")]
    );
    assert!(
        ov.selected_is_heading(),
        "the Headings lens rows are headings"
    );
    assert_eq!(
        ov.selected_line(),
        Some(3),
        "the first heading jumps to line 3"
    );
    // "This folder" (strip index 2): a file-only REFINEMENT — headings drop out.
    ov.set_facet_lens(2);
    let folder = ov.item_strings();
    assert!(
        !folder
            .iter()
            .any(|s| s == "Introduction" || s == "  Details"),
        "{folder:?}"
    );
}

#[test]
fn goto_headings_lens_is_empty_without_headings() {
    // A non-markdown buffer (or one with no headings) attaches nothing: the
    // Headings lens is still on the strip but reads empty ("no headings yet").
    let corpus = vec!["a.rs".to_string(), "b.rs".to_string()];
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![]);
    ov.attach_headings(Vec::new()); // no-op
    ov.focus_facet_id("headings");
    assert_eq!(ov.active_facet_id(), Some("headings"));
    assert!(ov.item_strings().is_empty(), "no headings → empty lens");
    assert_eq!(ov.empty_message(), "no headings yet");
}

#[test]
fn goto_headings_lens_fuzzy_filters_and_jumps_by_line() {
    // The retired Outline picker's fuzzy-jump behavior, now under Go-to's Headings
    // lens: filter to a heading, its accept is the LINE (titles can repeat), not
    // the file-open the other lenses do.
    let corpus = vec!["notes.md".to_string()];
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![]);
    ov.attach_headings(vec![
        ("Intro".to_string(), 0usize),
        ("  Setup".to_string(), 4usize),
        ("  Usage".to_string(), 9usize),
    ]);
    ov.focus_facet_id("headings");
    // Rows are the (indented) titles in order, marker-prefixed; lines stay parallel.
    const H: &str = OverlayKind::HEADING_MARKER_PREFIX;
    assert_eq!(
        ov.item_strings(),
        vec![
            format!("{H}Intro"),
            format!("{H}  Setup"),
            format!("{H}  Usage")
        ]
    );
    assert_eq!(ov.selected_line(), Some(0));
    // Fuzzy filter to "Usage" -> selected row jumps to its line (9), not its text.
    // `selected_value` reads the RAW corpus (unprefixed) — the marker is display-only.
    ov.push('u');
    ov.push('s');
    ov.push('a');
    assert_eq!(ov.selected_value(), Some("  Usage"));
    assert!(ov.selected_is_heading());
    assert_eq!(ov.selected_line(), Some(9));
    // No git / dir markers on heading rows; the indentation + kind-hint survive.
    assert!(
        ov.item_strings()
            .iter()
            .all(|s| !s.contains('•') && !s.ends_with('/'))
    );
    assert!(ov.item_strings().iter().all(|s| s.starts_with(H)));
}

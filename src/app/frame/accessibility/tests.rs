//! The incremental-update laws.
//!
//! Every one of these asserts a COUNT, not a duration. A latency number cannot
//! tell "one line was re-read" apart from "nothing was re-read", and CLAUDE.md
//! records a theme bench that measured 5 ms while nothing reshaped. The counts
//! come from [`ProjectionStats`], which the production refresh keeps whether or
//! not a test is watching.

use super::*;
use crate::app::App;
use crate::config::Config;
use std::path::PathBuf;

fn hermetic() -> App {
    App::new_hermetic(None, PathBuf::from("/"), Config::empty())
}

fn calm_globals() {
    crate::about::set_open(false);
    crate::lifetime::set_open(false);
    crate::streaks::set_open(false);
    crate::hud::set_held(false);
    crate::peek::set_open(false);
    crate::whichkey::set_force_shown(false);
    crate::menubar::set_menu_bar_on(false);
}

/// A document of `lines` lines, each with real prose so a line is not a
/// degenerate one-byte case — and each the SAME length, so a size sweep
/// compares like with like. (A zero-padded counter, because `line {n}` made
/// the 20 000-line arm's lines two bytes longer than the 100-line arm's and
/// the sweep reported a difference that was the fixture's, not the code's.)
fn document(lines: usize) -> String {
    (0..lines)
        .map(|n| format!("line {n:06} of some ordinary prose in a paragraph"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn attached(lines: usize) -> App {
    calm_globals();
    let mut app = hermetic();
    app.set_semantic_text_for_test(&document(lines));
    app.document.set_cursor(0);
    app.attach_assistive_technology_for_test();
    app
}

/// THE headline law. AccessKit expects a full tree at activation and changed
/// nodes afterwards; awl used to republish the whole document on every redraw,
/// which is what VoiceOver reported as "awl is not responding".
#[test]
fn a_full_tree_is_published_only_at_activation() {
    let _guard = crate::testlock::serial();
    let mut app = attached(400);
    assert_eq!(
        app.accessibility_stats().full_trees,
        0,
        "the synchronous handler already served the tree; the first update \
         must not send a second whole document",
    );

    for ch in "hello there".chars() {
        app.document.insert_char(ch);
        app.refresh_accessibility();
    }
    app.document.insert_char('\n');
    app.refresh_accessibility();
    app.document.set_cursor(0);
    app.refresh_accessibility();

    let stats = app.accessibility_stats();
    assert_eq!(
        stats.full_trees, 0,
        "a full tree was published after activation",
    );
    assert!(stats.refreshes > 10, "the sweep really drove frames");
}

/// The other branch of AccessKit's contract, and the one that is easy to get
/// silently wrong: when the activation handler could not answer, the platform
/// holds a PLACEHOLDER, and the next update is required to carry a full tree.
#[test]
fn a_placeholder_activation_is_owed_exactly_one_full_tree() {
    let _guard = crate::testlock::serial();
    calm_globals();
    let mut app = hermetic();
    app.set_semantic_text_for_test(&document(50));
    // Seed, then move the document on: the parked tree no longer describes the
    // app, so the handler must decline rather than serve a stale one.
    app.seed_accessibility_tree();
    app.document.insert_char('x');
    app.refresh_accessibility();
    let served = app.frame.activate_accessibility_for_test();
    assert!(
        served.is_none(),
        "a stale parked tree was served to a screen reader",
    );
    app.refresh_accessibility();
    assert_eq!(app.accessibility_stats().full_trees, 1);

    app.document.insert_char('y');
    app.refresh_accessibility();
    assert_eq!(
        app.accessibility_stats().full_trees,
        1,
        "the full tree owed to a placeholder must be paid once, not per frame",
    );
}

/// The synchronous branch: a screen reader that was already running when awl
/// launched gets a real tree on the spot, with no placeholder at all.
#[test]
fn an_activation_against_a_current_tree_is_served_synchronously_and_in_full() {
    let _guard = crate::testlock::serial();
    calm_globals();
    let mut app = hermetic();
    app.set_semantic_text_for_test(&document(30));
    let served = app
        .attach_assistive_technology_for_test()
        .expect("the parked tree was current and must be served");
    assert!(
        served.tree.is_some(),
        "an activation tree must carry the `Tree` metadata that declares the root",
    );
    assert_eq!(
        served.nodes.len(),
        app.semantic_snapshot().nodes.len(),
        "the activation tree must be the WHOLE tree",
    );
}

/// The measurement the item asks for, as a count rather than a clock, swept
/// across document sizes: the work a one-character edit does must not grow with
/// the document. The 20 000-line arm is the empirical worst case a prose user
/// reaches with a book, not a size an eye would check by hand.
#[test]
fn a_one_character_edit_costs_the_same_at_every_document_size() {
    let _guard = crate::testlock::serial();
    let mut measured: Vec<(usize, u64, u64, u64, u64)> = Vec::new();
    for lines in [100usize, 1_000, 20_000] {
        let mut app = attached(lines);
        // Park the caret mid-document, so nothing about the measurement is an
        // artifact of editing the first or the last line.
        let middle = app.document.buffer().line_col_to_char(lines / 2, 0);
        app.document.set_cursor(middle);
        app.refresh_accessibility();

        let before = app.accessibility_stats();
        app.document.insert_char('x');
        app.refresh_accessibility();
        let after = app.accessibility_stats();
        measured.push((
            lines,
            after.runs_rebuilt - before.runs_rebuilt,
            after.graphemes_segmented - before.graphemes_segmented,
            after.bytes_read - before.bytes_read,
            after.nodes_published - before.nodes_published,
        ));
    }

    for (lines, runs, _, _, nodes) in &measured {
        assert_eq!(
            *runs, 1,
            "{lines} lines: an edit rebuilt {runs} runs, not 1"
        );
        assert_eq!(
            *nodes, 2,
            "{lines} lines: an edit published {nodes} nodes; it must be the \
             changed run and the document node whose selection moved",
        );
    }
    let (_, _, graphemes, bytes, _) = measured[0];
    for (lines, _, other_graphemes, other_bytes, _) in &measured {
        assert_eq!(
            (*other_graphemes, *other_bytes),
            (graphemes, bytes),
            "{lines} lines: the cost of one keystroke moved with the document \
             size — segmented {other_graphemes} graphemes and read \
             {other_bytes} bytes against {graphemes}/{bytes} at 100 lines",
        );
    }
    // Anti-vacuity: one keystroke really is reading one line, not a document.
    assert!(
        bytes < 200,
        "one keystroke read {bytes} bytes; that is not one line",
    );
}

/// The structural case, and the one a run-based representation is FOR: pressing
/// Enter changes the parent's `children`, and must still leave every line below
/// the split alone.
#[test]
fn a_newline_republishes_the_parents_children_and_no_untouched_run() {
    let _guard = crate::testlock::serial();
    let lines = 2_000;
    let mut app = attached(lines);
    let middle = app.document.buffer().line_col_to_char(lines / 2, 10);
    app.document.set_cursor(middle);
    app.refresh_accessibility();

    let before = app.accessibility_stats();
    app.document.insert_char('\n');
    app.refresh_accessibility();
    let after = app.accessibility_stats();

    assert_eq!(
        after.runs_rebuilt - before.runs_rebuilt,
        2,
        "a split re-reads exactly the two halves it made",
    );
    assert_eq!(
        after.children_republished - before.children_republished,
        1,
        "the document node must republish its children exactly once",
    );
    assert_eq!(
        after.nodes_published - before.nodes_published,
        3,
        "two halves and the document node — never the lines below the split",
    );
}

/// A gliding caret must not re-announce anything. The dedup can only work if
/// the refresh finds nothing changed, so the claim under test is that a frame
/// with no input publishes zero nodes.
#[test]
fn frames_with_no_input_publish_nothing() {
    let _guard = crate::testlock::serial();
    let mut app = attached(500);
    let before = app.accessibility_stats();
    for _ in 0..20 {
        app.refresh_accessibility();
    }
    let after = app.accessibility_stats();
    assert_eq!(after.refreshes - before.refreshes, 20);
    assert_eq!(
        after.nodes_published, before.nodes_published,
        "an input-free frame published a node",
    );
    assert_eq!(
        after.runs_rebuilt, before.runs_rebuilt,
        "an input-free frame re-read a line",
    );
}

/// With no assistive technology attached, a frame must build nothing at all —
/// the gate ACCESSIBILITY.md promises. The only work left is the integer
/// compare that keeps the parked activation tree honest.
#[test]
fn a_frame_with_no_screen_reader_attached_builds_nothing() {
    let _guard = crate::testlock::serial();
    calm_globals();
    let mut app = hermetic();
    app.set_semantic_text_for_test(&document(5_000));
    for ch in "typing away with nobody listening".chars() {
        app.document.insert_char(ch);
        app.refresh_accessibility();
    }
    assert_eq!(
        app.accessibility_stats(),
        ProjectionStats::default(),
        "an unattached frame paid for projection work",
    );
}

/// A reattach gets a platform adapter that never saw what the last one held, so
/// a diff against the retained projection would describe a tree that no longer
/// exists.
#[test]
fn a_reattach_is_owed_a_full_tree_rather_than_a_diff() {
    let _guard = crate::testlock::serial();
    let mut app = attached(100);
    app.document.insert_char('a');
    app.refresh_accessibility();
    assert_eq!(app.accessibility_stats().full_trees, 0);

    app.frame.set_accessibility_active(false);
    app.document.insert_char('b');
    app.refresh_accessibility();
    app.frame.set_accessibility_active(true);
    app.refresh_accessibility();
    assert_eq!(
        app.accessibility_stats().full_trees,
        1,
        "a reattached screen reader was handed a diff against a tree it never saw",
    );
}

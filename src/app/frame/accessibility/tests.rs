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

// ── the platform mirror ────────────────────────────────────────────────────
struct Mirror {
    nodes: std::collections::HashMap<accesskit::NodeId, accesskit::Node>,
    root: Option<accesskit::NodeId>,
}

impl Mirror {
    fn of(app: &App) -> Self {
        let mut mirror = Self {
            nodes: std::collections::HashMap::new(),
            root: None,
        };
        for entry in app.frame.published_accessibility_trees() {
            // An activation that could not be served leaves the platform
            // holding a PLACEHOLDER, not the tree it had a moment ago.
            let Some(update) = entry else {
                mirror.nodes.clear();
                mirror.root = None;
                continue;
            };
            if let Some(tree) = update.tree.as_ref() {
                mirror.nodes.clear();
                mirror.root = Some(tree.root);
            }
            for (id, node) in &update.nodes {
                mirror.nodes.insert(*id, node.clone());
            }
        }
        mirror
    }

    fn document(&self) -> Option<&accesskit::Node> {
        let root = self.nodes.get(&self.root?)?;
        root.children().iter().find_map(|id| {
            let node = self.nodes.get(id)?;
            matches!(
                node.role(),
                accesskit::Role::MultilineTextInput | accesskit::Role::Document
            )
            .then_some(node)
        })
    }

    /// Do the document's selection endpoints name nodes the platform HAS?
    /// `None` when no selection was published at all.
    fn selection_resolves(&self) -> Option<bool> {
        let d = self.document()?;
        let sel = d.text_selection()?;
        Some(self.nodes.contains_key(&sel.anchor.node) && self.nodes.contains_key(&sel.focus.node))
    }

    /// The text the platform would read back, run by run.
    fn run_texts(&self) -> Vec<String> {
        self.document()
            .map(|d| {
                d.children()
                    .iter()
                    .filter_map(|id| self.nodes.get(id))
                    .map(|n| n.value().unwrap_or_default().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// What a fresh O(document) snapshot says the runs hold — the one-shot builder
/// is the independent oracle for the retained projection.
fn truth(app: &App) -> Vec<String> {
    app.semantic_snapshot()
        .nodes
        .iter()
        .filter(|n| crate::semantic::is_run_id(&n.id))
        .map(|n| n.value.clone().unwrap_or_default())
        .collect()
}

/// THE HEADLINE STALENESS LAW. A screen reader may ask for an INITIAL tree again in the
/// middle of a session — macOS re-asks whenever a window is cycled, which is the
/// user's exact trigger — and the answer comes from a slot parked before the
/// window was ever shown. The tree it is handed must describe the document as it
/// is NOW.
///
/// The axis swept is the one counting could not see: the run TEXT, not the run
/// count. Every earlier law here asserts a `ProjectionStats` counter, and a
/// counter says an update was published, never what was in it — a platform
/// holding the whole document as it stood at launch has exactly the right number
/// of runs in it.
#[test]
fn a_reasked_initial_tree_describes_the_document_as_it_is_now() {
    for edit in ["in-line", "newline", "selection"] {
        let _guard = crate::testlock::serial();
        calm_globals();
        let mut app = hermetic();
        app.set_semantic_text_for_test("alpha\nbeta\ngamma");
        app.document.set_cursor(0);
        app.seed_accessibility_tree();
        app.frame.activate_accessibility_for_test();
        app.refresh_accessibility();

        match edit {
            // An ordinary keystroke: the run sequence does not move.
            "in-line" => {
                for ch in "XYZ".chars() {
                    app.document.insert_char(ch);
                    app.refresh_accessibility();
                }
            }
            // A STRUCTURAL change, where a stale child list bites hardest: the
            // platform's document would name runs that no longer exist.
            "newline" => {
                for ch in "one\ntwo\n".chars() {
                    app.document.insert_char(ch);
                    app.refresh_accessibility();
                }
            }
            // No edit at all — only a selection, which is what a screen
            // reader announces and what a stale tree silently breaks.
            "selection" => {
                app.set_semantic_selection_for_test(2, 9);
                app.refresh_accessibility();
            }
            _ => unreachable!(),
        }

        // The window is cycled: the platform asks for an initial tree again.
        app.frame.activate_accessibility_for_test();
        app.refresh_accessibility();

        let mirror = Mirror::of(&app);
        assert_eq!(
            mirror.run_texts(),
            truth(&app),
            "{edit}: a re-asked initial tree handed the screen reader the \
             document as it stood at launch",
        );
        assert_eq!(
            mirror.selection_resolves(),
            Some(true),
            "{edit}: the document's selection names run nodes the platform \
             does not hold, so there is nothing for it to be announced against",
        );
    }
}

/// THE PLATFORM CONTRACT THAT READS AS A BUG, pinned so it is not re-filed.
///
/// `Role::TextRun` is EXCLUDED from a node's accessible children by
/// `accesskit_consumer::common_filter`, which `accesskit_macos` and
/// `accesskit_atspi_common` each re-export as their own `filter`. So a document
/// whose children are all text runs correctly exposes ZERO accessible children
/// on both platforms, and its text reaches a screen reader through the text
/// interface instead. An AT-SPI probe asserting "the document has one child per
/// line" is asserting a shape AccessKit deliberately does not publish.
///
/// Measured against that code rather than read off it: no local gate reaches
/// AT-SPI or VoiceOver, and this repo has been burned before by an oracle
/// derived from a careful source read.
#[test]
fn the_platform_filters_text_runs_out_of_the_documents_accessible_children() {
    let _guard = crate::testlock::serial();
    calm_globals();
    let mut app = hermetic();
    app.set_semantic_text_for_test(&document(3));
    app.document.set_cursor(0);
    let update = crate::semantic::native::tree_update(&app.semantic_snapshot());
    let tree = accesskit_consumer::Tree::new(update, true);
    let state = tree.state();
    let root = state.root();
    let doc = root
        .filtered_children(&accesskit_consumer::common_filter)
        .next()
        .expect("the document must survive the platform's own filter");
    assert!(
        matches!(
            doc.role(),
            accesskit::Role::MultilineTextInput | accesskit::Role::Document
        ),
        "the node under the root is not awl's document: {:?}",
        doc.role(),
    );
    // Unfiltered, the runs are there — awl really does publish them, and the
    // text interface is built from them.
    assert_eq!(
        doc.child_ids().count(),
        3,
        "awl stopped publishing its line runs at all",
    );
    assert_eq!(
        doc.filtered_children(&accesskit_consumer::common_filter)
            .count(),
        0,
        "a text run reached a screen reader as an accessible CHILD; if \
         accesskit's common_filter no longer excludes Role::TextRun, an \
         AT-SPI probe counting run children becomes correct and this law is \
         what says so",
    );
}

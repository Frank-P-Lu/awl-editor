//! Projection and request-decoding laws for the AccessKit bridge.

use super::*;
use crate::semantic::DOCUMENT_ID;

fn app_with(text: &str) -> crate::app::App {
    let mut app = crate::app::App::new_hermetic(
        None,
        std::path::PathBuf::from("/"),
        crate::config::Config::empty(),
    );
    app.set_semantic_text_for_test(text);
    app
}

fn runs_of(snapshot: &SemanticSnapshot) -> Vec<&SemanticNode> {
    document_runs(snapshot)
}

/// Ask the production locator where a document-wide grapheme offset lives.
/// A test that computed the position itself would only be checking its own
/// arithmetic against itself.
fn position(snapshot: &SemanticSnapshot, offset: usize) -> TextPosition {
    locate(&runs_of(snapshot), offset).expect("every offset names a run")
}

fn selection_request(snapshot: &SemanticSnapshot, anchor: usize, focus: usize) -> ActionRequest {
    ActionRequest {
        action: Action::SetTextSelection,
        target_tree: TreeId::ROOT,
        target_node: node_id(DOCUMENT_ID),
        data: Some(ActionData::SetTextSelection(TextSelection {
            anchor: position(snapshot, anchor),
            focus: position(snapshot, focus),
        })),
    }
}

/// Unicode that a run split can actually break: a combining sequence, a ZWJ
/// family, a regional-indicator flag — placed ON both sides of line breaks
/// and at a line's first and last position, because the interior cases are
/// the ones a run representation never gets wrong.
fn boundary_fixture() -> String {
    [
        "e\u{301}dge at the start",
        "👨‍👩‍👧‍👦",
        "ends with a flag 🇯🇵",
        "🇯🇵 starts with a flag",
        "",
        "e\u{301}",
        "plain tail",
    ]
    .join("\n")
}

#[test]
fn accesskit_projection_preserves_every_semantic_node_role_action_and_focus() {
    let _guard = crate::testlock::serial();
    let app = crate::app::App::new_hermetic(
        None,
        std::path::PathBuf::from("/"),
        crate::config::Config::empty(),
    );
    let snapshot = app.semantic_snapshot();
    let update = tree_update(&snapshot);
    assert_eq!(update.nodes.len(), snapshot.nodes.len());
    assert_eq!(update.focus, node_id(&snapshot.focus_id));
    let runs = runs_of(&snapshot);
    for semantic in &snapshot.nodes {
        let projected = update
            .nodes
            .iter()
            .find(|(id, _)| *id == node_id(&semantic.id))
            .map(|(_, node)| node)
            .unwrap();
        assert_eq!(projected.role(), role(semantic));
        for action in &semantic.actions {
            assert!(projected.supports_action(action_to_accesskit(*action)));
        }
    }
    assert!(!runs.is_empty(), "the document announced no text run");
}

/// THE identity a run-based document rests on: the runs, concatenated, are
/// the document, and their grapheme counts add up to the document's. If
/// this drifts, every offset a screen reader sends lands in the wrong
/// place — silently, because both sides are internally consistent.
#[test]
fn the_runs_reproduce_the_document_byte_for_byte_and_grapheme_for_grapheme() {
    let _guard = crate::testlock::serial();
    for text in [
        "",
        "one line",
        "a\nb\nc",
        "trailing newline\n",
        "\n\n\n",
        &boundary_fixture(),
    ] {
        let app = app_with(text);
        let snapshot = app.semantic_snapshot();
        let runs = runs_of(&snapshot);
        let joined: String = runs
            .iter()
            .map(|run| run.value.clone().unwrap_or_default())
            .collect();
        assert_eq!(joined, text, "the runs are not the document: {text:?}");
        let per_run: usize = runs.iter().map(|run| run.character_lengths.len()).sum();
        assert_eq!(
            per_run,
            crate::semantic::grapheme_lengths(text).len(),
            "run grapheme counts disagree with the document's: {text:?}",
        );
    }
}

/// The axis a run representation actually breaks on: every offset, swept,
/// including the ones that sit exactly ON a run boundary and the one past
/// the last grapheme.
#[test]
fn every_grapheme_offset_round_trips_across_run_boundaries() {
    let _guard = crate::testlock::serial();
    let text = boundary_fixture();
    let app = app_with(&text);
    let snapshot = app.semantic_snapshot();
    let runs = runs_of(&snapshot);
    let total = crate::semantic::grapheme_lengths(&text).len();
    assert!(runs.len() > 5, "the fixture must really be multi-run");
    for offset in 0..=total {
        let there = locate(&runs, offset).expect("every offset names a run");
        let back = delocate(&runs, &there).expect("every position names a run");
        assert_eq!(
            back, offset,
            "grapheme {offset} of {total} did not survive the run bridge",
        );
    }
}

/// The runs must be contiguous and in reading order in `snapshot.nodes` —
/// `document_runs` reads them positionally, and the document node's own
/// `children` list is what a screen reader walks.
#[test]
fn the_document_runs_are_contiguous_and_in_reading_order() {
    let _guard = crate::testlock::serial();
    let app = app_with(&boundary_fixture());
    let snapshot = app.semantic_snapshot();
    let indices: Vec<usize> = snapshot
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| crate::semantic::is_run_id(&node.id))
        .map(|(index, _)| index)
        .collect();
    assert!(!indices.is_empty());
    assert_eq!(
        indices.last().unwrap() - indices[0] + 1,
        indices.len(),
        "a non-run node was interleaved with the document's runs",
    );
    let document = snapshot
        .nodes
        .iter()
        .find(|node| node.id == DOCUMENT_ID)
        .expect("the document node");
    let ordered: Vec<&String> = indices.iter().map(|i| &snapshot.nodes[*i].id).collect();
    assert_eq!(
        ordered,
        document.children.iter().collect::<Vec<_>>(),
        "the document's children are not its runs in reading order",
    );
}

#[test]
fn document_selection_request_round_trips_graphemes() {
    let _guard = crate::testlock::serial();
    let app = app_with("e\u{301} 👨‍👩‍👧‍👦 🇯🇵");
    let snapshot = app.semantic_snapshot();
    let decoded = decode_request(&snapshot, selection_request(&snapshot, 1, 3));
    assert_eq!(
        decoded,
        Some(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: 1,
            focus: 3,
        })
    );
}

/// A MULTILINE selection names two different run nodes. The single-run
/// decoder rejected exactly this — `anchor.node != focus.node` returned
/// `None` — so every selection a screen reader made across a line break was
/// silently dropped.
#[test]
fn a_selection_spanning_run_boundaries_round_trips_both_ways() {
    let _guard = crate::testlock::serial();
    let text = boundary_fixture();
    let app = app_with(&text);
    let snapshot = app.semantic_snapshot();
    let total = crate::semantic::grapheme_lengths(&text).len();
    let runs = runs_of(&snapshot);

    // Sweep every ordered pair of run STARTS and ENDS: those are the
    // offsets a boundary bug hides at.
    let mut edges = vec![0usize];
    let mut consumed = 0;
    for run in &runs {
        consumed += run.character_lengths.len();
        edges.push(consumed);
    }
    for anchor in &edges {
        for focus in &edges {
            let decoded = decode_request(&snapshot, selection_request(&snapshot, *anchor, *focus));
            assert_eq!(
                decoded,
                Some(SemanticRequest::SetTextSelection {
                    id: DOCUMENT_ID.to_string(),
                    anchor: (*anchor).min(total),
                    focus: (*focus).min(total),
                }),
                "a selection from {anchor} to {focus} did not survive",
            );
        }
    }

    // And the projected direction: the document node's own text selection
    // must name the two runs the two ends really live in.
    let mut app = app_with(&text);
    app.set_semantic_selection_for_test(0, text.chars().count());
    let snapshot = app.semantic_snapshot();
    let update = tree_update(&snapshot);
    let document = update
        .nodes
        .iter()
        .find(|(id, _)| *id == node_id(DOCUMENT_ID))
        .map(|(_, node)| node)
        .expect("the document node is projected");
    let selection = document
        .text_selection()
        .expect("a whole-document selection is announced");
    assert_ne!(
        selection.anchor.node, selection.focus.node,
        "a selection over a multi-line document collapsed into one run",
    );
}

/// One extended grapheme cluster longer than 255 bytes: a base letter with
/// several hundred combining marks. UAX #29 keeps it as ONE cluster, but
/// AccessKit's `character_lengths` is a `Vec<u8>`, so the cluster has to be
/// carried across several character slots. That split is pure arithmetic
/// in three functions that must agree, and nothing had ever fed them a
/// cluster big enough to make them disagree.
fn oversized_cluster() -> String {
    format!("e{}", "\u{301}".repeat(200))
}

#[test]
fn a_grapheme_longer_than_255_bytes_crosses_the_accesskit_bridge_both_ways() {
    let cluster = oversized_cluster();
    assert!(
        cluster.len() > usize::from(u8::MAX),
        "the fixture must actually exceed one slot",
    );
    assert_eq!(
        crate::semantic::grapheme_lengths(&cluster).len(),
        1,
        "UAX #29 keeps this one cluster; the whole test rests on that",
    );

    // A document of three graphemes: a plain letter, the oversized
    // cluster, and another letter — so the split cluster has neighbours on
    // both sides and an off-by-one cannot hide at an edge.
    let text = format!("a{cluster}z");
    let lengths = crate::semantic::grapheme_lengths(&text);
    assert_eq!(lengths, vec![1, cluster.len(), 1]);

    let expanded = expanded_lengths(&lengths);
    assert_eq!(
        expanded.iter().map(|n| usize::from(*n)).sum::<usize>(),
        text.len(),
        "every byte of the document is accounted for exactly once",
    );
    assert_eq!(
        expanded,
        vec![1, u8::MAX, (cluster.len() - usize::from(u8::MAX)) as u8, 1],
        "the cluster occupies as many full slots as it needs plus a tail",
    );

    // FORWARD: semantic grapheme offset -> AccessKit character index.
    assert_eq!(expanded_index(&lengths, 0), 0);
    assert_eq!(expanded_index(&lengths, 1), 1, "before the big cluster");
    assert_eq!(expanded_index(&lengths, 2), 3, "the cluster spent 2 slots");
    assert_eq!(expanded_index(&lengths, 3), 4, "end of the document");

    // BACK: every AccessKit character index — including the one INSIDE the
    // split cluster, which a screen reader can and will produce — maps to a
    // grapheme boundary, never to the middle of a cluster.
    assert_eq!(semantic_index(&lengths, 0), 0);
    assert_eq!(semantic_index(&lengths, 1), 1);
    assert_eq!(
        semantic_index(&lengths, 2),
        1,
        "a position inside the split cluster clamps to the cluster's start",
    );
    assert_eq!(semantic_index(&lengths, 3), 2);
    assert_eq!(semantic_index(&lengths, 4), 3);

    // ROUND TRIP: forward then back is the identity on every boundary.
    for index in 0..=lengths.len() {
        assert_eq!(
            semantic_index(&lengths, expanded_index(&lengths, index)),
            index,
            "grapheme {index} did not survive the bridge",
        );
    }
}

/// The same oversized cluster, now with a LINE BREAK on each side of it, so
/// the slot-splitting arithmetic and the run-boundary arithmetic have to be
/// right at the same time.
#[test]
fn an_oversized_cluster_round_trips_through_a_real_selection_request() {
    let _guard = crate::testlock::serial();
    let cluster = oversized_cluster();
    let text = format!("before\na{cluster}z\nafter");
    let app = app_with(&text);
    let snapshot = app.semantic_snapshot();
    let runs = runs_of(&snapshot);
    assert_eq!(runs.len(), 3, "the fixture must be three runs");
    assert_eq!(
        runs[1].character_lengths.iter().map(|n| *n).sum::<usize>(),
        2 + cluster.len() + 1,
        "the middle run carries the cluster, its neighbours and its newline",
    );

    // "before\n" is 7 graphemes; the cluster is the 9th grapheme overall.
    let before = crate::semantic::grapheme_lengths("before\n").len();
    let decoded = decode_request(
        &snapshot,
        selection_request(&snapshot, before + 1, before + 2),
    );
    assert_eq!(
        decoded,
        Some(SemanticRequest::SetTextSelection {
            id: DOCUMENT_ID.to_string(),
            anchor: before + 1,
            focus: before + 2,
        }),
        "the split cluster must decode to ONE grapheme, not two",
    );
}

/// JSON is the agent's view and AccessKit is the screen reader's. Both are
/// projections of one snapshot, so an agent and a screen reader must never
/// be told different things — including after a full serialize/parse cycle,
/// which is what `--semantic-json` and the capture sidecar actually do.
#[test]
fn json_and_accesskit_are_projections_of_the_same_snapshot() {
    let _guard = crate::testlock::serial();
    for surface in ["editor", "overlay", "search"] {
        let mut app = app_with("e\u{301} 👨‍👩‍👧‍👦 prose\nand a second line");
        app.install_semantic_fixture_for_test(surface);
        let snapshot = app.semantic_snapshot();

        let json = serde_json::to_string(&snapshot).expect("the snapshot serializes");
        let parsed: SemanticSnapshot =
            serde_json::from_str(&json).expect("the snapshot parses back");
        assert_eq!(parsed, snapshot, "{surface}: JSON lost or changed a fact");

        let direct = tree_update(&snapshot);
        let via_json = tree_update(&parsed);
        assert_eq!(direct.focus, via_json.focus, "{surface}");
        assert_eq!(direct.nodes.len(), via_json.nodes.len(), "{surface}");
        for ((left_id, left), (right_id, right)) in direct.nodes.iter().zip(via_json.nodes.iter()) {
            assert_eq!(left_id, right_id, "{surface}");
            assert_eq!(left, right, "{surface}: node {left_id:?} diverged");
        }

        // The two views name the same nodes: every id the agent can read
        // hashes to a node the screen reader is given, and vice versa.
        let mut projected: Vec<NodeId> = direct.nodes.iter().map(|(id, _)| *id).collect();
        projected.sort_by_key(|id| id.0);
        let mut expected: Vec<NodeId> = parsed.nodes.iter().map(|node| node_id(&node.id)).collect();
        expected.sort_by_key(|id| id.0);
        assert_eq!(projected, expected, "{surface}");
        assert!(
            parsed.nodes.iter().any(|node| node.id == parsed.focus_id),
            "{surface}: focus_id names a node neither view contains",
        );
    }
}

/// An incremental update carries the changed nodes and NO `Tree`: AccessKit
/// reads a `Tree` as "this is a whole new tree", which is the very shape
/// the retained projector keeps off the per-frame path.
#[test]
fn an_incremental_update_names_only_what_changed_and_declares_no_tree() {
    let _guard = crate::testlock::serial();
    let app = app_with("alpha\nbeta\ngamma");
    let snapshot = app.semantic_snapshot();
    let run = snapshot
        .nodes
        .iter()
        .find(|node| crate::semantic::is_run_id(&node.id))
        .expect("a run")
        .id
        .clone();
    let update = incremental_tree_update(&snapshot, &[run.clone(), DOCUMENT_ID.to_string()]);
    assert!(
        update.tree.is_none(),
        "an incremental update declared a whole tree",
    );
    assert_eq!(update.nodes.len(), 2);
    assert_eq!(update.focus, node_id(&snapshot.focus_id));
    let ids: Vec<NodeId> = update.nodes.iter().map(|(id, _)| *id).collect();
    assert!(ids.contains(&node_id(&run)));
    assert!(ids.contains(&node_id(DOCUMENT_ID)));
}

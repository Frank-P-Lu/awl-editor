//! Pure AccessKit projection and request decoding.

use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Role, TextPosition, TextSelection, Toggled,
    Tree, TreeId, TreeUpdate,
};

use super::{
    SemanticAction, SemanticNode, SemanticRequest, SemanticRole, SemanticSelection,
    SemanticSnapshot,
};

pub fn node_id(id: &str) -> NodeId {
    // Stable FNV-1a; identity must survive process restarts and filtering.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    NodeId(hash)
}

pub fn tree_update(snapshot: &SemanticSnapshot) -> TreeUpdate {
    let nodes = snapshot
        .nodes
        .iter()
        .map(|semantic| (node_id(&semantic.id), project_node(snapshot, semantic)))
        .collect();
    let mut tree = Tree::new(node_id(&snapshot.root_id));
    tree.toolkit_name = Some("awl".to_string());
    TreeUpdate {
        nodes,
        tree: Some(tree),
        tree_id: TreeId::ROOT,
        focus: node_id(&snapshot.focus_id),
    }
}

fn project_node(snapshot: &SemanticSnapshot, semantic: &SemanticNode) -> Node {
    let mut node = Node::new(role(semantic));
    if !semantic.name.is_empty() {
        node.set_label(semantic.name.clone());
    }
    if let Some(value) = &semantic.value {
        node.set_value(value.clone());
    }
    if let Some(description) = &semantic.description {
        node.set_description(description.clone());
    }
    node.set_children(
        semantic
            .children
            .iter()
            .map(|id| node_id(id))
            .collect::<Vec<_>>(),
    );
    node.set_controls(
        semantic
            .controls
            .iter()
            .map(|id| node_id(id))
            .collect::<Vec<_>>(),
    );
    for action in &semantic.actions {
        node.add_action(action_to_accesskit(*action));
    }
    if let Some(selected) = semantic.selected {
        node.set_selected(selected);
    }
    if let Some(checked) = semantic.checked {
        node.set_toggled(Toggled::from(checked));
    }
    if let Some(expanded) = semantic.expanded {
        node.set_expanded(expanded);
    }
    if semantic.role == SemanticRole::Text && !semantic.character_lengths.is_empty() {
        node.set_character_lengths(expanded_lengths(&semantic.character_lengths));
    }
    if let Some(selection) = semantic.selection
        && let Some(text) = semantic
            .children
            .iter()
            .filter_map(|id| snapshot.nodes.iter().find(|node| node.id == *id))
            .find(|node| node.role == SemanticRole::Text)
    {
        node.set_text_selection(text_selection(text, selection));
    }
    node
}

fn role(node: &SemanticNode) -> Role {
    match node.role {
        SemanticRole::Application => Role::Application,
        SemanticRole::Document if node.editable && node.multiline => Role::MultilineTextInput,
        SemanticRole::Document => Role::Document,
        SemanticRole::Text => Role::TextRun,
        SemanticRole::Dialog => Role::Dialog,
        SemanticRole::Group => Role::Group,
        SemanticRole::TextInput if node.multiline => Role::MultilineTextInput,
        SemanticRole::TextInput => Role::TextInput,
        SemanticRole::ListBox => Role::ListBox,
        SemanticRole::Option => Role::ListBoxOption,
        SemanticRole::Button => Role::Button,
        SemanticRole::CheckBox => Role::CheckBox,
        SemanticRole::Slider => Role::Slider,
        SemanticRole::Status => Role::Status,
        SemanticRole::MenuBar => Role::MenuBar,
        SemanticRole::MenuItem => Role::MenuItem,
        SemanticRole::Heading => Role::Heading,
        SemanticRole::StaticText => Role::Label,
    }
}

fn action_to_accesskit(action: SemanticAction) -> Action {
    match action {
        SemanticAction::Focus => Action::Focus,
        SemanticAction::Click => Action::Click,
        SemanticAction::SetTextSelection => Action::SetTextSelection,
        SemanticAction::ReplaceSelectedText => Action::ReplaceSelectedText,
        SemanticAction::SetValue => Action::SetValue,
        SemanticAction::Increment => Action::Increment,
        SemanticAction::Decrement => Action::Decrement,
        SemanticAction::Expand => Action::Expand,
        SemanticAction::Collapse => Action::Collapse,
    }
}

fn expanded_lengths(lengths: &[usize]) -> Vec<u8> {
    lengths
        .iter()
        .flat_map(|&length| {
            let full = length / usize::from(u8::MAX);
            let tail = length % usize::from(u8::MAX);
            std::iter::repeat_n(u8::MAX, full).chain((tail > 0).then_some(tail as u8))
        })
        .collect()
}

fn expanded_index(lengths: &[usize], index: usize) -> usize {
    lengths
        .iter()
        .take(index)
        .map(|length| length.div_ceil(usize::from(u8::MAX)))
        .sum()
}

fn semantic_index(lengths: &[usize], expanded: usize) -> usize {
    let mut consumed = 0;
    for (index, length) in lengths.iter().enumerate() {
        let pieces = length.div_ceil(usize::from(u8::MAX));
        if expanded < consumed + pieces {
            return index;
        }
        consumed += pieces;
    }
    lengths.len()
}

fn text_selection(text: &SemanticNode, selection: SemanticSelection) -> TextSelection {
    let id = node_id(&text.id);
    TextSelection {
        anchor: TextPosition {
            node: id,
            character_index: expanded_index(&text.character_lengths, selection.anchor),
        },
        focus: TextPosition {
            node: id,
            character_index: expanded_index(&text.character_lengths, selection.focus),
        },
    }
}

pub fn decode_request(
    snapshot: &SemanticSnapshot,
    request: ActionRequest,
) -> Option<SemanticRequest> {
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node_id(&node.id) == request.target_node)?;
    let id = node.id.clone();
    match (request.action, request.data) {
        (Action::Focus, _) => Some(SemanticRequest::Focus { id }),
        (Action::Click, _) => Some(SemanticRequest::Click { id }),
        (Action::Increment, _) => Some(SemanticRequest::Increment { id }),
        (Action::Decrement, _) => Some(SemanticRequest::Decrement { id }),
        (Action::Expand, _) => Some(SemanticRequest::Expand { id }),
        (Action::Collapse, _) => Some(SemanticRequest::Collapse { id }),
        (Action::ReplaceSelectedText, Some(ActionData::Value(value))) => {
            Some(SemanticRequest::ReplaceSelectedText {
                id,
                value: value.into(),
            })
        }
        (Action::SetValue, Some(ActionData::Value(value))) => Some(SemanticRequest::SetValue {
            id,
            value: value.into(),
        }),
        (Action::SetTextSelection, Some(ActionData::SetTextSelection(selection))) => {
            let text = node
                .children
                .iter()
                .filter_map(|id| snapshot.nodes.iter().find(|candidate| candidate.id == *id))
                .find(|candidate| node_id(&candidate.id) == selection.anchor.node)?;
            if selection.anchor.node != selection.focus.node {
                return None;
            }
            Some(SemanticRequest::SetTextSelection {
                id,
                anchor: semantic_index(&text.character_lengths, selection.anchor.character_index),
                focus: semantic_index(&text.character_lengths, selection.focus.character_index),
            })
        }
        (Action::SetValue, Some(ActionData::NumericValue(value))) => {
            Some(SemanticRequest::SetValue {
                id,
                value: value.to_string(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{DOCUMENT_ID, DOCUMENT_TEXT_ID};

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
    }

    #[test]
    fn document_selection_request_round_trips_graphemes() {
        let _guard = crate::testlock::serial();
        let mut app = crate::app::App::new_hermetic(
            None,
            std::path::PathBuf::from("/"),
            crate::config::Config::empty(),
        );
        app.set_semantic_text_for_test("e\u{301} 👨‍👩‍👧‍👦 🇯🇵");
        let snapshot = app.semantic_snapshot();
        let selection = TextSelection {
            anchor: TextPosition {
                node: node_id(DOCUMENT_TEXT_ID),
                character_index: 1,
            },
            focus: TextPosition {
                node: node_id(DOCUMENT_TEXT_ID),
                character_index: 3,
            },
        };
        let decoded = decode_request(
            &snapshot,
            ActionRequest {
                action: Action::SetTextSelection,
                target_tree: TreeId::ROOT,
                target_node: node_id(DOCUMENT_ID),
                data: Some(ActionData::SetTextSelection(selection)),
            },
        );
        assert_eq!(
            decoded,
            Some(SemanticRequest::SetTextSelection {
                id: DOCUMENT_ID.to_string(),
                anchor: 1,
                focus: 3,
            })
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

    #[test]
    fn an_oversized_cluster_round_trips_through_a_real_selection_request() {
        let _guard = crate::testlock::serial();
        let mut app = crate::app::App::new_hermetic(
            None,
            std::path::PathBuf::from("/"),
            crate::config::Config::empty(),
        );
        let cluster = oversized_cluster();
        app.set_semantic_text_for_test(&format!("a{cluster}z"));
        let snapshot = app.semantic_snapshot();
        let update = tree_update(&snapshot);
        let text_node = update
            .nodes
            .iter()
            .find(|(id, _)| *id == node_id(DOCUMENT_TEXT_ID))
            .map(|(_, node)| node)
            .expect("the document text node is projected");
        assert_eq!(
            text_node
                .character_lengths()
                .iter()
                .map(|n| usize::from(*n))
                .sum::<usize>(),
            2 + cluster.len(),
        );

        // Select from just before the cluster to just after it, expressed in
        // AccessKit's expanded character space, and decode it back.
        let selection = TextSelection {
            anchor: TextPosition {
                node: node_id(DOCUMENT_TEXT_ID),
                character_index: 1,
            },
            focus: TextPosition {
                node: node_id(DOCUMENT_TEXT_ID),
                character_index: 3,
            },
        };
        let decoded = decode_request(
            &snapshot,
            ActionRequest {
                action: Action::SetTextSelection,
                target_tree: TreeId::ROOT,
                target_node: node_id(DOCUMENT_ID),
                data: Some(ActionData::SetTextSelection(selection)),
            },
        );
        assert_eq!(
            decoded,
            Some(SemanticRequest::SetTextSelection {
                id: DOCUMENT_ID.to_string(),
                anchor: 1,
                focus: 2,
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
            let mut app = crate::app::App::new_hermetic(
                None,
                std::path::PathBuf::from("/"),
                crate::config::Config::empty(),
            );
            app.set_semantic_text_for_test("e\u{301} 👨‍👩‍👧‍👦 prose");
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
            for ((left_id, left), (right_id, right)) in
                direct.nodes.iter().zip(via_json.nodes.iter())
            {
                assert_eq!(left_id, right_id, "{surface}");
                assert_eq!(left, right, "{surface}: node {left_id:?} diverged");
            }

            // The two views name the same nodes: every id the agent can read
            // hashes to a node the screen reader is given, and vice versa.
            let mut projected: Vec<NodeId> = direct.nodes.iter().map(|(id, _)| *id).collect();
            projected.sort_by_key(|id| id.0);
            let mut expected: Vec<NodeId> =
                parsed.nodes.iter().map(|node| node_id(&node.id)).collect();
            expected.sort_by_key(|id| id.0);
            assert_eq!(projected, expected, "{surface}");
            assert!(
                parsed.nodes.iter().any(|node| node.id == parsed.focus_id),
                "{surface}: focus_id names a node neither view contains",
            );
        }
    }
}

//! Pure AccessKit projection and request decoding.

use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Role, TextPosition, TextSelection, Toggled,
    Tree, TreeId, TreeUpdate,
};

use super::{SemanticAction, SemanticNode, SemanticRequest, SemanticRole, SemanticSnapshot};

pub fn node_id(id: &str) -> NodeId {
    // Stable FNV-1a; identity must survive process restarts and filtering.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    NodeId(hash)
}

/// Retained NATIVE projection state.
///
/// One thing lives here, and it is the difference between "publish the changed
/// nodes" and "publish something document-sized anyway". The document node is
/// republished on every keystroke — its selection moved — and an AccessKit
/// `Node` is a VALUE, so its whole child list travels with it. Re-deriving
/// those ids from their id strings each time cost more than the rest of the
/// update put together: 0.79 ms of 0.84 at 50 000 lines, measured. They are
/// rebuilt only when the run sequence moves.
#[derive(Default)]
pub struct TreeProjector {
    document_children: Vec<NodeId>,
    shape: Option<u64>,
}

impl TreeProjector {
    /// Forget the cache — a new platform adapter, or a document swap.
    pub fn invalidate(&mut self) {
        self.shape = None;
        self.document_children.clear();
    }

    fn document_children(&mut self, snapshot: &SemanticSnapshot, shape: u64) -> &[NodeId] {
        if self.shape != Some(shape) {
            self.document_children = snapshot
                .nodes
                .iter()
                .find(|node| node.id == super::DOCUMENT_ID)
                .map(|document| document.children.iter().map(|id| node_id(id)).collect())
                .unwrap_or_default();
            self.shape = Some(shape);
        }
        &self.document_children
    }

    /// THE FULL TREE — every node, plus the `Tree` metadata that declares the
    /// root.
    ///
    /// AccessKit wants this exactly once per activation: it is what a platform
    /// adapter needs before it can hold anything, and what
    /// [`accesskit_winit::Adapter::update_if_active`] requires when the
    /// activation handler returned `None`. Publishing it on every REDRAW
    /// instead is the stall this projector exists to retire.
    pub fn full(&mut self, snapshot: &SemanticSnapshot, shape: u64) -> TreeUpdate {
        let runs = document_runs(snapshot);
        let children = self.document_children(snapshot, shape).to_vec();
        let nodes = snapshot
            .nodes
            .iter()
            .map(|semantic| {
                (
                    node_id(&semantic.id),
                    project_node(&runs, semantic, &children),
                )
            })
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

    /// THE CHANGED NODES, and nothing else.
    ///
    /// `tree` is `None`: AccessKit treats an update without it as a change to a
    /// tree the platform already holds, and a node the update does not name
    /// keeps the state it already had. A node dropped from its parent's
    /// `children` is released by that parent's own update, so removals need no
    /// entry here.
    pub fn incremental(
        &mut self,
        snapshot: &SemanticSnapshot,
        changed: &[String],
        shape: u64,
    ) -> TreeUpdate {
        // Only pay for the run index and the child list when the node that
        // reads them is actually being published.
        let touches_document = changed.iter().any(|id| id == super::DOCUMENT_ID);
        let runs = if touches_document {
            document_runs(snapshot)
        } else {
            Vec::new()
        };
        let children = if touches_document {
            self.document_children(snapshot, shape).to_vec()
        } else {
            Vec::new()
        };
        let mut nodes = Vec::with_capacity(changed.len());
        let mut seen: Vec<&str> = Vec::with_capacity(changed.len());
        for id in changed {
            if seen.contains(&id.as_str()) {
                continue;
            }
            let Some(semantic) = snapshot.nodes.iter().find(|node| node.id == *id) else {
                continue;
            };
            seen.push(id.as_str());
            nodes.push((node_id(id), project_node(&runs, semantic, &children)));
        }
        TreeUpdate {
            nodes,
            tree: None,
            tree_id: TreeId::ROOT,
            focus: node_id(&snapshot.focus_id),
        }
    }
}

/// A whole tree with no retained state — for the one-shot consumers and the
/// laws, which have no projector to carry.
pub fn tree_update(snapshot: &SemanticSnapshot) -> TreeUpdate {
    TreeProjector::default().full(snapshot, 0)
}

pub fn incremental_tree_update(snapshot: &SemanticSnapshot, changed: &[String]) -> TreeUpdate {
    TreeProjector::default().incremental(snapshot, changed, 0)
}

/// The document's text runs, in reading order.
///
/// They are contiguous and already ordered in `snapshot.nodes` — the projection
/// builds them that way, and `the_document_runs_are_contiguous_and_in_reading_order`
/// holds it — so this is one filtered pass rather than a lookup per child.
fn document_runs(snapshot: &SemanticSnapshot) -> Vec<&SemanticNode> {
    snapshot
        .nodes
        .iter()
        .filter(|node| super::is_run_id(&node.id))
        .collect()
}

/// A document-wide GRAPHEME offset as the AccessKit position it names: which
/// run holds it, and where inside that run's expanded character space.
///
/// Selections that cross a run boundary are the ordinary case here, not an edge
/// one — every multi-line selection is one — so the anchor and the focus are
/// located independently and may name different nodes.
fn locate(runs: &[&SemanticNode], offset: usize) -> Option<TextPosition> {
    let mut consumed = 0;
    for run in runs {
        let length = run.character_lengths.len();
        if offset < consumed + length {
            return Some(TextPosition {
                node: node_id(&run.id),
                character_index: expanded_index(&run.character_lengths, offset - consumed),
            });
        }
        consumed += length;
    }
    // The end of the document: the last run's end, never a position in a node
    // that does not exist.
    runs.last().map(|run| TextPosition {
        node: node_id(&run.id),
        character_index: expanded_index(&run.character_lengths, run.character_lengths.len()),
    })
}

/// The inverse of [`locate`]: an AccessKit position back to a document-wide
/// grapheme offset. A position INSIDE a grapheme that had to be split across
/// several `character_lengths` slots clamps to that grapheme's start, exactly
/// as it did when the document was one run.
fn delocate(runs: &[&SemanticNode], position: &TextPosition) -> Option<usize> {
    let mut consumed = 0;
    for run in runs {
        if node_id(&run.id) == position.node {
            return Some(
                consumed + semantic_index(&run.character_lengths, position.character_index),
            );
        }
        consumed += run.character_lengths.len();
    }
    None
}

fn project_node(
    runs: &[&SemanticNode],
    semantic: &SemanticNode,
    document_children: &[NodeId],
) -> Node {
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
    // The document's child list is the one that is document-sized, so it comes
    // from the projector's cache rather than being re-derived per keystroke.
    if semantic.id == super::DOCUMENT_ID && document_children.len() == semantic.children.len() {
        node.set_children(document_children.to_vec());
    } else {
        node.set_children(
            semantic
                .children
                .iter()
                .map(|id| node_id(id))
                .collect::<Vec<_>>(),
        );
    }
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
        && let Some(anchor) = locate(runs, selection.anchor)
        && let Some(focus) = locate(runs, selection.focus)
    {
        node.set_text_selection(TextSelection { anchor, focus });
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
            // A multi-line selection names two DIFFERENT run nodes, so the two
            // ends are resolved independently. Requiring one shared node — as
            // this did while the document was a single run — would silently
            // drop every selection a screen reader made across a line break.
            let runs = document_runs(snapshot);
            Some(SemanticRequest::SetTextSelection {
                id,
                anchor: delocate(&runs, &selection.anchor)?,
                focus: delocate(&runs, &selection.focus)?,
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
mod tests;

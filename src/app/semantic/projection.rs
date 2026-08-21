//! The RETAINED semantic projection — the state that makes an update
//! incremental.
//!
//! `SemanticSnapshot` is still the one semantic owner; nothing here is a second
//! description of the UI. What this file adds is memory: the snapshot is kept
//! between frames and UPDATED IN PLACE, so an ordinary keystroke re-reads one
//! line of the rope instead of cloning the whole document, re-segmenting it
//! under UAX #29 and reprojecting every node.
//!
//! The document half is retained — one node per stable line run, refreshed only
//! where [`crate::semantic::runs::RunTable`] says a line changed. The surface
//! half (pickers, search, cards, which-key, the menu bar, the notice) is
//! rebuilt every refresh and diffed, because it is bounded by what is on screen
//! and never by the document's size.
//!
//! Every refresh records what it actually did in [`ProjectionStats`]. Those
//! counters are the witness: a latency number alone cannot tell "one line was
//! re-read" from "nothing was re-read", and a bench that measures a no-op is
//! how a green perf test hides a regression.

use super::*;
use crate::semantic::runs::{Run, RunId};

/// Where the retained document nodes sit in `snapshot.nodes`: the root and the
/// document node first, the runs after them in line order, the surface tail
/// last. Fixed positions, because a projection that had to SEARCH for the
/// document node every frame would be O(document) again.
const ROOT_INDEX: usize = 0;
const DOCUMENT_INDEX: usize = 1;
const RUN_BASE: usize = 2;

/// What one refresh actually cost, in units a law can assert on.
///
/// `bytes_read` and `graphemes_segmented` used to be the whole document on
/// every frame; `runs_rebuilt` is how many line runs were reprojected;
/// `nodes_published` is how many nodes reached the platform. A witness asserts
/// these counts, not just the clock — CLAUDE.md records a theme bench that
/// "measured" 5 ms while nothing reshaped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProjectionStats {
    pub(crate) refreshes: u64,
    pub(crate) seeds: u64,
    pub(crate) runs_rebuilt: u64,
    pub(crate) bytes_read: u64,
    pub(crate) graphemes_segmented: u64,
    pub(crate) nodes_published: u64,
    pub(crate) full_trees: u64,
    pub(crate) children_republished: u64,
}

#[derive(Clone, Debug)]
struct Slot {
    id: RunId,
    /// The run's `rev` at the moment this projection last read its line.
    rev: u64,
    /// How many graphemes the projected run holds. Kept beside the node so a
    /// document-wide grapheme offset is located by summing integers rather than
    /// by walking the document's text.
    graphemes: usize,
}

pub(crate) struct SemanticProjection {
    snapshot: SemanticSnapshot,
    slots: Vec<Slot>,
    content_rev: u64,
    shape_rev: u64,
    seeded: bool,
    document_present: bool,
    /// The surface tail as last published, so a refresh can tell which of those
    /// nodes actually moved.
    tail: Vec<SemanticNode>,
    /// The last resolved selection, keyed by the char offsets and the content
    /// revision it was resolved against — so a gliding caret with no input, and
    /// a frame with no edit, re-read nothing at all.
    resolved: Option<(usize, usize, u64, SemanticSelection)>,
    changed: Vec<String>,
    stats: ProjectionStats,
}

impl Default for SemanticProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticProjection {
    pub(crate) fn new() -> Self {
        Self {
            snapshot: SemanticSnapshot {
                schema: crate::semantic::SCHEMA.to_string(),
                root_id: ROOT_ID.to_string(),
                focus_id: DOCUMENT_ID.to_string(),
                nodes: Vec::new(),
            },
            slots: Vec::new(),
            content_rev: 0,
            shape_rev: 0,
            seeded: false,
            document_present: false,
            tail: Vec::new(),
            resolved: None,
            changed: Vec::new(),
            stats: ProjectionStats::default(),
        }
    }

    pub(crate) fn snapshot(&self) -> &SemanticSnapshot {
        &self.snapshot
    }

    pub(crate) fn into_snapshot(self) -> SemanticSnapshot {
        self.snapshot
    }

    /// The node ids this refresh changed — exactly what an incremental
    /// `TreeUpdate` publishes.
    pub(crate) fn changed(&self) -> &[String] {
        &self.changed
    }

    pub(crate) fn stats(&self) -> ProjectionStats {
        self.stats
    }

    /// The run SEQUENCE's revision — the cache key for anything derived from
    /// the document's child list.
    pub(crate) fn shape_rev(&self) -> u64 {
        self.shape_rev
    }

    /// Has this projection ever been built? A projection that has not is owed a
    /// full tree, because the platform is holding a placeholder.
    pub(crate) fn is_seeded(&self) -> bool {
        self.seeded
    }

    /// Forget everything and rebuild from scratch on the next refresh — used
    /// when a screen reader lets go, so a later reattach cannot be handed a
    /// diff against a tree the new platform adapter never saw.
    pub(crate) fn invalidate(&mut self) {
        self.seeded = false;
        self.resolved = None;
    }

    /// Bring the retained snapshot up to date. The narrow view is the whole
    /// input: this projection cannot see the application state behind it.
    pub(in crate::app) fn refresh(&mut self, view: &SemanticView<'_>) {
        self.changed.clear();
        self.stats.refreshes += 1;
        if view.buffer().is_none() {
            self.refresh_start(view);
            return;
        }
        if !self.document_present {
            self.invalidate();
        }
        let shape_moved = if self.seeded {
            self.sync_runs(view)
        } else {
            self.seed(view);
            true
        };
        self.sync_document(view, shape_moved);
        self.rebuild_tail(view);
        self.document_present = true;
        debug_assert_eq!(
            self.snapshot
                .nodes
                .iter()
                .filter(|node| node.focused)
                .count(),
            1,
            "the projection published more or less than one focus owner",
        );
    }

    // --- the document half -------------------------------------------------

    fn refresh_start(&mut self, view: &SemanticView<'_>) {
        let mut root = SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl");
        root.children = vec![START_NEW_ID.to_string(), START_GOTO_ID.to_string()];
        let button = |id, name| {
            let mut node = SemanticNode::new(id, SemanticRole::Button, name);
            node.focusable = true;
            node.actions = vec![SemanticAction::Focus, SemanticAction::Click];
            node
        };
        self.snapshot.nodes = vec![
            root,
            button(START_NEW_ID, "New document"),
            button(START_GOTO_ID, "Go to"),
        ];
        let focus_id = view.fold_surfaces(&mut self.snapshot.nodes);
        self.snapshot.focus_id = focus_id.clone();
        for node in &mut self.snapshot.nodes {
            node.focused = node.id == focus_id;
        }
        self.slots.clear();
        self.tail.clear();
        self.resolved = None;
        self.content_rev = 0;
        self.shape_rev = 0;
        self.seeded = true;
        self.document_present = false;
        self.changed = self
            .snapshot
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect();
        debug_assert_eq!(
            self.snapshot
                .nodes
                .iter()
                .filter(|node| node.focused)
                .count(),
            1,
            "the start projection must publish one focus owner"
        );
    }

    fn seed(&mut self, view: &SemanticView<'_>) {
        self.stats.seeds += 1;
        let buffer = view.buffer().expect("document projection has a buffer");
        let table = buffer.runs();
        let mut root = SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl");
        root.children.push(DOCUMENT_ID.to_string());
        let mut document = SemanticNode::new(DOCUMENT_ID, SemanticRole::Document, String::new());
        document.focusable = true;
        document.editable = true;
        document.multiline = true;
        document.actions = vec![
            SemanticAction::Focus,
            SemanticAction::SetTextSelection,
            SemanticAction::ReplaceSelectedText,
            SemanticAction::SetValue,
        ];
        self.snapshot.nodes.clear();
        self.snapshot.nodes.push(root);
        self.snapshot.nodes.push(document);
        self.slots.clear();
        for (line, run) in table.runs().iter().enumerate() {
            let (slot, node) = self.build_run(buffer, line, *run);
            self.slots.push(slot);
            self.snapshot.nodes.push(node);
        }
        self.content_rev = table.content_rev();
        self.shape_rev = table.shape_rev();
        self.resolved = None;
        self.seeded = true;
        self.tail.clear();
        // A seed is paired with a full tree by its caller; naming every node in
        // `changed` as well would double-publish the whole document.
        self.changed.clear();
    }

    /// Returns whether the run SEQUENCE moved.
    fn sync_runs(&mut self, view: &SemanticView<'_>) -> bool {
        let buffer = view.buffer().expect("document projection has a buffer");
        let table = buffer.runs();
        if table.content_rev() == self.content_rev {
            return false;
        }
        if table.shape_rev() == self.shape_rev {
            // The common case: the same lines, one of them re-typed. Only the
            // marked runs are re-read, and no parent republishes its children.
            for (line, run) in table.runs().iter().enumerate() {
                if self.slots[line].rev == run.rev {
                    continue;
                }
                let (slot, node) = self.build_run(buffer, line, *run);
                self.changed.push(node.id.clone());
                self.slots[line] = slot;
                self.snapshot.nodes[RUN_BASE + line] = node;
            }
            self.content_rev = table.content_rev();
            return false;
        }
        self.resplice(buffer, table.runs());
        self.content_rev = table.content_rev();
        self.shape_rev = table.shape_rev();
        true
    }

    /// A line was added or removed. Every run whose id AND rev survived is
    /// reused untouched — that is what keeps a newline typed at the top of a
    /// long document from reprojecting the lines below it — and the document
    /// node republishes its `children`, which is the one list a structural
    /// change genuinely invalidates.
    fn resplice(&mut self, buffer: &Buffer, runs: &[Run]) {
        let mut retained: std::collections::HashMap<RunId, (Slot, SemanticNode)> = self
            .slots
            .drain(..)
            .zip(self.snapshot.nodes.drain(RUN_BASE..))
            .map(|(slot, node)| (slot.id, (slot, node)))
            .collect();
        self.slots.reserve(runs.len());
        self.snapshot.nodes.reserve(runs.len());
        for (line, run) in runs.iter().enumerate() {
            match retained.remove(&run.id) {
                Some((slot, node)) if slot.rev == run.rev => {
                    self.slots.push(slot);
                    self.snapshot.nodes.push(node);
                }
                _ => {
                    let (slot, node) = self.build_run(buffer, line, *run);
                    self.changed.push(node.id.clone());
                    self.slots.push(slot);
                    self.snapshot.nodes.push(node);
                }
            }
        }
        self.resolved = None;
        self.stats.children_republished += 1;
    }

    fn build_run(&mut self, buffer: &Buffer, line: usize, run: Run) -> (Slot, SemanticNode) {
        let text = buffer.run_text(line);
        let lengths = crate::semantic::grapheme_lengths(&text);
        self.stats.runs_rebuilt += 1;
        self.stats.bytes_read += text.len() as u64;
        self.stats.graphemes_segmented += lengths.len() as u64;
        let mut node = SemanticNode::new(
            crate::semantic::run_node_id(run.id),
            SemanticRole::Text,
            "Markdown",
        );
        let slot = Slot {
            id: run.id,
            rev: run.rev,
            graphemes: lengths.len(),
        };
        node.value = Some(text);
        node.character_lengths = lengths;
        (slot, node)
    }

    /// The document node: its name, its focus, its selection, and — only when
    /// the run sequence moved — its children.
    fn sync_document(&mut self, view: &SemanticView<'_>, shape_moved: bool) {
        let buffer = view.buffer().expect("document projection has a buffer");
        let name = buffer
            .path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled document".to_string());
        let focused = matches!(view.layer(), workspace::Layer::Editor);
        let selection = self.selection(buffer);

        let mut moved = false;
        if shape_moved {
            let children: Vec<String> = self
                .slots
                .iter()
                .map(|slot| crate::semantic::run_node_id(slot.id))
                .collect();
            self.snapshot.nodes[DOCUMENT_INDEX].children = children;
            moved = true;
        }
        let node = &mut self.snapshot.nodes[DOCUMENT_INDEX];
        if node.name != name {
            node.name = name;
            moved = true;
        }
        if node.focused != focused {
            node.focused = focused;
            moved = true;
        }
        if node.selection != Some(selection) {
            node.selection = Some(selection);
            moved = true;
        }
        if moved {
            self.changed.push(DOCUMENT_ID.to_string());
        }
    }

    fn selection(&mut self, buffer: &Buffer) -> SemanticSelection {
        let cursor = buffer.cursor_char();
        let anchor = buffer.anchor_char().unwrap_or(cursor);
        if let Some((was_anchor, was_cursor, rev, selection)) = self.resolved
            && was_anchor == anchor
            && was_cursor == cursor
            && rev == self.content_rev
        {
            return selection;
        }
        let selection = SemanticSelection {
            anchor: self.document_grapheme_offset(buffer, anchor),
            focus: self.document_grapheme_offset(buffer, cursor),
        };
        self.resolved = Some((anchor, cursor, self.content_rev, selection));
        selection
    }

    /// A char offset in the rope as a document-wide GRAPHEME offset, without
    /// reading the document: the runs before the caret's line contribute their
    /// stored counts, and only the caret's own line is segmented.
    fn document_grapheme_offset(&mut self, buffer: &Buffer, char_index: usize) -> usize {
        let (line, column) = buffer.char_to_line_col(char_index);
        let line = line.min(self.slots.len().saturating_sub(1));
        let prefix: usize = self.slots[..line].iter().map(|slot| slot.graphemes).sum();
        let text = buffer.run_text(line);
        self.stats.bytes_read += text.len() as u64;
        prefix + crate::semantic::char_to_grapheme(&text, column)
    }

    // --- the surface half --------------------------------------------------

    /// Rebuild everything that is not the document, and diff it. The tail is
    /// what is on SCREEN — a picker's visible rows, a card's lines, the notice
    /// — so its size is bounded by the surface, never by the document.
    fn rebuild_tail(&mut self, view: &SemanticView<'_>) {
        let base = RUN_BASE + self.slots.len();
        let root_children_before = self.snapshot.nodes[ROOT_INDEX].children.clone();
        self.snapshot.nodes.truncate(base);
        self.snapshot.nodes[ROOT_INDEX].children.truncate(1);

        let focus_id = view.fold_surfaces(&mut self.snapshot.nodes);
        self.snapshot.focus_id = focus_id;

        if self.snapshot.nodes[ROOT_INDEX].children != root_children_before {
            self.changed.push(ROOT_ID.to_string());
        }
        let previous: std::collections::HashMap<&str, &SemanticNode> = self
            .tail
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();
        for node in &self.snapshot.nodes[base..] {
            if previous.get(node.id.as_str()) != Some(&node) {
                self.changed.push(node.id.clone());
            }
        }
        self.tail = self.snapshot.nodes[base..].to_vec();
    }

    /// Record that this projection's nodes were just published in full.
    pub(crate) fn note_full_tree(&mut self) {
        self.stats.full_trees += 1;
        self.stats.nodes_published += self.snapshot.nodes.len() as u64;
        self.changed.clear();
    }

    pub(crate) fn note_incremental(&mut self, count: usize) {
        self.stats.nodes_published += count as u64;
        self.changed.clear();
    }
}

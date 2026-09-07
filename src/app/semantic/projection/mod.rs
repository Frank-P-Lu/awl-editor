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

mod buffer_source;
mod transcript;

/// Where the retained document nodes sit in `snapshot.nodes`: the root and the
/// document node first, the runs after them in line order, the surface tail
/// last. Fixed positions, because a projection that had to SEARCH for the
/// document node every frame would be O(document) again.
const ROOT_INDEX: usize = 0;
const DOCUMENT_INDEX: usize = 1;
const RUN_BASE: usize = 2;

/// What one refresh actually cost, in units a law can assert on.
///
/// `bytes_read` and `graphemes_segmented` are what the refresh actually
/// touched, which an unincremental one makes the whole document every frame;
/// `runs_rebuilt` is how many line runs were reprojected;
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
    /// Was the CURRENTLY SEEDED tree built from a comparison transcript
    /// (`true`) or the real buffer (`false`)? Compared against this frame's
    /// source at the top of [`Self::refresh`]: the two halves keep disjoint
    /// run identities (a transcript renumbers `RunId`s from zero every
    /// rebuild; the buffer's are minted once and never reused), so resuming
    /// either incremental path across a crossing would hand out ids the other
    /// side never minted. A crossing forces [`Self::invalidate`] instead —
    /// exactly the reseed an assistive technology reattaching already gets.
    built_from_transcript: bool,
    /// The last transcript this projection actually published, so an
    /// unrelated refresh (a caret blink, a resize) with the SAME comparison
    /// showing republishes nothing — the transcript's own twin of `content_rev`,
    /// which has no meaning here because a transcript carries no `RunTable`.
    last_transcript: Option<String>,
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
            built_from_transcript: false,
            last_transcript: None,
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
        self.last_transcript = None;
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
        let transcript = view.comparison_text();
        if self.seeded && self.built_from_transcript != transcript.is_some() {
            // THE SUBSTITUTION BOUNDARY ITSELF MOVED (buffer <-> comparison),
            // not just its content — see the field's own doc for why neither
            // incremental path may resume across it.
            self.invalidate();
        }
        let shape_moved = if self.seeded {
            match transcript {
                Some(text) => self.sync_transcript(text),
                None => self.sync_runs(view),
            }
        } else {
            match transcript {
                Some(text) => self.seed_transcript(text),
                None => self.seed(view),
            }
            true
        };
        self.built_from_transcript = transcript.is_some();
        self.sync_document(view, shape_moved, transcript.is_some());
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

    /// The document node: its name, its focus, its selection, and — only when
    /// the run sequence moved — its children.
    ///
    /// `transcript_mode` is [`SemanticView::comparison_text`]'s presence,
    /// threaded down rather than re-asked: **THE PUBLISHED RUNS ARE THE
    /// SUBSTITUTED PROSE**, not the buffer, whenever it is `true`, so the
    /// buffer's own cursor/anchor name a position in text nobody on this tree
    /// can see — reporting it would be exactly the leak this fold exists to
    /// close, one field over from the run text itself. Zero is inert on both
    /// sides of the substitution boundary, and matches the caret layer, which
    /// draws no caret at all over a comparison transcript
    /// (`TextPipeline::document_is_a_transcript`).
    fn sync_document(&mut self, view: &SemanticView<'_>, shape_moved: bool, transcript_mode: bool) {
        let buffer = view.buffer().expect("document projection has a buffer");
        let name = buffer
            .path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled document".to_string());
        let focused = matches!(view.layer(), workspace::Layer::Editor);
        let selection = if transcript_mode {
            SemanticSelection {
                anchor: 0,
                focus: 0,
            }
        } else {
            self.selection(buffer)
        };
        // The read-only fact is per-FRAME, not per-revision: a reading surface
        // opens and closes without touching the buffer's content or shape, so
        // the seed's answer would go stale here. Re-asked every refresh, like
        // the name and the focus flag beside it.
        let read_only = view.document_is_read_only();

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
        if node.editable == read_only {
            node.editable = !read_only;
            moved = true;
        }
        let actions = document_actions(read_only);
        if node.actions != actions {
            node.actions = actions;
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

/// **WHAT AN ASSISTIVE TECHNOLOGY MAY DO TO THE DOCUMENT NODE** — one owner,
/// read by the seed and by every refresh.
///
/// A node that advertises an action nothing routes is worse than a node that
/// advertises nothing (`requests.rs`'s own doc), and while a READ-ONLY prose
/// surface is up both text-WRITING requests are refused at the one wall
/// (`App::text_door_open`). So they leave the roster with the refusal, and
/// `editable` leaves with them: a reader is told the document cannot be written
/// rather than offered an edit that silently does nothing. Focus and selection
/// stay — the surface is for reading, and reading is what they are for.
fn document_actions(read_only: bool) -> Vec<SemanticAction> {
    let mut actions = vec![SemanticAction::Focus, SemanticAction::SetTextSelection];
    if !read_only {
        actions.push(SemanticAction::ReplaceSelectedText);
        actions.push(SemanticAction::SetValue);
    }
    actions
}

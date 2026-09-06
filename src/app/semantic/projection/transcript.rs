//! The comparison-transcript half of the retained projection: seeding and
//! resyncing the document's text runs from a substituted prose string instead
//! of the buffer's own `RunTable`.
//!
//! Split out of `projection.rs` at its natural size ceiling — the buffer path
//! and the transcript path are two distinct sources for the same document
//! runs, never entangled with each other, so a reader of one need not load
//! the other.

use super::*;

impl SemanticProjection {
    /// **THE ONE ALTERNATE SEED**, taken exactly when a comparison transcript
    /// is showing instead of the buffer — parallel to [`Self::seed`], reading a
    /// plain string instead of a [`Buffer`]'s [`RunTable`](crate::semantic::runs::RunTable).
    ///
    /// Why a transcript cannot reuse `seed`/`build_run`: those mint no
    /// identity of their own, they READ it off `buffer.runs()`, which knows
    /// nothing about a transcript — the substitution never touches the
    /// document (`crate::comparison::prose_for`'s own doc), so there is no
    /// `RunTable` to ask. Every call therefore renumbers `RunId`s from zero,
    /// which is the right cost: a comparison transcript is replaced WHOLE on a
    /// row selection, never edited character-by-character, so there is no
    /// per-keystroke case to keep incremental the way the buffer path must.
    pub(super) fn seed_transcript(&mut self, transcript: &str) {
        self.stats.seeds += 1;
        let mut root = SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl");
        root.children.push(DOCUMENT_ID.to_string());
        let mut document = SemanticNode::new(DOCUMENT_ID, SemanticRole::Document, String::new());
        // `focusable`/`multiline` are set once at seed time and never
        // revisited by `sync_document` (mirroring `seed`'s own contract) —
        // read-only is a fact `sync_document` DOES re-derive every frame, so
        // it is deliberately left for that one owner rather than guessed here.
        document.focusable = true;
        document.multiline = true;
        self.snapshot.nodes.clear();
        self.snapshot.nodes.push(root);
        self.snapshot.nodes.push(document);
        self.slots.clear();
        self.push_transcript_runs(transcript);
        self.content_rev = 0;
        self.shape_rev = 0;
        self.resolved = None;
        self.seeded = true;
        self.tail.clear();
        // A seed is paired with a full tree by its caller; naming every node in
        // `changed` as well would double-publish the whole document — the same
        // contract `seed`'s own comment states.
        self.changed.clear();
        self.last_transcript = Some(transcript.to_string());
    }

    /// The transcript's twin of [`Self::sync_runs`]. Returns whether the run
    /// SEQUENCE moved — always `true` on an actual change, because every
    /// `RunId` is reminted from zero every rebuild (see [`Self::seed_transcript`]),
    /// so the document's `children` list is never merely reordered in place.
    pub(super) fn sync_transcript(&mut self, transcript: &str) -> bool {
        if self.last_transcript.as_deref() == Some(transcript) {
            // The common case while a reader sits on one row: an unrelated
            // refresh (a caret blink elsewhere, a resize) with the SAME
            // comparison showing republishes nothing, exactly like the buffer
            // path's `content_rev` short-circuit above.
            return false;
        }
        self.slots.clear();
        self.snapshot.nodes.truncate(RUN_BASE);
        self.push_transcript_runs(transcript);
        for slot in &self.slots {
            self.changed.push(crate::semantic::run_node_id(slot.id));
        }
        self.resolved = None;
        self.stats.children_republished += 1;
        self.last_transcript = Some(transcript.to_string());
        true
    }

    /// Push one synthetic text run per transcript LINE — the transcript's
    /// twin of [`Self::build_run`], reading a plain `&str` instead of a
    /// [`Buffer`].
    fn push_transcript_runs(&mut self, transcript: &str) {
        for (line, text) in transcript.split('\n').enumerate() {
            let lengths = crate::semantic::grapheme_lengths(text);
            self.stats.runs_rebuilt += 1;
            self.stats.bytes_read += text.len() as u64;
            self.stats.graphemes_segmented += lengths.len() as u64;
            let id = RunId(line as u64);
            let mut node = SemanticNode::new(
                crate::semantic::run_node_id(id),
                SemanticRole::Text,
                "Markdown",
            );
            node.value = Some(text.to_string());
            node.character_lengths = lengths;
            self.slots.push(Slot {
                id,
                rev: 0,
                graphemes: node.character_lengths.len(),
            });
            self.snapshot.nodes.push(node);
        }
    }
}

//! The buffer half of the retained projection: seeding and incrementally
//! resyncing the document's text runs from the real `Buffer`'s own
//! `RunTable` — the path that runs on every ordinary keystroke, so it stays
//! the one that reuses identity wherever the `RunTable` says a line survived.
//!
//! Sibling to `transcript.rs`, split at the same seam: this file is the
//! source-specific half, `mod.rs` keeps everything that doesn't care which
//! source fed the document node.

use super::*;

impl SemanticProjection {
    pub(super) fn seed(&mut self, view: &SemanticView<'_>) {
        self.stats.seeds += 1;
        let buffer = view.buffer().expect("document projection has a buffer");
        let table = buffer.runs();
        let mut root = SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl");
        root.children.push(DOCUMENT_ID.to_string());
        let mut document = SemanticNode::new(DOCUMENT_ID, SemanticRole::Document, String::new());
        document.focusable = true;
        document.multiline = true;
        let read_only = view.document_is_read_only();
        document.editable = !read_only;
        document.actions = document_actions(read_only);
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
    pub(super) fn sync_runs(&mut self, view: &SemanticView<'_>) -> bool {
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
}

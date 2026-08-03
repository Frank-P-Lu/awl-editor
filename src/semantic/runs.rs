//! Stable per-line identity for the document's accessibility text runs.
//!
//! AccessKit wants a document as a sequence of text runs, and it wants an
//! update after activation to carry the nodes that CHANGED rather than the
//! whole tree. Both halves need run identity that survives an edit, and the
//! line NUMBER cannot supply it: inserting one line renumbers every run below
//! it, so a screen reader would be handed the entire document every time a
//! user pressed Enter. Each line therefore carries a minted id that moves with
//! it.
//!
//! **This table stores no text.** A run knows who it is and when it last
//! changed; the rope stays the one document. That is deliberate — a side copy
//! of every line would be a second document model, and the copy would drift
//! from the rope the first time a mutation site forgot to update it.
//!
//! The table is maintained by [`crate::buffer::Buffer`]'s three rope-mutation
//! sites and nowhere else, which is why `buffer/tests.rs`'s sweep can prove it
//! agrees with a table rebuilt from scratch after every editing command.

/// A line's identity. Minted once, retired when the line is deleted, and never
/// reused within a buffer's lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    pub id: RunId,
    /// The table's `content_rev` at the moment this line's text last changed.
    /// A projection that remembers the rev it last published for a run knows,
    /// with one integer compare, whether it must re-read the line.
    pub rev: u64,
}

#[derive(Clone, Debug)]
pub struct RunTable {
    table_id: u64,
    runs: Vec<Run>,
    next_id: u64,
    content_rev: u64,
    shape_rev: u64,
}

/// Process-wide, so `(table_id, content_rev)` names one document state for the
/// whole run and never collides across a buffer swap. CLAUDE.md's cache-key
/// discipline in one field: `version` alone restarts at 0 per open, and the
/// collision has already served a stale document once.
static NEXT_TABLE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl RunTable {
    pub fn new(line_count: usize) -> Self {
        let mut table = Self {
            table_id: NEXT_TABLE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            runs: Vec::new(),
            next_id: 0,
            content_rev: 0,
            shape_rev: 0,
        };
        table.reset(line_count);
        table
    }

    /// The one value that names this document's exact state: identity plus
    /// content revision. Equal means "the same text in the same buffer".
    pub fn state_key(&self) -> (u64, u64) {
        (self.table_id, self.content_rev)
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    /// Bumped once per edit that changes any line's TEXT.
    pub fn content_rev(&self) -> u64 {
        self.content_rev
    }

    /// Bumped only when the id SEQUENCE changes — a line added or removed.
    /// A parent republishes its `children` exactly when this moves, which is
    /// what keeps ordinary typing from reprojecting a 20 000-entry child list.
    pub fn shape_rev(&self) -> u64 {
        self.shape_rev
    }

    fn mint(&mut self) -> RunId {
        let id = RunId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Every line is new: a fresh buffer, or a load.
    pub fn reset(&mut self, line_count: usize) {
        let count = line_count.max(1);
        self.content_rev += 1;
        self.shape_rev += 1;
        let rev = self.content_rev;
        self.runs.clear();
        self.runs.reserve(count);
        for _ in 0..count {
            let id = self.mint();
            self.runs.push(Run { id, rev });
        }
    }

    /// Record that an edit replaced the lines `first..=last` (indices into the
    /// table as it stood BEFORE the edit) with `new_line_count` lines.
    ///
    /// The first line of the span keeps its id: an edit that begins inside a
    /// line leaves that line's prefix in place, so it is the same line to a
    /// reader whose cursor is sitting in it. Everything after it in the span is
    /// retired and replaced by freshly minted runs — which is honest, because
    /// after a multi-line paste those really are different lines.
    pub fn splice(&mut self, first: usize, last: usize, new_line_count: usize) {
        if self.runs.is_empty() {
            self.reset(new_line_count);
            return;
        }
        let last = last.min(self.runs.len() - 1);
        let first = first.min(last);
        let new_line_count = new_line_count.max(1);
        self.content_rev += 1;
        let rev = self.content_rev;

        if last - first + 1 == new_line_count {
            // Same lines, new text: no structural change, so no parent
            // republishes its children and no id is spent.
            for run in &mut self.runs[first..=last] {
                run.rev = rev;
            }
            return;
        }

        self.shape_rev += 1;
        let mut replacement = Vec::with_capacity(new_line_count);
        replacement.push(Run {
            id: self.runs[first].id,
            rev,
        });
        for _ in 1..new_line_count {
            let id = self.mint();
            replacement.push(Run { id, rev });
        }
        self.runs.splice(first..=last, replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(table: &RunTable) -> Vec<u64> {
        table.runs().iter().map(|run| run.id.0).collect()
    }

    #[test]
    fn an_edit_inside_one_line_keeps_every_id_and_moves_no_shape() {
        let mut table = RunTable::new(5);
        let before = ids(&table);
        let shape = table.shape_rev();
        table.splice(2, 2, 1);
        assert_eq!(ids(&table), before, "an ordinary edit renamed a run");
        assert_eq!(table.shape_rev(), shape, "an ordinary edit moved the shape");
        assert!(table.content_rev() > 0);
        // Exactly the edited line is marked, so a projection re-reads one line.
        let touched: Vec<usize> = table
            .runs()
            .iter()
            .enumerate()
            .filter(|(_, run)| run.rev == table.content_rev())
            .map(|(index, _)| index)
            .collect();
        assert_eq!(touched, vec![2]);
    }

    /// The axis that makes identity worth having: a line inserted in the MIDDLE
    /// must leave every line below it with the id it already had. Keyed by
    /// position, the ids below would all shift and the whole document would be
    /// republished on one Enter.
    #[test]
    fn inserting_a_line_leaves_every_later_run_with_its_own_id() {
        let mut table = RunTable::new(5);
        let before = ids(&table);
        table.splice(1, 1, 2);
        let after = ids(&table);
        assert_eq!(after.len(), 6);
        assert_eq!(after[0], before[0]);
        assert_eq!(after[1], before[1], "the split line keeps its identity");
        assert_eq!(
            &after[3..],
            &before[2..],
            "lines below the insertion were renamed",
        );
        assert!(table.shape_rev() > 1, "a structural change moved no shape");
    }

    #[test]
    fn joining_lines_retires_ids_without_renaming_the_survivors() {
        let mut table = RunTable::new(5);
        let before = ids(&table);
        table.splice(1, 2, 1);
        let after = ids(&table);
        assert_eq!(after.len(), 4);
        assert_eq!(after[1], before[1]);
        assert_eq!(&after[2..], &before[3..]);
    }

    #[test]
    fn a_minted_id_is_never_reused_after_its_line_is_deleted() {
        let mut table = RunTable::new(3);
        let retired = ids(&table)[2];
        table.splice(1, 2, 1);
        table.splice(1, 1, 3);
        assert!(
            !ids(&table).contains(&retired),
            "a retired id came back and would alias a stale node",
        );
    }
}

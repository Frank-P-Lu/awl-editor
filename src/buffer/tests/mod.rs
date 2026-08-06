//! Tests for the `buffer` module (cursor / motion / selection / undo-redo /
//! quick-note naming) -- split by SUBJECT out of one 2241-line `buffer::tests`
//! file into this `buffer/tests/` directory -- every test's NAME is
//! unchanged, only its module path grew one segment
//! (`buffer::tests::foo` -> `buffer::tests::<subject>::foo`). `use super::*;`
//! here still resolves to the `buffer` root exactly as before the split;
//! each child module re-derives buffer access directly via its own
//! `use super::super::*;` plus `use super::*;` for the shared `b()` builder
//! and `note_tmp()` scratch-dir helper defined here.

use super::*;

use crate::testscratch::ScratchDir;

fn b(s: &str) -> Buffer {
    Buffer::from_str(s)
}

/// A fresh, uniquely-named tempdir under the OS temp root, owned by a
/// [`ScratchDir`] guard that removes it on drop (queue item 168).
fn note_tmp(name: &str) -> ScratchDir {
    let mut p = std::env::temp_dir();
    p.push(format!("awl_note_test_{}_{}", std::process::id(), name));
    ScratchDir::new(p)
}

mod cursor_motion;
mod edit_ops;
mod kill_yank;
mod mark_selection;
mod paste_and_bounds;
mod undo_redo;
mod script_roundtrip;
mod eol_crlf;
mod quick_notes;
mod syntax_lang;
mod word_delete_boundary;

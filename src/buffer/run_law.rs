//! THE soundness law for the accessibility run table.
//!
//! The whole incremental projection rests on one promise the three
//! rope-mutation sites make: **a run whose id and revision are unchanged holds
//! unchanged text.** If a splice's arithmetic is off by one, or a mutation site
//! forgets to resplice, the table stays internally consistent and a screen
//! reader is quietly read the wrong line — the failure mode that no amount of
//! "the tree is correct" testing catches, because the tree IS correct about a
//! document that has moved on.
//!
//! So this file never inspects the splice. It applies real edits through the
//! real public API, then asks the table's own claim to survive against the
//! rope. The axis swept is the one arithmetic actually breaks on: the FIRST and
//! LAST line, empty lines, multi-line inserts and deletes, undo and redo — and
//! a long pseudo-random run, because a hand-picked case list is a list of the
//! cases the author thought of.

use super::Buffer;
use crate::semantic::runs::RunId;
use std::collections::HashMap;

/// Every run's `(rev, text)` right now.
fn observe(buffer: &Buffer) -> HashMap<RunId, (u64, String)> {
    buffer
        .runs()
        .runs()
        .iter()
        .enumerate()
        .map(|(line, run)| (run.id, (run.rev, buffer.run_text(line))))
        .collect()
}

/// Assert the promise, and that the table still describes the rope's shape.
fn assert_sound(
    buffer: &Buffer,
    before: &HashMap<RunId, (u64, String)>,
    what: &str,
) -> HashMap<RunId, (u64, String)> {
    let runs = buffer.runs().runs();
    assert_eq!(
        runs.len(),
        buffer.line_count(),
        "{what}: the run table has {} runs for {} lines",
        runs.len(),
        buffer.line_count(),
    );
    let mut ids: Vec<RunId> = runs.iter().map(|run| run.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), runs.len(), "{what}: a run id appears twice");

    let after = observe(buffer);
    for (id, (rev, text)) in &after {
        if let Some((was_rev, was_text)) = before.get(id)
            && was_rev == rev
        {
            assert_eq!(
                was_text, text,
                "{what}: run {id:?} kept its revision while its text changed \
                 from {was_text:?} to {text:?} — a screen reader would be read \
                 the stale line",
            );
        }
    }
    // The concatenation identity the projection's offset arithmetic rests on.
    let joined: String = (0..buffer.line_count())
        .map(|l| buffer.run_text(l))
        .collect();
    assert_eq!(
        joined,
        buffer.text(),
        "{what}: the runs are not the document"
    );
    after
}

/// Unicode a line-based split can break: combining marks, a ZWJ family, a flag,
/// empty lines, and a line that is nothing but a newline.
fn fixture() -> Buffer {
    Buffer::from_str("e\u{301}dge\n👨‍👩‍👧‍👦\n\nplain\n🇯🇵 tail\n")
}

#[test]
fn every_public_edit_keeps_unchanged_runs_honest() {
    type Step = (&'static str, fn(&mut Buffer));
    let steps: &[Step] = &[
        ("insert_char at start", |b| {
            b.set_cursor(0);
            b.insert_char('x');
        }),
        ("insert_char at end", |b| {
            b.set_cursor(usize::MAX);
            b.insert_char('y');
        }),
        ("insert_newline mid-line", |b| {
            b.set_cursor(2);
            b.insert_newline();
        }),
        ("insert multi-line text", |b| {
            b.set_cursor(3);
            b.insert_text("one\ntwo\nthree");
        }),
        ("insert_tab", |b| {
            b.set_cursor(1);
            b.insert_tab();
        }),
        ("indent_lines", Buffer::indent_lines),
        ("outdent_lines", Buffer::outdent_lines),
        ("delete_backward at a line start", |b| {
            b.set_cursor(b.line_col_to_char(2, 0));
            b.delete_backward();
        }),
        ("delete_forward at a line end", |b| {
            b.set_cursor(b.line_col_to_char(1, usize::MAX));
            b.delete_forward();
        }),
        ("delete_word_backward", Buffer::delete_word_backward),
        ("delete_word_forward", Buffer::delete_word_forward),
        ("delete_to_line_start", Buffer::delete_to_line_start),
        ("kill_line", |b| {
            b.set_cursor(b.line_col_to_char(1, 0));
            b.kill_line();
        }),
        ("kill_region over three lines", |b| {
            b.set_anchor(b.line_col_to_char(0, 1));
            b.set_cursor(b.line_col_to_char(3, 1));
            b.kill_region();
        }),
        ("yank the killed region back", Buffer::yank),
        ("delete_selection", |b| {
            b.set_anchor(0);
            b.set_cursor(4);
            b.delete_selection();
        }),
        ("replace_char_range across lines", |b| {
            let end = b.line_col_to_char(2, 0);
            b.replace_char_range(1, end, "replacement\nspanning\nthree");
        }),
        ("replace_before_cursor", |b| {
            b.set_cursor(6);
            b.replace_before_cursor(3, "zz");
        }),
        ("set_text wholesale", |b| {
            b.set_text("brand\nnew\ndocument\n");
        }),
        ("set_text to a single line", |b| b.set_text("just one line")),
        ("set_text to empty", |b| b.set_text("")),
        ("undo", Buffer::undo),
        ("undo again", Buffer::undo),
        ("redo", Buffer::redo),
        ("redo again", Buffer::redo),
    ];

    for (name, step) in steps {
        // A fresh fixture per step, and then the step applied twice, because a
        // splice that is wrong only on a table it already spliced is a real
        // shape (the second edit is the one that reads a stale index).
        let mut buffer = fixture();
        let mut state = observe(&buffer);
        for round in 0..2 {
            step(&mut buffer);
            state = assert_sound(&buffer, &state, &format!("{name} (round {round})"));
        }
    }
}

/// The hand-written roster above is a list of the cases its author thought of.
/// This one is not: a long deterministic random walk over the same operations,
/// asserting the promise after every single step.
#[test]
fn a_long_random_edit_walk_never_leaves_a_stale_run() {
    let mut buffer = fixture();
    let mut state = observe(&buffer);
    // xorshift64*, so the sequence is fixed and a failure is reproducible by
    // its step number alone.
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for step in 0..2_000u32 {
        let roll = next();
        let chars = buffer.text().chars().count().max(1);
        buffer.set_cursor((roll >> 8) as usize % chars);
        match roll % 12 {
            0 => buffer.insert_char('a'),
            1 => buffer.insert_char('\u{301}'),
            2 => buffer.insert_newline(),
            3 => buffer.insert_text("👨‍👩‍👧‍👦\nsecond\n"),
            4 => buffer.delete_backward(),
            5 => buffer.delete_forward(),
            6 => buffer.kill_line(),
            7 => {
                buffer.set_anchor((roll >> 20) as usize % chars);
                buffer.delete_selection();
            }
            8 => buffer.delete_word_backward(),
            9 => buffer.undo(),
            10 => buffer.redo(),
            _ => buffer.insert_text("🇯🇵"),
        }
        state = assert_sound(&buffer, &state, &format!("random step {step}"));
    }
    assert!(
        buffer.runs().content_rev() > 100,
        "the walk must really have edited; it reached rev {}",
        buffer.runs().content_rev(),
    );
}

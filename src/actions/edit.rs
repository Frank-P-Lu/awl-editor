//! The MARKDOWN smart-Enter edit — the one dispatch arm whose behavior is richer
//! than a bare buffer call. `apply_transition`'s `Newline` arm asks [`smart_newline`] to
//! continue a list / blockquote (ordered lists AUTO-INCREMENT), unconditionally END
//! the block on an empty BLOCKQUOTE, PRESERVE-or-END an empty LIST item (bullet /
//! numbered / task) by provenance, or carry leading
//! indentation forward; a `false` return falls through to a plain
//! `insert_newline`. The DECISION ([`SmartNewline`] + [`smart_newline_for`])
//! is pure over one line's text + cursor column, so it is unit-testable without a
//! buffer/GPU; the ONE impure bit — reading/writing the buffer's short-lived
//! list-continuation provenance flag — lives here in [`smart_newline`] itself.
//! The pure decision and impure provenance update remain separated here.

use super::*;

fn table_row_source(columns: usize) -> String {
    format!("|{}", " |".repeat(columns.max(1)))
}

/// Bare Enter inserts a correctly-columned source row immediately below a real
/// GFM table row. Shift-Enter deliberately bypasses this owner in `actions.rs`.
pub(super) fn table_newline(ctx: &mut ActionCtx) -> bool {
    if !ctx.buffer.is_markdown() || ctx.buffer.has_selection() {
        return false;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    let (line, _) = ctx.buffer.cursor_line_col();
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, line) else {
        return false;
    };
    let columns = (start..end)
        .filter(|&row| row != start + 1)
        .map(|row| crate::markdown::split_row_cells(lines[row]).len())
        .max()
        .unwrap_or(1);
    // Header + separator are inseparable GFM structure: inserting "below the
    // header" means the first body row, below its separator.
    let below = if line == start { start + 1 } else { line };
    let at = ctx.buffer.line_col_to_char(below, usize::MAX);
    let row = table_row_source(columns);
    ctx.buffer.replace_char_range(at, at, &format!("\n{row}"));
    ctx.buffer.set_cursor(at + 3);
    true
}

/// Tab/Shift-Tab walk real table cells in source order. Reaching the forward
/// end appends one scaffold row; non-table source keeps the established list
/// indentation behavior.
pub(super) fn table_tab(ctx: &mut ActionCtx, forward: bool) -> bool {
    if !ctx.buffer.is_markdown() || ctx.buffer.has_selection() {
        return false;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    let (line, col) = ctx.buffer.cursor_line_col();
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, line) else {
        return false;
    };
    let mut cells = Vec::new();
    for (row, line_text) in lines.iter().enumerate().take(end).skip(start) {
        if row == start + 1 {
            continue;
        }
        for range in crate::markdown::table_cell_ranges(line_text) {
            let cell_col = line_text[..range.start].chars().count();
            cells.push((row, cell_col));
        }
    }
    if cells.is_empty() {
        return false;
    }
    let current = cells
        .iter()
        .rposition(|&(row, cell_col)| row == line && cell_col <= col)
        .or_else(|| cells.iter().position(|&(row, _)| row == line))
        .unwrap_or_else(|| if forward { 0 } else { cells.len() - 1 });
    if forward && current + 1 == cells.len() {
        let columns = cells.iter().filter(|(row, _)| *row == start).count().max(1);
        let at = ctx.buffer.line_col_to_char(end - 1, usize::MAX);
        ctx.buffer
            .replace_char_range(at, at, &format!("\n{}", table_row_source(columns)));
        ctx.buffer.set_cursor(at + 3);
    } else {
        let next = if forward {
            current + 1
        } else {
            current.saturating_sub(1)
        };
        let (target_line, target_col) = cells[next];
        ctx.buffer
            .set_cursor(ctx.buffer.line_col_to_char(target_line, target_col));
    }
    true
}

/// MARKDOWN-only smart Enter. Returns `true` when it performed the edit; `false`
/// tells the caller to do a plain `insert_newline`. Reads only the current line's
/// text + cursor column and mutates through the buffer's atomic edit seam, so it
/// stays pure and `--keys`-drivable (live and replay can't drift). Gated on
/// `is_markdown`, and skipped while a selection is active (a plain newline, which
/// overwrites the selection, is the right thing there).
pub(super) fn smart_newline(ctx: &mut ActionCtx) -> bool {
    if !ctx.buffer.is_markdown() || ctx.buffer.has_selection() {
        return false;
    }
    let (line, col) = ctx.buffer.cursor_line_col();
    let text = ctx.buffer.line_text(line);
    match smart_newline_for(&text, col) {
        Some(SmartNewline::Continue(prefix)) => {
            let mut s = String::with_capacity(prefix.len() + 1);
            s.push('\n');
            s.push_str(&prefix);
            ctx.buffer.replace_before_cursor(0, &s);
            true
        }
        Some(SmartNewline::ContinueListItem { prefix, bare }) => {
            let mut s = String::with_capacity(prefix.len() + 1);
            s.push('\n');
            s.push_str(&prefix);
            ctx.buffer.replace_before_cursor(0, &s);
            if bare {
                // Nothing followed the cursor on the split line, so the
                // line this Enter just opened is ITSELF a bare, otherwise-empty
                // list-item continuation — mark its provenance so the very next
                // smart-newline decision on this line (if nothing intervenes)
                // knows awl generated it, rather than guessing from its bytes.
                ctx.buffer.mark_list_continuation_generated();
            }
            true
        }
        Some(SmartNewline::EndBlockquote { strip }) => {
            // Empty blockquote: drop the dangling `>` run, leaving the line blank
            // with the caret at column 0 — the quote has ended. Unconditional
            // (the provenance law does not cover blockquotes — see the
            // type's own doc).
            ctx.buffer.replace_before_cursor(strip, "");
            true
        }
        Some(SmartNewline::EmptyListItem { strip }) => {
            if ctx.buffer.take_list_continuation_generated() {
                // Enter on the empty continuation awl JUST generated
                // (the immediately preceding action, nothing intervening) — strip
                // the dangling marker, ending the list/run. Matches the ordinary
                // "Enter twice to leave a list" gesture.
                ctx.buffer.replace_before_cursor(strip, "");
            } else {
                // The provenance-gated rule covers numbered and task items
                // alongside bullets: a marker of ANY OTHER
                // provenance — typed, loaded from disk, undone/redone back into
                // place, or reached after any other edit — is PRESERVED
                // byte-semantically, and a fresh PLAIN line opens below it. Park
                // at line end first (regardless of where the caret sits in the
                // marker's trailing whitespace), then insert ONE plain newline —
                // one atomic undo group — so the caret lands at column 0 of the
                // new empty line and the off-cursor marker renders concealed. No
                // second marker is emitted; the required trailing space is
                // untouched.
                let (line, _) = ctx.buffer.cursor_line_col();
                let end = ctx.buffer.line_col_to_char(line, usize::MAX);
                ctx.buffer.set_cursor(end);
                ctx.buffer.insert_newline();
            }
            true
        }
        None => false,
    }
}

/// ALIGN TABLE: re-pad the GFM table under the caret so its `|` line up. Finds the
/// table block around the caret line via [`crate::markdown::table_block_lines`],
/// re-emits it with [`crate::markdown::align_table`], and replaces exactly those
/// lines as ONE undoable edit (Cmd-Z restores the pre-align source). A calm no-op
/// when the caret is not in a table, when the buffer isn't markdown, or when the
/// table is ALREADY aligned (no edit → the undo history stays meaningful). Reads +
/// mutates only through the buffer's public seam, so `--keys` drives it identically
/// live and in replay. The manual palette command; [`auto_align_table_on_row_leave`]
/// is the same re-pad fired automatically.
pub(super) fn align_table_at_cursor(ctx: &mut ActionCtx) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    let (cur_line, _) = ctx.buffer.cursor_line_col();
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, cur_line) else {
        return; // caret not inside a table — calm no-op
    };
    let block = lines[start..end].join("\n");
    let aligned = crate::markdown::align_table(&block);
    if aligned == block {
        return; // already aligned — skip the edit so undo stays meaningful
    }
    // Char range covering exactly the table's lines (last line's end, before its
    // trailing newline): `line_col_to_char` clamps the huge col to the line length.
    let start_char = ctx.buffer.line_col_to_char(start, 0);
    let end_char = ctx.buffer.line_col_to_char(end - 1, usize::MAX);
    ctx.buffer
        .replace_char_range(start_char, end_char, &aligned);
}

/// AUTO-ALIGN ON ROW-LEAVE: the [`align_table_at_cursor`] re-pad, fired
/// automatically the moment the caret leaves the table ROW it was editing (see
/// `actions.rs::finish_action`, the ONE seam every dispatched action passes
/// through — this IS the whole trigger; no new caret-tracking state lives on
/// `Buffer`). ROW-leave rather than TABLE-leave: a table can carry many rows,
/// and finishing one row is the natural moment ITS drifted padding should snap
/// back — waiting for the caret to clear the WHOLE table would leave every row
/// already finished sitting misaligned while later rows are still being typed,
/// which is exactly the "doesn't self-heal" complaint this exists to fix.
///
/// Idempotent (a calm no-op when `row_before` wasn't in a table, or the table's
/// already aligned) and ALWAYS its own sealed undo group: `replace_char_range`'s
/// edit is a REPLACE (both `removed` and `inserted` non-empty), which the undo
/// engine's `record_edit` never coalesces (`is_replace` short-circuits
/// `can_coalesce`) regardless of what ran immediately before it — so one Cmd-Z
/// right after an auto-align undoes exactly the re-pad, revealing the user's
/// last raw (typed, misaligned) edit rather than merging with or eating it.
///
/// `row_before` is the LOGICAL LINE the caret sat on immediately before the
/// action that just finished (captured pre-dispatch, in `ActionSnapshot`); the
/// block is re-derived fresh against the CURRENT (post-action) text rather than
/// trusted from before, so a same-action structural edit (Enter splitting the
/// row, a merge-up backspace) is still read correctly — a `row_before` the
/// action deleted outright, or that no longer looks like a table row, is a calm
/// no-op via `table_block_lines` returning `None`.
///
/// CARET PRESERVATION: column re-padding shifts char offsets across the whole
/// row, so a raw pre-edit char offset would land on padding whitespace or a
/// different cell after the replace. If the CURRENT caret sits inside the same
/// table block being rewritten (it may have moved to a different row of the
/// SAME table, not only out of it), its position is captured as a LOGICAL
/// (cell, intra-cell offset) pair ([`crate::markdown::locate_table_caret`])
/// before the replace, then re-resolved on the realigned row
/// ([`crate::markdown::table_caret_col`]) after — landing on the same cell/
/// content offset rather than a byte count that now means something else. A
/// caret that already sits outside the block (it left the table, not only the
/// row) is shifted by the block's char-length delta when it sits AFTER the
/// table, or left untouched when it sits before — a replace strictly ahead of
/// it can never move it.
pub(super) fn auto_align_table_on_row_leave(ctx: &mut ActionCtx, row_before: usize) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    if row_before >= lines.len() {
        return; // the row the caret left no longer exists post-action
    }
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, row_before) else {
        return; // row_before wasn't in a table (or no longer looks like one)
    };
    let block = lines[start..end].join("\n");
    let aligned = crate::markdown::align_table(&block);
    if aligned == block {
        return; // already aligned — skip the edit so undo stays meaningful
    }

    // Capture the CURRENT caret's logical position before the replace shifts
    // every char offset from `start_char` onward.
    let cur = ctx.buffer.cursor_char();
    let (cur_line, cur_col) = ctx.buffer.cursor_line_col();
    let caret_cell = (cur_line >= start && cur_line < end).then(|| {
        (
            cur_line - start,
            crate::markdown::locate_table_caret(lines[cur_line], cur_col),
        )
    });

    let start_char = ctx.buffer.line_col_to_char(start, 0);
    let end_char = ctx.buffer.line_col_to_char(end - 1, usize::MAX);
    let old_len = end_char - start_char;
    ctx.buffer
        .replace_char_range(start_char, end_char, &aligned);
    let new_len = aligned.chars().count();

    let new_cursor = match caret_cell {
        Some((row_in_block, pos)) => {
            let new_row = aligned.split('\n').nth(row_in_block).unwrap_or("");
            let new_col = crate::markdown::table_caret_col(new_row, pos);
            ctx.buffer.line_col_to_char(start + row_in_block, new_col)
        }
        // Caret already sat outside the block: a replace strictly ahead of it
        // (it sat before the table) can't move it; one strictly behind it (it
        // sat after) shifts it by exactly the block's char-length delta.
        None if cur >= end_char => (cur as i64 + (new_len as i64 - old_len as i64)) as usize,
        None => cur,
    };
    ctx.buffer.set_cursor(new_cursor);
}

/// TAG DOCUMENT LANGUAGE — the ONE door that writes a `lang:` frontmatter tag
/// into the user's document, and it only ever opens because the user asked for
/// it (the palette's "Tag document language"). Detects the document's dominant
/// CJK script from the buffer text ([`crate::script::dominant_cjk`]), resolves
/// it to a tag through the live ambiguity ladder
/// ([`crate::frontmatter::cjk_priority`] — the same global the Settings row
/// reads and the CJK picker promotes, so a `--keys` replay that changes the
/// ladder and then tags observes the new front), and inserts
/// `---\nlang: ..\n---\n` at byte 0 as ONE undoable edit (Cmd-Z restores the
/// pre-tag text and cursor), then acknowledges the applied writer-visible name
/// through the shared self-clearing notice channel.
///
/// A calm no-op — no edit, no version bump, no undo entry — when the buffer is
/// not markdown (frontmatter is a markdown/notes convention; literal
/// `---`/`lang:` text in a `.rs` file is corruption), when a frontmatter block
/// is already present (never a second block; the existing tag already wins the
/// ladder), or when the document carries no CJK at all (nothing to detect —
/// the ladder only disambiguates CJK).
///
/// This detection used to run AUTOMATICALLY on every fresh open, which meant
/// opening a document to read it edited it, and the stamped tag then outranked
/// the user's own `cjk_priority` setting for the life of the file. Resolution
/// needs nothing written: the render ladder re-reads the frontmatter tag from
/// the buffer on every reshape, and an untagged document's ambiguous Han runs
/// follow the configured ladder. Only this action writes.
pub(super) fn tag_document_language(ctx: &mut ActionCtx) -> Effect {
    if !ctx.buffer.is_markdown() {
        return Effect::None;
    }
    let text = ctx.buffer.text();
    if crate::frontmatter::detect(&text).is_some() {
        return Effect::None; // already carries a frontmatter block — never a second one
    }
    let Some(script) = crate::script::dominant_cjk(&text) else {
        return Effect::None; // no CJK — nothing this ladder can name
    };
    let lang = crate::script::doc_lang_for(script, &crate::frontmatter::cjk_priority());
    let block = format!("---\nlang: {}\n---\n", lang.code());
    ctx.buffer.replace_char_range(0, 0, &block);
    Effect::Notice(NoticeEffect::Toast(format!(
        "Document language: {}",
        lang.label()
    )))
}

/// TAB dispatch: on a markdown LIST context (the caret line — or ANY line of an
/// active selection — is a list item), indent one nesting level; ELSEWHERE fall back
/// to the soft-tab insert. Keeping the list-vs-plain gate
/// here (over the SHARED [`crate::markdown::list_item`] detection) is what makes Tab
/// `--keys`-drivable and testable without a GPU.
pub(super) fn list_tab(ctx: &mut ActionCtx) {
    if ctx.buffer.is_markdown() && selection_or_cursor_on_list(ctx) {
        ctx.buffer.indent_lines();
    } else {
        ctx.buffer.insert_tab();
    }
}

/// SHIFT-TAB dispatch: outdent one nesting level across the caret line or selection.
/// Uniform (not list-gated): off a list it simply strips up to two leading spaces —
/// a clean no-op when there are none, so it never surprises on plain prose.
pub(super) fn list_outdent(ctx: &mut ActionCtx) {
    ctx.buffer.outdent_lines();
}

/// True when the Tab should INDENT rather than soft-tab: the caret line, or any line
/// an active selection spans, is a markdown [`crate::markdown::list_item`].
fn selection_or_cursor_on_list(ctx: &ActionCtx) -> bool {
    let is_list = |l: usize| crate::markdown::list_item(&ctx.buffer.line_text(l)).is_some();
    match ctx.buffer.selection_line_col() {
        Some(((l0, _), (l1, _))) => (l0.min(l1)..=l0.max(l1)).any(is_list),
        None => is_list(ctx.buffer.cursor_line_col().0),
    }
}

/// The outcome of a markdown smart Enter, computed purely from one line.
pub(super) enum SmartNewline {
    /// Insert a newline then this continuation prefix — a BLOCKQUOTE run or bare
    /// indentation. Not a list marker, so the provenance law does not apply:
    /// a blockquote's own empty-item Enter is unconditional (`EndBlockquote`), and
    /// bare indentation has no "end" concept at all.
    Continue(String),
    /// Insert a newline then this LIST marker continuation (bullet / numbered /
    /// task). `bare` is true when nothing on the split line followed the cursor,
    /// so the line this opens is ITSELF a bare, otherwise-empty item — the
    /// provenance flag is set (by the caller, `smart_newline`) iff `bare`.
    ContinueListItem { prefix: String, bare: bool },
    /// The current BLOCKQUOTE is EMPTY: strip `strip` chars before the cursor (the
    /// dangling indent + `>` run) and insert nothing, unconditionally ending the
    /// block — provenance-independent (blockquotes sit outside this law).
    EndBlockquote { strip: usize },
    /// The current LIST item (bullet `- `/`* `/`+ `, numbered `N.`/`N)`, or task
    /// `- [ ] `/`- [x] `) is EMPTY. Whether Enter here PRESERVES the marker
    /// (opening a fresh plain line below) or ENDS the list
    /// (stripping `strip` chars before the cursor) depends on provenance the
    /// buffer tracks, not on which marker kind this is — see `smart_newline`'s
    /// `EmptyListItem` arm, the ONE place that reads it.
    EmptyListItem { strip: usize },
}

/// Decide the markdown smart-Enter behavior for the current `line` text and
/// cursor `col` (chars from the line start). Pure — no buffer / GPU. After any
/// leading indentation it recognizes, in order:
///  * a blockquote (`>`…) — continued with the same `>` run;
///  * a list item ([`crate::markdown::list_item`] — the SAME shared grammar the
///    Tab/Shift-Tab indent gate uses, so a line the two commands disagree about
///    can't exist), with a task checkbox (`[ ]`/`[x]`/`[X]` + space right after
///    an unordered marker) layered on top here since `list_item` doesn't know
///    task syntax — continued with the same bullet (a task item's checkbox
///    continues UNCHECKED, never carrying `[x]` forward) or, for an ordered
///    item, the number INCREMENTED;
///  * else bare indentation — preserved on a plain Enter.
///    An EMPTY blockquote unconditionally ends the block (`EndBlockquote`); an EMPTY
///    list item (bullet / numbered / task) is `EmptyListItem` — its caller decides
///    preserve-vs-end by provenance; bare indentation
///    is only ever carried, never ended. Returns `None` when there's nothing to
///    continue (plain prose, or the caret sits inside the marker), so the caller does
///    an ordinary newline.
pub(super) fn smart_newline_for(line: &str, col: usize) -> Option<SmartNewline> {
    let chars: Vec<char> = line.chars().collect();
    // Leading indentation (spaces / tabs) — shared by every branch below.
    let mut i = 0;
    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
        i += 1;
    }

    // Blockquote: a run of '>' and spaces; continue with the same run.
    if i < chars.len() && chars[i] == '>' {
        let mut j = i;
        while j < chars.len() && (chars[j] == '>' || chars[j] == ' ') {
            j += 1;
        }
        if col < j {
            return None; // caret inside the marker → plain newline
        }
        if chars[j..].iter().all(|c| c.is_whitespace()) {
            return Some(SmartNewline::EndBlockquote { strip: col });
        }
        return Some(SmartNewline::Continue(chars[..j].iter().collect()));
    }

    // List item (bullet or ordered): the SHARED grammar, `list_item` — the same
    // primitive the Tab/Shift-Tab indent gate calls — decides is-list; a task
    // checkbox (`[ ]`/`[x]`/`[X]` then a space, right after an unordered marker)
    // is layered on top here since `list_item` doesn't parse task syntax.
    if let Some(it) = crate::markdown::list_item(line) {
        let is_task = !it.ordered
            && chars.len() >= it.content + 4
            && chars[it.content] == '['
            && matches!(chars[it.content + 1], ' ' | 'x' | 'X')
            && chars[it.content + 2] == ']'
            && chars[it.content + 3] == ' ';
        let prefix_len = if is_task { it.content + 4 } else { it.content };
        if col < prefix_len {
            return None; // caret inside the marker → plain newline
        }
        if chars[prefix_len..].iter().all(|c| c.is_whitespace()) {
            // For every supported marker, an empty list item's
            // preserve-vs-end is the CALLER's provenance-gated decision.
            return Some(SmartNewline::EmptyListItem { strip: col });
        }
        let indent: String = chars[..it.indent].iter().collect();
        // The marker text (indent..content) minus its required trailing space
        // ends in the bullet char (unordered) or the digit run + delimiter
        // (ordered) — decompose it rather than re-deriving the grammar.
        let mut marker: Vec<char> = chars[it.indent..it.content].to_vec();
        marker.pop(); // the required trailing space
        let last = marker.pop().expect("list_item guarantees a marker char");
        let bare = chars[col..].iter().all(|c| c.is_whitespace());
        let prefix = if it.ordered {
            let n: usize = marker.iter().collect::<String>().parse().unwrap_or(0);
            // `saturating_add` so a pathological `usize::MAX.` marker can't
            // overflow (panic in debug, wrap-to-0 in release) — it simply pins
            // at usize::MAX.
            format!("{indent}{}{last} ", n.saturating_add(1))
        } else if is_task {
            format!("{indent}{last} [ ] ") // continuation opens UNCHECKED
        } else {
            format!("{indent}{last} ")
        };
        return Some(SmartNewline::ContinueListItem { prefix, bare });
    }

    // Bare indentation: carry it forward on a plain Enter (only when the caret is
    // at/after the indentation). No "end on empty" — indentation is just kept.
    if i > 0 && col >= i {
        return Some(SmartNewline::Continue(chars[..i].iter().collect()));
    }

    None
}

//! The source-level footnote insertion command.

use super::{ActionCtx, FormatResult};

/// Insert one reference/definition pair as a single undoable source edit.
/// Numeric labels are identifiers only; display numbering is independently
/// derived from reference order by `markdown::footnotes`.
pub(in crate::actions) fn apply_insert_footnote(ctx: &mut ActionCtx) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    let result = insert_footnote(&text, anchor, cursor);
    ctx.buffer
        .apply_format(&result.text, result.anchor, result.cursor);
}

fn insert_footnote(text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    let mut label = 1usize;
    while text.contains(&format!("[^{label}]")) {
        label += 1;
    }
    let insert = anchor.map_or(cursor, |anchor| anchor.max(cursor));
    let mut chars: Vec<char> = text.chars().collect();
    let reference = format!("[^{label}]");
    chars.splice(insert..insert, reference.chars());
    let mut out: String = chars.into_iter().collect();
    if !out.is_empty() {
        if out.ends_with("\n\n") {
            // already at a block boundary
        } else if out.ends_with('\n') {
            out.push('\n');
        } else {
            out.push_str("\n\n");
        }
    }
    out.push_str(&format!("[^{label}]: "));
    let end = out.chars().count();
    FormatResult {
        text: out,
        anchor: None,
        cursor: end,
    }
}

#[cfg(test)]
mod tests {
    use super::insert_footnote;

    #[test]
    fn insert_footnote_chooses_a_collision_free_label_and_places_definition_caret() {
        let source = "Alpha[^1].\n\n[^1]: existing\n";
        let result = insert_footnote(source, None, 5);
        assert_eq!(
            result.text, "Alpha[^2][^1].\n\n[^1]: existing\n\n[^2]: ",
            "the first unused numeric ID is inserted at the caret and its definition is appended"
        );
        assert_eq!(result.anchor, None);
        assert_eq!(result.cursor, result.text.chars().count());
    }

    #[test]
    fn insert_footnote_uses_the_selection_end_without_replacing_unicode_prose() {
        let source = "αβ gamma";
        let result = insert_footnote(source, Some(0), 2);
        assert_eq!(result.text, "αβ[^1] gamma\n\n[^1]: ");
        let reversed = insert_footnote(source, Some(2), 0);
        assert_eq!(reversed.text, result.text, "selection direction is inert");
    }

    #[test]
    fn insert_footnote_reuses_an_existing_block_boundary_without_extra_blank_lines() {
        let result = insert_footnote("note\n\n", None, 4);
        assert_eq!(result.text, "note[^1]\n\n[^1]: ");
    }
}

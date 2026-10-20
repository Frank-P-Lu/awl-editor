//! One typed-space input rule; saved Markdown keeps the standard `[ ]` spelling.
use crate::buffer::Buffer;

pub(super) fn insert_typed(buffer: &mut Buffer, text: &str) {
    let eligible = text == " " && buffer.is_markdown() && !buffer.has_selection();
    buffer.insert_chars(text);
    if !eligible {
        return;
    }
    let (line, col) = buffer.cursor_line_col();
    let source = buffer.line_text(line);
    let Some(item) = crate::markdown::list_item(&source) else {
        return;
    };
    if item.ordered || col != item.content + 3 || source.get(item.content..col) != Some("[] ") {
        return;
    }
    let doc = buffer.text();
    let start = buffer.char_to_byte(buffer.line_col_to_char(line, 0));
    let spans = crate::markdown::spans(&doc);
    if crate::markdown::rich_unordered_list_item(&source, start, spans.iter()).is_none() {
        return;
    }
    // Separate conversion from the trigger: one undo restores literal `[] `.
    buffer.replace_before_cursor(3, "[ ] ");
}

//! Export's markdown gate. On a non-Markdown buffer the row stays enabled
//! (see `apply_export_action`'s own doc) but must never silently no-op — the
//! File-menu row's own half of the same defect has its law in
//! `menu::ellipsis_law`.

use super::super::*;
use crate::overlay::OverlayKind;

fn drive(buffer: &mut Buffer, action: Action) -> Transition {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, &action, false)
}

/// MUTATION TARGET: revert `apply_export_action`'s non-markdown arms to
/// `Effect::None` and this goes red on every one of the three formats, naming
/// which action stayed silent.
#[test]
fn export_on_a_non_markdown_buffer_raises_an_explicit_notice_never_a_silent_no_op() {
    for (action, format) in [
        (Action::ExportWord, crate::export::Format::Docx),
        (Action::ExportHtml, crate::export::Format::Html),
        (Action::ExportPdf, crate::export::Format::Pdf),
    ] {
        let mut plain = Buffer::from_str("plain text, no markdown syntax");
        plain.set_path("/notes/todo.txt".into());
        let transition = drive(&mut plain, action.clone());
        assert_eq!(
            transition.primary(),
            Effect::Notice(NoticeEffect::Sticky(
                "can't export a non-Markdown file".to_string()
            )),
            "{action:?} on a .txt buffer must raise an explicit notice, never Effect::None"
        );

        // The gate is on document kind, not on the action itself: the exact
        // same action over a Markdown buffer still exports for real.
        let mut markdown = Buffer::from_str("# heading");
        markdown.set_path("/notes/todo.md".into());
        let markdown_transition = drive(&mut markdown, action);
        assert_eq!(
            markdown_transition.primary(),
            Effect::Export(format),
            "a Markdown buffer must still export normally"
        );
    }
}

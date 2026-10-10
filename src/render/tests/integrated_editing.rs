//! Formatter, rich-list shaping and composition share one document source.
use super::{headless_dqp, view_md};
use crate::{actions, buffer::Buffer, keymap::Action, overlay};

fn toggle_code(buffer: &mut Buffer) {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = overlay::Journey::default();
    let mut make = |_| None;
    let mut browse = |_, _| None;
    let mut ctx = actions::ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make,
        browse_to: &mut browse,
        oracle: None,
    };
    assert_eq!(
        actions::apply_transition(&mut ctx, &Action::ToggleCodeBlock, false).primary(),
        actions::Effect::None
    );
}

#[test]
fn nested_code_action_reconciles_retained_list_geometry_through_undo_redo() {
    let _guard = crate::testlock::serial();
    let source = "- parent\n  - 日本語 child\n\nanchor\n";
    for world in ["Tawny", "Quokka", "Paperbark", "Brolga", "Mangrove"] {
        let _world = crate::theme::WorldPin::world(world).unwrap();
        for dpi in [1.0, 2.0] {
            let (_, _, mut p) = headless_dqp(960.0 * dpi, 800.0 * dpi).expect("GPU required");
            p.set_dpi(dpi);
            let mut b = Buffer::from_str(source);
            let original_cursor = b.line_col_to_char(1, 6);
            b.set_cursor(original_cursor);
            p.set_view(&view_md(source, 3, 0));
            assert!(p.buffer.lines[1].hanging_inset() > 0.0);
            toggle_code(&mut b);
            let fenced = b.text();
            assert!(fenced.contains("  - ```\n    日本語 child\n    ```"));
            let emitted_cursor = b.cursor_char();
            let emitted_selection = b.selection_range();
            assert!(emitted_selection.is_some());
            p.set_view(&view_md(&fenced, 5, 0));
            assert_eq!(
                p.list_marks().len(),
                2,
                "{world} {dpi}: both markers stay outside code"
            );
            assert_eq!(p.buffer.lines[2].hanging_inset(), 0.0);
            assert_eq!(p.buffer.lines[2].text(), "    日本語 child");
            let xs = p.line_glyph_xs(2);
            assert!(
                xs[..5]
                    .windows(2)
                    .all(|pair| pair[1] - pair[0] > 0.51 * dpi),
                "{world} {dpi}: every literal source space keeps a visible advance: {xs:?}"
            );
            b.undo();
            assert_eq!(b.text(), source);
            assert_eq!(b.cursor_char(), original_cursor);
            assert_eq!(b.selection_range(), None);
            p.set_view(&view_md(&b.text(), 3, 0));
            assert!(p.buffer.lines[1].hanging_inset() > 0.0);
            b.redo();
            assert_eq!(b.text(), fenced);
            assert_eq!(b.cursor_char(), emitted_cursor);
            assert_eq!(
                b.selection_range(),
                None,
                "redo keeps the shared cleared-selection contract"
            );
            p.set_view(&view_md(&b.text(), 5, 0));
            assert_eq!(p.buffer.lines[2].hanging_inset(), 0.0);
        }
    }
}

#[test]
fn document_preedit_offsets_preserve_nested_list_reveal_and_full_underline() {
    let _guard = crate::testlock::serial();
    let text = "- parent\n  - 日本語 child\nanchor\n";
    for world in ["Tawny", "Quokka", "Paperbark", "Brolga", "Mangrove"] {
        let _world = crate::theme::WorldPin::world(world).unwrap();
        for dpi in [1.0, 2.0] {
            let (_, _, mut p) = headless_dqp(960.0 * dpi, 800.0 * dpi).expect("GPU required");
            p.set_dpi(dpi);
            for preedit in ["にほん", "a\u{301}b"] {
                let mut v = view_md(text, 1, 6);
                v.preedit = preedit.into();
                p.set_view(&v);
                assert_eq!(
                    p.buffer.lines[1].hanging_inset(),
                    0.0,
                    "composition row reveals source"
                );
                let underline = p.preedit_rects();
                assert!(underline.iter().any(|r| r[2] > 1.0));
                let reshapes = p.reshape_count;
                let composed = p.shaped_key.clone();
                for offset in [0, 1, 3, 99] {
                    v.preedit_cursor = Some(offset);
                    p.set_view(&v);
                    assert_eq!(p.cursor_col, 6 + offset.min(preedit.chars().count()));
                    assert_eq!(p.preedit_rects(), underline);
                    assert_eq!(p.reshape_count, reshapes);
                    assert_eq!(p.shaped_key, composed);
                }
                v.preedit.clear();
                v.cursor_line = 2;
                v.cursor_col = 0;
                p.set_view(&v);
                assert!(p.preedit_rects().is_empty());
                assert!(p.buffer.lines[1].hanging_inset() > 0.0);
            }
        }
    }
}

//! Laws at the real formatting-action and measured-popover seams.

use super::*;
use crate::actions::{ActionCtx, apply_transition};
use crate::buffer::Buffer;
use crate::overlay::{Journey, OverlayKind, OverlayState};
use crate::render::tests::headless_dqp;

const SOURCE: &str = concat!(
    "# A document\n\nFirst paragraph.\n\n",
    "A little prefix before selected words and a tail.\n\n",
    "Second paragraph with another selection.\n",
);

struct Editing {
    buffer: Buffer,
    journey: Journey,
}

impl Editing {
    fn new() -> Self {
        let mut buffer = Buffer::from_str(SOURCE);
        let start = SOURCE.find("selected").unwrap();
        buffer.set_cursor(start + "selected words".len());
        buffer.set_anchor(start);
        Self {
            buffer,
            journey: Journey::default(),
        }
    }

    fn press(&mut self, button: PopoverButton) -> crate::actions::Effect {
        let mut shift = false;
        let mut zoom = 1.0;
        let mut search = None;
        let mut make = |kind: OverlayKind| Some(OverlayState::new(kind, vec![], vec![], vec![]));
        let mut browse = |_: OverlayKind, _: Option<String>| None;
        let mut ctx = ActionCtx {
            buffer: &mut self.buffer,
            shift_selecting: &mut shift,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut self.journey,
            make_overlay: &mut make,
            browse_to: &mut browse,
            oracle: None,
        };
        apply_transition(&mut ctx, &button.action(), false).primary()
    }

    fn view(&self) -> ViewState {
        let (line, col) = self.buffer.cursor_line_col();
        let text = self.buffer.text();
        let mut view = ViewState {
            text: text.clone(),
            cursor_line: line,
            cursor_col: col,
            selection: self.buffer.selection_line_col(),
            is_markdown: true,
            ..ViewState::base()
        };
        view.popover = crate::actions::popover::plan(
            &text,
            self.buffer.anchor_char(),
            self.buffer.cursor_char(),
            true,
        );
        view.overlay_active = self.journey.card().is_some();
        view
    }
}

fn measured(p: &mut TextPipeline, view: &ViewState, w: u32, h: u32) -> PopoverGeom {
    p.set_view(view);
    p.popover_layout(w, h)
        .expect("selected Markdown has a popover")
}

fn exercise_button(p: &mut TextPipeline, mut edit: Editing, button: PopoverButton, w: u32, h: u32) {
    p.set_view(&ViewState::base());
    let source = edit.buffer.text();
    let selection = edit.buffer.selection_line_col();
    // Highlight's equals scan cannot pair across a newline. The single-line
    // roster sweep still requires Highlight to make a real edit.
    let refuses_highlight = button == PopoverButton::Highlight
        && edit
            .buffer
            .selected_text()
            .is_some_and(|s| s.contains('\n'));
    let before = measured(p, &edit.view(), w, h);
    let target = before.buttons.iter().find(|b| b.button == button).unwrap();
    let pointer = [
        (target.x0 + target.x1) * 0.5,
        before.card[1] + before.card[3] * 0.5,
    ];
    assert_eq!(before.hit(pointer[0], pointer[1]), Some(button));
    let repeats = match button {
        PopoverButton::Bold
        | PopoverButton::Italic
        | PopoverButton::Highlight
        | PopoverButton::Code
        | PopoverButton::Strike => 4,
        PopoverButton::Heading => 8,
        PopoverButton::Link => 1,
    };
    for press in 0..repeats {
        let effect = edit.press(button);
        let view = edit.view();
        if button == PopoverButton::Link {
            assert!(view.overlay_active, "Link interrupts with its URL editor");
            p.set_view(&view);
            assert!(
                p.popover_layout(w, h).is_none(),
                "interrupted popover is gone"
            );
            continue;
        }
        if refuses_highlight {
            assert_eq!(
                effect,
                crate::actions::Effect::Notice(crate::actions::NoticeEffect::Sticky(
                    "markdown can't mark that up".to_string(),
                )),
                "{button:?}: the multiline refusal explains itself",
            );
            assert_eq!(
                edit.buffer.text(),
                source,
                "{button:?}: refusal preserves source"
            );
            assert_eq!(
                edit.buffer.selection_line_col(),
                selection,
                "{button:?}: refusal preserves the selection"
            );
        } else if press == 0 {
            assert_ne!(
                edit.buffer.text(),
                source,
                "{button:?}: first-format premise"
            );
        }
        let after = measured(p, &view, w, h);
        assert_eq!(
            &after.card[..2],
            &before.card[..2],
            "{} dpi={} {w}x{h} {button:?} press {press}: format must not move its target",
            theme::active().name,
            p.dpi
        );
        assert_eq!(
            after.hit(pointer[0], pointer[1]),
            Some(button),
            "{} {button:?} press {press}: stationary repeat still hits the same button",
            theme::active().name
        );
    }
}

#[test]
fn formatting_preserves_popover_origin_and_repeat_hit_across_worlds_and_dpi() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping formatting origin/repeat-target sweep: no wgpu adapter");
        return;
    };
    let mut cells = 0;
    for world in 0..theme::THEMES.len() {
        theme::set_active(world);
        p.sync_theme();
        for dpi in [1.0, 1.5, 2.0] {
            p.set_dpi(dpi);
            for logical_width in [800.0, 1400.0] {
                let (w, h) = ((logical_width * dpi) as u32, (800.0 * dpi) as u32);
                p.set_size(w as f32, h as f32);
                for button in PopoverButton::ALL {
                    exercise_button(&mut p, Editing::new(), button, w, h);
                    cells += 1;
                }
            }
        }
    }
    assert!(cells > 0, "the world/button roster must enroll real cases");
    assert_eq!(
        cells,
        theme::THEMES.len() * 3 * 2 * PopoverButton::ALL.len()
    );
}

#[test]
fn popover_dismissal_and_interruptions_clear_placement_before_paint() {
    let _guard = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping popover interruption/dismissal law: no wgpu adapter");
        return;
    };
    for interruption in 0..4 {
        let mut edit = Editing::new();
        p.set_view(&edit.view());
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let (before, buttons) = p.popover_report().unwrap();
        let x = (buttons[0].2[0] + buttons[0].2[1]) * 0.5;
        let y = before[1] + before[3] * 0.5;
        assert!(p.resolve_popover_hover(x, y));
        let mut down = edit.view();
        match interruption {
            0 => down.popover = None,
            1 => down.selection = None,
            2 => down.overlay_active = true,
            3 => down.search_active = true,
            _ => unreachable!(),
        }
        p.set_view(&down);
        assert!(p.popover_report().is_none() && p.popover_hit(x, y).is_none());
        assert!(
            p.popover_hover.is_none(),
            "dismissal clears the old hover immediately"
        );
        let start = SOURCE.find("another").unwrap();
        edit.buffer.set_cursor(start + "another".len());
        edit.buffer.set_anchor(start);
        // There is deliberately no prepare between the down/up snapshots.
        let after = measured(&mut p, &edit.view(), 1200, 800);
        assert_ne!(&before[..2], &after.card[..2], "a new selection re-anchors");
    }
}

#[test]
fn popover_reanchors_for_new_selection_scroll_resize_zoom_and_dpi() {
    let _guard = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping popover viewport/selection re-anchor law: no wgpu adapter");
        return;
    };
    for change in 0..6 {
        p.set_dpi(1.0);
        p.set_size(1200.0, 800.0);
        p.set_view(&ViewState::base());
        let mut edit = Editing::new();
        let before = measured(&mut p, &edit.view(), 1200, 800);
        edit.press(PopoverButton::Bold);
        let mut view = edit.view();
        assert_eq!(measured(&mut p, &view, 1200, 800).card, before.card);
        let old_scroll_top = p.rendered_scroll_top_px(view.scroll);
        let (mut w, mut h) = (1200, 800);
        match change {
            0 => {
                view.selection = Some(((6, 0), (6, 6)));
            }
            1 => {
                view.scroll = ScrollPos::at_row(2);
            }
            2 => {
                w = 560;
            }
            3 => {
                h = 140;
            }
            4 => {
                view.zoom = 1.5;
            }
            5 => {
                p.set_dpi(2.0);
            }
            _ => unreachable!(),
        }
        p.set_size(w as f32, h as f32);
        let moved = measured(&mut p, &view, w, h);
        assert_ne!(
            &moved.card[..2],
            &before.card[..2],
            "change {change} re-anchors"
        );
        if change == 1 {
            let delta = p.rendered_scroll_top_px(view.scroll) - old_scroll_top;
            assert_eq!(
                moved.card[0], before.card[0],
                "scrolling retains horizontal placement"
            );
            assert_eq!(
                moved.card[1],
                before.card[1] - delta,
                "popover follows document scrolling"
            );
            view.scroll = ScrollPos::default();
            assert_eq!(
                measured(&mut p, &view, w, h).card,
                before.card,
                "scrolling back restores the original target"
            );
            continue;
        }
        let mut closed = view;
        closed.popover = None;
        p.set_view(&closed);
        closed.popover = edit.view().popover;
        let fresh = measured(&mut p, &closed, w, h);
        assert_eq!(
            moved, fresh,
            "change {change} matches a freshly opened popover"
        );
    }
}

#[test]
fn formatting_popover_keeps_its_target_for_reversed_wrapped_unicode_selections() {
    let _guard = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(800.0, 800.0) else {
        eprintln!("skipping reversed/wrapped Unicode formatting law: no wgpu adapter");
        return;
    };
    let text = concat!(
        "# Document\n\nA long introduction before café 🐦 漢字 and more words ",
        "that wrap over the narrow writing column.\n",
        "The second selected line has a tail.\n",
    );
    let start = text[..text.find("café").unwrap()].chars().count();
    let end = text[..text.find(" has a tail").unwrap()].chars().count();
    for reverse in [false, true] {
        for button in PopoverButton::ALL {
            let mut edit = Editing {
                buffer: Buffer::from_str(text),
                journey: Journey::default(),
            };
            edit.buffer.set_cursor(if reverse { start } else { end });
            edit.buffer.set_anchor(if reverse { end } else { start });
            exercise_button(&mut p, edit, button, 800, 800);
        }
    }
}

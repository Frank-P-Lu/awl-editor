//! ITEM 114 — THE SUMMONED WORKSPACE'S OWN ACTION SEAM: the two-region keyboard,
//! and the Cmd-P deep link into it.
//!
//! Carved out of `overlay_nav.rs` (a grandfathered file already at its own
//! code-health high-water mark) so the workspace's navigation reads as one
//! thing rather than as two arms inside the picker intercept. The content model
//! is `crate::overlay::workspace`; the lifecycle it advances is
//! `crate::overlay::Journey`.

use super::*;

/// ITEM 114 — THE SUMMONED WORKSPACE'S TWO-REGION KEYBOARD.
///
/// A workspace is one task in two coordinated regions: a navigation RAIL (its
/// primary list) and a CONTENT pane (its detail stage). This is the whole of what
/// that costs at the action layer — everything else a workspace does is the
/// picker it already was.
///
///   * `Tab` moves focus between the two regions, at any width.
///   * While the RAIL holds focus, the vertical keys step CATEGORIES (through the
///     picker's own lens owner, so there is no second category state), `→` / `↵`
///     enter the rows, and typing hands focus to the rows because what you are
///     typing into is their search field.
///   * While the ROWS hold focus this intercept declines every key, so the rows
///     pane is byte-for-byte the picker's existing keyboard — `↑/↓` rows, `←/→`
///     the category (or a range row's own rail), `↵` the row's control.
///
/// `Esc` is deliberately absent from both arms: it belongs to
/// [`crate::overlay::Journey`], whose table already says a cancel on the detail
/// stage lands on the primary list and a cancel on the primary list lands in the
/// editor. Spelling it here would be the second owner item 173 exists to prevent.
pub(super) fn workspace_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let ov = ctx.journey.card().unwrap();
    ov.workspace_shape()?;
    if matches!(action, Action::InsertTab) {
        ctx.journey.toggle_detail();
        return Some(Effect::None);
    }
    if ov.detail_focus {
        return None;
    }
    // ── THE RAIL HOLDS FOCUS ──────────────────────────────────────────────
    let rail = |ctx: &mut ActionCtx, delta: isize| {
        ctx.journey.card_mut().unwrap().rail_move(delta);
        Some(Effect::None)
    };
    match action {
        Action::NextLine | Action::PageScrollDown => rail(ctx, 1),
        Action::PreviousLine | Action::PageScrollUp => rail(ctx, -1),
        // The rail is short and clamped at both ends, so a jump is just a big
        // step through the same owner rather than a second selection rule.
        Action::LineEnd | Action::BufferEnd => rail(ctx, isize::MAX / 2),
        Action::LineStart | Action::BufferStart => rail(ctx, isize::MIN / 2),
        // Rightward and Enter both mean "into the content".
        Action::ForwardChar | Action::Newline => {
            ctx.journey.toggle_detail();
            Some(Effect::None)
        }
        // There is nothing to the left of the primary list; swallowing this
        // keeps `←` from falling through to the lens cycle and moving the rail
        // sideways as well as vertically.
        Action::BackwardChar => Some(Effect::None),
        // Typing is searching, and the results are rows — so the query edit and
        // the focus hand-off are one gesture, never "type, then wonder why
        // nothing moved".
        Action::InsertChar(_) | Action::DeleteBackward | Action::DeleteWordBackward => {
            ctx.journey.toggle_detail();
            None
        }
        _ => None,
    }
}

/// THE Cmd-P DEEP LINK. A Range row's control is item 94's rail, and the palette
/// structurally cannot show one: its settings rows are appended by
/// `attach_settings_rows`, which carries no `RangeCell`, so `item_range_fracs`
/// reports nothing and the row offers a bare text field with no thumb, no band
/// and no neighbours. Rather than teach the palette a second copy of a Settings
/// control, the row takes you to the place that owns it. Every other kind keeps
/// its existing palette path: a Toggle completes in one keystroke, a Picker opens
/// the real audition, an Action runs, a Path opens the real navigator.
///
/// OPEN THE SETTINGS WORKSPACE STANDING ON `row`: its category
/// selected in the navigation rail, its own row selected in the content pane,
/// and the content pane focused. Returns whether the deep link was taken.
///
/// Only ever taken from somewhere that is NOT already the Settings workspace —
/// inside Settings the row is already where you are, and re-entering would drop
/// the parked position item 173 exists to preserve. The caller is parked, not
/// replaced, so `Esc` walks back out the way you came in.
pub(super) fn deep_link_settings(ctx: &mut ActionCtx, row: crate::settings::SettingRow) -> bool {
    if ctx.journey.card().map(|o| o.kind) == Some(crate::overlay::OverlayKind::Settings) {
        return false;
    }
    let Some(mut card) = (ctx.make_overlay)(crate::overlay::OverlayKind::Settings) else {
        return false;
    };
    card.rail_focus_category(row.category);
    if !card.select_accept(row.name) {
        return false;
    }
    ctx.journey.descend(card, crate::overlay::Bind::Value);
    // The row is the destination, so the content pane is what should hold the
    // keyboard — routed through the lifecycle, never by writing the focus bit.
    ctx.journey.toggle_detail();
    ctx.journey.card().map(|o| o.kind) == Some(crate::overlay::OverlayKind::Settings)
}

//! The summoned workspace's action seam: the two-region keyboard,
//! and the Cmd-P deep link into it.
//!
//! Carved out of `overlay_nav.rs` (a grandfathered file already at its own
//! code-health high-water mark) so the workspace's navigation reads as one
//! thing rather than as two arms inside the picker intercept. The content model
//! is `crate::overlay::workspace`; the lifecycle it advances is
//! `crate::overlay::Journey`.

use super::*;

/// The summoned workspace's two-region keyboard.
///
/// A workspace is one task in two coordinated regions: a navigation RAIL (its
/// primary list) and a CONTENT pane (its detail stage). This is the whole of what
/// that costs at the action layer — everything else a workspace does is the
/// picker it already was.
///
///   * `Tab` and `Shift-Tab` move focus between the two regions, at any width.
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
/// editor. Spelling it here would create a second transition owner.
///
/// [`crate::overlay::workspace::WorkspaceShape::rows_are_primary`] is
/// the one fact every consumer reduces to; this intercept reads it exactly once
/// and never re-branches on which kind is open. History is the one named
/// exception at the gate, not in the body: History has card presentation but
/// its comparison keyboard follows the same primary-row rule. Keeping the
/// exception here prevents kind checks from spreading through the handler.
pub(super) fn workspace_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let ov = ctx.journey.card().unwrap();
    let rows_primary = ov.workspace_shape()?.rows_are_primary();
    // IS THERE A CONTENT REGION TO GO INTO? On the timeline shape the content is
    // read-only prose that has to come from somewhere — a version selected in the
    // timeline — and an empty history (or a query that filters every version away)
    // has none. `comparison_request()` is that fact, typed and kind-neutral; with
    // no payload this intercept declines every key it would otherwise own, so
    // `Enter` falls through to the ordinary close and `Tab` cannot hand the
    // keyboard to a blank region. A `RailOverRows` workspace's rows always exist.
    let has_content = !rows_primary || ov.comparison_request().is_some();
    if !has_content {
        return None;
    }
    // Tab AND Shift-Tab move focus between the two regions at any width, in
    // either shape. Both do the same thing because there are exactly two regions,
    // and the user's Esc decision (2026-08-02) makes them the ONLY way across:
    // `Esc` now leaves the workspace from either stage, so `Shift-Tab` — which is
    // `Action::Outdent` in the document — has to answer here or the footer's
    // advertised Back would be true for one of the two keys the decision names.
    // On the timeline shape, `CompareVersion` is History's own long-standing
    // second door to the same toggle (its palette command, "Compare with
    // version…"). Checked before either region's own keys so it can never be
    // shadowed by them.
    if matches!(action, Action::InsertTab | Action::Outdent)
        || (rows_primary && matches!(action, Action::CompareVersion))
    {
        ctx.journey.toggle_detail();
        return Some(Effect::None);
    }
    if rows_primary {
        return rows_primary_intercept(ctx, action);
    }
    // ── RailOverRows (Settings, today): THE RAIL HOLDS FOCUS ─────────────
    if ov.detail_focus {
        return None;
    }
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
        // Rightward and Enter both mean "into the content". `AcceptAlternate`
        // rides the same door — it has no separate meaning on a rail of bare
        // category labels, so it defaults to exactly what `Newline` does.
        Action::ForwardChar | Action::Newline | Action::AcceptAlternate => {
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

/// The primary-rows arm, reached through `rows_primary` rather than a kind
/// check. Diff-paging
/// (`PageUp`/`PageDown`) always pages the comparison, focused on it or not — a
/// browsing convenience predating this fold, kept verbatim. `CompareVersion`/
/// `Tab` are handled by the caller before this runs.
///
/// Bare `Newline` never restores: unfocused it does
/// what `CompareVersion`/`Tab` already do (move focus into the comparison);
/// focused, there is nothing further to "enter", so it is a calm no-op. Only
/// `AcceptAlternate` (⇧↵) restores, regardless of which region holds focus —
/// deliberately absent from every arm below so it falls through to the
/// ordinary accept path (`accept_overlay` → the kind's own accept), exactly
/// where bare `Enter` used to land.
fn rows_primary_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let ov = ctx.journey.card().unwrap();
    let page = ctx.scroll_page_lines.max(1);
    let focused = ov.detail_focus;
    match action {
        Action::PageScrollDown => {
            let ov = ctx.journey.card_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_add(page);
            Some(Effect::None)
        }
        Action::PageScrollUp => {
            let ov = ctx.journey.card_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_sub(page);
            Some(Effect::None)
        }
        Action::Newline if !focused => {
            ctx.journey.toggle_detail();
            Some(Effect::None)
        }
        Action::Newline => Some(Effect::None),
        Action::AcceptAlternate => None,
        _ if focused => match action {
            Action::NextLine => {
                let ov = ctx.journey.card_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_add(1);
                Some(Effect::None)
            }
            Action::PreviousLine => {
                let ov = ctx.journey.card_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_sub(1);
                Some(Effect::None)
            }
            // Esc falls through to the shared owner: the table already says a
            // cancel on the detail stage lands on the primary list.
            Action::Cancel => None,
            _ => Some(Effect::None),
        },
        _ => None,
    }
}

/// The Cmd-P deep link. A Range row's control is a rail, and the palette
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
/// the parked position this path preserves. The caller is parked, not
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

/// Open the "Keep version…" naming minibuffer, PARKING whatever is
/// already open rather than replacing it outright. `Action::KeepVersion`'s own
/// dispatch calls this (today always reached with nothing open — the Command
/// palette closes itself before its `RunAction` re-dispatch, same door every
/// other palette-launched picker uses), and it is the door a future
/// in-workspace "keep" gesture (116d's History timeline) must call DIRECTLY —
/// like [`deep_link_settings`] above, never by re-dispatching an `Action`
/// through `apply_transition`'s top-level intercept gate, which would
/// swallow it outright while a card is already open — proven by the sibling
/// test in `actions::tests::overlay_drive`, which calls this function
/// directly for exactly that reason.
///
/// `overlay::Journey` owns suspend/return — this reuses it
/// rather than writing a second parking mechanism the way the old
/// unconditional `ctx.journey.enter(...)` effectively was (it replaced
/// whatever was up and parked nothing, silently stranding it).
pub(super) fn open_keep_version(ctx: &mut ActionCtx) {
    let card = crate::overlay::OverlayState::new_keep_name();
    if ctx.journey.card().is_some() {
        ctx.journey.descend(card, crate::overlay::Bind::Value);
    } else {
        ctx.journey.enter(Some(card));
    }
}

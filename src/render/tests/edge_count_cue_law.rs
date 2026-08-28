//! THE POSITIONAL COUNT CUE LAW.
//!
//! A windowed candidate list gave no indication of how much sat off-screen —
//! "there is no scroll bar so like how do i even know where my files are".
//! DECIDED: no literal scrollbar, a faint text cue at the window's edges
//! ("↑ 3 more" / "↓ 41 more"), extending the `+ N more…` idiom
//! (`workingset.rs`'s resting-stack overflow row — an EXPAND affordance, and
//! untouched by this item; this cue lives only in the scrolling views). One
//! windowing owner: [`crate::render::chrome::window_edge_counts`], read at
//! both card families (flat `overlay.rs`, grouped `theme_picker.rs`) through
//! the shared budget fixed point `resolve_window_and_cue`. **The SUMMONED
//! WORKSPACE family is deliberately excluded** — its row origin is shared
//! with its own RAIL (a `RailOverRows` shape's category-label column), so
//! shifting it for the cue moved the rail whenever the CONTENT pane's item
//! count happened to clip, even though the rail itself never changed
//! (`workspace.rs::workspace_geometry`'s own doc has the measured defect and
//! the pre-existing law — `render/tests/rail_ink_law.rs` — that caught it);
//! decoupling the two is future work, not part of this item's shipped scope.
//!
//! **THE SCROLL-INVARIANCE BUG THIS LAW NAMES.** An earlier cut of the
//! reservation charged exactly as many extra display lines as the edges
//! clipping AT THE CURRENT SCROLL POSITION (0, 1 or 2) — so scrolling from
//! "only below clips" (top of a long list) to "both clip" (the middle) grew
//! the card under the reader's own cursor, caught immediately by
//! `render/tests/palette_scroll_anchor.rs`'s pre-existing "scrolling moves
//! only the list, never the surface" law on an unrelated composition. The
//! fix (`resolve_window_and_cue`'s own doc) reserves BOTH edges
//! unconditionally whenever the corpus is windowed at all — a fact
//! `scroll_window`'s own `visible` already answers without reading the
//! current scroll position — so only the CONTENT of each reserved slot
//! (text or blank) varies with scroll, never the reservation itself.
//! `cue_card_geometry_is_scroll_invariant_while_windowed` below is the
//! direct regression law for exactly this class of defect.
//!
//! **THE TWO TRAPS THE ITEM NAMED EXPLICITLY.** (a) A sectioned card (the
//! theme picker) windows DISPLAY LINES (section headers interleaved with
//! item rows) but the cue counts hidden ITEMS — `window_edge_counts` is fed
//! `item_top`/`item_visible` straight off `scroll_window`, read BEFORE
//! `window_plan` turns them into a display-line count. This law's own
//! arithmetic oracle (`visible_items`, below) is read off `PlannedRow::item`
//! rather than a line count, so a family that regressed into billing
//! headers as hidden items would fail it directly. (b) The resting stack's
//! `+ N more…` row stays exactly as it is — it is never touched by this file.
//!
//! **THE GEOMETRY AXIS** (the item's own callout: one window alone is
//! exactly the shape of law that would go green while blind to this) is
//! swept two ways: a roomy canvas with a corpus bigger than any picker's own
//! `window_rows()` (the per-KIND cap binds — `command_palette_default_window_shows_the_below_edge_cue`,
//! acceptance case 1), and a short canvas with a modest corpus (the CANVAS
//! binds — `theme_picker_short_window_shows_the_below_edge_cue`, acceptance
//! case 2) — plus the full roster swept at both a tall-fits and a
//! corpus-forced-clip geometry (`every_picker_kinds_cue_is_present_iff_the_window_clips`).

use super::super::*;
use super::overlay_height_clamp_law::{Family, family, overlay_view};
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// `(hidden_above, hidden_below, visible_items, top_idx)` for the currently
/// summoned overlay, read off THREE INDEPENDENT SOURCES: the sidecar-facing
/// cue report (`overlay_edge_cue_report`), the sidecar-facing window report
/// (`overlay_window_report`, for `top_idx`), and the PLANNED rows themselves
/// (`overlay_row_plan`, counting real `item: Some(_)` rows — never a
/// display-line count, which is the sectioned-family trap this law exists
/// to catch). `n_items` is not read back at all: the caller already knows
/// it, having built the fixture's corpus itself.
fn cue_state(p: &TextPipeline, width: u32) -> (Option<usize>, Option<usize>, usize, usize) {
    let geom = p.overlay_geometry(width);
    let plan = p.overlay_row_plan(&geom);
    let (above, below) = p.overlay_edge_cue_report().unwrap_or_default();
    let visible_items = plan.rows().iter().filter(|r| r.item.is_some()).count();
    let top_idx = p.overlay_window_report().map_or(0, |r| r.0);
    (above, below, visible_items, top_idx)
}

/// The item's own arithmetic identity, asserted against the independent
/// sources `cue_state` reads: hidden-above + shown + hidden-below must equal
/// the CALLER'S OWN `n_items` (the fixture's real corpus size, never read
/// back through the same formula this checks), and hidden-above must equal
/// `top_idx` (the sidecar's own window report) exactly.
fn assert_cue_arithmetic(p: &TextPipeline, width: u32, n_items: usize, ctx: &str) {
    let (above, below, visible_items, top_idx) = cue_state(p, width);
    assert_eq!(
        above.unwrap_or(0),
        top_idx,
        "{ctx}: hidden-above ({above:?}) must equal the window's own top_idx ({top_idx})"
    );
    assert_eq!(
        above.unwrap_or(0) + visible_items + below.unwrap_or(0),
        n_items,
        "{ctx}: hidden-above + shown + hidden-below must equal the roster \
         (above={above:?} visible_items={visible_items} below={below:?} n_items={n_items})"
    );
}

/// A corpus/canvas pair that CLIPS: at least one edge cue is reserved, and
/// each reserved slot's own text (when it has one) is genuinely present in
/// the shaped `panel_buffer` — read back by its literal text, never inferred
/// from row counts (the Wagtail tripwire: a sidecar can report state a frame
/// never drew).
fn assert_clips_and_cue_present(p: &TextPipeline, width: u32, n_items: usize, ctx: &str) {
    let (above, below, ..) = cue_state(p, width);
    assert!(
        above.is_some() || below.is_some(),
        "{ctx}: this fixture is deliberately windowed but neither edge cue fired"
    );
    let (above_line, below_line) = p.overlay_cue_lines(width);
    if above.is_some() {
        assert!(
            above_line.is_some(),
            "{ctx}: cue_above={above:?} but no shaped line carries its exact text"
        );
    }
    if below.is_some() {
        assert!(
            below_line.is_some(),
            "{ctx}: cue_below={below:?} but no shaped line carries its exact text"
        );
    }
    assert_cue_arithmetic(p, width, n_items, ctx);
}

/// A corpus/canvas pair that FITS: neither edge cue fires, and no line in
/// the shaped buffer carries either arrow glyph (a presence-floor's own
/// companion — a law that only checked `cue_above`/`cue_below` would pass
/// even if the shaper drew the text anyway).
fn assert_fits_and_no_cue(p: &TextPipeline, width: u32, n_items: usize, ctx: &str) {
    let (above, below, visible_items, _) = cue_state(p, width);
    assert_eq!(
        (above, below),
        (None, None),
        "{ctx}: this fixture fits entirely but a cue fired anyway"
    );
    assert_eq!(
        visible_items, n_items,
        "{ctx}: fits entirely means every item is shown, got {visible_items} of {n_items}"
    );
    let (above_line, below_line) = p.overlay_cue_lines(width);
    assert_eq!(
        (above_line, below_line),
        (None, None),
        "{ctx}: no cue fired but a shaped line still carries cue text"
    );
}

/// A corpus/canvas pair the SUMMONED WORKSPACE family clips WITHOUT a cue —
/// the family's own deliberate exclusion (`workspace.rs::workspace_geometry`'s
/// own doc: the cue's `first_top` shift is shared with the rail's row
/// origin, so it would move the rail's category labels whenever the
/// CONTENT pane's item count happened to clip, even though the rail itself
/// never changed — caught by `render/tests/rail_ink_law.rs`). Neither edge
/// fires here, but (unlike a genuine fit) the corpus is NOT shown in full.
fn assert_clips_with_no_cue_workspace_exclusion(
    p: &TextPipeline,
    width: u32,
    n_items: usize,
    ctx: &str,
) {
    let (above, below, visible_items, _) = cue_state(p, width);
    assert_eq!(
        (above, below),
        (None, None),
        "{ctx}: the workspace family must never fire a cue"
    );
    assert!(
        visible_items < n_items,
        "{ctx}: this fixture must actually clip ({visible_items} of {n_items} shown) for the \
         exclusion to mean anything"
    );
}

const ROOMY: (u32, u32) = (1200, 800);

#[test]
fn every_picker_kinds_cue_is_present_iff_the_window_clips() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(ROOMY.0 as f32, ROOMY.1 as f32) else {
        eprintln!(
            "skipping every_picker_kinds_cue_is_present_iff_the_window_clips: no wgpu adapter"
        );
        return;
    };
    let mut fit_cells = 0usize;
    let mut clip_cells = 0usize;
    for kind in OverlayKind::ALL {
        let fam = family(kind);
        // TALL-FITS: a corpus no bigger than the kind's own window cap, at a
        // roomy canvas — nothing clips anywhere, at any family.
        let small_n = kind.window_rows().min(3).max(1);
        let v = overlay_view(kind, small_n, false);
        p.set_view(&v);
        p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
        let ctx = format!("{kind:?}/{fam:?} tall-fits");
        assert_fits_and_no_cue(&p, ROOMY.0, small_n, &ctx);
        fit_cells += 1;

        // CORPUS-FORCED CLIP: bigger than the kind's own window cap — the
        // PER-KIND cap binds regardless of how roomy the canvas is.
        //
        // The CONTEXTUAL family (Spell) is the one exception: its own
        // fixture caps the corpus to `window_rows()` before it ever reaches
        // the geometry (mirroring production, where suggestions are already
        // bounded upstream — `OverlayKind::MAX_SUGGESTIONS`), so it cannot
        // be driven into clipping through this door at all. Graded as a
        // SECOND tall-fits cell instead, so the roster is still swept by
        // name rather than silently skipped.
        let big_n = kind.window_rows() + 25;
        let v = overlay_view(kind, big_n, false);
        p.set_view(&v);
        p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
        let ctx = format!("{kind:?}/{fam:?} corpus-forced-clip");
        if fam == Family::Contextual {
            assert_fits_and_no_cue(&p, ROOMY.0, kind.window_rows().max(1), &ctx);
            fit_cells += 1;
        } else if fam == Family::Workspace {
            assert_clips_with_no_cue_workspace_exclusion(&p, ROOMY.0, big_n, &ctx);
            fit_cells += 1;
        } else {
            assert_clips_and_cue_present(&p, ROOMY.0, big_n, &ctx);
            clip_cells += 1;

            // SCROLLED TO THE TOP: `overlay_view`'s own default selection is
            // the corpus's LAST item, so every cell above exercises the
            // ABOVE edge's arithmetic (`hidden-above == top_idx`) but never
            // the BELOW edge's — a gap a mutated `below` formula sailed
            // through here while still being caught by the acceptance
            // captures below, which happen to open scrolled to the top.
            // Selecting item 0 here closes that gap across the WHOLE
            // roster rather than leaving it to two named kinds.
            let mut v_top = overlay_view(kind, big_n, false);
            v_top.overlay_selected = 0;
            p.set_view(&v_top);
            p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
            let ctx = format!("{kind:?}/{fam:?} corpus-forced-clip, scrolled to the top");
            let (above, below, ..) = cue_state(&p, ROOMY.0);
            assert_eq!(
                above, None,
                "{ctx}: scrolled to item 0, nothing should be hidden above"
            );
            assert!(
                below.is_some(),
                "{ctx}: a corpus this much bigger than the window must hide something below"
            );
            assert_clips_and_cue_present(&p, ROOMY.0, big_n, &ctx);
        }

        // The GROUPED family's own sectioned lens carries real section
        // headers — the shape `window_edge_counts` must not double-bill.
        if fam == Family::Grouped {
            let v = overlay_view(kind, big_n, true);
            p.set_view(&v);
            p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
            let ctx = format!("{kind:?}/{fam:?} sectioned corpus-forced-clip");
            assert_clips_and_cue_present(&p, ROOMY.0, big_n, &ctx);
            clip_cells += 1;
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    // NON-VACUITY: both arms of the geometry axis actually graded real
    // cells — an aggregate count alone would pass even if one arm silently
    // reported nothing.
    assert!(
        fit_cells > 20,
        "the tall-fits arm graded too few cells: {fit_cells}"
    );
    assert!(
        clip_cells > 20,
        "the clipping arm graded too few cells: {clip_cells}"
    );
}

/// Acceptance case 1: the command palette's unfiltered All lens at the
/// default window — a dozen-ish rows drawn from the whole command roster,
/// ending at an ordinary row with nothing saying the list continues before
/// this item. After the fix, the cue is present and arithmetic-correct.
#[test]
fn command_palette_default_window_shows_the_below_edge_cue() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(ROOMY.0 as f32, ROOMY.1 as f32) else {
        eprintln!(
            "skipping command_palette_default_window_shows_the_below_edge_cue: no wgpu adapter"
        );
        return;
    };
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands".to_string();
    v.overlay_items = crate::commands::names();
    v.overlay_bindings = crate::commands::effective_bindings(&[], &[]);
    v.overlay_selected = 0;
    v.overlay_window_rows = OverlayKind::Command.window_rows();
    v.overlay_lens = crate::facets::scheme(OverlayKind::Command)
        .map(|s| {
            s.strip
                .iter()
                .enumerate()
                .map(|(i, f)| (f.label.to_string(), i == 0))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        v.overlay_items.len() > v.overlay_window_rows,
        "the command roster ({}) must exceed the palette's own window ({}) for this to be \
         the reported shape at all",
        v.overlay_items.len(),
        v.overlay_window_rows
    );
    let n_items = v.overlay_items.len();
    p.set_view(&v);
    p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
    let (above, below, ..) = cue_state(&p, ROOMY.0);
    assert_eq!(
        above, None,
        "the unfiltered All lens opens scrolled to the top — nothing should be hidden above"
    );
    assert!(
        below.is_some(),
        "the palette's default window must show the below-edge cue at its own default size"
    );
    assert_clips_and_cue_present(
        &p,
        ROOMY.0,
        n_items,
        "command palette / All / default window",
    );
}

/// Acceptance case 2 (and the law's own geometry axis, above): the theme
/// picker in a SHORT window — the cue must fire because the CANVAS clips,
/// not because the corpus outgrew the per-kind cap.
#[test]
fn theme_picker_short_window_shows_the_below_edge_cue() {
    let _g = crate::testlock::serial();
    const SHORT: (u32, u32) = (900, 460);
    let Some((device, queue, mut p)) = headless_dqp(SHORT.0 as f32, SHORT.1 as f32) else {
        eprintln!("skipping theme_picker_short_window_shows_the_below_edge_cue: no wgpu adapter");
        return;
    };
    // A modest corpus — well under the theme picker's own (whole-roster)
    // window cap — so ANY clipping here is the canvas binding, not the
    // per-kind cap; the roster sweep above already covers the cap-bound arm.
    let n_items = crate::theme::THEMES.len();
    let v = overlay_view(OverlayKind::Theme, n_items, false);
    p.set_view(&v);
    p.prepare(&device, &queue, SHORT.0, SHORT.1).unwrap();
    let (_, _, visible_items, _) = cue_state(&p, SHORT.0);
    assert!(
        visible_items < n_items,
        "the short canvas must actually clip the theme roster ({visible_items} of {n_items} shown) \
         — otherwise this cell proves nothing about the canvas axis"
    );
    assert_clips_and_cue_present(&p, SHORT.0, n_items, "theme picker / short window");
}

/// **THE DIRECT REGRESSION LAW.** For a windowed corpus, the card's own
/// exterior geometry (`card_x`, `card_y`, `card_w`, `card_h`) must be
/// BYTE-IDENTICAL at every scroll position — the reservation is a property
/// of the corpus/canvas/query alone, never of which edge happens to clip at
/// the current selection. Swept over the flat, grouped and workspace
/// families; the mutation this law is named for (reserving `above.is_some()
/// as usize + below.is_some() as usize` instead of a fixed `2`) made the
/// SAME command palette's card 27px taller mid-scroll than at either end.
#[test]
fn cue_card_geometry_is_scroll_invariant_while_windowed() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(ROOMY.0 as f32, ROOMY.1 as f32) else {
        eprintln!("skipping cue_card_geometry_is_scroll_invariant_while_windowed: no wgpu adapter");
        return;
    };
    let mut swept = 0usize;
    for kind in [
        OverlayKind::Command,  // flat, per-kind-cap-bound
        OverlayKind::Theme,    // grouped
        OverlayKind::Settings, // summoned workspace
    ] {
        let n = kind.window_rows() + 25;
        let mut rects: Vec<[f32; 4]> = Vec::new();
        for sel in [0usize, n / 4, n / 2, (3 * n) / 4, n - 1] {
            let mut v = overlay_view(kind, n, false);
            v.overlay_selected = sel;
            p.set_view(&v);
            p.prepare(&device, &queue, ROOMY.0, ROOMY.1).unwrap();
            rects.push(
                p.overlay_card_rect()
                    .expect("an open overlay has a card rect"),
            );
        }
        for (i, rect) in rects.iter().enumerate().skip(1) {
            assert_eq!(
                *rect, rects[0],
                "{kind:?}: the card's own rect moved between selection 0 and {i} — \
                 {:?} -> {rect:?} (the cue's reservation must not depend on scroll position)",
                rects[0],
            );
        }
        swept += 1;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        swept, 3,
        "the scroll-invariance law must sweep all three named families"
    );
}

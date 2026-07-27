//! ITEM 104 — THE SETTINGS "EVERY SECOND ROW" LAW, the `apply_core`-seam half.
//! User report (2026-07-26, world Mopoke): Settings → Editor visibly contains
//! every row, but moving/selecting through it reaches only ALTERNATING rows
//! ("every second item being selectable bug is back" — a recurrence of the
//! 2026-07-17 Firetail report, `e10b9fa`, whose law proved shaper/plate/band
//! pitch agreement but never reproduced the live symptom).
//!
//! This sweeps the axis the earlier probes (all keyed to the Editor facet,
//! n=0..8 Downs from a fresh summon) did NOT cover: EVERY Settings facet (not
//! just Editor), BOTH directions, BOTH starting parities (even/odd start
//! index), a FILTERED list, a WINDOW-SCROLLED list (the full `All` home
//! exceeds `window_rows()`), and the Zoom RANGE row's enter/exit adjacency —
//! through the exact `apply_core` seam the live keymap resolves into
//! (`Action::NextLine`/`PreviousLine`/`ForwardChar`/`BackwardChar`), so a
//! `--keys` replay and this law can never disagree.
//!
//! Every clause asserts TWO things per step: `selected` moves by EXACTLY one
//! row (never zero, never two — the literal "every second row" failure
//! mode), and walking from one end to the other visits every row's NAME
//! exactly once (so a name-preserving swap/skip can't hide behind an
//! index-only check).

use super::super::*;
use super::{settings_drive, settings_overlay};

/// Switch `ov` to the facet whose sidecar id is `fid` (mirrors a `Right`/`Left`
/// press or a lens-strip click — `set_facet_lens`, the one owner both use).
fn goto_facet(ov: &mut OverlayState, fid: &str) {
    let idx = ov
        .facet_scheme()
        .expect("Settings always facets")
        .strip
        .iter()
        .position(|f| f.id == fid)
        .unwrap_or_else(|| panic!("no {fid:?} facet on the Settings strip"));
    ov.set_facet_lens(idx);
}

/// Walk `ov.selected` from `start` to one end via repeated `action`
/// (`NextLine`/`PreviousLine`), through the REAL `apply_core` seam
/// (`settings_drive`), asserting every step advances by EXACTLY one row and
/// recording every visited row's NAME. Returns the visited names in walk
/// order (length == rows walked, including the start).
fn walk(overlay: &mut Option<OverlayState>, action: &Action, steps: usize) -> Vec<String> {
    let mut names =
        vec![overlay.as_ref().unwrap().item_strings()[overlay.as_ref().unwrap().selected].clone()];
    for step in 0..steps {
        let before = overlay.as_ref().unwrap().selected;
        let eff = settings_drive(overlay, action);
        assert!(
            matches!(eff, Effect::None),
            "step {step}: {action:?} on an ordinary row must not open a sub-overlay \
             or otherwise signal ({eff:?})"
        );
        let after = overlay.as_ref().unwrap().selected;
        let delta = after as isize - before as isize;
        let expected = if matches!(action, Action::NextLine) {
            1
        } else {
            -1
        };
        assert_eq!(
            delta, expected,
            "step {step}: {action:?} must move `selected` by exactly one row \
             (before={before} after={after}) — a delta of 0 or ±2 IS the \
             \"every second row\" bug"
        );
        names.push(
            overlay.as_ref().unwrap().item_strings()[overlay.as_ref().unwrap().selected].clone(),
        );
    }
    names
}

/// EVERY SETTINGS FACET, BOTH DIRECTIONS, BOTH STARTING PARITIES: walking
/// from an even-start / odd-start row to the far end, one `NextLine`/
/// `PreviousLine` at a time, must reach every row's name EXACTLY once — no
/// facet, direction, or parity may skip or repeat a row.
///
/// The FORWARD half starts at row 0/1 and walks the FULL remaining distance
/// to the last row; the BACKWARD half is the true mirror image — it starts
/// at the LAST row / second-to-last row and walks the FULL remaining
/// distance down to row 0 (not a 0-or-1-step stub from the top: a `start`
/// near the top with a walk length of only `start` steps would make the
/// backward sweep exercise multiple steps only in facets with 0/1 rows,
/// leaving every longer facet's `PreviousLine` chain unswept beyond a
/// single step).
#[test]
fn every_settings_facet_reaches_every_row_forward_and_backward_from_both_parities() {
    let _g = crate::testlock::serial();
    // DERIVED, not hand-typed: reading the ids straight off `SETTINGS_FACETS.strip`
    // (the single owner `settings::settings_bucket` also reads) means a facet added
    // to production is automatically swept here too — mirroring this project's own
    // precedent for sweeping a `FacetScheme::strip` axis (`commands.rs`'s
    // `command_facets_land_on_all_home_then_group_by_menu_section`,
    // `history::tests::history_facets_land_on_all_home_then_group_by_time`), both of
    // which derive their id list from the strip rather than duplicating it. A
    // hand-typed literal here would make a NEW facet — and any "every second row"
    // bug scoped to it — invisible to the entire law, exactly the failure class this
    // law exists to catch.
    let facets: Vec<&str> = crate::settings::SETTINGS_FACETS
        .strip
        .iter()
        .map(|f| f.id)
        .collect();
    for fid in facets {
        // FORWARD from index 0 (even) and index 1 (odd, when present) to the end.
        for start in [0usize, 1usize] {
            let mut overlay = Some(settings_overlay());
            let ov = overlay.as_mut().unwrap();
            goto_facet(ov, fid);
            let n = ov.items.len();
            assert!(n > 0, "facet {fid:?} has no rows");
            let start = start.min(n - 1);
            ov.selected = start;
            let all_names: Vec<String> = ov.item_strings();
            let visited = walk(&mut overlay, &Action::NextLine, n - 1 - start);
            let expected: Vec<String> = all_names[start..].to_vec();
            assert_eq!(
                visited, expected,
                "facet {fid:?} start={start}: forward walk must visit rows \
                 {start}..{n} in order with no skip/repeat"
            );
        }
        // BACKWARD from the LAST row (n-1) and second-to-last (n-2, when
        // present) down to row 0 — the full-length mirror of the forward
        // sweep above, so a backward-only regression in any facet beyond a
        // single step is caught here, not just in the (editor-only) Zoom
        // adjacency test.
        for end_offset in [0usize, 1usize] {
            let mut overlay = Some(settings_overlay());
            let ov = overlay.as_mut().unwrap();
            goto_facet(ov, fid);
            let n = ov.items.len();
            let start = (n - 1).saturating_sub(end_offset);
            ov.selected = start;
            let all_names: Vec<String> = ov.item_strings();
            let visited = walk(&mut overlay, &Action::PreviousLine, start);
            let expected: Vec<String> = all_names[..=start].iter().rev().cloned().collect();
            assert_eq!(
                visited, expected,
                "facet {fid:?} backward start={start} (of {n} rows): backward \
                 walk must visit rows {start}..=0 in order with no skip/repeat"
            );
        }
    }
}

/// A FILTERED list (typing narrows the fuzzy query): the reduced row set is
/// still reached one-per-step in both directions — a filter re-ranks `items`
/// but must not change `move_sel`'s one-row-per-step contract.
#[test]
fn a_filtered_settings_list_still_steps_one_row_at_a_time() {
    let _g = crate::testlock::serial();
    let mut overlay = Some(settings_overlay());
    // "Page" fuzzy-matches "Page mode", "Page width (prose)", "Page width
    // (code)" — a real, non-trivial multi-row filter within the All home.
    for c in "page".chars() {
        settings_drive(&mut overlay, &Action::InsertChar(c));
    }
    let n = overlay.as_ref().unwrap().items.len();
    assert!(
        n >= 2,
        "the \"page\" filter should match more than one row (got {n})"
    );
    overlay.as_mut().unwrap().selected = 0;
    let names = walk(&mut overlay, &Action::NextLine, n - 1);
    assert_eq!(
        names.len(),
        n,
        "a filtered list of {n} rows must be walked in exactly {n} steps with no skip"
    );
    let uniq: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(
        uniq.len(),
        n,
        "every filtered row must be reached exactly once: {names:?}"
    );
}

/// A WINDOW-SCROLLED list: the full `All` home exceeds `window_rows()`, so this
/// sweep forces the scroll window to advance mid-walk — the class of bug a small,
/// always-fully-visible test corpus can't exercise. The law below derives its
/// count from the registry rather than copying it here.
#[test]
fn a_window_scrolled_settings_list_reaches_every_row_exactly_once() {
    let _g = crate::testlock::serial();
    let mut overlay = Some(settings_overlay());
    let ov = overlay.as_mut().unwrap();
    let n = ov.items.len();
    let window = ov.window_rows();
    assert!(
        n > window,
        "the All home ({n} rows) must exceed window_rows ({window}) \
             for this law to actually force a scroll — else it is vacuous"
    );
    ov.selected = 0;
    let names = walk(&mut overlay, &Action::NextLine, n - 1);
    let uniq: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(
        uniq.len(),
        n,
        "walking the whole scrolled All list must visit every one of its {n} rows \
         exactly once (a scroll-window desync would skip or repeat rows near a \
         window boundary): {names:?}"
    );
}

/// ITEM 94 — RANGE-ROW (Zoom) ENTER/EXIT ADJACENCY: stepping ONTO the Zoom
/// row and back OFF it, in both directions, must not poison the neighbouring
/// rows' reachability — `range_step`'s Left/Right claim on a Range row must
/// stay confined to Left/Right, never leak into Up/Down's one-row-per-step
/// contract for the row before or after it.
#[test]
fn the_zoom_range_row_does_not_poison_neighbouring_row_reachability() {
    let _g = crate::testlock::serial();
    let mut overlay = Some(settings_overlay());
    let ov = overlay.as_mut().unwrap();
    goto_facet(ov, "editor");
    let zoom_row = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .expect("Editor facet has a Zoom row");
    assert!(
        zoom_row > 0 && zoom_row + 1 < ov.items.len(),
        "Zoom needs a neighbour on both sides"
    );

    // Approach from ABOVE: land on Zoom via NextLine, then step off DOWN.
    ov.selected = zoom_row - 1;
    let before_name = ov.item_strings()[zoom_row - 1].clone();
    let names = walk(&mut overlay, &Action::NextLine, 2); // -1 -> zoom -> +1
    assert_eq!(names[0], before_name);
    assert_eq!(names[1], "Zoom");
    assert_eq!(
        names[2],
        overlay.as_ref().unwrap().item_strings()[zoom_row + 1],
        "stepping DOWN off the Zoom row must land on exactly the next row, not skip it"
    );

    // Approach from BELOW: land on Zoom via PreviousLine, then step off UP.
    let mut overlay = Some(settings_overlay());
    let ov = overlay.as_mut().unwrap();
    goto_facet(ov, "editor");
    ov.selected = zoom_row + 1;
    let names = walk(&mut overlay, &Action::PreviousLine, 2); // +1 -> zoom -> -1
    assert_eq!(names[1], "Zoom");
    assert_eq!(
        names[2],
        overlay.as_ref().unwrap().item_strings()[zoom_row - 1],
        "stepping UP off the Zoom row must land on exactly the previous row, not skip it"
    );

    // LEFT/RIGHT on the Zoom row steps its VALUE, not the selection — confirm
    // it stays selected (range_step's claim), then Up/Down off it still work.
    let mut overlay = Some(settings_overlay());
    let ov = overlay.as_mut().unwrap();
    goto_facet(ov, "editor");
    ov.selected = zoom_row;
    let before = overlay.as_ref().unwrap().selected;
    let eff = settings_drive(&mut overlay, &Action::ForwardChar);
    assert!(
        matches!(eff, Effect::SettingRangeStep { .. }),
        "Right on Zoom must step its value: {eff:?}"
    );
    assert_eq!(
        overlay.as_ref().unwrap().selected,
        before,
        "Right on Zoom must not move the selection"
    );
    let names = walk(&mut overlay, &Action::NextLine, 1);
    assert_eq!(
        names[1],
        overlay.as_ref().unwrap().item_strings()[zoom_row + 1],
        "after stepping its value, Down off Zoom must still land on exactly the next row"
    );
}

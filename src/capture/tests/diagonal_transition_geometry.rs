//! **THE FILTER/SCROLL TRANSITION LAW for the diagonal row composition.**
//!
//! A prior pixel-search measurement (over `--keys`-free `set_view` frames)
//! found the diagonal spine's ink column never moves across a base / filtered
//! / scrolled sequence, but no regression guard exists for that door — and
//! the guard has to be driven through REAL CHORDS —
//! `crate::run::ReplaySession::apply_chord`, the same seam `--keys` itself
//! runs, resolving through the real keymap and `actions::apply_transition` —
//! never a directly-built `ViewState`.
//!
//! The published row geometry (`overlay.window.rows[]`: each planned row's
//! drawn `x`/`y`/`w`/`h` alongside `item`, its corpus index, and `display`,
//! its window slot) makes the read cheap: this file's `read_band` copies
//! `plan_geometry.rs`'s own JSON reader rather than a Rust struct, so a
//! serializer regression fails here too.
//!
//! # Why FILTER is keyed by ITEM and SCROLL is keyed by SLOT
//!
//! `render/plan/row_extent.rs::apply_row_extent` (a file this lane does not
//! own or touch) sets `row.dx = base_dx + dx_step * row.display` — a diagonal
//! world's per-row horizontal step is a pure function of the row's WINDOW SLOT
//! (`display`, 0 at the top of the visible band), never of which item occupies
//! it. Two consequences, and they are not symmetric:
//!
//! * A FILTER that removes items without disturbing which items occupy the
//!   visible window leaves every survivor's SLOT unchanged, so its geometry is
//!   provably unchanged too — and the only honest way to state that is by
//!   ITEM identity (`row.item`). A row-INDEX comparison would still pass here
//!   (the slots really are the same rows), but it could not tell "the item I
//!   was looking at didn't move" from "whatever landed at slot i this time
//!   happens to match" — it grades the mapping, not the promise a user reads
//!   the composition by. `filter_never_displaces_a_surviving_row` below
//!   engineers the corpus so real, substantial filtering (removing most of a
//!   180-item corpus) provably leaves the on-screen survivors' slots alone,
//!   then asserts by item.
//! * A SCROLL, by definition, moves EVERY visible item to a different slot
//!   (`top_idx` advances) — so no item can occupy the same slot in the
//!   before/after frames, and an item-keyed equality claim across a genuine
//!   scroll would be FALSE, not weak: the per-row cascade is real, authored
//!   product behavior (the composition's own doc: "the spine leans"). What
//!   IS invariant — and what the prior pixel-search measurement actually
//!   shows, x = 504 in a base AND a filtered AND a *scrolled* Mangrove frame
//!   alike — is that the MAPPING from slot to x is a property of the CARD,
//!   not of what is scrolled under it. `scroll_never_moves_the_slot_to_x_map`
//!   below compares slot i's published rect before vs after a real scroll:
//!   `diagonal_pixel_composition.rs`'s own attachment-inset claim, reverified
//!   here through real chords instead of a directly-built `ViewState`.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::testscratch::ScratchDir;
use crate::theme;

/// EVERY WORLD THAT AUTHORS A DIAGONAL SPINE — read off the roster (never a
/// named list), mirroring `render/tests/diagonal_pixel_composition.rs`'s own
/// `diagonal_worlds()` so a third diagonal world enrolls here by shipping,
/// not by being remembered.
fn diagonal_worlds() -> Vec<&'static str> {
    let out: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|w| matches!(w.render_caps.list_style, theme::ListStyle::Diagonal(_)))
        .map(|w| w.name)
        .collect();
    assert!(
        out.len() >= 2,
        "the roster sweep found {} diagonal worlds — it is not reading the roster it thinks it is",
        out.len()
    );
    out
}

/// One `overlay.window.rows[]` entry, read back out of the JSON — never out
/// of a Rust struct — so a serializer regression fails here exactly like
/// `plan_geometry.rs`'s own reader.
#[derive(Clone, Copy, Debug)]
struct Row {
    display: u64,
    item: Option<u64>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

struct Band {
    rows: Vec<Row>,
    sel_row: u64,
}

fn read_band(png: &std::path::Path) -> Band {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    let w = &v["overlay"]["window"];
    assert!(!w.is_null(), "an open picker must report a window");
    let rows = w["rows"]
        .as_array()
        .expect("schema /201: `rows` is an array")
        .iter()
        .map(|r| Row {
            display: r["display"].as_u64().expect("display"),
            item: r["item"].as_u64(),
            x: r["x"].as_f64().expect("x"),
            y: r["y"].as_f64().expect("y"),
            w: r["w"].as_f64().expect("w"),
            h: r["h"].as_f64().expect("h"),
        })
        .collect();
    Band {
        rows,
        sel_row: w["sel_row"].as_u64().expect("sel_row"),
    }
}

/// The corpus: `KEEP` items FIRST (natural corpus order, so they occupy the
/// window's top slots the instant the card opens, before any query is typed),
/// then a much larger `TAIL` block whose characters share NOTHING with the
/// `keep` query — no `k`/`e`/`p` at all — so it matches the query subsequence
/// ZERO times. That asymmetry is what makes the filter transition provable
/// rather than merely plausible: the survivors are EXACTLY the items already
/// on screen, in exactly the order they already held (`fuzzy::score` ties on
/// an equal match and `fuzzy::rank` breaks ties by original corpus index —
/// `src/fuzzy.rs`), so filtering can remove 150 real items from the corpus
/// without moving a single visible one.
const KEEP: usize = 30;
const TAIL: usize = 150;

fn build_corpus() -> Vec<String> {
    let mut v: Vec<String> = (0..KEEP).map(|i| format!("keep{i:03}")).collect();
    v.extend((0..TAIL).map(|i| format!("zzzzz{i:03}")));
    v
}

/// Open the unified Go-to picker (`corpus`-backed) through TWO real chords —
/// `s-p` (Command Palette) then filtering to and accepting "Go to…" — the
/// same two-step route a live user takes, so this law never invents a
/// synthetic direct-open door the keymap does not actually expose (`OpenGoto`
/// carries no default binding of its own — `commands/catalog/navigation.rs`).
fn open_goto(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("s-p g o Space t o Enter").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// Type `text`'s characters as individual real chords (each a plain key
/// press — no modifier), exactly what `--keys "a b c"` would replay.
fn type_chars(session: &mut crate::run::ReplaySession, text: &str) {
    let spec: String = text
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let chords = crate::keyspec::parse_chords(&spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// Press Down (`NextLine`) `n` times — real arrow-key chords, driving the same
/// scroll a keyboard user drives.
fn press_down(session: &mut crate::run::ReplaySession, n: usize) {
    let spec = vec!["Down"; n].join(" ");
    let chords = crate::keyspec::parse_chords(&spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// One capture at the session's CURRENT state, canvas/DPI explicit, returning
/// the published band. `label` disambiguates the scratch PNG per checkpoint.
fn capture_checkpoint(
    session: &crate::run::ReplaySession,
    root: &std::path::Path,
    config: &Config,
    dir: &std::path::Path,
    label: &str,
    canvas: (u32, u32),
    dpi: f32,
) -> Band {
    let project = crate::run::project_info(root, &None, None, config);
    let mut opts = crate::run::fold_capture_state(session, project);
    opts.canvas = Some(canvas);
    opts.dpi = Some(dpi);
    let out = dir.join(format!("{label}.png"));
    capture_with(&out, session.buffer(), &opts).expect("capture succeeds");
    read_band(&out)
}

/// Non-vacuity + selection-exclusion shared by both arms: neither band may be
/// degenerate, and the row the OWNER colours (`sel_row`) is excluded from
/// either side of a comparison — it legitimately steps outward on a
/// staggering composition, and the published rows carry no per-row selection
/// flag to check that against (the plan's LOGICAL selected row is a
/// different fact from the drawn one, and no render path may read it outside
/// the one transaction that colours the band).
fn assert_real_band(who: &str, b: &Band) {
    assert!(
        b.rows.len() >= 4,
        "{who}: only {} published rows — too few to mean anything",
        b.rows.len()
    );
    for r in &b.rows {
        assert!(
            r.w > 1.0 && r.h > 1.0,
            "{who}: row {} is a {}x{} rect — a degenerate rect would make every \
             invariance claim below vacuous",
            r.display,
            r.w,
            r.h
        );
    }
    assert!(
        (b.sel_row as usize) < b.rows.len(),
        "{who}: sel_row {} is outside the {} published rows",
        b.sel_row,
        b.rows.len()
    );
}

/// **LAW 1 — A FILTER THAT DOES NOT DISTURB THE VISIBLE WINDOW MUST NOT MOVE
/// ANY ROW IN IT, ASSERTED BY ITEM.**
///
/// Real chords open the Go-to picker over a 180-item corpus, capture the
/// unfiltered band, type `"keep"` (4 real chords), capture again, then PROVE
/// the transition was real (150 items actually left the corpus, not a no-op)
/// before comparing every surviving item's rect keyed by `item` — excluding
/// each state's own selected row.
#[test]
fn filter_never_displaces_a_surviving_row() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping filter_never_displaces_a_surviving_row: no wgpu adapter");
        return;
    }
    let entry_world = theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();

    for world in diagonal_worlds() {
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for &(canvas, dpi) in &[((1400u32, 900u32), 1.0f32), ((2800, 1800), 2.0)] {
                theme::set_active_by_name(world).unwrap();
                check_filter_transition(world, bar, canvas, dpi);
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(entry_world);
}

/// One cell of `filter_never_displaces_a_surviving_row`'s sweep: open the
/// picker, capture before/after typing `"keep"`, prove the narrowing was
/// real, then assert the item-keyed invariance.
fn check_filter_transition(world: &str, bar: bool, canvas: (u32, u32), dpi: f32) {
    let ctx = format!("{world} bar={bar} dpi={dpi} canvas={canvas:?}");

    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl_diag_filter_{}_{}_{}_{}",
        world,
        bar,
        dpi as u32,
        std::process::id()
    )));
    let corpus = build_corpus();
    let mut buffer = Buffer::from_str("hello world\n");
    let config = Config::empty();
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let root = dir.to_path_buf();

    let mut session = crate::run::ReplaySession::new(
        crate::run::ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        &root,
        None,
        &config,
        None,
        &mut km,
    );

    open_goto(&mut session);
    let before = capture_checkpoint(&session, &root, &config, &dir, "before", canvas, dpi);
    type_chars(&mut session, "keep");
    let after = capture_checkpoint(&session, &root, &config, &dir, "after", canvas, dpi);

    assert_real_band(&format!("{ctx} before"), &before);
    assert_real_band(&format!("{ctx} after"), &after);
    assert_filter_genuinely_narrowed(&ctx, &before, &after);
    assert_survivors_keyed_by_item_hold_their_rect(&ctx, &before, &after);
}

/// PRESENCE / non-vacuity: the filter must have genuinely narrowed the
/// corpus. Every published row after filtering must carry an item index
/// below `KEEP` (the `zzzzz` tail is gone), and the total published band
/// must not have grown.
fn assert_filter_genuinely_narrowed(ctx: &str, before: &Band, after: &Band) {
    for r in &after.rows {
        if let Some(item) = r.item {
            assert!(
                item < KEEP as u64,
                "{ctx}: row {} still reports item {item} after filtering to \"keep\" — \
                 the {TAIL}-item tail did not actually leave",
                r.display
            );
        }
    }
    assert!(
        after.rows.len() <= before.rows.len(),
        "{ctx}: the filtered band published MORE rows ({}) than the unfiltered one ({}) \
         — the corpus shrank, the band cannot have grown",
        after.rows.len(),
        before.rows.len()
    );
}

/// THE CLAIM, keyed by ITEM: every item present in BOTH bands (excluding
/// each band's own selected row) lands at the exact same rect it held before
/// typing anything.
fn assert_survivors_keyed_by_item_hold_their_rect(ctx: &str, before: &Band, after: &Band) {
    use std::collections::HashMap;
    let by_item = |b: &Band| -> HashMap<u64, Row> {
        b.rows
            .iter()
            .enumerate()
            .filter(|(i, r)| *i as u64 != b.sel_row && r.item.is_some())
            .map(|(_, r)| (r.item.unwrap(), *r))
            .collect()
    };
    let (map_before, map_after) = (by_item(before), by_item(after));
    let mut compared = 0usize;
    for (item, rb) in &map_before {
        let Some(ra) = map_after.get(item) else {
            continue; // this item was itself part of the tail, or is this
            // frame's OWN selected row — not a claim this law makes.
        };
        assert!(
            (rb.x - ra.x).abs() < 0.02
                && (rb.y - ra.y).abs() < 0.02
                && (rb.w - ra.w).abs() < 0.02
                && (rb.h - ra.h).abs() < 0.02,
            "{ctx}: item {item} survived the filter but its published rect MOVED — \
             before [{:.2},{:.2},{:.2}x{:.2}], after [{:.2},{:.2},{:.2}x{:.2}] — a \
             surviving row jumped horizontally",
            rb.x,
            rb.y,
            rb.w,
            rb.h,
            ra.x,
            ra.y,
            ra.w,
            ra.h
        );
        compared += 1;
    }
    assert!(
        compared >= 3,
        "{ctx}: only {compared} items were comparable across the filter — the corpus \
         construction did not keep enough survivors on screen to mean anything"
    );
}

/// **LAW 2 — A REAL SCROLL DOES NOT MOVE THE SLOT-TO-X MAPPING.**
///
/// A `dx_per_row` cascade is real, authored product behaviour
/// (`render/chrome/diagonal/cluster.rs`'s `RowSpan`), so an item's ABSOLUTE
/// rect legitimately changes when a scroll moves it to a different window
/// slot. What must NOT change is the mapping itself: slot `i`'s rect (minus
/// whichever slot is that frame's own selected row) must read identically
/// before and after a real, substantial scroll — exactly the prior lane's own
/// measurement (x = 504 in a base, a filtered, AND a scrolled Mangrove frame
/// alike), now reverified through real Down-arrow chords instead of a
/// directly-built `ViewState`.
#[test]
fn scroll_never_moves_the_slot_to_x_map() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping scroll_never_moves_the_slot_to_x_map: no wgpu adapter");
        return;
    }
    let entry_world = theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();

    // A large, UNIFORM corpus (100 items) — no query typed here, so ranking
    // never enters; only real Down-arrow chords move the window.
    let corpus: Vec<String> = (0..100).map(|i| format!("row{i:03}")).collect();

    for world in diagonal_worlds() {
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for &(canvas, dpi) in &[((1400u32, 900u32), 1.0f32), ((2800, 1800), 2.0)] {
                theme::set_active_by_name(world).unwrap();
                let ctx = format!("{world} bar={bar} dpi={dpi} canvas={canvas:?}");

                let dir = ScratchDir::new(std::env::temp_dir().join(format!(
                    "awl_diag_scroll_{}_{}_{}_{}",
                    world,
                    bar,
                    dpi as u32,
                    std::process::id()
                )));
                let mut buffer = Buffer::from_str("hello world\n");
                let config = Config::empty();
                let mut km = crate::keymap::KeymapState::new_with_convention(
                    crate::convention::Convention::Mac,
                );
                let root = dir.to_path_buf();

                let mut session = crate::run::ReplaySession::new(
                    crate::run::ReplayPolicy::ordinary(),
                    &mut buffer,
                    &corpus,
                    &root,
                    None,
                    &config,
                    None,
                    &mut km,
                );

                open_goto(&mut session);
                let before =
                    capture_checkpoint(&session, &root, &config, &dir, "before", canvas, dpi);
                assert_real_band(&format!("{ctx} before"), &before);

                // Scroll well past the visible window, so `top_idx` genuinely
                // advances (not just the in-window selection).
                let depth = before.rows.len() + 12;
                press_down(&mut session, depth);
                let after =
                    capture_checkpoint(&session, &root, &config, &dir, "after", canvas, dpi);
                assert_real_band(&format!("{ctx} after"), &after);

                // PRESENCE: the scroll must have actually moved the window — the
                // item at slot 0 must differ, or this arm compares nothing real.
                assert_ne!(
                    before.rows[0].item, after.rows[0].item,
                    "{ctx}: {depth} Down presses left slot 0 showing the same item — the \
                     window never actually scrolled, so the invariance claim below would \
                     be checked against a no-op"
                );

                let n = before.rows.len().min(after.rows.len());
                let mut compared = 0usize;
                for i in 0..n as u64 {
                    if i == before.sel_row || i == after.sel_row {
                        continue; // the selected row legitimately steps outward.
                    }
                    let (rb, ra) = (
                        before.rows.iter().find(|r| r.display == i).unwrap(),
                        after.rows.iter().find(|r| r.display == i).unwrap(),
                    );
                    assert!(
                        (rb.x - ra.x).abs() < 0.02
                            && (rb.y - ra.y).abs() < 0.02
                            && (rb.w - ra.w).abs() < 0.02
                            && (rb.h - ra.h).abs() < 0.02,
                        "{ctx}: slot {i} read [{:.2},{:.2},{:.2}x{:.2}] before the scroll and \
                         [{:.2},{:.2},{:.2}x{:.2}] after — the slot-to-x mapping moved, which \
                         means the composition's own surface shifted under a scroll rather \
                         than staying the fixed surface-relative line the composition promises",
                        rb.x,
                        rb.y,
                        rb.w,
                        rb.h,
                        ra.x,
                        ra.y,
                        ra.w,
                        ra.h
                    );
                    compared += 1;
                }
                assert!(
                    compared >= 3,
                    "{ctx}: only {compared} slots were comparable across the scroll"
                );
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(entry_world);
}

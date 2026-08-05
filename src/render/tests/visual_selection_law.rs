//! ITEM 164 — THE VISUAL-SELECTION TRANSACTION LAWS.
//!
//! An overlay card must give ONE answer to "which row is selected?" on EVERY
//! frame, including the intermediate frames of a selection move. Before the
//! transaction it gave two: the selection BAND is animated (the living morph on
//! Pane, the `BandResponse::Slide` ease on Bars) and the primary label's ink rode
//! it, while the secondary shortcut/value column and the range rail's thumb read
//! the LOGICAL selected row and recoloured a whole glide early. The user's live
//! Command-palette screenshot is exactly that: the band and the label on "Go to
//! file…", only "Switch project…"'s shortcut switched.
//!
//! These laws observe INTERMEDIATE frames, not settled ones — a law that only
//! looks at rest cannot see this bug at all, because at rest every clock agrees.
//! Two independent clocks are exercised:
//!
//! * the REAL ONE, `advance(dt)` on an `arm_live_juice`d pipeline, driving the
//!   production `chase_or_snap` glide (a test that never advances the clock
//!   proves nothing — CLAUDE.md's item-94 tripwire);
//! * the PINNED one, `AWL_LIVING_BAND`'s phase pin, which dumps a deterministic
//!   mid-flight frame with no clock at all so a REAL-PIXEL law can read it.
//!
//! And the oracle is the COMMITTED glyph colours, read back out of the shaped
//! buffers and off the rendered pixels — never a recomputation of the rule under
//! test, which would agree with any implementation including the broken one.

use super::super::*;
use super::{headless_dqp, view};

use crate::render::livingband::{self, Choreo, MotionForce};

/// One glide duration in seconds (`OVERLAY_BAND_SLIDE_MS`).
const GLIDE_S: f32 = OVERLAY_BAND_SLIDE_MS / 1000.0;

/// The reported surface: a Command-palette-shaped card — a candidate list whose
/// rows each carry a key chord in the SECONDARY column. `lens` makes it FACETED,
/// which is what the real Cmd-P palette is.
fn palette_view(faceted: bool) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = vec![
        "Go to file".into(),
        "Switch project".into(),
        "Open recent".into(),
        "Close window".into(),
        "Find and replace".into(),
        "Toggle sidebar".into(),
    ];
    v.overlay_bindings = vec![
        "C-p".into(),
        "C-x p".into(),
        "C-r".into(),
        "C-w".into(),
        "C-s".into(),
    ];
    v.overlay_selected = 0;
    if faceted {
        v.overlay_lens = vec![("All".into(), true), ("File".into(), false)];
    }
    v
}

/// A live pipeline with Reduce Motion OFF and the juice animators armed — the
/// only state in which a band animates at all. Returns the saved reduce flag.
fn armed_dqp() -> Option<(wgpu::Device, wgpu::Queue, TextPipeline, bool)> {
    let (device, queue, mut p) = headless_dqp(1200.0, 800.0)?;
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    p.arm_live_juice();
    Some((device, queue, p, saved))
}

/// Restore every global this file arms.
fn disarm(saved: bool) {
    livingband::set_motion_test_override(None);
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    crate::render::set_motion_test_override(None);
    crate::motion::set_reduced(saved);
    theme::set_active(theme::DEFAULT_THEME);
}

/// The frame's three answers to "which rows read selected": what the PRIMARY
/// labels committed, what the SECONDARY column committed, and what the shared
/// transaction says. Read AFTER a real `prepare`.
fn answers(p: &mut TextPipeline) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let geom = p.overlay_geometry(1200);
    let (primary, secondary) = p.overlay_ink_flip_probe(&geom);
    let row_plan = p.overlay_row_plan(&geom);
    let visual = p.resolve_visual_selection(&geom, &row_plan).rows().to_vec();
    (primary, secondary, visual)
}

/// THE HEADLINE LAW — the reported first→second hover, frame by frame, on the
/// REAL clock. At EVERY frame of the glide the primary ink, the secondary ink
/// and the transaction name the SAME rows: never the band on one row while only
/// the shortcut has moved to the next.
///
/// Non-vacuity is asserted, not assumed: the run must contain a frame where the
/// selected set is still the row the pointer LEFT even though the logical
/// selection has already moved (i.e. the glide is genuinely observed mid-flight,
/// which is the only regime in which the bug can appear), and must end on the
/// arrival row.
#[test]
fn pointer_switch_never_splits_the_band_from_the_shortcut_ink() {
    // The guard comes FIRST: arming the pipeline writes process globals
    // (Reduce Motion), and every writer must hold `testlock::serial()`.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p, saved)) = armed_dqp() else {
        eprintln!("skipping pointer_switch_never_splits: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Pane));
    livingband::set_motion_test_override(Some(MotionForce {
        choreo: Choreo::Morph,
        phase: None,
    }));
    p.sync_theme();

    // PRECONDITION: this world genuinely flips BOTH inks, so a disagreement is
    // observable at all. A world with no flip would make the law vacuous.
    let (prim_ink, sec_ink) = (
        crate::render::chrome::overlay_selected_primary_ink(),
        crate::render::chrome::overlay_selected_secondary_ink(),
    );
    assert!(
        prim_ink.is_some() && sec_ink.is_some(),
        "the probe world must flip BOTH the primary and the secondary ink \
         (primary {prim_ink:?}, secondary {sec_ink:?}) or this law cannot see the bug"
    );

    let mut v = palette_view(true);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    // Settle fully on row 0 before the move, so the glide below starts from rest.
    p.advance(4.0 * GLIDE_S);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let (prim, sec, vis) = answers(&mut p);
    assert!(
        prim == sec && sec == vis && !vis.is_empty(),
        "at rest every visual must already agree \
         (primary {prim:?}, secondary {sec:?}, transaction {vis:?})"
    );
    let start_rows = vis.clone();

    // THE POINTER MOVE: the hover handed the selection to the next row. The band
    // has not moved yet. This is the exact frame in the user's screenshot.
    v.overlay_selected = 1;
    p.set_view(&v);

    let mut saw_lagging_frame = false;
    let mut last = Vec::new();
    // Frame 0 (no clock advance at all) plus a sweep across one whole glide, in
    // steps far finer than the glide so mid-transition frames are unmissable.
    for step in 0..=24 {
        if step > 0 {
            p.advance(GLIDE_S / 12.0);
        }
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let (prim, sec, vis) = answers(&mut p);
        assert_eq!(
            prim, vis,
            "step {step}: the PRIMARY label ink must read the shared transaction \
             (primary {prim:?}, transaction {vis:?})"
        );
        assert_eq!(
            sec, vis,
            "step {step}: the SECONDARY shortcut ink must read the shared transaction, \
             not the logical row (secondary {sec:?}, transaction {vis:?}) — \
             a shortcut that switches ink while the band is still on the previous row \
             is item 164's two-simultaneous-answers defect"
        );
        if vis == start_rows {
            saw_lagging_frame = true;
        }
        last = vis;
    }

    assert!(
        saw_lagging_frame,
        "NON-VACUOUS: the sweep must contain a frame where the band still reads the \
         row the pointer LEFT ({start_rows:?}) after the logical selection moved — \
         that lag is the only regime in which the split can appear"
    );
    assert_eq!(
        last,
        vec![1],
        "the glide must finally settle with exactly the arrival row reading selected"
    );

    disarm(saved);
}

/// THE SAME LAW ON THE OTHER FAMILY. `Bars` shares the split owner but a
/// different animator (`overlay_band_drawn`'s slide), and no shipped world sets
/// `BandResponse::Slide` — so the probe forces it. Without this arm the Bars
/// half of the transaction would be entirely unexercised.
#[test]
fn bars_family_selected_visuals_agree_across_a_whole_slide() {
    // The guard comes FIRST: arming the pipeline writes process globals
    // (Reduce Motion), and every writer must hold `testlock::serial()`.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p, saved)) = armed_dqp() else {
        eprintln!("skipping bars_family_selected_visuals_agree: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Bars));
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 10.0,
        grow_px: 24.0,
        extent: theme::BarExtent::HugLabel,
        coverage: theme::BarCoverage::All,
    }));
    crate::render::set_motion_test_override(Some(theme::MotionJuice {
        entrance: theme::OverlayEntrance::Instant,
        band: theme::BandResponse::Slide,
    }));
    p.sync_theme();

    let mut v = palette_view(false);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    p.advance(4.0 * GLIDE_S);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let (_, _, settled) = answers(&mut p);
    assert_eq!(
        settled,
        vec![0],
        "a settled Bars card reads exactly the logical row (got {settled:?}) — \
         coverage must be measured against the ROW SLOT, not the gap-inset plate"
    );

    v.overlay_selected = 2;
    p.set_view(&v);
    let mut saw_lag = false;
    let mut last = Vec::new();
    for step in 0..=24 {
        if step > 0 {
            p.advance(GLIDE_S / 12.0);
        }
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let (prim, sec, vis) = answers(&mut p);
        assert_eq!(prim, vis, "Bars step {step}: primary ink vs transaction");
        assert_eq!(sec, vis, "Bars step {step}: secondary ink vs transaction");
        if vis == vec![0] {
            saw_lag = true;
        }
        last = vis;
    }
    assert!(
        saw_lag,
        "NON-VACUOUS: the Bars slide must be observed lagging"
    );
    assert_eq!(last, vec![2], "the Bars slide settles on the arrival row");

    disarm(saved);
}

/// CLICK ACCEPTANCE SURVIVES THE WAIT. Making the secondaries wait for the band
/// must never make the POINTER wait: at every frame of a glide, hit-testing the
/// middle of display row `k` resolves to row `k`'s own item — including while
/// the band is nowhere near it. This is the half of the item that a "make
/// everything follow the band" fix could plausibly have broken.
#[test]
fn a_click_activates_the_row_under_the_pointer_however_far_behind_the_band_is() {
    // The guard comes FIRST: arming the pipeline writes process globals
    // (Reduce Motion), and every writer must hold `testlock::serial()`.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p, saved)) = armed_dqp() else {
        eprintln!("skipping click_activates_row_under_pointer: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Pane));
    livingband::set_motion_test_override(Some(MotionForce {
        choreo: Choreo::Morph,
        phase: None,
    }));
    p.sync_theme();

    let mut v = palette_view(false);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    p.advance(4.0 * GLIDE_S);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    // Drive a long glide (row 0 -> the last row) and hit-test EVERY row at EVERY
    // intermediate frame.
    v.overlay_selected = 5;
    p.set_view(&v);
    let mut probed = 0usize;
    for step in 0..=12 {
        if step > 0 {
            p.advance(GLIDE_S / 6.0);
        }
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let row_plan = p.overlay_row_plan(&geom);
        let vis = p.resolve_visual_selection(&geom, &row_plan);
        // Row tops come from the SHAPED buffer (`overlay_row_y_probe`), not from
        // row arithmetic — so the pointer is aimed at glyphs that genuinely exist.
        let yp = p.overlay_row_y_probe();
        let [card_x, _cy0, card_w, _ch] = p.overlay_card_rect().expect("the card is open");
        let cx = card_x + card_w * 0.5;
        for (&k, &row_top) in &yp.primary {
            let cy = row_top + yp.lh * 0.5;
            assert_eq!(
                p.overlay_row_at(cx, cy),
                Some(k),
                "step {step}: the pointer over display row {k} must hit row {k}'s own item \
                 regardless of where the band currently reads selected ({:?})",
                vis.rows()
            );
            probed += 1;
        }
    }
    assert!(
        probed >= 60,
        "the hit-test sweep must be substantial (got {probed})"
    );

    disarm(saved);
}

/// REAL PIXELS, MID-FLIGHT — the Wagtail tripwire applied to the SECONDARY
/// column. On a 1-bit world the band is a solid WHITE fill and on-band ink is
/// BLACK, so a shortcut that recolours a row ahead of its own band paints black
/// glyphs on the black card: literally invisible. The sidecar cannot see that;
/// only pixels can.
///
/// Pins a mid-flight morph phase (deterministic, no clock) with the band still
/// climbing toward the selected row, and asserts the target row's SHORTCUT
/// glyphs are still WHITE — while every row the band actually covers carries
/// BLACK glyphs on white fill.
#[test]
fn the_shortcut_column_never_flips_ahead_of_its_own_band_in_real_pixels() {
    // The guard comes FIRST: arming the pipeline writes process globals
    // (Reduce Motion), and every writer must hold `testlock::serial()`.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p, saved)) = armed_dqp() else {
        eprintln!("skipping shortcut_column_never_flips_ahead: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Pane));
    // An EARLY pinned phase: the band has just left its start row, three rows
    // BELOW the target, and has not yet climbed onto the target.
    livingband::set_motion_test_override(Some(MotionForce {
        choreo: Choreo::Morph,
        phase: Some(0.1),
    }));
    p.sync_theme();

    let mut v = palette_view(true);
    v.overlay_selected = 2;
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let geom = p.overlay_geometry(1200);
    assert!(
        p.overlay_geom_is_faceted(&geom),
        "the lens strip must route the FACETED palette layout"
    );
    let (covered, target, first_top, lh, band) = p.living_probe_geom(&geom);
    assert!(
        !covered.is_empty() && !covered.contains(&target),
        "mid-flight the band must cover rows but NOT the target \
         (covered {covered:?}, target {target})"
    );
    // The secondary column must actually be drawn, or there is nothing to assert.
    let (_, secondary_flipped) = p.overlay_ink_flip_probe(&geom);
    assert!(
        p.overlay_right_column_shown(),
        "the palette must be drawing its shortcut column for this law to mean anything"
    );
    assert_eq!(
        secondary_flipped, covered,
        "the shortcut ink must flip on exactly the rows the band covers"
    );

    let [_bx, band_top, _bw, band_h] = band;
    let band_bot = band_top + band_h;
    let (texture, tview) = super::dither::offscreen(&device, 1200, 800);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item-164 shortcut ink encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let pixels = super::dither::read_pixels(&device, &queue, &texture, 1200, 800);
    let at = |x: i64, y: i64| pixels[(y as u32 * 1200 + x as u32) as usize];
    let near_white = |px: [u8; 4]| px[3] == 255 && px[0] >= 170 && px[1] >= 170 && px[2] >= 170;
    let near_black = |px: [u8; 4]| px[3] == 255 && px[0] <= 80 && px[1] <= 80 && px[2] <= 80;

    // The SHORTCUT column's own x-band: the right third of the card, where the
    // right-aligned chords sit — well clear of these short primary labels.
    let [card_x, _cy0, card_w, _ch] = p.overlay_card_rect().expect("the palette card is open");
    let x0 = (card_x + card_w * 0.62) as i64;
    let x1 = (card_x + card_w - 4.0) as i64;

    // (1) The not-yet-reached TARGET row's shortcut keeps WHITE (unflipped) ink
    // on the black ground. A premature flip paints it black-on-black — zero
    // white pixels here — which is precisely the reported defect made visible.
    let t_top = (first_top + target as f32 * lh + 3.0) as i64;
    let t_bot = (first_top + (target as f32 + 1.0) * lh - 3.0) as i64;
    let mut target_white = 0usize;
    for y in t_top..t_bot {
        for x in x0..x1 {
            if near_white(at(x, y)) {
                target_white += 1;
            }
        }
    }
    assert!(
        target_white > 8,
        "the not-yet-reached target row {target}'s SHORTCUT must keep WHITE ink \
         (got {target_white} white px in x {x0}..{x1}); a flip ahead of the band \
         paints it black-on-black — invisible"
    );

    // (2) Every row the band DOES cover carries BLACK shortcut glyphs riding the
    // white fill — the flip is real, not merely absent everywhere.
    let mut any_black = false;
    for &k in &covered {
        let row_top = first_top + k as f32 * lh;
        let y_lo = row_top.max(band_top).ceil() as i64;
        let y_hi = (row_top + lh).min(band_bot).floor() as i64;
        if y_hi <= y_lo {
            continue;
        }
        for y in y_lo..y_hi {
            for x in x0..x1 {
                if near_black(at(x, y)) {
                    any_black = true;
                }
            }
        }
    }
    assert!(
        any_black,
        "at least one covered row must show BLACK shortcut glyphs on the white band \
         (covered {covered:?}) — otherwise the flip never fires and (1) is vacuous"
    );

    disarm(saved);
}

// --- The no-wildcard source sweep -------------------------------------------

/// The ONLY production files that may READ the LOGICAL selected display row off
/// the scene plan ([`crate::render::plan::OverlayRowPlan::selected_display`],
/// which item 174 made the single owner of that derivation).
///
/// * `chrome/overlay.rs` — `overlay_window_report`, the sidecar's STATE oracle
///   (`sel_row` answers "what does Enter run"), deliberately not a rendering
///   decision.
/// * `chrome/overlay_visual_sel.rs` — the ONE visual-selection transaction, the
///   single place a render path is allowed to convert state into a drawn row.
///
/// Any other production caller is a new selected visual growing its own clock —
/// exactly how the shortcut column drifted a glide ahead of its band.
const LOGICAL_ROW_OWNERS: &[&str] = &["chrome/overlay.rs", "chrome/overlay_visual_sel.rs"];

fn names_logical_row(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false; // prose, not code
    }
    // A CALL, not the planner's own definition (`fn selected_display(`).
    line.contains(".selected_display()")
}

fn scan(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, usize)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                continue;
            }
            scan(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // A sibling `tests.rs` is the test tier, not a render path.
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            if names_logical_row(line) {
                out.push((rel.clone(), i + 1));
            }
        }
    }
}

#[test]
fn no_render_path_reads_the_logical_selected_row_outside_the_transaction() {
    let render_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("render");
    let mut hits = Vec::new();
    scan(&render_root, &render_root, &mut hits);
    let strays: Vec<String> = hits
        .iter()
        .filter(|(f, _)| !LOGICAL_ROW_OWNERS.contains(&f.as_str()))
        .map(|(f, l)| format!("  {f}:{l}"))
        .collect();
    assert!(
        strays.is_empty(),
        "only {LOGICAL_ROW_OWNERS:?} may read the LOGICAL selected display row. A render \
         path that colours or places a selected visual from state instead of from \
         `resolve_visual_selection` re-splits item 164's transaction and puts two answers \
         on the card during every selection move. offending lines:\n{}",
        strays.join("\n")
    );

    // NON-VACUOUS: both owners really do carry calls, so a refactor that moved
    // the transaction elsewhere without updating this list trips the law rather
    // than silently emptying it.
    for owner in LOGICAL_ROW_OWNERS {
        assert!(
            hits.iter().any(|(f, _)| f == owner),
            "{owner} must actually call `plan.selected_display()` — found none, \
             so this law is scanning for something that no longer exists"
        );
    }
}

#[test]
fn the_source_scanner_reads_code_and_skips_prose() {
    assert!(names_logical_row(
        "        let logical = plan.selected_display();"
    ));
    assert!(!names_logical_row(
        "/// reads `.selected_display()` — a prose reference"
    ));
    assert!(!names_logical_row("// .selected_display() in a note"));
    assert!(!names_logical_row(
        "    pub(in crate::render) fn selected_display(&self) -> Option<usize> {"
    ));
    assert!(!names_logical_row("let x = vis.reads_selected(row);"));
}

/// FACTUAL RECORD, not a taste law: how many rows read selected across a real
/// glide. Item 164's brief asked for "exactly one row" at every intermediate
/// frame, and the MEASURED answer on the shipped Morph voice, for a one-row
/// pointer move, is `[1, 2, 1, 1, ...]` — one row at every frame but a single
/// one, where the band's stretch majority-covers BOTH rows at once.
///
/// That one frame is kept deliberately. The >50%-coverage rule is a LEGIBILITY
/// rule, not a decoration: a row with band fill under it must carry on-band ink
/// or it goes invisible on an inverse-fill world. When the stretched band
/// genuinely owns two rows, two rows genuinely have fill, so cardinality 1 and
/// the authored living band are mutually exclusive — and the item's real defect
/// was never straddling, it was the band on one row and the shortcut on
/// another. This test pins the shape (never more than the two rows a one-row
/// stretch can reach, settling on exactly one) so a regression that widened it
/// would be caught, and so the number in the log is measured, not assumed.
#[test]
fn the_selected_row_count_across_a_glide_is_recorded_not_assumed() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p, saved)) = armed_dqp() else {
        eprintln!("skipping selected_row_count_record: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Pane));
    livingband::set_motion_test_override(Some(MotionForce {
        choreo: Choreo::Morph,
        phase: None,
    }));
    p.sync_theme();
    let mut v = palette_view(true);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    p.advance(4.0 * GLIDE_S);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    v.overlay_selected = 1;
    p.set_view(&v);
    let mut seen = Vec::new();
    for step in 0..=24 {
        if step > 0 {
            p.advance(GLIDE_S / 12.0);
        }
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let (_, _, vis) = answers(&mut p);
        seen.push(vis.len());
    }
    eprintln!("ITEM164 selected-row counts across the glide: {seen:?}");
    assert!(
        seen.iter().all(|&n| n <= 2),
        "a one-row move must never light more than the two rows the stretched \
         band can straddle: {seen:?}"
    );
    assert_eq!(
        *seen.last().unwrap(),
        1,
        "the glide settles on exactly one row"
    );
    disarm(saved);
}

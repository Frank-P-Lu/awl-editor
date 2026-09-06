//! The thematic-break line's reveal state has ONE owner: `wysiwyg_reveals`'s
//! "caret line OR selection touch" test, threaded into BOTH the row-scale
//! decision (`build_line_attrs`'s `confirmed_rule` gate,
//! [`crate::render::spans::md_line_scale`]) and the ornament draw gate
//! (`rule_lines`, consumed by `prepare_ornaments`). The two halves are proved
//! together here because a fix to one side alone reintroduces the other from
//! the opposite direction: scale the row down on reveal but keep drawing the
//! fleuron, or drop the fleuron on reveal but keep the row at ornament size.
//!
//! # The row-scale half — the revealed row drops the ornament's room entirely
//!
//! Measured on Saltpan: a body row is 32px, an unrevealed `---` row is
//! 70.4px (`ornament_scale` 2.2×) — and, pre-fix, STAYED 70.4px once the
//! caret landed, with the raw `---` shaping at the ornament's 2.2× advance
//! against ~14px body text (a ~33×70px block caret slab). The decided fix:
//! a REVEALED rule line (caret on it, or a selection touching it) is body
//! size in every dimension — row height, glyph advances, caret cell.
//!
//! # The ornament-draw half — a selection-revealed rule line no longer double-draws
//!
//! `rule_lines()` used to drop only the CARET's own line before handing its
//! set to `prepare_ornaments`; a selection touching a rule line (caret
//! elsewhere) left the line in that set, so the fleuron drew ON TOP of the
//! now-revealed raw markup. Fixed by widening `rule_lines()`'s gate with the
//! same `selection_touch_bytes`/`selection_touches` pair `footnote_marks`/
//! `bare_url_marks` already use — one owner, no per-layer re-derivation.
//!
//! # What is measured, and how
//!
//! The row-scale/advance claims are GEOMETRY (the sealed `layout_report`),
//! never a magic-number comparison: the revealed row's per-char advances are
//! compared against the SAME BYTES shaped as an ordinary non-markdown line —
//! the renderer's own body-scale ground truth, not an authored constant. The
//! "no fleuron ink" claim is an APPEARANCE claim (CLAUDE.md: the
//! sidecar/geometry is a state oracle, not an appearance oracle) and is
//! asserted by real pixel arithmetic over a rendered frame, swept across
//! canvas widths that move the adaptive column (the probe that caught the
//! original bug moved the column 80px right).

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view_md};

/// Logical line indices in [`DOC`] below (blank-line-separated so each
/// construct shapes alone): a real thematic break (2), a fenced code block
/// whose BODY happens to contain a `---`-shaped line (8) — the confirmed-rule
/// control that must never grow or draw an ornament, in ANY reveal state,
/// because the real parse tags it `Code`, never `Rule`.
const DOC: &str =
    "intro\n\n---\n\nmore\n\n```\ncode\n---\nmore code\n```\n\nend\n";
const RULE_LINE: usize = 2;
const FENCED_DASH_LINE: usize = 8;

const W: u32 = 1200;
const H: u32 = 800;

/// `view_md(DOC, ..)` with a two-line selection touching `line` while the
/// caret sits on a DIFFERENT line entirely (`caret_line`) — the skip-gate
/// axis both items name: a selection-only reveal must serve the same state a
/// caret-on-the-line reveal does.
fn view_selecting(caret_line: usize, sel_from: usize, sel_to: usize) -> ViewState {
    let mut v = view_md(DOC, caret_line, 0);
    v.selection = Some(((sel_from, 0), (sel_to, 0)));
    v
}

// ---------------------------------------------------------------------------
// The row-scale half — row height + advances
// ---------------------------------------------------------------------------

/// For one world: caret OFF the rule line keeps the ornament-scaled row (AND
/// draws the fleuron — the presence floor a pure geometry check could miss by
/// deleting its own subject); caret ON it, and a SELECTION merely touching it
/// with the caret elsewhere, both drop to EXACTLY the body row height with
/// EXACTLY the body-scale per-char advances — measured against the same
/// bytes shaped as plain (non-markdown) text, never an authored constant.
fn assert_reveal_drops_ornament_scale(world: &'static str) {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping assert_reveal_drops_ornament_scale({world}): no wgpu adapter");
        return;
    };
    crate::theme::set_active_by_name(world).unwrap();
    p.sync_theme();
    let body_h = p.metrics.line_height;
    let ornament_h = body_h * crate::theme::active().ornament_scale;
    assert!(
        (ornament_h - body_h).abs() > 1.0,
        "{world}: ornament_scale must genuinely differ from 1.0 or this law is vacuous"
    );

    let render = |p: &mut TextPipeline, v: &ViewState| {
        p.set_view(v);
        p.prepare(&device, &queue, W, H).unwrap();
        p.layout_report().expect("sealed frame is reportable")
    };

    // The BODY-SCALE ground truth: the identical bytes, non-markdown, so
    // `md_line_scale` never applies a heading/rule scale at all — the
    // renderer's own answer for "what does this text look like at 1.0×",
    // not a guessed constant.
    let mut plain = view_md(DOC, RULE_LINE, 0);
    plain.is_markdown = false;
    let plain_report = render(&mut p, &plain);
    let plain_row = plain_report
        .rows
        .iter()
        .find(|r| r.logical_line == RULE_LINE)
        .expect("plain rule-shaped row present");

    // OFF: caret elsewhere, no selection — the ornament's own reserved room,
    // and the fleuron actually draws (presence, via `rule_tops`).
    let off_report = render(&mut p, &view_md(DOC, 0, 0));
    let off_row = off_report
        .rows
        .iter()
        .find(|r| r.logical_line == RULE_LINE)
        .unwrap();
    assert!(
        (off_row.height - ornament_h).abs() < 0.6,
        "{world}: caret-off rule row height {} must equal ornament scale {ornament_h}",
        off_row.height
    );
    assert_eq!(
        p.rule_tops().len(),
        1,
        "{world}: caret-off, the fleuron must actually be drawn (presence floor)"
    );

    // ON: caret lands on the rule line itself.
    let on_report = render(&mut p, &view_md(DOC, RULE_LINE, 0));
    let on_row = on_report
        .rows
        .iter()
        .find(|r| r.logical_line == RULE_LINE)
        .unwrap();
    assert!(
        (on_row.height - body_h).abs() < 0.6,
        "{world}: caret-on rule row height {} must drop to body height {body_h}, not stay at \
         the ornament's {ornament_h}",
        on_row.height
    );
    assert_eq!(
        on_row.xs.len(),
        plain_row.xs.len(),
        "{world}: caret-on revealed glyph count must match the plain (body-scale) shaping"
    );
    for (a, b) in on_row.xs.iter().zip(plain_row.xs.iter()) {
        assert!(
            (a - b).abs() < 0.6,
            "{world}: caret-on revealed advances {:?} must equal body advances {:?}",
            on_row.xs,
            plain_row.xs
        );
    }

    // SEL: the skip-gate axis — a selection touches the rule line while the
    // caret sits on a DIFFERENT line. Same metrics as ON, not the stale
    // ornament-scaled row a caret-line-only gate would serve.
    let sel_report = render(&mut p, &view_selecting(4, 0, 4));
    let sel_row = sel_report
        .rows
        .iter()
        .find(|r| r.logical_line == RULE_LINE)
        .unwrap();
    assert!(
        (sel_row.height - body_h).abs() < 0.6,
        "{world}: selection-touch (caret elsewhere) rule row height {} must drop to body \
         height {body_h} exactly like the caret-on case",
        sel_row.height
    );
    for (a, b) in sel_row.xs.iter().zip(plain_row.xs.iter()) {
        assert!(
            (a - b).abs() < 0.6,
            "{world}: selection-touch revealed advances {:?} must equal body advances {:?}",
            sel_row.xs,
            plain_row.xs
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// THE ROSTER, NOT ONE NAMED WORLD: the default launch world plus the
/// roster's own largest-`ornament_scale` member — so the law is proven at
/// the tightest real value (a small scale could hide a geometry bug under
/// rounding) as well as the everyday default.
#[test]
fn revealed_rule_row_drops_to_body_scale_across_the_roster() {
    let biggest = crate::theme::THEMES
        .iter()
        .max_by(|a, b| a.ornament_scale.total_cmp(&b.ornament_scale))
        .expect("a non-empty roster");
    assert!(
        biggest.ornament_scale > 1.0,
        "the roster's largest ornament_scale must genuinely exceed 1.0 or the second world \
         below proves nothing beyond the default"
    );
    let default_name = crate::theme::THEMES[crate::theme::DEFAULT_THEME].name;
    for world in [default_name, biggest.name] {
        assert_reveal_drops_ornament_scale(world);
    }
}

/// THE CONFIRMED-RULE CONTROL: a `---`-shaped line living inside a fenced
/// code block is never a real `Rule` span, so it must stay body height and
/// draw no ornament in EVERY reveal state — caret elsewhere, caret ON that
/// exact line, and a selection touching it. A regression that dropped the
/// `confirmed_rule` gate (or forgot to thread `revealed` through it) would
/// either always grow this line, or grow it only when revealed — this law
/// catches both directions.
#[test]
fn a_fenced_dash_line_stays_body_height_through_every_reveal_state() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) =
        headless_dqp(W as f32, H as f32)
    else {
        eprintln!(
            "skipping a_fenced_dash_line_stays_body_height_through_every_reveal_state: no \
             wgpu adapter"
        );
        return;
    };
    let body_h = p.metrics.line_height;

    let render = |p: &mut TextPipeline, v: &ViewState| {
        p.set_view(v);
        p.prepare(&device, &queue, W, H).unwrap();
        p.layout_report().expect("sealed frame is reportable")
    };

    let states = [
        ("caret elsewhere", view_md(DOC, 0, 0)),
        ("caret on the fenced dash", view_md(DOC, FENCED_DASH_LINE, 0)),
        (
            "selection touching the fenced dash",
            view_selecting(0, FENCED_DASH_LINE, FENCED_DASH_LINE),
        ),
    ];
    for (ctx, v) in states {
        let report = render(&mut p, &v);
        let row = report
            .rows
            .iter()
            .find(|r| r.logical_line == FENCED_DASH_LINE)
            .expect("fenced dash row present");
        assert!(
            (row.height - body_h).abs() < 0.6,
            "{ctx}: a fenced `---` body line must stay body height ({body_h}), got {}",
            row.height
        );
        // Exactly one real ornament in the whole document (line RULE_LINE) —
        // the fenced dash never contributes a second one, whatever state
        // IT is in.
        assert!(
            p.rule_tops().len() <= 1,
            "{ctx}: the fenced dash line must never itself draw an ornament"
        );
    }
}

// ---------------------------------------------------------------------------
// The ornament-draw half — real pixels
// ---------------------------------------------------------------------------

/// The canvases this sweep renders at — two widths under PAGE mode, wide
/// enough to hold the fixed measure but different enough to actually move
/// the adaptive column's left edge (asserted below, not assumed).
const CANVASES: &[(u32, u32)] = &[(1400, 900), (900, 900)];

/// Ink present past [`INK_DIFF_FLOOR`] anywhere in `[x0,x1) x [y0,y1)`,
/// against a `bg` sample taken from empty page ground. Well above 8-bit
/// quantization noise, well under a real glyph edge's step.
const INK_DIFF_FLOOR: i32 = 24;

fn has_ink(
    pixels: &[[u8; 4]],
    w: i64,
    bg: [u8; 4],
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
) -> bool {
    for y in y0.max(0)..y1 {
        for x in x0.max(0)..x1 {
            let idx = (y * w + x) as usize;
            let Some(p) = pixels.get(idx) else { continue };
            let diff = (0..3).map(|c| (p[c] as i32 - bg[c] as i32).abs()).sum::<i32>();
            if diff > INK_DIFF_FLOOR {
                return true;
            }
        }
    }
    false
}

/// THE HEADLINE ORNAMENT-DRAW LAW. A selection merely TOUCHING the rule line (caret on
/// a different line entirely) must draw ZERO fleuron ink in the column's
/// CENTER band (where the fleuron centers, well clear of the left-aligned
/// revealed raw markup near `text_left`) — swept across canvases that move
/// the adaptive column, with a presence companion (caret-off) proving the
/// sample window actually finds real ink when the fleuron IS drawn, and a
/// caret-on control re-confirming the half that already worked.
#[test]
fn selection_touching_a_rule_line_draws_no_fleuron_ink() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping selection_touching_a_rule_line_draws_no_fleuron_ink: no wgpu adapter");
        return;
    };
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(50);

    let render = |p: &mut TextPipeline,
                  device: &wgpu::Device,
                  queue: &wgpu::Queue,
                  w: u32,
                  h: u32,
                  v: &ViewState| {
        p.set_view(v);
        p.prepare(device, queue, w, h).unwrap();
        let (texture, tview) = offscreen(device, w, h);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl rule-reveal test encoder"),
        });
        p.render(&mut encoder, &tview).expect("render failed");
        queue.submit(Some(encoder.finish()));
        read_pixels(device, queue, &texture, w, h)
    };

    let mut lefts = Vec::new();
    let mut graded = 0usize;
    for &(cw, ch) in CANVASES {
        p.set_size(cw as f32, ch as f32);
        p.set_view(&view_md(DOC, 0, 0));
        // The fleuron's own vertical band, read off the CARET-OFF (ornament-
        // scaled) layout — the geometry the ornament actually draws into.
        p.prepare(&device, &queue, cw, ch).unwrap();
        let report = p.layout_report().unwrap();
        let rule_row = report
            .rows
            .iter()
            .find(|r| r.logical_line == RULE_LINE)
            .unwrap();
        let (top, height) = (rule_row.top, rule_row.height);
        let col_left = p.column_left();
        let col_w = p.column_width();
        lefts.push(col_left);
        // CENTER THIRD of the column: the fleuron centers there; the raw
        // markup (revealed, left-aligned at `text_left`) never reaches this
        // far right for a 3-glyph `---` at any scale in this fixture.
        let (bx0, bx1) = (
            (col_left + col_w * 0.35) as i64,
            (col_left + col_w * 0.65) as i64,
        );
        let (y0, y1) = (top as i64, (top + height).max(top + 32.0) as i64);

        let bg_px = read_pixels(&device, &queue, &{
            // A throwaway render of the SAME view to sample empty page ground
            // far from any text, in the same frame's own palette.
            let (t, v) = offscreen(&device, cw, ch);
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("awl rule-reveal bg encoder"),
            });
            p.render(&mut enc, &v).expect("render failed");
            queue.submit(Some(enc.finish()));
            t
        }, cw, ch);
        let bg = bg_px[(10 * cw as i64 + 10) as usize];

        // PRESENCE (caret off, no selection): the fleuron really is there.
        let off_px = render(&mut p, &device, &queue, cw, ch, &view_md(DOC, 0, 0));
        assert!(
            has_ink(&off_px, cw as i64, bg, bx0, bx1, y0, y1),
            "canvas {cw}x{ch}: caret-off must draw real fleuron ink in the column's center \
             band [{bx0},{bx1})x[{y0},{y1}) — a law that finds nothing here proves nothing \
             about the states below"
        );

        // THE ORNAMENT-DRAW CLAIM: selection touches the rule line, caret elsewhere.
        let sel_px = render(&mut p, &device, &queue, cw, ch, &view_selecting(4, 0, 4));
        assert!(
            !has_ink(&sel_px, cw as i64, bg, bx0, bx1, y0, y1),
            "canvas {cw}x{ch}: a selection touching the rule line (caret elsewhere) must draw \
             NO fleuron ink in the center band — the double-draw this item fixes"
        );

        // CARET-ON CONTROL: already correct pre-fix, must stay correct.
        let on_px = render(&mut p, &device, &queue, cw, ch, &view_md(DOC, RULE_LINE, 0));
        assert!(
            !has_ink(&on_px, cw as i64, bg, bx0, bx1, y0, y1),
            "canvas {cw}x{ch}: caret-on must still draw no fleuron ink"
        );

        // SELECTION CLEARED: the fleuron returns (non-vacuity companion for
        // the absence claim above — an off state that never draws again
        // would make the absence claim meaningless).
        let cleared_px = render(&mut p, &device, &queue, cw, ch, &view_md(DOC, 4, 0));
        assert!(
            has_ink(&cleared_px, cw as i64, bg, bx0, bx1, y0, y1),
            "canvas {cw}x{ch}: clearing the selection must bring the fleuron back"
        );

        graded += 1;
    }

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    assert!(
        (lefts[0] - lefts[1]).abs() > 20.0,
        "the canvas sweep must actually move the adaptive column's left edge ({lefts:?}) or \
         this law never swept the axis the original bug needed to be found on"
    );
    assert_eq!(graded, CANVASES.len(), "every canvas must be graded");
}

/// A fenced `---` body line draws no fleuron in any reveal state — the same
/// confirmed-rule control as [`a_fenced_dash_line_stays_body_height_through_every_reveal_state`],
/// asked of the ORNAMENT layer instead of the row-scale layer, so a
/// regression that fixed one gate but not the other still gets caught.
#[test]
fn a_fenced_dash_line_never_draws_a_fleuron() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping a_fenced_dash_line_never_draws_a_fleuron: no wgpu adapter");
        return;
    };
    for v in [
        view_md(DOC, 0, 0),
        view_md(DOC, FENCED_DASH_LINE, 0),
        view_selecting(0, FENCED_DASH_LINE, FENCED_DASH_LINE),
    ] {
        p.set_view(&v);
        // Exactly one ornament top in the whole doc (the real rule at
        // RULE_LINE) in every state — the fenced dash never adds a second.
        assert_eq!(
            p.rule_tops().len(),
            1,
            "the fenced dash line must never itself contribute an ornament"
        );
    }
}

//! **THE BACK IS THE SAME BACK AT EVERY WIDTH, AND IT IS ON SCREEN AT EVERY
//! WIDTH.**
//!
//! The reported strangeness of `Tab`-as-Back was a NARROW-STAGE complaint: below
//! `workspace_is_wide` the workspace shows one region at a time, so "return
//! focus to the rail" names a region that is not drawn. The fix — `⌫`, through
//! `crate::overlay::workspace::BackKey` — is width-independent by construction,
//! because neither the footer nor the action seam takes a width. This file is
//! where that construction is checked against real geometry rather than trusted.
//!
//! # Two claims, and why neither is the other
//!
//!   * **INVARIANCE.** Across the whole swept geometry the footer resolves to
//!     ONE sentence. A wide layout and a staged layout that taught different
//!     Backs would be two products; a wide layout and a staged layout that
//!     agreed only because the sweep never crossed the threshold would be a
//!     vacuous law, which is why the sweep asserts it reached BOTH regimes and
//!     names how many cells landed in each.
//!   * **PRESENCE.** The footer that carries the Back is actually planned,
//!     actually shaped, and actually inked — inside the card, at every cell.
//!     Invariance alone is satisfied perfectly by a workspace that draws no
//!     footer anywhere: an identical sentence nobody can read is identical.
//!     The presence floor is what makes the invariance worth having, and the
//!     narrow stage is exactly where a footer has historically gone missing (a
//!     stage showing the other region plans `hint_rows = 0`).
//!
//! # The axis, and the one that produced the misreading
//!
//! `workspace_is_wide` is a threshold in LOGICAL px over scaled text, so it
//! MOVES with zoom and with the display face's own metrics — a single quoted
//! width is the threshold at one zoom and nowhere else
//! (`workspace_stage_reach`'s module doc records how that turned into a false
//! defect report). So the sweep crosses width × zoom × scale rather than picking
//! a width, and derives its enrolment from the workspace roster instead of
//! naming Settings.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};
use crate::overlay::workspace::{BackKey, WorkspaceShape};
use crate::overlay::{OverlayKind, OverlayState};

/// A LOCAL LUMINANCE STEP big enough to be a glyph edge rather than a gradient —
/// the same threshold the foot-hint pixel law uses, for the same reason: card
/// grounds and washes move slowly, type does not.
const GLYPH_STEP: f32 = 24.0;

fn luma(p: [u8; 4]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// THE ENROLLED KINDS, asked of the roster rather than named: every kind that
/// `workspace_shape()` claims and whose own rows live in the CONTENT pane —
/// which is the stage this Back is reached from, and which is asked through
/// `rows_are_primary()`, the one owner, rather than by naming a shape variant.
/// Today that is Settings; a second such member enrols itself.
fn enrolled() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| {
            k.workspace_shape()
                .is_some_and(|s| !WorkspaceShape::rows_are_primary(s))
        })
        .collect()
}

/// A real Settings card standing in its CONTENT pane, with focus placed by the
/// LIFECYCLE rather than assigned — the same walk a user makes.
fn card_in_content(kind: OverlayKind) -> OverlayState {
    let mut ov = OverlayState::new(kind, crate::settings::visible_names(), Vec::new(), Vec::new());
    ov.set_facet_lens(0);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    journey.toggle_detail();
    journey.card().expect("the card is up").clone()
}

/// The card projected the way `App::sync_view` projects it — every workspace
/// field read off the kind's own owners, never written as a literal.
fn content_view(ov: &OverlayState) -> ViewState {
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = ov.kind.title();
    v.overlay_items = ov.item_strings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    v.overlay_sections = ov.item_sections();
    v.overlay_hint = ov.foot_hint();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = ov.window_rows();
    v
}

/// One swept cell's coordinates, and the key its failures are reported in.
#[derive(Clone, Copy)]
struct Cell {
    w: u32,
    h: u32,
    zoom: f32,
    dpi: f32,
}

impl Cell {
    fn describe(&self, kind: OverlayKind) -> String {
        format!(
            "{} at {}x{} logical, zoom={}, dpi={}",
            kind.as_str(),
            self.w,
            self.h,
            self.zoom,
            self.dpi
        )
    }
}

/// Logical windows the sweep crosses: the app's OWN enforced minimum (derived
/// from the same metrics `app::lifecycle` enforces, so a change to either moves
/// this cell with it), a ladder through the staging threshold, and comfortably
/// wide. The threshold's own value is deliberately never written down — it moves
/// with zoom and with the display face, and a law that pinned it would be
/// testing this machine.
fn windows() -> Vec<(u32, u32)> {
    let min_w = (30.0 * CHAR_WIDTH + 2.0 * TEXT_LEFT.0).ceil() as u32;
    let min_h = (8.0 * LINE_HEIGHT + 2.0 * TEXT_TOP.0).ceil() as u32;
    vec![
        (min_w, min_h),
        (560, 480),
        (700, 620),
        (860, 720),
        (1000, 760),
        (1100, 800),
        (1400, 900),
        (1800, 1000),
    ]
}

/// **THE LAW.** One sentence everywhere, drawn everywhere, on both sides of the
/// staging threshold.
#[test]
fn the_workspaces_back_reads_and_draws_the_same_on_both_sides_of_the_staging_threshold() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the_workspaces_back_reads_and_draws...: no wgpu adapter");
        return;
    }
    let kinds = enrolled();
    assert!(
        !kinds.is_empty(),
        "no kind enrolled — the roster's `RailOverRows` members are this law's subject, and \
         an enrolment that matches nothing sweeps nothing"
    );

    let mut sentences: std::collections::BTreeSet<String> = Default::default();
    let mut backs: std::collections::BTreeSet<&'static str> = Default::default();
    let mut overrun: Vec<String> = Vec::new();
    let (mut staged, mut wide, mut graded) = (0usize, 0usize, 0usize);

    for kind in &kinds {
        let ov = card_in_content(*kind);
        let back = ov
            .detail_back()
            .expect("the content pane must have a Back to be invariant about");
        backs.insert(back.glyph());
        for (lw, lh) in windows() {
            for zoom in [1.0f32, 1.4, 2.0] {
                for dpi in [1.0f32, 2.0] {
                    let cell = Cell {
                        w: lw,
                        h: lh,
                        zoom,
                        dpi,
                    };
                    let (pw, ph) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    let Some((device, queue, mut p)) = headless_dqp(pw as f32, ph as f32) else {
                        return;
                    };
                    p.set_dpi(dpi);
                    p.set_size(pw as f32, ph as f32);
                    let mut v = content_view(&ov);
                    v.zoom = zoom;
                    p.set_view(&v);
                    p.prepare(&device, &queue, pw, ph).unwrap();

                    let what = cell.describe(*kind);
                    let geom = p.workspace_geometry(pw);
                    match p.workspace_is_wide(pw) {
                        true => wide += 1,
                        false => staged += 1,
                    }
                    graded += 1;

                    // PRESENCE, and INVARIANCE OVER THE DRAWN SENTENCE. The
                    // footer's own SHAPED line is the subject of both, because a
                    // stage that plans no footer (the staging regime's other
                    // stage does exactly that) shapes none, and a sentence read
                    // off the card rather than off the frame would agree with
                    // itself at every width for free.
                    let line = p.overlay_hint_line().unwrap_or_else(|| {
                        panic!(
                            "{what}: the content stage shaped no footer at all, so the Back it \
                             teaches is unreadable exactly where it is needed most"
                        )
                    });
                    let drawn = p.panel_buffer.lines[line].text().to_string();
                    sentences.insert(drawn.clone());
                    assert_eq!(
                        drawn,
                        ov.foot_hint(),
                        "{what}: the drawn footer is not the card's own sentence"
                    );
                    assert!(
                        drawn
                            .split(crate::overlay::HINT_SEP)
                            .any(|c| c == format!("{} back", back.glyph())),
                        "{what}: the footer stopped naming the Back. got {drawn:?}"
                    );

                    let (ink_w, top, height) = p
                        .panel_buffer
                        .layout_runs()
                        .find_map(|run| {
                            (run.line_i == line).then_some((
                                run.line_w,
                                geom.text_top + run.line_top,
                                run.line_height,
                            ))
                        })
                        .unwrap_or_else(|| panic!("{what}: the shaped footer has no ink run"));
                    assert!(
                        ink_w > 0.0 && height > 0.0,
                        "{what}: the footer shaped to nothing ({ink_w}x{height})"
                    );
                    let [cx, cy, cw, ch] = p.workspace_regions(pw).card;
                    // WHERE THE FOOTER STARTS is a seating claim and is asserted
                    // outright: a line that begins off its own card is misplaced,
                    // not merely too big for the room.
                    assert!(
                        geom.text_left >= cx && top >= cy,
                        "{what}: the footer's ink box starts at ({:.1},{:.1}), outside the \
                         card ({cx:.1},{cy:.1} {cw:.1}x{ch:.1})",
                        geom.text_left,
                        top
                    );
                    // WHETHER IT FITS is LEDGERED, not asserted — see `OVERRUN`.
                    // A cell that overflows and is not listed fails; a listed
                    // cell that stops overflowing fails too.
                    let fits = geom.text_left + ink_w <= cx + cw + 0.5
                        && top + height <= cy + ch + 0.5;
                    if !fits {
                        overrun.push(what.clone());
                        continue;
                    }

                    // AND IT IS REALLY DRAWN. The sidecar-style facts above are
                    // a state oracle; whether type reached the frame is a
                    // question for the pixels.
                    let (texture, tview) = offscreen(&device, pw, ph);
                    let mut enc =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("awl workspace back footer"),
                        });
                    p.render(&mut enc, &tview).unwrap();
                    queue.submit(Some(enc.finish()));
                    let px = read_pixels(&device, &queue, &texture, pw, ph);
                    let lum: Vec<f32> = px.iter().map(|q| luma(*q)).collect();
                    let half = (height * 0.5 - 1.0).max(1.0);
                    let mid = top + height * 0.5;
                    let y0 = ((mid - half) as i64).max(0);
                    let y1 = ((mid + half) as i64).min(ph as i64 - 2);
                    let x0 = (geom.text_left as i64).max(0);
                    let x1 = ((geom.text_left + ink_w) as i64).min(pw as i64 - 2);
                    let inked = (x0..x1)
                        .filter(|x| {
                            (y0..y1).any(|y| {
                                let i = (y * pw as i64 + x) as usize;
                                (lum[i] - lum[i + pw as usize]).abs() > GLYPH_STEP
                                    || (lum[i] - lum[i + 1]).abs() > GLYPH_STEP
                            })
                        })
                        .count();
                    assert!(
                        inked >= 4,
                        "{what}: only {inked} inked columns in the footer's own band \
                         ({x0}..{x1}) — the Back cell is planned and shaped but not on screen"
                    );
                }
            }
        }
    }

    // THE SWEEP CROSSED THE THRESHOLD, and says so. Without this the invariance
    // above is a statement about one regime wearing the clothes of two.
    assert!(
        staged > 0 && wide > 0,
        "the sweep never crossed the staging threshold (staged {staged}, wide {wide}) across \
         {graded} cells over {:?} — one regime went ungraded, so the width-invariance claim \
         is about nothing",
        kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        sentences.len(),
        1,
        "the footer taught {} different sentences across {graded} cells — the whole point of \
         a Back the action seam derives without a width is that a staged layout and a wide \
         one cannot disagree: {sentences:?}",
        sentences.len()
    );
    assert_eq!(
        backs,
        std::collections::BTreeSet::from([BackKey::Erase.glyph()]),
        "the enrolled roster's content panes must all teach the erase key as their Back — \
         the focus key is the fallback for a live query, and no cell here types one"
    );
    assert_eq!(
        overrun,
        OVERRUN,
        "the set of cells whose footer runs past the card's right edge changed. A cell that \
         is here and not in OVERRUN is a NEW overrun — fix it. A cell in OVERRUN that is no \
         longer here has been fixed — delete its entry rather than leave a ledger that \
         grades nothing."
    );
}

/// **THE CELLS WHERE THE FOOTER IS WIDER THAN ITS CARD** — a ledger of an
/// EXISTING defect this law found and does not own, kept two-sided so it cannot
/// quietly grow or quietly rot.
///
/// All of them are the same corner: the app's own enforced MINIMUM window
/// (`app::lifecycle`) at a zoom above 100%, where the card is ~425 logical px
/// and the rows line shapes past 470. The footer is a single unwrapped line, so
/// it is the first thing a tiny card cannot hold, and this is true of the
/// workspace's footer regardless of which key it names — replacing the focus
/// cell with the erase cell made this line NARROWER, not wider, because `⌫`
/// shapes shorter than `tab`.
///
/// It is a ledger and not an exclusion because the fix is a composition
/// question — elide a cell, wrap the line, or refuse the zoom — that belongs to
/// whoever owns the card's minimum, not to the key the footer names.
const OVERRUN: &[&str] = &[
    "settings at 464x288 logical, zoom=1.4, dpi=1",
    "settings at 464x288 logical, zoom=1.4, dpi=2",
    "settings at 464x288 logical, zoom=2, dpi=1",
    "settings at 464x288 logical, zoom=2, dpi=2",
    "settings at 560x480 logical, zoom=1.4, dpi=1",
    "settings at 560x480 logical, zoom=1.4, dpi=2",
    "settings at 560x480 logical, zoom=2, dpi=1",
    "settings at 560x480 logical, zoom=2, dpi=2",
    "settings at 700x620 logical, zoom=2, dpi=1",
    "settings at 700x620 logical, zoom=2, dpi=2",
];

/// **THE BACK COSTS THE FOOTER NO WIDTH** — which is what makes the ledger above
/// a finding about the card's minimum rather than about this change.
///
/// The rows line already ran to four cells before the erase key replaced the
/// focus key in the fourth, and `OVERRUN` records where four cells were already
/// too many. This shapes BOTH sentences through the same pipeline at the same
/// cell and requires the shipped one to be no wider — so the ledger cannot be
/// read as damage this change did, and a future rewording that does widen the
/// line reddens here instead of silently growing the ledger.
#[test]
fn naming_the_erase_key_shapes_no_wider_than_naming_the_focus_key() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping naming_the_erase_key_shapes_no_wider...: no wgpu adapter");
        return;
    }
    let mut graded = 0usize;
    for kind in enrolled() {
        let ov = card_in_content(kind);
        let shipped = ov.foot_hint();
        let was = shipped.replace(
            &format!("{} back", BackKey::Erase.glyph()),
            &format!("{} back", BackKey::Focus.glyph()),
        );
        assert_ne!(
            shipped, was,
            "{}: the substitution matched nothing, so this measures one sentence twice",
            kind.as_str()
        );
        for (lw, lh) in windows() {
            for zoom in [1.0f32, 1.4, 2.0] {
                let Some((device, queue, mut p)) = headless_dqp(lw as f32, lh as f32) else {
                    return;
                };
                let mut widths = Vec::new();
                for hint in [&shipped, &was] {
                    let mut v = content_view(&ov);
                    v.zoom = zoom;
                    v.overlay_hint = hint.clone();
                    p.set_view(&v);
                    p.prepare(&device, &queue, lw, lh).unwrap();
                    let line = p
                        .overlay_hint_line()
                        .expect("the content stage shapes its footer");
                    widths.push(
                        p.panel_buffer
                            .layout_runs()
                            .find_map(|run| (run.line_i == line).then_some(run.line_w))
                            .expect("the shaped footer has an ink run"),
                    );
                }
                assert!(
                    widths[0] <= widths[1],
                    "{} at {lw}x{lh} zoom={zoom}: `{} back` shapes {:.1}px, wider than the \
                     `{} back` it replaced ({:.1}px) — naming the Back must not cost the \
                     footer width it does not have",
                    kind.as_str(),
                    BackKey::Erase.glyph(),
                    widths[0],
                    BackKey::Focus.glyph(),
                    widths[1]
                );
                graded += 1;
            }
        }
    }
    assert!(graded >= 20, "the comparison must actually run, got {graded}");
}

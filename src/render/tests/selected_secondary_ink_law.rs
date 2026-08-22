//! THE SELECTED ROW'S SECONDARY COLUMN, AND THE GROUND IT IS ACTUALLY DRAWN ON.
//!
//! A row's secondary column — the chord, the value readout, the git cell — flips
//! its ink when the row reads selected, and the flip is chosen for the SELECTION
//! BAND's colour (`theme::selected_row_secondary_ink`). So the ink is only right
//! if the band is *there*. A list style that puts no fill under the column spends
//! an ink picked for a surface the reader never sees, and the failure mode is not
//! subtle: on one `Bars` world that same resolution once landed a range rail's
//! thumb byte-for-byte on the world's own page.
//!
//! `rail_thumb_over_fill` answers "is there a fill under the RAIL" and is the
//! reason that thumb is `muted` again. Nothing answered the same question for the
//! TEXT column, and every existing floor on it grades a different subject: the
//! disabled-row accessory floor grades UNSELECTED rows on a contextual card, and
//! the document-selection floor grades a wash over prose. **This file is the
//! missing floor, and its axis is the one that could hide the defect: SELECTION
//! ITSELF.**
//!
//! # The oracle
//!
//! Every claim is a comparison between two pixel populations the SAME frame drew,
//! so no number here is a claim about a rasterizer:
//!
//! * the GROUND is the modal exact colour over the column's own ink box — which
//!   reads whatever is actually there (plain card, an unselected world's row, a
//!   `Bars` plate, a `Pane` band), never a recomputation of what should be;
//! * the INK is the FURTHEST exact colour from that ground **that still holds real
//!   area** in the same box. ⚠️ The obvious oracle — the most COMMON non-ground
//!   colour — measures antialiasing on this surface, not ink: a chord is a handful
//!   of thin strokes in a box that is mostly ground, so its edge pixels outnumber
//!   its cores and the mode reported a near-ground colour whose ΔE swung by 40
//!   between two window heights of the same world. An area floor plus "furthest"
//!   reads the stroke core, which is what a reader sees.
//!
//! The box itself comes from the owners the draw seats the column from — the
//! measured per-row secondary width, and the diagonal cluster's own accessory end
//! where there is one — so a law and a frame cannot disagree about where to look.
//!
//! # The three arms
//!
//! * **PRESENCE** — the column must have ink at all. Asserted first and counted
//!   separately, because a contrast floor over a treatment is satisfiable by
//!   deleting the treatment, and the whole failure this file names ends with an
//!   ink that IS its own ground.
//! * **FLOOR** — the selected row's ink clears a perceptual distance from whatever
//!   it actually sits on.
//! * **NO FILL, NO TRADE** — the arm that would catch an ink chosen for an absent
//!   band, and the reason it is not a slack tolerance on the arm above. A flip onto
//!   a real band legitimately SPENDS distinctness: measured over the roster, a
//!   `Bars` world's chord on its own plate reads ΔE 32.7 where the same chord on
//!   plain page reads 47.6, and that is correct — the ink is legible on the surface
//!   it is on. So a global "never worse than unselected" slack would have to be
//!   ~15 ΔE wide, and would then tolerate most of the defect. **The arm that holds
//!   asks instead whether there is a band at all**: when the selected column's
//!   measured ground is the SAME surface as an unselected column's, nothing was
//!   traded against, and the flip may not cost a single ΔE. Enrolment in that arm
//!   is measured off pixels and then cross-checked against the roster — every world
//!   whose list style emits no row fill must have landed in it.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::overlay::OverlayKind;

/// ΔE an ink must clear against the ground it is ACTUALLY drawn on to count as
/// present and legible — the same number the disabled-row accessory floor and the
/// range rail's own thumb floor already spend on this identical question, reused
/// rather than invented a second time.
const SECONDARY_INK_FLOOR: f64 = 3.0;

/// Two measured grounds count as the SAME surface below this ΔE — the classic
/// just-noticeable difference. A frame that draws no fill under the selected row
/// lands its two grounds byte-identical; this is the tolerance, not the signal.
const SAME_GROUND: f64 = 2.3;

/// The distinctness a flip may cost when there is no band under the column at all.
/// One ΔE of antialiasing slack and nothing more: with nothing traded against,
/// "the same ink, or a more distinct one" is the whole claim.
const NO_TRADE_SLACK: f64 = 1.0;

/// A card with a real secondary column on every row and a wide spread of label
/// widths — the shape a chord column is drawn on, and the one place the flip has a
/// subject.
fn chord_view(n: usize, selected: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Command.title();
    v.overlay_items = (0..n)
        .map(|i| {
            if i % 3 == 0 {
                format!("short {i}")
            } else {
                format!("a considerably longer candidate label {i}")
            }
        })
        .collect();
    v.overlay_bindings = (0..n).map(|_| "C-x C-s".to_string()).collect();
    v.overlay_selected = selected.min(n.saturating_sub(1));
    v.overlay_hint = "type to filter".into();
    v
}

/// The `(left, right)` ink box of display row `display`'s secondary column, off
/// the SAME owners the emitter seats it from: the measured per-row width, the
/// diagonal cluster's own accessory end where a cluster exists, and the text
/// column's far edge where one does not.
fn accessory_box(
    p: &TextPipeline,
    geom: &crate::render::chrome::OverlayGeom,
    display: usize,
) -> Option<(f32, f32)> {
    let w = *p.overlay_row_secondary_px(geom).get(&display)?;
    if w <= 0.0 {
        return None;
    }
    match p.diagonal_cluster_probe() {
        Some(probe) => {
            let (lo, hi) = probe.accessory_span(display);
            // The measured box is the rail's whole accessory lane; this row's own
            // ink hangs on its outer end and is `w` wide.
            let flow = crate::render::chrome::diagonal::accessory_flow(p);
            let anchor = if flow.sign() > 0.0 { lo } else { hi };
            Some(flow.span(anchor, w))
        }
        None => {
            let anchor = geom.text_left + geom.text_w;
            Some(crate::render::rowlayout::ColumnFlow::Leftward.span(anchor, w))
        }
    }
}

/// One row's reading: the ground its secondary column sits on, the ink drawn
/// there, and the perceptual distance between them. `None` when the row drew no
/// secondary column at all — which every caller counts, so "nothing was graded"
/// cannot pass for a sweep.
struct ColumnReading {
    ground: [u8; 4],
    ink: Option<[u8; 4]>,
    de: f64,
}

fn read_column(
    p: &TextPipeline,
    geom: &crate::render::chrome::OverlayGeom,
    plan: &crate::render::plan::OverlayRowPlan,
    pixels: &[[u8; 4]],
    cw: u32,
    display: usize,
) -> Option<ColumnReading> {
    let row = plan.rows().iter().find(|r| r.display == display)?;
    row.item?;
    let (l, r) = accessory_box(p, geom, display)?;
    // A pixel of slack at each edge for the glyph's own antialiasing, and the
    // row's vertical slot inset by a pixel so an adjacent row's shadow bleed —
    // a real rendering fact, not this defect — stays out of the population.
    let region = pixeldiff::Region::new(l - 1.0, row.top + 1.0, (r - l) + 2.0, row.height - 2.0);
    if region.w <= 0 || region.h <= 0 {
        return None;
    }
    let ground = region_mode(pixels, cw as i64, region);
    let ink = extreme_ink(pixels, cw as i64, region, ground);
    Some(ColumnReading {
        ground,
        de: ink.map_or(0.0, |i| pixeldiff::delta_e(i, ground)),
        ink,
    })
}

/// The least area one exact colour must hold to count as this column's INK rather
/// than as an antialiased edge. A stroke core covers many more pixels than this at
/// every scale the sweep runs; a fleck covers fewer.
const INK_AREA_FLOOR: usize = 4;

/// The exact colour furthest from `ground` that still holds [`INK_AREA_FLOOR`]
/// pixels of area — see the header on why this is not the modal non-ground colour.
fn extreme_ink(
    pixels: &[[u8; 4]],
    width: i64,
    r: pixeldiff::Region,
    ground: [u8; 4],
) -> Option<[u8; 4]> {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter(|&(c, n)| n >= INK_AREA_FLOOR && c != ground)
        .max_by(|a, b| {
            pixeldiff::delta_e(a.0, ground)
                .partial_cmp(&pixeldiff::delta_e(b.0, ground))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(c, _)| c)
}

/// The modal exact colour over a region — the surface actually drawn there, since
/// glyph ink is a minority of a text row's area.
fn region_mode(pixels: &[[u8; 4]], width: i64, r: pixeldiff::Region) -> [u8; 4] {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}

/// One world's reading at one configuration: the SELECTED row's column against
/// its own ground, and an unselected row's column against its own.
struct WorldReading {
    style: String,
    selected: ColumnReading,
    plain: ColumnReading,
}

/// Render one card and read the selected row's column and a distant unselected
/// row's. The two graded rows are three apart, so neither is the other's
/// neighbour: adjacent-row shadow bleed is real and is not this subject.
fn probe_world(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cw: u32,
    ch: u32,
) -> Option<WorldReading> {
    const SELECTED: usize = 3;
    const PLAIN: usize = 7;
    p.set_view(&chord_view(12, SELECTED));
    p.prepare(device, queue, cw, ch).ok()?;
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    if !p.overlay_right_column_shown() {
        return None;
    }
    let pixels = pixeldiff::render_frame(p, device, queue, cw, ch);
    let selected = read_column(p, &geom, &plan, &pixels, cw, SELECTED)?;
    let plain = read_column(p, &geom, &plan, &pixels, cw, PLAIN)?;
    Some(WorldReading {
        style: format!("{:?}", crate::render::effective_list_style()),
        selected,
        plain,
    })
}

/// Every world whose list style puts NO fill under a selected row, read off the
/// roster's own composition answer rather than named. These are the worlds the
/// no-trade arm must enrol, and the cross-check that the pixel-measured enrolment
/// is not a property of the GPU.
fn no_row_fill_worlds() -> Vec<&'static str> {
    theme::THEMES
        .iter()
        .filter(|t| match t.render_caps.list_style {
            // A diagonal selection is the bright mark alone; the composition
            // deliberately has no row-fill fallback.
            theme::ListStyle::Diagonal(_) => true,
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Ruled(_) => false,
        })
        .map(|t| t.name)
        .collect()
}

/// **THE LAW.** Over the WHOLE roster — every list style enrols, because the
/// question ("is the ink the flip chose right for the surface under it") is asked
/// of every world by the same owner — at 1×/2× DPI and both menu-bar arms.
#[test]
fn a_selected_rows_secondary_column_is_never_less_visible_than_an_unselected_ones() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the selected-secondary ink floor: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut present = 0usize;
    let mut styles = std::collections::BTreeSet::new();
    let mut no_fill_enrolled: std::collections::BTreeSet<&'static str> = Default::default();
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for world in theme::world_names() {
                let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
                p.sync_theme();
                let Some(r) = probe_world(&mut p, &device, &queue, cw, ch) else {
                    continue;
                };
                let ctx = format!("{world} [{}] bar={bar} dpi={dpi}", r.style);
                styles.insert(r.style.split('(').next().unwrap_or("?").to_string());

                // PRESENCE, asserted before any distance is believed: a floor over
                // an ink is satisfied by the ink not being drawn.
                for (what, c) in [("selected", &r.selected), ("unselected", &r.plain)] {
                    assert!(
                        c.ink.is_some(),
                        "{ctx}: the {what} row's secondary column drew NO ink over its own \
                         box (ground {:?}) — a contrast floor here would be satisfied by \
                         the chord having vanished",
                        c.ground
                    );
                }
                present += 1;

                // THE FLOOR: the selected row's column is legible against whatever
                // it actually sits on.
                assert!(
                    r.selected.de >= SECONDARY_INK_FLOOR,
                    "{ctx}: the SELECTED row's secondary ink {:?} clears only ΔE {:.2} \
                     against the ground it is drawn on ({:?}), under the {SECONDARY_INK_FLOOR} \
                     floor — an ink resolved for a selection band that is not under this \
                     column",
                    r.selected.ink,
                    r.selected.de,
                    r.selected.ground
                );

                // NO FILL, NO TRADE. Enrolment is the frame's own answer: the two
                // rows' grounds are the same surface, so nothing was traded against.
                let ground_gap = pixeldiff::delta_e(r.selected.ground, r.plain.ground);
                if ground_gap < SAME_GROUND {
                    no_fill_enrolled.insert(world);
                    assert!(
                        r.selected.de >= r.plain.de - NO_TRADE_SLACK,
                        "{ctx}: the selected row's secondary column sits on the SAME ground \
                         as an unselected row's ({:?} vs {:?}, ΔE {ground_gap:.2}) — so there \
                         is no selection fill under it to stay legible against — and yet \
                         selecting the row made that column HARDER to see: ΔE {:.2} where the \
                         unselected row reads ΔE {:.2}. The flip is spending an ink chosen for \
                         a band that is not there.",
                        r.selected.ground,
                        r.plain.ground,
                        r.selected.de,
                        r.plain.de
                    );
                }
                graded += 1;
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert_eq!(
        graded, present,
        "every graded cell cleared the presence arm"
    );
    assert!(
        graded > 60,
        "the roster sweep graded only {graded} cells — it is not reading the roster it \
         thinks it is"
    );
    // ENROLMENT, named rather than assumed: all four list families must have been
    // reached, or this law is about a subset of the product.
    assert_eq!(
        styles.len(),
        4,
        "the sweep reached list styles {styles:?} — the flip's owner is shared by all \
         four families, so a sweep that misses one cannot see a family-specific defect"
    );
    // ⚠️ THE NO-TRADE ARM'S ENROLMENT IS ITSELF A HYPOTHESIS, so it is checked
    // against the ROSTER's own declared composition rather than trusted: a world
    // that draws no row fill must have been measured as drawing none. A pixel-
    // derived enrolment that quietly stopped matching would otherwise leave the
    // sharp arm sweeping nothing while the law still passed.
    for world in no_row_fill_worlds() {
        assert!(
            no_fill_enrolled.contains(world),
            "{world}'s list style emits no row fill, and yet its selected row's secondary \
             column was measured on a DIFFERENT ground from an unselected one — so the \
             no-trade arm did not enrol it. Enrolled: {no_fill_enrolled:?}"
        );
    }
    assert!(
        no_fill_enrolled.len() >= 2,
        "the no-trade arm enrolled {no_fill_enrolled:?} — under two worlds it cannot tell \
         a shared defect from one world's accident"
    );
}

/// MEASUREMENT REPORT, not a law — the roster table the relative arm's slack was
/// calibrated from, worst margin first. `#[ignore]`d by default:
/// `cargo test --bin awl selected_secondary_ink_report -- --ignored --nocapture`
#[test]
#[ignore]
fn selected_secondary_ink_report() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping selected_secondary_ink_report: no wgpu adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    let (cw, ch) = (1200u32, 800u32);
    p.set_size(cw as f32, ch as f32);
    let mut rows: Vec<(f64, String)> = Vec::new();
    for world in theme::world_names() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        let flip = crate::render::chrome::overlay_selected_secondary_ink();
        let Some(r) = probe_world(&mut p, &device, &queue, cw, ch) else {
            rows.push((f64::MAX, format!("{world:>10} — no secondary column drawn")));
            continue;
        };
        let margin = r.selected.de - r.plain.de;
        rows.push((
            margin,
            format!(
                "{world:>10} {:<18} sel ΔE {:6.2} (ink {:?} on {:?})  plain ΔE {:6.2} \
                 (ink {:?} on {:?})  margin {margin:+7.2}  flip token {:?}",
                r.style,
                r.selected.de,
                r.selected.ink,
                r.selected.ground,
                r.plain.de,
                r.plain.ink,
                r.plain.ground,
                flip,
            ),
        ));
    }
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    eprintln!("\n=== selected vs unselected secondary-column ink, {cw}x{ch} ===");
    for (_, line) in &rows {
        eprintln!("{line}");
    }
}

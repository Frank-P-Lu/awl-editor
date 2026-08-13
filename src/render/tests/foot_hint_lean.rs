//! THE FOOT HINT HANGS ON THE SPINE THE ROWS HANG ON, INSTEAD OF
//! HOLDING THE CARD'S LEFT EDGE UNDER A LEANING LIST.
//!
//! Every line of a card is one rich-text run in one `panel_buffer`, so the hint had
//! no x of its own: it inherited the buffer's left edge while every row above it
//! raked with the spine. The fix gives the FOOT BAND — the hint, its separator, and
//! the tips list on the one kind that carries one — the same independent x the
//! leaning ROWS already had: `overlay_upload_text` emits one `TextArea` per band over
//! the single buffer, and this band simply had never been asked for a left of its
//! own. See `chrome/diagonal/foot.rs` for the shape and for the terminus question.
//!
//! # What these laws measure, and how each avoids being satisfied by a collapse
//!
//! * **The lean is READ from the drawn composition.** The headline law compares the
//!   foot's own displacement from row 0 against the displacement the DRAWN SPINE
//!   takes over the same vertical distance, computed from two rows' anchors off the
//!   measured rail. It sweeps a CRAMPED card as well as a roomy one, because
//!   `TRAVEL_MAX_BAND_FRACTION` makes the spine give up rake there and the authored
//!   `ROW_STEP` no longer equals the drawn step — a second reading of the constant is
//!   green on every roomy card and red only there.
//! * **The offset is not measured against nothing.** "The hint is offset from the
//!   card edge" gets HAPPIER as the hint disappears, so every reading below carries
//!   a presence floor: the hint's shaped ink width against a floor under the
//!   roster's tightest real value, and — in the pixel law — real stepped glyph
//!   structure counted inside the band the owner claims.
//! * **The enrolment is derived from the roster**, `ListStyle::Diagonal(_)` on each
//!   world's own `render_caps`, and both spine DIRECTIONS are required to appear, so
//!   the mirror half of the law cannot lose its subject to a world changing style.
//! * **Both menu-bar states run.** `MENU_BAR_ON` is off on macOS and on everywhere
//!   else, taking `menubar_reserve` off every card's height budget — which is how a
//!   picker came to draw zero candidate rows on Linux and nowhere else. A law about
//!   where a card's foot sits is a law about a card with rows in it, so each cell
//!   asserts it got some.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// THE PRESENCE FLOOR on the hint's own shaped ink, in physical px at 1×. The
/// roster's real values sit around 340 px for the standard
/// `"type to filter   ↵ keep   esc revert"` line at the picker's chrome size; this
/// floor is set well under the tightest of them so it can only be tripped by the
/// SUBJECT collapsing, never by a face being narrow. Every offset assertion in this
/// file is paired with it: an offset from the card's edge is trivially satisfied by a
/// hint that is not there.
// Narrow cards deliberately yield the optional `type to filter` lead. The
// shortest remaining real action sentence (`↵ apply`) is about 75 logical px
// in the shipped faces, so 60 still proves non-vacuous ink without rejecting
// the authored narrow fallback.
const HINT_INK_FLOOR_PX: f32 = 60.0;

/// A local luma step (of 255) that a GLYPH EDGE produces and a world's ambient
/// ground does not. The pixel law counts columns carrying one inside the hint's own
/// band; a `Diagonal` world backs its rows with nothing, so the ground under the hint
/// is the world's own (frosted, and on two worlds dithered) page — hence a threshold
/// well above a dither level and well under an ink/page contrast.
const GLYPH_STEP: f32 = 24.0;

fn luma(p: [u8; 4]) -> f32 {
    0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
}

/// Every world whose composition LEANS, from the roster's own list style rather than
/// by name — so a world that changes style changes what this sweeps.
fn leaning_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.list_style, theme::ListStyle::Diagonal(_)))
        .map(|t| t.name)
        .collect()
}

fn upright_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| !matches!(t.render_caps.list_style, theme::ListStyle::Diagonal(_)))
        .map(|t| t.name)
        .collect()
}

/// A summoned picker with a foot hint and enough rows for the spine to rake across.
/// `rows` is the corpus size and `label` the row text — a one-glyph label is how the
/// card is made to HUG, which is what makes the composition give up rake.
fn picker(kind: OverlayKind, rows: usize, label: &str) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = kind == OverlayKind::Theme;
    v.overlay_title = kind.title();
    v.overlay_hint = kind.hint();
    v.overlay_items = (0..rows).map(|i| format!("{label}{i}")).collect();
    // THE WINDOW IS THE CANVAS'S, not a per-kind row cap. `OverlayKind::window_rows`
    // is a FIXED cap (12 for most kinds) and on an ordinary canvas that cap — not the
    // pixel budget — is what binds, so every kind drew the same 12 rows and no cell in
    // the sweep could make the spine's travel exceed its responsive bound. Handing the
    // corpus size makes the HEIGHT budget the binding constraint, exactly as the
    // capture path does through the kind's own owner.
    v.overlay_window_rows = rows;
    // A MIDDLE row, deliberately: the selected row steps its whole cluster outward,
    // so a law comparing the foot against row 0's or the last row's anchor would be
    // reading a shifted one at either end.
    v.overlay_selected = rows / 2;
    v.overlay_sections = vec![String::new(); rows];
    v
}

/// The kinds that carry a foot hint at all. `Spell` is the contextual popup, whose
/// geometry owner sets `hint_rows = 0` outright, so it has no subject here — and the
/// exclusion is CHECKED rather than trusted: every kind this DOES return must shape a
/// hint, asserted in each sweep below.
fn hinted_kinds() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .into_iter()
        .filter(|k| *k != OverlayKind::Spell && !k.hint().is_empty())
        .collect()
}

/// One prepared frame's foot placement beside the composition it must agree with.
struct Graded {
    steps: f32,
    /// The foot's own displacement from row 0's anchor, and the displacement the
    /// DRAWN spine takes over the same vertical distance.
    foot_dx: f32,
    spine_dx: f32,
    ink_w: f32,
    left: f32,
    clamped: bool,
    text_left: f32,
    text_w: f32,
    /// The WHOLE band's widest shaped ink — the hint's own, or a tips line beneath
    /// it on the one kind that carries them. On some world × kind pairs this already
    /// EXCEEDS the text column and is clipped by the emitter; that is the state
    /// before this item and the clamp's job is to leave it exactly there.
    band_w: f32,
    rows: usize,
}

fn grade(p: &TextPipeline, width: u32) -> Option<Graded> {
    let geom = p.overlay_geometry(width);
    let plan = p.overlay_row_plan(&geom);
    let foot = p.overlay_foot_placement(&geom, &plan)?;
    let probe = p.diagonal_cluster_probe()?;
    let rows = plan.rows().len();
    let last = rows.saturating_sub(1);
    // THE SPINE'S OWN SLOPE, from the two ends of the DRAWN rail — never
    // `DiagonalComposition::row_step`, which is the authored constant before the
    // responsive yield. `spine_x` carries no selected shift, so neither end can be
    // contaminated by which row is selected.
    let per_row = if last > 0 {
        (probe.spine_x(last) - probe.spine_x(0)) / last as f32
    } else {
        0.0
    };
    Some(Graded {
        steps: foot.steps,
        foot_dx: foot.anchor - probe.label_anchor(0),
        spine_dx: per_row * foot.steps,
        ink_w: foot.ink_w,
        left: foot.left,
        clamped: foot.clamped,
        text_left: geom.text_left,
        text_w: geom.text_w,
        band_w: p
            .overlay_footer_content_px(&geom, plan.content_rows())
            .max(foot.ink_w),
        rows,
    })
}

/// GRADE ONE CELL of the headline sweep, returning the DRAWN rake it measured in
/// LOGICAL px per row — the space the authored `ROW_STEP` is written in, so the two are
/// comparable at either device scale, and so the sweep can prove it contains a geometry
/// where they differ.
fn grade_on_the_spines_line(g: &Graded, dpi: f32, ctx: &str) -> f32 {
    // PRESENCE, before any claim about where the band sits: rows to rake (the Linux
    // menu-bar reserve once left a card with none) and a hint with real ink in it.
    assert!(
        g.rows >= 3,
        "{ctx}: the card drew only {} rows — the fixture, or the height budget under \
         this menu-bar state, is what failed",
        g.rows
    );
    assert!(
        g.ink_w >= HINT_INK_FLOOR_PX * dpi,
        "{ctx}: the hint's shaped ink is {}px, under the {}px presence floor — every \
         offset assertion here is satisfied by a hint that is not there",
        g.ink_w,
        HINT_INK_FLOOR_PX * dpi
    );
    // THE CLAIM: the foot's displacement IS the drawn spine's over the same vertical
    // distance.
    assert!(
        (g.foot_dx - g.spine_dx).abs() <= 0.02 * dpi,
        "{ctx}: the foot band is {} px off row 0's anchor, but the DRAWN spine moves {} \
         px over the same {} row pitches — the band is not on the composition's own line",
        g.foot_dx,
        g.spine_dx,
        g.steps
    );
    // …AT THE TERMINUS OR PAST IT, whichever the shipped answer to the open design
    // question is. The claim READS the switch rather than restating one of its branches,
    // so flipping it stays the one-word change its own doc promises.
    let last = (g.rows - 1) as f32;
    if crate::render::chrome::diagonal::FOOT_CONTINUES_THE_LEAN {
        assert!(
            g.steps > last,
            "{ctx}: the band's own row is {} steps down and the last candidate row is \
             {last} — the shipped answer CONTINUES the lean past the spine's terminus, \
             so the band must sit below the list",
            g.steps
        );
    } else {
        assert!(
            (g.steps - last).abs() < 1e-3,
            "{ctx}: the shipped answer seats the band AT the spine's terminus (step \
             {last}), but it is at {}",
            g.steps
        );
    }
    (g.spine_dx / g.steps).abs() / dpi
}

/// THE HEADLINE LAW — the foot band sits on the line the DRAWN spine would have
/// drawn there, at every device scale, in both menu-bar states, over every kind that
/// carries a hint, and on a CRAMPED card as well as a roomy one.
///
/// The cramped cell is the one that distinguishes the drawn step from the authored
/// constant: `TRAVEL_MAX_BAND_FRACTION` bounds the spine's whole travel by a share of
/// the card's side territory, so a hugging card rakes at a fraction of `ROW_STEP` and
/// a foot that re-derived the constant would lean further than the spine beside it.
#[test]
fn the_foot_band_hangs_on_the_line_the_drawn_spine_takes() {
    let _g = crate::testlock::serial();
    let worlds = leaning_worlds();
    assert!(
        !worlds.is_empty(),
        "no world leans — this law has no subject; the enrolment is \
         ListStyle::Diagonal off the roster, so a roster with none of them makes \
         every assertion below vacuous"
    );
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    let mut cramped_graded = 0usize;
    let mut tightest_rake = f32::MAX;
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            // ROOMY vs CRAMPED is a fixture pair, and the second one's job is to make
            // the spine give up rake: `TRAVEL_MAX_BAND_FRACTION` bounds the whole
            // travel by a share of the card's SIDE TERRITORY, so a narrow card with
            // MANY rows is what drives the drawn step under the authored 7.0 — a
            // narrow card alone does not (13 rows on a 520px canvas still afford the
            // constant outright, which is how the first draft of this law measured
            // 7.0 in every cell).
            for (shape, lw, rows, label) in [
                ("roomy", 1200.0f32, 14usize, "candidate row "),
                ("cramped", 420.0, 30, "a"),
            ] {
                let (cw, ch) = ((lw * dpi) as u32, (760.0 * dpi) as u32);
                let Some((device, queue, mut p)) = headless_dqp(cw as f32, ch as f32) else {
                    eprintln!("skipping the_foot_band_hangs_on_...: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_menu_bar);
                    return;
                };
                p.set_dpi(dpi);
                for world in &worlds {
                    crate::theme::set_active_by_name(world).unwrap();
                    p.sync_theme();
                    for kind in hinted_kinds() {
                        let v = picker(kind, rows, label);
                        p.set_view(&v);
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let ctx = format!("{world}/{kind:?} dpi={dpi} {shape} menu_bar={bar}");
                        let g = grade(&p, cw).unwrap_or_else(|| {
                            panic!(
                                "{ctx}: a leaning world with a hinted picker open drew no \
                                 foot placement at all"
                            )
                        });
                        let rake = grade_on_the_spines_line(&g, dpi, &ctx);
                        if shape == "cramped" {
                            eprintln!("MEASURED {ctx}: rows={} rake={rake} logical px/row", g.rows);
                            tightest_rake = tightest_rake.min(rake);
                            cramped_graded += 1;
                        }
                        graded += 1;
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    crate::theme::set_active(entry);
    assert!(graded >= 40, "the sweep must actually run, got {graded}");
    assert!(
        cramped_graded >= 10,
        "the cramped cell must be reached, got {cramped_graded}"
    );
    // THE SWEEP CONTAINS A GEOMETRY WHERE THE DRAWN STEP AND THE AUTHORED CONSTANT
    // DIFFER — without one, every cell above compares the two where they are equal
    // and the "read, never re-authored" claim is untested. `ROW_STEP` is 7.0 logical.
    eprintln!("MEASURED tightest drawn rake across the sweep: {tightest_rake} px/row");
    assert!(
        tightest_rake < 6.5,
        "no cell in the sweep made the spine give up rake (tightest {tightest_rake} \
         px/row against an authored 7.0), so nothing here distinguishes the drawn \
         step from the constant"
    );
}

/// THE FOOT BAND MIRRORS WITH THE CLUSTER — its ink hangs on the same end of the
/// spine a row's NAME does, so on an ascending world it ends on the spine instead of
/// crossing it.
///
/// Both directions must be present in the enrolled roster, asserted rather than
/// assumed: with only one of them the claim is true of one world and says nothing
/// about the mirror.
#[test]
fn the_foot_band_hangs_on_the_same_spine_end_a_row_name_does() {
    let _g = crate::testlock::serial();
    let worlds = leaning_worlds();
    let directions: Vec<theme::DiagonalDirection> = crate::theme::THEMES
        .iter()
        .filter_map(|t| match t.render_caps.list_style {
            theme::ListStyle::Diagonal(s) => Some(s.direction),
            _ => None,
        })
        .collect();
    for want in [
        theme::DiagonalDirection::Descending,
        theme::DiagonalDirection::Ascending,
    ] {
        assert!(
            directions.contains(&want),
            "no enrolled world spines {want:?} — the mirror half of this law has no \
             subject, and a claim true of one orientation says nothing about its \
             mirror (enrolled: {directions:?})"
        );
    }
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_foot_band_hangs_on_the_same_spine_end...: no wgpu adapter");
        return;
    };
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    for world in &worlds {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for kind in hinted_kinds() {
            let v = picker(kind, 14, "candidate row ");
            p.set_view(&v);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            let geom = p.overlay_geometry(1200);
            let plan = p.overlay_row_plan(&geom);
            let foot = p
                .overlay_foot_placement(&geom, &plan)
                .expect("a leaning world with a hint open has a foot placement");
            let probe = p
                .diagonal_cluster_probe()
                .expect("a leaning world has a rail");
            let flow = probe.label_flow();
            // A ROW'S OWN INK, on the side the composition puts it: measured from the
            // rail, at an UNSELECTED row so the outward step cannot enter it.
            let row = 1usize;
            let row_w = 40.0f32;
            let row_mid = probe.label_origin(row, row_w) + row_w * 0.5;
            let row_side = (row_mid - probe.spine_x(row)).signum();
            // The spine's own abscissa at the foot band's row, recovered from the
            // anchor by the connector the ROWS are offset by — so the two sides are
            // measured against the same line without this law re-deriving either.
            let connector = probe.label_anchor(row) - probe.spine_x(row);
            let foot_mid = foot.left + foot.ink_w * 0.5;
            let foot_side = (foot_mid - (foot.anchor - connector)).signum();
            assert!(
                foot.ink_w >= HINT_INK_FLOOR_PX,
                "{world}/{kind:?}: the hint's ink is {}px, under the presence floor — \
                 a side claim about ink that is not there is free",
                foot.ink_w
            );
            assert_eq!(
                foot_side, row_side,
                "{world}/{kind:?}: the rows' ink sits on the {flow:?} side of the \
                 spine and the foot band's on the other — the band crossed the spine \
                 instead of mirroring with the cluster"
            );
            graded += 1;
        }
    }
    crate::theme::set_active(entry);
    assert!(graded >= 8, "the sweep must actually run, got {graded}");
}

/// THE FOOT BAND'S INK NEVER LEAVES THE TEXT COLUMN — the emitter clips to it, so an
/// offset that outran the column would eat glyphs rather than move them. Swept down
/// to widths where the clamp genuinely binds, which is asserted: a clamp no cell
/// reaches is a clamp no law has run.
#[test]
fn the_foot_bands_ink_stays_inside_the_text_column_at_every_width() {
    let _g = crate::testlock::serial();
    let worlds = leaning_worlds();
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    let mut clamped = 0usize;
    for lw in [1200u32, 900, 760, 640, 560, 480, 420] {
        let Some((device, queue, mut p)) = headless_dqp(lw as f32, 800.0) else {
            eprintln!("skipping the_foot_bands_ink_stays_inside...: no wgpu adapter");
            return;
        };
        for world in &worlds {
            crate::theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for kind in hinted_kinds() {
                let v = picker(kind, 14, "candidate row ");
                p.set_view(&v);
                p.prepare(&device, &queue, lw, 800).unwrap();
                let Some(g) = grade(&p, lw) else { continue };
                let ctx = format!("{world}/{kind:?} width={lw}");
                assert!(
                    g.ink_w >= HINT_INK_FLOOR_PX.min(g.text_w),
                    "{ctx}: the hint's ink is {}px against a {}px column — under the \
                     presence floor with room to spare",
                    g.ink_w,
                    g.text_w
                );
                assert!(
                    g.left >= g.text_left - 0.01,
                    "{ctx}: the band is seated at {} — LEFT of the text column's own \
                     edge {}",
                    g.left,
                    g.text_left
                );
                // THE CLAIM: the lean never pushes the band past the column the
                // emitter clips to. Expressed against the column's own room for THIS
                // band rather than against the column's edge outright, because a few
                // world × kind pairs already shape a hint wider than their card's
                // text column and clip it — the state before this item, which the
                // clamp must leave exactly where it found it (the band keeps the
                // card's edge) rather than improve or worsen.
                let room = (g.text_w - g.band_w).max(0.0);
                assert!(
                    g.left <= g.text_left + room + 0.01,
                    "{ctx}: the band is seated {} px into a column with {room} px of \
                     room for its {} px of ink ({} px wide column) — the lean pushed \
                     it into the emitter's clip",
                    g.left - g.text_left,
                    g.band_w,
                    g.text_w
                );
                if g.clamped {
                    clamped += 1;
                }
                graded += 1;
            }
        }
    }
    crate::theme::set_active(entry);
    assert!(graded >= 20, "the sweep must actually run, got {graded}");
    assert!(
        clamped >= 1,
        "no width in the sweep drove the band back to the card's edge, so the column \
         clamp — the one path that can silently eat the lean — is untested"
    );
}

/// EVERY UPRIGHT WORLD SEATS ITS FOOT BAND EXACTLY WHERE IT ALWAYS DID, bit for bit:
/// the feature is inert off the enrolled roster, which is the structural half of the
/// byte-identity claim (the measured half is a gallery sweep of the whole roster's
/// PNGs and sidecars).
#[test]
fn an_upright_world_seats_its_foot_band_at_the_cards_own_text_edge() {
    let _g = crate::testlock::serial();
    let worlds = upright_worlds();
    assert!(
        worlds.len() > 10,
        "only {} worlds are upright — this law's subject is the rest of the roster",
        worlds.len()
    );
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping an_upright_world_seats_its_foot_band...: no wgpu adapter");
        return;
    };
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for world in &worlds {
            crate::theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for kind in hinted_kinds() {
                let v = picker(kind, 14, "candidate row ");
                p.set_view(&v);
                p.prepare(&device, &queue, 1200, 800).unwrap();
                let geom = p.overlay_geometry(1200);
                let plan = p.overlay_row_plan(&geom);
                let ctx = format!("{world}/{kind:?} menu_bar={bar}");
                // PRESENCE: this frame really did shape a hint, so "inert" is a
                // statement about a hint that exists.
                assert!(
                    p.overlay_hint_line().is_some(),
                    "{ctx}: no hint was shaped, so the inertness below is vacuous"
                );
                assert!(
                    p.overlay_foot_placement(&geom, &plan).is_none(),
                    "{ctx}: an upright world resolved a foot placement — the lean must \
                     be reached only through a drawn spine"
                );
                assert_eq!(
                    p.overlay_foot_left(&geom, &plan).to_bits(),
                    geom.text_left.to_bits(),
                    "{ctx}: the band's seat is not the card's own text edge, bit for bit"
                );
                graded += 1;
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    crate::theme::set_active(entry);
    assert!(graded >= 100, "the sweep must actually run, got {graded}");
}

/// THE PIXEL LAW — the hint's GLYPHS are where the owner says they are.
///
/// Real stepped glyph structure is counted inside the band the placement claims, and
/// the strip the hint VACATED — from the card's own text edge to the band's new left
/// — must carry none. The second half is what a placement reverted to `text_left`
/// fails; the first is what a placement that moved the glyphs off the card, or lost
/// them to the emitter's clip, fails. Together they are the drawn↔owner agreement
/// this band's non-interactive nature leaves no hit-test to provide.
#[test]
fn the_hints_glyphs_are_drawn_in_the_band_the_placement_claims() {
    let _g = crate::testlock::serial();
    let worlds = leaning_worlds();
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        let (w, h) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping the_hints_glyphs_are_drawn_in_the_band...: no wgpu adapter");
            return;
        };
        p.set_dpi(dpi);
        for world in &worlds {
            crate::theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            let v = picker(OverlayKind::Theme, 14, "candidate row ");
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let geom = p.overlay_geometry(w);
            let plan = p.overlay_row_plan(&geom);
            let foot = p
                .overlay_foot_placement(&geom, &plan)
                .expect("a leaning world with a hint open has a foot placement");
            let (texture, tview) = offscreen(&device, w, h);
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("awl foot hint encoder"),
            });
            p.render(&mut enc, &tview).unwrap();
            queue.submit(Some(enc.finish()));
            let px = read_pixels(&device, &queue, &texture, w, h);
            let lum: Vec<f32> = px.iter().map(|q| luma(*q)).collect();
            // The hint's own band, a hair inside its line box at both ends so a
            // neighbouring line's ink cannot enter the count.
            let half = (p.overlay_hint_h() * 0.5 - 1.0).max(1.0);
            let y0 = ((foot.center_y - half) as i64).max(0);
            let y1 = ((foot.center_y + half) as i64).min(h as i64 - 2);
            // Columns carrying a glyph edge: a strong local step inside the band.
            let inked = |x: i64| -> bool {
                (y0..y1).any(|y| {
                    let i = (y * w as i64 + x) as usize;
                    (lum[i] - lum[i + w as usize]).abs() > GLYPH_STEP
                        || (lum[i] - lum[i + 1]).abs() > GLYPH_STEP
                })
            };
            let count = |a: f32, b: f32| -> usize {
                let (a, b) = (a.max(0.0) as i64, (b as i64).min(w as i64 - 2));
                (a..b).filter(|x| inked(*x)).count()
            };
            let ctx = format!("{world} dpi={dpi}");
            let in_band = count(foot.left, foot.left + foot.ink_w);
            // PRESENCE: the hint's glyphs really are in the band the owner claims.
            assert!(
                in_band as f32 >= 0.25 * foot.ink_w,
                "{ctx}: only {in_band} of the {} columns the placement claims \
                 ([{}, {}]) carry glyph structure — the band is where the owner says \
                 the hint is, and the hint is not in it",
                foot.ink_w,
                foot.left,
                foot.left + foot.ink_w
            );
            // …AND THE VACATED STRIP IS EMPTY. Two glyph widths of slack at the seam
            // so the hint's own first stem cannot be counted against it.
            let vacated = foot.left - geom.text_left;
            if vacated > 24.0 * dpi {
                let stray = count(geom.text_left, foot.left - 8.0 * dpi);
                assert!(
                    stray as f32 <= 0.06 * vacated,
                    "{ctx}: {stray} of the {vacated} columns between the card's text \
                     edge and the band's new left still carry glyph structure — the \
                     hint did not move, or it is drawn twice"
                );
                graded += 1;
            }
        }
        p.set_dpi(1.0);
    }
    crate::theme::set_active(entry);
    assert!(
        graded >= 2,
        "no cell offset the band far enough to check the strip it vacated, so the \
         half of this law that fails on a reverted placement never ran ({graded})"
    );
}

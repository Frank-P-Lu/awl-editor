//! **A WORKSPACE MAY NOT SHOW BOTH REGIONS AND KEEP ONLY ONE OF THEM WORKING.**
//!
//! A `RailOverRows` workspace's content pane draws rows, and a row is a NAME and
//! its ACCESSORY — the value readout, and for a Range row the rail you drag.
//! Going two-column adds the category rail beside that pane, which SHRINKS it;
//! the accessory column is the first thing `rowlayout` drops under width
//! pressure. So the two decisions — go wide, and grant the accessory — are the
//! same question asked of two different widths, and getting the order wrong
//! puts both regions on screen with the controls missing from one of them.
//!
//! The user's call: stage one region a little longer instead. This file grades
//! that as an implication over real geometry rather than trusting it.
//!
//! # Three claims, and why none of them is another
//!
//!   * **THE IMPLICATION.** Wherever the frame went two-column, the rows it
//!     drew kept their accessory. Asked of `overlay_right_shown`, the one gate
//!     the accessory upload, the frost's surface list and the published lanes
//!     all read.
//!   * **PRESENCE.** The accessory is not merely granted, it REACHES THE FRAME:
//!     the value lane's own shaped band carries glyph edges, counted off the
//!     rendered pixels. Without it the implication is satisfied perfectly by a
//!     workspace that grants an accessory column and inks nothing into it — and
//!     "the accessory survives the transition" is a claim about something
//!     visible, not about a flag.
//!   * **MACHINE-INDEPENDENCE.** The whole outcome — wide or staged, granted or
//!     yielded, per cell — is IDENTICAL for two readers whose project roots are
//!     both past the settings roster's own allowance. This is the claim the item
//!     was really about: the un-elided path made the reader's own filesystem an
//!     input to awl's layout, so the same build at the same window size laid out
//!     differently on two machines. `settings::value_cells` bounds the cell;
//!     this checks what the bound was FOR. It is deliberately NOT a claim that a
//!     two-character root draws the same column as a hundred-character one —
//!     see [`ROOTS`] for why that difference is honest and the other is not.
//!
//! # The axes, and the one that is not a number
//!
//! `workspace_is_wide` is a threshold in LOGICAL px over scaled text, so it
//! MOVES with zoom and with the display face's own metrics — the same trap
//! `workspace_back_width` records. No width is written down here: the sweep
//! crosses width x zoom x dpi and asserts it REACHED both regimes, naming how
//! many cells landed in each.
//!
//! The LENS is swept from the facet scheme rather than picked, because which
//! rows are on show decides both the widest name and the widest value — and the
//! Path rows, the subject of the whole item, live in exactly one of them.
//!
//! **AND THE MENU BAR.** `menubar::platform_default` is `false` on macOS and
//! `true` everywhere else, and the bar's reserve comes straight off the
//! workspace card (`plan_workspace_regions`), so an unpinned law measures a
//! different product locally than in CI. Both arms are swept, and the AMBIENT
//! value is captured to restore — never `cfg!(target_os = …)`, which reports the
//! host that COMPILED the test rather than the branch the initialiser took.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{SETTINGS_VIEW_PARKED_WINDOW_ROWS, headless_dqp, settings_overlay_view};
use crate::overlay::workspace::WorkspaceShape;
use crate::overlay::{OverlayKind, OverlayState};

/// A LOCAL LUMINANCE STEP big enough to be a glyph edge rather than a gradient,
/// the same threshold the sibling workspace pixel laws use. It compares a
/// RENDERED pixel with its RENDERED neighbour, never with an authored constant,
/// so it survives the backend-to-backend difference in the exact bytes a frame
/// lands on.
const GLYPH_STEP: f32 = 24.0;

fn luma(p: [u8; 4]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// THE ENROLLED KINDS, asked of the roster: every kind that claims a workspace
/// shape whose ROWS live in the content pane — the only place an accessory
/// column can be lost to the rail's arrival. A `TimelineOverComparison`
/// workspace opens a relocated document there and has no accessory to lose, so
/// it excludes itself through the one owner (`rows_are_primary`) rather than by
/// name. Today that is Settings; a second such member enrols itself.
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

/// Three readers' machines, fed through the real `SettingsValues` the live
/// Settings menu is built from: one whose project root is a couple of
/// characters, one of the shape that produced the report — a home directory
/// several folders deep with a descriptive project name on the end — and one
/// deeper still.
///
/// **The machine-independence claim is over the last two, and deliberately not
/// over the first.** A reader whose root is `/p` has less to show and honestly
/// draws a narrower cell; what must stop mattering is HOW deep a deep path is,
/// because that is the difference no reader can see and no reader chose. Which
/// machines qualify is asked of the product ([`is_elided`]), never assumed from
/// the string, so a change to the allowance re-partitions the ladder by itself.
const ROOTS: &[(&str, &str)] = &[
    ("shallow", "/p"),
    (
        "deep",
        "/Users/someone/Documents/writing/projects/2026/the-long-novel-working-draft",
    ),
    (
        "deeper",
        "/Users/someone/Documents/writing/projects/2026/drafts/chapters/revisions/final/really/quite/deep/indeed",
    ),
];

/// Does this machine's root actually overflow the settings roster's allowance —
/// i.e. does any drawn value cell differ from its un-elided readout? Asked of the
/// two production owners rather than by measuring the string here, so this test
/// cannot disagree with the module it is grading.
fn is_elided(root: &str) -> bool {
    let v = values(root);
    crate::settings::visible_rows()
        .iter()
        .zip(crate::settings::visible_value_cells(&v))
        .any(|(r, cell)| cell != crate::settings::value_for(r, &v))
}

fn values(root: &str) -> crate::settings::SettingsValues {
    let mut v = super::settings_values(1.0, 1.0);
    v.default_folder = root.to_string();
    v.workspace = root.to_string();
    v.project_root = root.to_string();
    v
}

/// A real Settings card on lens `lens`, standing in its CONTENT pane with focus
/// placed by the LIFECYCLE rather than assigned — the same walk a user makes —
/// and carrying the production value cells and range cells for `root`.
fn card_in_content(kind: OverlayKind, lens: usize, root: &str) -> OverlayState {
    let vals = values(root);
    let mut ov = OverlayState::new(
        kind,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov.set_facet_lens(lens);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    journey.toggle_detail();
    journey.card().expect("the card is up").clone()
}

/// The card projected the way `App::sync_view` projects it, through the shared
/// settings folder plus the four workspace fields that folder does not carry.
fn content_view(ov: &OverlayState) -> ViewState {
    let mut v = settings_overlay_view(ov, SETTINGS_VIEW_PARKED_WINDOW_ROWS);
    v.overlay_title = ov.kind.title();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    v.overlay_hint = ov.foot_hint();
    v
}

/// Logical windows the sweep crosses: the app's OWN enforced minimum (derived
/// from the same metrics `app::lifecycle` enforces), one in the middle of the
/// staging regime, and one comfortably past the threshold. The threshold's own
/// value is deliberately never written down — it moves with zoom and with the
/// display face, and a law that pinned it would be testing this machine.
fn windows() -> Vec<(u32, u32)> {
    let min_w = (30.0 * CHAR_WIDTH + 2.0 * TEXT_LEFT.0).ceil() as u32;
    let min_h = (8.0 * LINE_HEIGHT + 2.0 * TEXT_TOP.0).ceil() as u32;
    vec![(min_w, min_h), (760, 640), (1400, 900)]
}

/// One swept cell's coordinates, and the key its failures are reported in.
#[derive(Clone, Copy)]
struct Cell {
    w: u32,
    h: u32,
    zoom: f32,
    dpi: f32,
    menu_bar: bool,
}

impl Cell {
    fn describe(&self, kind: OverlayKind, lens: &str, root: &str) -> String {
        format!(
            "{} lens={lens} root={root} at {}x{} logical, zoom={}, dpi={}, menu_bar={}",
            kind.as_str(),
            self.w,
            self.h,
            self.zoom,
            self.dpi,
            if self.menu_bar { "on" } else { "off" }
        )
    }

    /// The key the two machines are compared under — everything BUT the root.
    fn key(&self, kind: OverlayKind, lens: &str) -> String {
        format!(
            "{} lens={lens} at {}x{} zoom={} dpi={} menu_bar={}",
            kind.as_str(),
            self.w,
            self.h,
            self.zoom,
            self.dpi,
            self.menu_bar
        )
    }
}

/// **DID THE VALUE ACTUALLY REACH THE FRAME?** Renders the prepared pipeline and
/// counts, over the value lane's own published band, the canvas columns carrying
/// a glyph edge — `(inked columns, band width)`.
fn value_ink(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &TextPipeline,
    pw: u32,
    ph: u32,
    lane: (f32, f32),
    top: f32,
    height: f32,
) -> (usize, usize) {
    let (texture, tview) = offscreen(device, pw, ph);
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl workspace two-column accessory"),
    });
    p.render(&mut enc, &tview).unwrap();
    queue.submit(Some(enc.finish()));
    let px = read_pixels(device, queue, &texture, pw, ph);
    let lum: Vec<f32> = px.iter().map(|q| luma(*q)).collect();
    let half = (height * 0.5 - 1.0).max(1.0);
    let mid = top + height * 0.5;
    let y0 = ((mid - half) as i64).max(0);
    let y1 = ((mid + half) as i64).min(ph as i64 - 2);
    let x0 = (lane.0 as i64).max(0);
    let x1 = ((lane.0 + lane.1) as i64).min(pw as i64 - 2);
    let inked = (x0..x1)
        .filter(|x| {
            (y0..y1).any(|y| {
                let i = (y * pw as i64 + x) as usize;
                (lum[i] - lum[i + pw as usize]).abs() > GLYPH_STEP
                    || (lum[i] - lum[i + 1]).abs() > GLYPH_STEP
            })
        })
        .count();
    (inked, (x1 - x0).max(0) as usize)
}

/// **THE LAW.** Two-column implies an accessory, the accessory is ink, and
/// neither answer moves with the reader's own filesystem.
#[test]
fn a_two_column_workspace_keeps_the_rows_accessory_and_neither_reads_off_the_machine() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping a_two_column_workspace_keeps_the_rows_accessory...: no wgpu adapter");
        return;
    }
    let kinds = enrolled();
    assert!(
        !kinds.is_empty(),
        "no kind enrolled — this law's subject is the roster's content-pane workspaces, and an \
         enrolment that matches nothing sweeps nothing"
    );

    // THE AMBIENT MENU-BAR VALUE, captured rather than derived.
    let ambient_menu_bar = crate::menubar::menu_bar_on();

    let (mut staged, mut wide, mut graded, mut inked_cells, mut yielded, mut no_readout) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    // Per (cell key) -> per root: (wide, granted). The machine-independence
    // claim is that every entry holds one distinct answer.
    let mut across_machines: std::collections::BTreeMap<String, Vec<(&str, bool, bool)>> =
        Default::default();
    let mut lenses_seen: std::collections::BTreeSet<String> = Default::default();
    let mut tightest = f32::INFINITY;

    for kind in &kinds {
        // THE LENSES, asked of the kind's own facet scheme rather than named —
        // which rows are on show sets both the widest name and the widest value,
        // and the Path rows this item is about live in exactly one of them.
        let scheme = card_in_content(*kind, 0, ROOTS[0].1)
            .facet_scheme()
            .expect("a content-pane workspace facets");
        assert!(
            !scheme.strip.is_empty(),
            "{}: the facet scheme is empty, so the lens axis sweeps nothing",
            kind.as_str()
        );

        for menu_bar in [false, true] {
            crate::menubar::set_menu_bar_on(menu_bar);
            for (lens_i, facet) in scheme.strip.iter().enumerate() {
                lenses_seen.insert(facet.id.to_string());
                for (lw, lh) in windows() {
                    for zoom in [1.0f32, 2.0] {
                        for dpi in [1.0f32, 2.0] {
                            let cell = Cell {
                                w: lw,
                                h: lh,
                                zoom,
                                dpi,
                                menu_bar,
                            };
                            for (machine, root) in ROOTS {
                                let ov = card_in_content(*kind, lens_i, root);
                                if ov.item_strings().is_empty() {
                                    continue;
                                }
                                let what = cell.describe(*kind, facet.id, machine);
                                let (pw, ph) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                                let Some((device, queue, mut p)) =
                                    headless_dqp(pw as f32, ph as f32)
                                else {
                                    crate::menubar::set_menu_bar_on(ambient_menu_bar);
                                    return;
                                };
                                p.set_dpi(dpi);
                                p.set_size(pw as f32, ph as f32);
                                let mut v = content_view(&ov);
                                v.zoom = zoom;
                                p.set_view(&v);
                                p.prepare(&device, &queue, pw, ph).unwrap();

                                let is_wide = p.workspace_is_wide(pw);
                                let granted = p.overlay_right_shown;
                                match is_wide {
                                    true => wide += 1,
                                    false => staged += 1,
                                }
                                graded += 1;
                                across_machines
                                    .entry(cell.key(*kind, facet.id))
                                    .or_default()
                                    .push((machine, is_wide, granted));

                                // **THE IMPLICATION.**
                                assert!(
                                    !is_wide || granted,
                                    "{what}: the workspace drew BOTH regions and the content \
                                     pane's rows lost their accessory column — the rail's \
                                     arrival shrank the pane past what the rows need, which is \
                                     the transition this stage exists to delay"
                                );
                                if !granted {
                                    yielded += 1;
                                    continue;
                                }

                                let Some(g) = p.overlay_row_geometry() else {
                                    panic!("{what}: a granted accessory but no planned geometry");
                                };
                                // Grade the row whose value is the WIDEST cell on
                                // show: it is the one the shared column is sized
                                // by, so it is the one an overflow shows up in.
                                let widest =
                                    g.rows.iter().filter(|r| r.lanes.value.is_some()).max_by(
                                        |a, b| {
                                            a.lanes
                                                .value
                                                .unwrap()
                                                .w
                                                .total_cmp(&b.lanes.value.unwrap().w)
                                        },
                                    );
                                // A DRAWN ROW WHOSE READOUT IS EMPTY publishes no
                                // value lane even with the column granted — the
                                // affordance rows (a submenu, an action) have no
                                // value to state, and a whole lens can be made of
                                // them. That is a lens with nothing to grade, not a
                                // lost accessory, and the two are told apart by the
                                // card's OWN drawn rows rather than by a lens name.
                                let binds = ov.item_bindings();
                                let expects_value = g.rows.iter().any(|r| {
                                    r.item
                                        .and_then(|i| binds.get(i))
                                        .is_some_and(|s| !s.trim().is_empty())
                                });
                                let Some(row) = widest else {
                                    assert!(
                                        !expects_value,
                                        "{what}: the accessory column is granted and a drawn row \
                                         has a readout to state, but no row published a value \
                                         lane — the grant and the lanes read the same gate, so \
                                         they cannot disagree"
                                    );
                                    no_readout += 1;
                                    continue;
                                };
                                let lane = row.lanes.value.expect("filtered on Some");

                                // THE CELL STAYS INSIDE THE BAND. An elided value
                                // that still overflows its own column has moved
                                // the defect rather than fixed it.
                                assert!(
                                    lane.x >= g.band_x - 0.5
                                        && lane.x + lane.w <= g.band_x + g.band_w + 0.5,
                                    "{what}: the widest value lane runs {:.1}..{:.1} outside its \
                                     own band {:.1}..{:.1}",
                                    lane.x,
                                    lane.x + lane.w,
                                    g.band_x,
                                    g.band_x + g.band_w
                                );

                                // **PRESENCE**, off the rendered pixels.
                                let (inked, band) = value_ink(
                                    &device,
                                    &queue,
                                    &p,
                                    pw,
                                    ph,
                                    (lane.x, lane.w),
                                    row.y,
                                    row.h,
                                );
                                assert!(
                                    band >= 4,
                                    "{what}: the value lane clamped to {band} canvas columns — \
                                     there is nothing here for a pixel floor to grade"
                                );
                                // THE FLOOR IS SET UNDER THE ROSTER'S TIGHTEST
                                // REAL VALUE. A settings readout is a word or two
                                // of type across its own shaped band; the tightest
                                // cell this sweep reaches is reported below so the
                                // headroom stays visible, and a quarter leaves room
                                // for a shaper's antialiasing to differ across
                                // backends without leaving room for an empty column.
                                assert!(
                                    inked * 4 >= band,
                                    "{what}: only {inked} of the value lane's own {band} band \
                                     columns carry a glyph edge — the accessory column is \
                                     granted, planned and seated, but what reached the frame is \
                                     not a readout"
                                );
                                tightest = tightest.min(inked as f32 / band as f32);
                                inked_cells += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);

    // THE SWEEP CROSSED THE THRESHOLD, and says so. Without this the implication
    // above is vacuously true of a sweep that never went two-column at all.
    assert!(
        staged > 0 && wide > 0,
        "the sweep never crossed the staging threshold (staged {staged}, wide {wide}) across \
         {graded} cells over {:?} — the implication grades nothing",
        kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>()
    );
    assert!(
        inked_cells > 0,
        "no cell drew an inked value lane across {graded} swept cells — the presence floor \
         graded nothing, and an accessory that never appears satisfies the implication for free"
    );

    // **MACHINE-INDEPENDENCE**, over the machines whose paths ALL overflow — the
    // partition is the product's own answer, and it is asserted to have found at
    // least two members before it grades anything, or the comparison is between a
    // machine and itself.
    let overflowing: Vec<&str> = ROOTS
        .iter()
        .filter(|(_, root)| is_elided(root))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        overflowing.len() >= 2,
        "only {} of the {} machines in the ladder overflow the roster's allowance ({overflowing:?}) \
         — the invariance claim needs two readers whose paths are BOTH past it, or it compares \
         nothing",
        overflowing.len(),
        ROOTS.len()
    );
    let mut differed: Vec<String> = Vec::new();
    let mut compared = 0usize;
    for (key, seen) in &across_machines {
        let deep: Vec<_> = seen
            .iter()
            .filter(|(m, _, _)| overflowing.contains(m))
            .collect();
        let answers: std::collections::BTreeSet<(bool, bool)> =
            deep.iter().map(|(_, w, g)| (*w, *g)).collect();
        compared += deep.len();
        if answers.len() > 1 {
            differed.push(format!("{key}: {deep:?}"));
        }
    }
    assert!(
        differed.is_empty(),
        "the layout answered differently for two readers whose paths are BOTH past the roster's \
         allowance, so their only difference is how much further past it they are — (wide, \
         accessory granted) per machine:\n{}",
        differed.join("\n")
    );

    eprintln!(
        "workspace two-column accessory: {graded} cells over {} lenses {:?} x {} machines x both \
         menu-bar arms (ambient here is {ambient_menu_bar}); {wide} wide, {staged} staged, \
         {inked_cells} inked, {yielded} yielded while staged, {no_readout} with no readout to \
         state; {compared} cells compared across the {} overflowing machines {overflowing:?}; \
         tightest ink share {:.3} against a floor of 0.250",
        lenses_seen.len(),
        lenses_seen,
        ROOTS.len(),
        overflowing.len(),
        tightest
    );
}

/// **THE DELAY IS A MECHANISM, NOT A COINCIDENCE.**
///
/// The law above is an implication, and today's Settings roster satisfies it
/// with room to spare — the value cells are bounded by
/// `settings::value_cells`'s allowance, which is the widest row NAME, so the
/// rows' whole demand lands exactly on the `MIN_PANE_CHARS` legibility floor
/// that was already there. That makes the implication true and says nothing
/// about WHY, and a roster that grew one long name would re-open the defect
/// with every existing law still green.
///
/// So this asks the gate directly, at one fixed geometry, with two rosters that
/// differ only in how much room their rows want: the demanding one must still be
/// staged at a width where the modest one has already gone two-column. It is the
/// only check here that fails if `workspace_min_pane` stops reading the rows.
#[test]
fn the_wide_gate_delays_for_rows_that_ask_for_more_room() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the_wide_gate_delays_for_rows_that_ask_for_more_room: no adapter");
        return;
    }
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);

    // Two rosters at the same row COUNT — only the widths differ, so nothing but
    // the demand can move the threshold.
    let modest: Vec<String> = (0..8).map(|i| format!("Row {i}")).collect();
    let modest_vals: Vec<String> = (0..8).map(|_| "on".to_string()).collect();
    let demanding: Vec<String> = (0..8)
        .map(|i| format!("A considerably longer settings row name {i}"))
        .collect();
    let demanding_vals: Vec<String> = (0..8)
        .map(|_| "a considerably longer readout".to_string())
        .collect();

    let threshold = |items: &[String], vals: &[String]| -> Option<u32> {
        let ov = card_in_content(OverlayKind::Settings, 0, ROOTS[0].1);
        for w in (200u32..4000).step_by(4) {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, 900.0) else {
                return None;
            };
            p.set_dpi(1.0);
            p.set_size(w as f32, 900.0);
            let mut v = content_view(&ov);
            v.overlay_items = items.to_vec();
            v.overlay_bindings = vals.to_vec();
            v.overlay_ranges = Vec::new();
            p.set_view(&v);
            p.prepare(&device, &queue, w, 900).unwrap();
            if p.workspace_is_wide(w) {
                return Some(w);
            }
        }
        None
    };

    let modest_at = threshold(&modest, &modest_vals);
    let demanding_at = threshold(&demanding, &demanding_vals);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);

    let (Some(modest_at), Some(demanding_at)) = (modest_at, demanding_at) else {
        eprintln!("skipping the_wide_gate_delays_for_rows_that_ask_for_more_room: no adapter");
        return;
    };
    assert!(
        demanding_at > modest_at,
        "the wide gate flipped at {demanding_at}px for rows that want a lot of room and at \
         {modest_at}px for rows that want little — it is not reading the rows at all, so the \
         two-column stage can still arrive at a pane the accessory does not fit in"
    );
    eprintln!(
        "wide-gate delay: modest rows go two-column at {modest_at}px, demanding rows at \
         {demanding_at}px (+{}px)",
        demanding_at - modest_at
    );
}

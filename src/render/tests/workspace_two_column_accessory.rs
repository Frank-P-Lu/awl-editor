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
//!     differently on two machines. `settings::visible_value_cells` bounds the cell;
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
        concat!(
            "/Users/someone/Documents/writing/projects/2026/drafts/chapters/",
            "revisions/final/really/quite/deep/indeed"
        ),
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
/// settings folder (which now derives `overlay_workspace`/`overlay_detail_focus`
/// itself) plus the two workspace fields that folder does not carry.
fn content_view(ov: &OverlayState) -> ViewState {
    let mut v = settings_overlay_view(ov, SETTINGS_VIEW_PARKED_WINDOW_ROWS);
    v.overlay_title = ov.kind.title().to_string();
    v.overlay_lens = ov.lens_strip();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(WorkspaceShape::rows_are_primary);
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

    /// The key the fresh-pipeline control's enrolment is taken over: the
    /// GEOMETRY, which is what a hoisted pipeline carries between cells, paired
    /// with the outcome class the sweep's claims are stated in. Deliberately
    /// free of the lens and the machine — those change the card's corpus, not
    /// the pipeline state a reuse could stale, and folding them in would make
    /// the control a second full sweep.
    fn shape(&self, class: (bool, bool, bool)) -> String {
        format!(
            "{}x{} zoom={} dpi={} menu_bar={} class={class:?}",
            self.w, self.h, self.zoom, self.dpi, self.menu_bar
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
    (pw, ph): (u32, u32),
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

/// What one swept cell came to. `ink_share` is `Some` only where a value lane
/// was actually graded against the pixels, so the caller can report how much
/// headroom the presence floor had without re-deriving it.
struct Outcome {
    wide: bool,
    granted: bool,
    ink_share: Option<f32>,
    no_readout: bool,
}

/// **ONE CELL'S READING, AS BITS** — the form the fresh-pipeline control
/// compares, named so the audit's own shape stays readable.
type Reading = (bool, bool, bool, Option<u32>);

impl Outcome {
    /// **THE READING, AS BITS** — the form the fresh-pipeline control compares.
    /// The ink share goes through `f32::to_bits`, so agreement means agreement
    /// in the last mantissa bit rather than `==`'s tolerance for `-0.0`.
    fn bits(&self) -> Reading {
        (
            self.wide,
            self.granted,
            self.no_readout,
            self.ink_share.map(f32::to_bits),
        )
    }

    /// WHAT KIND OF CELL THIS IS, as the control's enrolment reads it: the three
    /// booleans the sweep's claims are stated over. Derived from the reading
    /// rather than from a cell's coordinates, so a geometry that starts
    /// answering differently enrols itself.
    fn class(&self) -> (bool, bool, bool) {
        (self.wide, self.granted, self.no_readout)
    }
}

/// What the sweep saw, in aggregate — the counts every closing assertion and the
/// receipt are stated in.
struct Tally {
    wide: usize,
    staged: usize,
    graded: usize,
    inked: usize,
    yielded: usize,
    no_readout: usize,
    tightest: f32,
}

impl Tally {
    fn new() -> Self {
        Self {
            wide: 0,
            staged: 0,
            graded: 0,
            inked: 0,
            yielded: 0,
            no_readout: 0,
            tightest: f32::INFINITY,
        }
    }

    fn add(&mut self, out: &Outcome) {
        match out.wide {
            true => self.wide += 1,
            false => self.staged += 1,
        }
        self.graded += 1;
        self.yielded += usize::from(!out.granted);
        self.no_readout += usize::from(out.no_readout);
        if let Some(share) = out.ink_share {
            self.tightest = self.tightest.min(share);
            self.inked += 1;
        }
    }
}

/// **ONE SWEPT CELL, GRADED.** Every per-cell assertion lives here so the sweep
/// above stays a sweep; `None` means the machine has no adapter to answer with.
fn grade_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    kind: OverlayKind,
    lens: (usize, &str),
    machine: (&str, &str),
    cell: Cell,
) -> Outcome {
    let (lens_i, lens_id) = lens;
    let (name, root) = machine;
    let ov = card_in_content(kind, lens_i, root);
    let what = cell.describe(kind, lens_id, name);
    assert!(
        !ov.item_strings().is_empty(),
        "{what}: the lens filtered every row away, so this cell has no accessory to grade and \
         the sweep's lens axis is narrower than it reports"
    );
    let (pw, ph) = (
        (cell.w as f32 * cell.dpi) as u32,
        (cell.h as f32 * cell.dpi) as u32,
    );
    // THE CELL'S OWN MENU-BAR ARM, set here rather than by the caller, so the
    // fresh-pipeline control below cannot re-measure a cell under the ambient
    // arm and grade a different card.
    crate::menubar::set_menu_bar_on(cell.menu_bar);
    p.set_dpi(cell.dpi);
    p.set_size(pw as f32, ph as f32);
    let mut v = content_view(&ov);
    v.zoom = cell.zoom;
    p.set_view(&v);
    p.prepare(device, queue, pw, ph).unwrap();

    let wide = p.workspace_is_wide(pw);
    let granted = p.overlay_right_shown;
    // **THE IMPLICATION.**
    assert!(
        !wide || granted,
        "{what}: the workspace drew BOTH regions and the content pane's rows lost their \
         accessory column — the rail's arrival shrank the pane past what the rows need, which \
         is the transition this stage exists to delay"
    );
    let mut out = Outcome {
        wide,
        granted,
        ink_share: None,
        no_readout: false,
    };
    if !granted {
        return out;
    }

    let Some(g) = p.overlay_row_geometry() else {
        panic!("{what}: a granted accessory but no planned geometry");
    };
    // Grade the row whose value is the WIDEST cell on show: it is the one the
    // shared column is sized by, so it is the one an overflow shows up in.
    let widest = g
        .rows
        .iter()
        .filter(|r| r.lanes.value.is_some())
        .max_by(|a, b| {
            a.lanes
                .value
                .unwrap()
                .w
                .total_cmp(&b.lanes.value.unwrap().w)
        });
    // A DRAWN ROW WHOSE READOUT IS EMPTY publishes no value lane even with the
    // column granted — the affordance rows (a submenu, an action) have no value
    // to state, and a whole lens can be made of them. That is a lens with nothing
    // to grade, not a lost accessory, and the two are told apart by the card's
    // OWN drawn rows rather than by a lens name.
    let binds = ov.item_bindings();
    let expects_value = g.rows.iter().any(|r| {
        r.item
            .and_then(|i| binds.get(i))
            .is_some_and(|s| !s.trim().is_empty())
    });
    let Some(row) = widest else {
        assert!(
            !expects_value,
            "{what}: the accessory column is granted and a drawn row has a readout to state, but \
             no row published a value lane — the grant and the lanes read the same gate, so they \
             cannot disagree"
        );
        out.no_readout = true;
        return out;
    };
    let lane = row.lanes.value.expect("filtered on Some");

    // THE CELL STAYS INSIDE THE BAND. An elided value that still overflows its
    // own column has moved the defect rather than fixed it.
    assert!(
        lane.x >= g.band_x - 0.5 && lane.x + lane.w <= g.band_x + g.band_w + 0.5,
        "{what}: the widest value lane runs {:.1}..{:.1} outside its own band {:.1}..{:.1}",
        lane.x,
        lane.x + lane.w,
        g.band_x,
        g.band_x + g.band_w
    );

    // **PRESENCE**, off the rendered pixels.
    let (inked, band) = value_ink(device, queue, p, (pw, ph), (lane.x, lane.w), row.y, row.h);
    assert!(
        band >= 4,
        "{what}: the value lane clamped to {band} canvas columns — there is nothing here for a \
         pixel floor to grade"
    );
    // THE FLOOR IS SET UNDER THE ROSTER'S TIGHTEST REAL VALUE. A settings readout
    // is a word or two of type across its own shaped band; the tightest cell this
    // sweep reaches is reported by the caller so the headroom stays visible, and
    // a quarter leaves room for a shaper's antialiasing to differ across backends
    // without leaving room for an empty column.
    assert!(
        inked * 4 >= band,
        "{what}: only {inked} of the value lane's own {band} band columns carry a glyph edge — \
         the accessory column is granted, planned and seated, but what reached the frame is not \
         a readout"
    );
    out.ink_share = Some(inked as f32 / band as f32);
    out
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

    let mut tally = Tally::new();
    // Per (cell key) -> per root: (wide, granted). The machine-independence
    // claim is that every entry holds one distinct answer.
    let mut across_machines: std::collections::BTreeMap<String, Vec<(&str, bool, bool)>> =
        Default::default();
    let mut lenses_seen: std::collections::BTreeSet<String> = Default::default();
    // ONE PIPELINE for the whole sweep, checked against fresh ones below rather
    // than trusted: five hundred cells at a pipeline each is most of this
    // module's cost, and reuse across a size, zoom and lens swap is exactly the
    // cache-key discipline CLAUDE.md records.
    let Some((device, queue, mut p)) = headless_dqp(64.0, 64.0) else {
        return;
    };
    // THE CONTROL'S ENROLMENT, derived rather than pinned: the first cell of
    // each distinct (geometry x outcome class) the sweep actually produced, plus
    // every cell that held the tightest ink share — the reading the presence
    // floor's headroom is stated from. A geometry or a class that starts
    // appearing enrols itself, and one that stops appearing takes its
    // representative with it, so the control cannot go on covering a shape the
    // sweep no longer has.
    let mut audit: Vec<(String, Site, Reading)> = Vec::new();
    let mut represented: std::collections::BTreeSet<String> = Default::default();

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
                                let site = Site {
                                    kind: *kind,
                                    lens: (lens_i, facet.id),
                                    machine: (machine, root),
                                    cell,
                                };
                                let out = grade_cell(
                                    &device,
                                    &queue,
                                    &mut p,
                                    *kind,
                                    (lens_i, facet.id),
                                    (machine, root),
                                    cell,
                                );
                                let tighter =
                                    out.ink_share.is_some_and(|share| share < tally.tightest);
                                if represented.insert(cell.shape(out.class())) || tighter {
                                    audit.push((site.describe(), site, out.bits()));
                                }
                                tally.add(&out);
                                across_machines
                                    .entry(cell.key(*kind, facet.id))
                                    .or_default()
                                    .push((machine, out.wide, out.granted));
                            }
                        }
                    }
                }
            }
        }
    }
    let rechecked = assert_the_hoist_carries_no_state(&audit);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    assert!(
        rechecked >= audit.len(),
        "only {rechecked} of the {} cells this law's claims rest on were re-measured against a \
         pipeline built for them alone — the reuse check stopped covering the sweep",
        audit.len()
    );
    conclude(
        &tally,
        &across_machines,
        &lenses_seen,
        &kinds,
        ambient_menu_bar,
    );
    eprintln!(
        "workspace two-column accessory: {rechecked} cells re-measured against fresh pipelines"
    );
}

/// One swept cell's full coordinates — everything `grade_cell` needs — carried
/// whole so the fresh-pipeline control rebuilds the cell rather than parsing it
/// back out of the label. A control that re-derived its coordinates from a
/// second copy of the sweep's own arithmetic would agree with itself, not with
/// the sweep.
#[derive(Clone, Copy)]
struct Site {
    kind: OverlayKind,
    lens: (usize, &'static str),
    machine: (&'static str, &'static str),
    cell: Cell,
}

impl Site {
    fn describe(&self) -> String {
        self.cell.describe(self.kind, self.lens.1, self.machine.0)
    }
}

/// **THE HOISTED PIPELINE IS CHECKED, NOT TRUSTED.** Every cell the enrolment
/// above picked out, measured again against a pipeline that has seen no other
/// geometry, through [`super::assert_the_hoist_carries_no_state`] — the one
/// owner of that rule across the laws that hoist.
fn assert_the_hoist_carries_no_state(audit: &[(String, Site, Reading)]) -> usize {
    let sites: std::collections::BTreeMap<&str, Site> = audit
        .iter()
        .map(|(what, site, _)| (what.as_str(), *site))
        .collect();
    let recorded: Vec<(String, Reading)> = audit
        .iter()
        .map(|(what, _, bits)| (what.clone(), *bits))
        .collect();
    super::assert_the_hoist_carries_no_state(&recorded, |what| {
        let s = sites[what];
        let (pw, ph) = (
            (s.cell.w as f32 * s.cell.dpi) as u32,
            (s.cell.h as f32 * s.cell.dpi) as u32,
        );
        let (d2, q2, mut fresh) = headless_dqp(pw as f32, ph as f32)?;
        Some(grade_cell(&d2, &q2, &mut fresh, s.kind, s.lens, s.machine, s.cell).bits())
    })
}

/// **THE SWEEP'S OWN NON-VACUITY, AND THE CLAIM IT WAS FOR.** Kept apart from
/// the sweep because every assertion here is about the sweep AS A WHOLE — that
/// it crossed the threshold, that it graded ink anywhere, and that the two
/// overflowing machines agreed — rather than about any one cell.
fn conclude(
    tally: &Tally,
    across_machines: &std::collections::BTreeMap<String, Vec<(&str, bool, bool)>>,
    lenses_seen: &std::collections::BTreeSet<String>,
    kinds: &[OverlayKind],
    ambient_menu_bar: bool,
) {
    let (wide, staged, graded) = (tally.wide, tally.staged, tally.graded);
    // THE SWEEP CROSSED THE THRESHOLD, and says so. Without this the implication
    // in `grade_cell` is vacuously true of a sweep that never went two-column.
    assert!(
        staged > 0 && wide > 0,
        "the sweep never crossed the staging threshold (staged {staged}, wide {wide}) across \
         {graded} cells over {:?} — the implication grades nothing",
        kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>()
    );
    assert!(
        tally.inked > 0,
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
        "only {} of the {} machines in the ladder overflow the roster's allowance \
         ({overflowing:?}) — the invariance claim needs two readers whose paths are BOTH past \
         it, or it compares nothing",
        overflowing.len(),
        ROOTS.len()
    );
    let mut differed: Vec<String> = Vec::new();
    let mut compared = 0usize;
    for (key, seen) in across_machines {
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
        "workspace two-column accessory: {graded} cells over {} lenses {lenses_seen:?} x {} \
         machines x both menu-bar arms (ambient here is {ambient_menu_bar}); {wide} wide, \
         {staged} staged, {} inked, {} yielded while staged, {} with no readout to state; \
         {compared} cells compared across the {} overflowing machines {overflowing:?}; tightest \
         ink share {:.3} against a floor of 0.250",
        lenses_seen.len(),
        ROOTS.len(),
        tally.inked,
        tally.yielded,
        tally.no_readout,
        overflowing.len(),
        tally.tightest
    );
}

/// **THE DELAY IS A MECHANISM, NOT A COINCIDENCE.**
///
/// The law above is an implication, and today's Settings roster satisfies it
/// with room to spare — the value cells are bounded by
/// `settings::visible_value_cells`'s allowance, which is the widest row NAME, so the
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

    // ONE PIPELINE for the whole walk. The widths this closure returns are
    // re-derived below against pipelines built for a single width, so the reuse
    // is checked rather than trusted.
    let Some((device, queue, mut p)) = headless_dqp(64.0, 900.0) else {
        eprintln!("skipping the_wide_gate_delays_for_rows_that_ask_for_more_room: no adapter");
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        return;
    };
    let mut threshold = |items: &[String], vals: &[String]| -> Option<u32> {
        (200u32..4000)
            .step_by(4)
            .find(|w| wide_at(&device, &queue, &mut p, items, vals, *w))
    };

    let modest_at = threshold(&modest, &modest_vals);
    let demanding_at = threshold(&demanding, &demanding_vals);

    let (Some(modest_at), Some(demanding_at)) = (modest_at, demanding_at) else {
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
        panic!(
            "the wide gate never flipped anywhere in 200..4000px for one of the two rosters \
             (modest {modest_at:?}, demanding {demanding_at:?}) — the walk graded nothing"
        );
    };
    // **THE HOISTED PIPELINE IS CHECKED, NOT TRUSTED**, over the four readings
    // this law's whole claim rests on: each threshold IS the pair `(not wide at
    // w-4, wide at w)`, so re-deriving both sides of both flips against a
    // pipeline built for that width alone re-establishes the two numbers
    // outright rather than sampling near them.
    let rechecked = assert_the_two_thresholds_survive_the_hoist(&[
        (&modest, &modest_vals, modest_at),
        (&demanding, &demanding_vals, demanding_at),
    ]);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    assert!(
        rechecked == 4,
        "only {rechecked} of the 4 readings the two thresholds rest on were re-derived against a \
         fresh pipeline"
    );
    assert!(
        demanding_at > modest_at,
        "the wide gate flipped at {demanding_at}px for rows that want a lot of room and at \
         {modest_at}px for rows that want little — it is not reading the rows at all, so the \
         two-column stage can still arrive at a pane the accessory does not fit in"
    );
    eprintln!(
        "wide-gate delay: modest rows go two-column at {modest_at}px, demanding rows at \
         {demanding_at}px (+{}px); {rechecked} readings re-derived against fresh pipelines",
        demanding_at - modest_at
    );
}

/// **DOES THE GATE READ WIDE AT THIS ONE WIDTH?** The single owner of that
/// reading, so the hoisted walk above and the fresh-pipeline control below
/// cannot ask two different questions and agree about nothing.
fn wide_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    items: &[String],
    vals: &[String],
    w: u32,
) -> bool {
    let ov = card_in_content(OverlayKind::Settings, 0, ROOTS[0].1);
    p.set_dpi(1.0);
    p.set_size(w as f32, 900.0);
    let mut v = content_view(&ov);
    v.overlay_items = items.to_vec();
    v.overlay_bindings = vals.to_vec();
    v.overlay_ranges = Vec::new();
    p.set_view(&v);
    p.prepare(device, queue, w, 900).unwrap();
    p.workspace_is_wide(w)
}

/// A THRESHOLD IS A PAIR OF READINGS — staged at the step before it, wide at it
/// — and both are re-derived here against a pipeline that has walked no other
/// width. A hoisted walk that carried state would land the flip somewhere a
/// single-width pipeline does not.
fn assert_the_two_thresholds_survive_the_hoist(found: &[(&[String], &[String], u32); 2]) -> usize {
    let recorded: Vec<(String, bool)> = found
        .iter()
        .flat_map(|(_, _, at)| {
            [
                (format!("the step below the {at}px flip"), false),
                (format!("the {at}px flip itself"), true),
            ]
        })
        .collect();
    super::assert_the_hoist_carries_no_state(&recorded, |what| {
        let (items, vals, at) = found
            .iter()
            .find(|(_, _, at)| what.contains(&at.to_string()))
            .expect("every label names one of the two thresholds");
        let w = if what.ends_with("itself") {
            *at
        } else {
            at - 4
        };
        let (d2, q2, mut fresh) = headless_dqp(w as f32, 900.0)?;
        Some(wide_at(&d2, &q2, &mut fresh, items, vals, w))
    })
}

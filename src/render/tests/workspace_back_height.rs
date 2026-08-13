//! **THE BACK'S FOOTER HAS ROOM TO STAND IN, AND HOW MUCH ROOM IS A PROPERTY
//! OF THE WORLD — SO EVERY WORLD IS ASKED.**
//!
//! `workspace_back_width` grades the footer's HORIZONTAL fit in three bands,
//! because half of that comparison is a fact about the host: `⌫` is carried by
//! no bundled face, so its advance comes from whatever the system font DB
//! answers with, and a boolean at the card's right edge said one thing on a Mac
//! and another on CI. This file grades the OTHER axis, and it deliberately does
//! NOT band it.
//!
//! # WHY THERE IS NO BAND HERE
//!
//! Nothing a host can substitute moves the vertical measurement. The chain from
//! the card's top to the footer's baseline is closed arithmetic over quantities
//! awl sets:
//!
//!   * `Metrics::line_height` is `LINE_HEIGHT * zoom * dpi` — a constant, never
//!     a face's own ascent/descent;
//!   * `overlay_lh()` adds theme-authored LOGICAL lengths (the density's leading
//!     and the list style's row gap) through the same scale;
//!   * the footer's own row is `round(overlay_lh * OVERLAY_HINT_ROW)`, and the
//!     shaper hands that height to the line rather than letting the line ask for
//!     one;
//!   * the panel buffer shapes at `Wrap::None`, so no substituted advance can
//!     spill a line into a second row and push the footer down a pitch;
//!   * the card's own height is the canvas less the menu bar's reserve.
//!
//! A substituted glyph gets no vote anywhere in that list, and item 408
//! measured the consequence rather than assuming it: across all 94 laid-out
//! cells of the width law's sweep, on macOS against Apple Symbols and on Ubuntu
//! 24.04 against DejaVu Sans, the vertical demand agreed TO THE BYTE while every
//! shaped width differed by one common factor.
//!
//! So a `HOST_BAND` analogue here would open a grade no host could ever populate
//! — a ledger that reads as coverage and holds nothing. The vertical grade stays
//! exact, and what it gains instead is NOTICE: a cell that walks toward the edge
//! is ledgered by name at the demand it walked to ([`CROWDED`]), so getting
//! closer reddens rather than passing quietly until the day it crosses.
//!
//! # AND THE AXIS THAT DOES MOVE IT IS THE ROSTER'S, WHICH IS WHY THIS FILE
//! EXISTS
//!
//! The one non-constant in `overlay_lh()` is the LIST STYLE's row gap, and that
//! is `theme::active().render_caps.list_style` — a per-WORLD value. It is not a
//! rounding difference: a `Rules` world buys its rules the air either side of
//! them, which is a quarter of a row added to every row the card stacks above
//! its footer.
//!
//! Measured over the whole roster at the app's own enforced minimum window,
//! zoom 1.4, menu bar shown — the cell the width law left one and a half percent
//! from its edge — the same footer reads:
//!
//! | pitch | vertical demand |
//! |---|---|
//! | `Pane` / `Diagonal` | 0.9849 — fits |
//! | `Rules` | 1.2433 — past the card's bottom |
//! | `Bars` | never laid out at all |
//!
//! One cell, three outcomes, none of them a host's doing. The width law's own
//! vertical readings are all taken at whatever world happens to be active when
//! it runs, so its `STARVED` ledger is a single pitch's answer wearing no label.
//! This law asks the ROSTER.
//!
//! # HOW THE ENROLMENT IS DERIVED
//!
//! Every world is swept over the full geometry grid, and the worlds are then
//! GROUPED BY WHAT THEY MEASURED — not by their list style, which would be
//! assuming the answer. Each group is labelled with the list-style tags its
//! members carry, and the law requires those labels to be DISTINCT: two groups
//! wearing one label would mean two worlds with the same row pitch disagreed
//! vertically, i.e. that something other than the pitch moved this axis, and it
//! reddens naming both worlds rather than quietly ledgering one of them.
//!
//! The three ledgers are keyed by that label and stay exact and two-sided: a
//! cell that arrives is a new degradation, a cell that has left has been fixed
//! and its entry is stale.
//!
//! # THE SWEEP REUSES ONE PIPELINE, AND THAT IS CHECKED RATHER THAN ASSUMED
//!
//! Twenty worlds over the width law's grid is 1920 cells, and a fresh
//! `TextPipeline` per cell costs sixteen times what reusing one does. Reuse is a
//! cache-staleness bet — the exact class CLAUDE.md's cache-key discipline is
//! about — so every cell that EARNS A LEDGER ENTRY, and the tightest fitting
//! cell in the sweep, is measured a second time against a pipeline built for
//! that cell alone, and the two readings must agree.

use super::super::*;
use super::headless_dqp;
use super::workspace_back_width::{
    assert_the_budget_could_not_hold_it, card_in_content, content_view, enrolled, windows,
};
use crate::overlay::{OverlayKind, OverlayState};

/// **HOW CLOSE TO ITS CARD'S BOTTOM EDGE A FOOTER MAY SIT BEFORE THIS LAW
/// LEDGERS IT BY NAME.** Not a tolerance — a crowded cell still counts as
/// fitting, and nothing about the product's behaviour changes at this line. It
/// is the NOTICE threshold: the point past which "how close" stops being
/// invisible and becomes an entry in [`CROWDED`] carrying the demand it reached.
///
/// Sized off the measured distribution rather than picked. Over the swept roster
/// the fitting cells fall in two clumps with a 2.7% gap between them — four
/// cells crowd the edge at 0.9824…0.9982, and the next-tightest cell in the
/// whole sweep sits at 0.9551. This threshold stands in that gap with room on
/// both sides, and the law measures and reports both clearances rather than
/// trusting them ([`THRESHOLD_CLEARANCE`]).
const CROWDING: f32 = 0.03;

/// **THE STEP A CROWDED CELL'S DEMAND IS LEDGERED AT**, which is what makes
/// "the footer got closer to the edge" a RED rather than a silent pass. A cell
/// already inside [`CROWDING`] cannot be caught by membership alone — it is
/// already a member — so its entry carries its demand floored to this step, and
/// a cell that walks half a percent nearer changes its own entry.
///
/// This is a notice granularity and not a measurement tolerance: the vertical
/// chain is exact arithmetic and agrees to the byte across hosts (see the module
/// doc), so the step is not absorbing any variation. It is coarse enough that a
/// one-pixel change in an unrelated rounding does not rewrite the ledger, and
/// fine enough that no cell can cross a whole percent of its card unremarked.
const NOTICE_STEP: f32 = 0.005;

/// **THE NOTICE THRESHOLD'S OWN NON-VACUITY.** A threshold with a cell sitting
/// on it is re-decided by the next unrelated change, which is the failure the
/// width law's `GRADE_HEADROOM` exists to end, one axis over. So the sweep
/// requires the tightest FITTING cell and the loosest CROWDED cell to each stand
/// this factor clear of [`CROWDING`]'s line, and reports both.
///
/// 1.01 sits under the measured pair (1.0156 below the line, 1.0128 above it),
/// so it is a floor with room rather than a restatement of today's numbers.
const THRESHOLD_CLEARANCE: f32 = 1.01;

/// The pitch tag a world contributes to its group's label. Exhaustive with NO
/// wildcard, so a new [`theme::ListStyle`] cannot join a group under a borrowed
/// name — it fails to compile, and whoever adds it decides what it is called.
fn pitch_tag(style: theme::ListStyle) -> &'static str {
    match style {
        theme::ListStyle::Pane => "pane",
        theme::ListStyle::Bars => "bars",
        theme::ListStyle::Rules(_) => "rules",
        theme::ListStyle::Diagonal(_) => "diagonal",
    }
}

/// One swept cell's coordinates. The MENU BAR is part of the key for the same
/// reason it is in the width law's: its reserve is subtracted straight from the
/// card's height budget, so a vertical grade is not well defined without it.
#[derive(Clone, Copy)]
struct Cell {
    w: u32,
    h: u32,
    zoom: f32,
    dpi: f32,
    menu_bar: bool,
}

impl Cell {
    fn describe(&self, kind: OverlayKind) -> String {
        format!(
            "{} at {}x{} logical, zoom={}, dpi={}, menu_bar={}",
            kind.as_str(),
            self.w,
            self.h,
            self.zoom,
            self.dpi,
            if self.menu_bar { "on" } else { "off" }
        )
    }
}

/// The whole geometry grid, in the order the ledgers read in. The same windows,
/// zooms and scales the width law crosses, so a cell name means the same thing
/// in both files.
fn grid() -> Vec<Cell> {
    let mut cells = Vec::new();
    for menu_bar in [false, true] {
        for (w, h) in windows() {
            for zoom in [1.0f32, 1.4, 2.0] {
                for dpi in [1.0f32, 2.0] {
                    cells.push(Cell {
                        w,
                        h,
                        zoom,
                        dpi,
                        menu_bar,
                    });
                }
            }
        }
    }
    cells
}

/// Where one cell's footer stood in its card's height budget.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Room {
    /// The budget could not lay the line out at all — [`STARVED`].
    Starved,
    /// The line is drawn past the card's bottom edge — [`SPILLED`].
    Spilled,
    /// It fits, with less than [`CROWDING`] of the card left under it —
    /// [`CROWDED`].
    Crowded,
    /// It fits with room. Ledgered nowhere.
    Fits,
}

/// One cell's reading: the grade, and the share of its card's height the footer
/// asked for. A starved cell has no ink box to measure, so its demand is
/// infinite and it takes no part in the threshold arithmetic.
#[derive(Clone, Copy)]
struct Reading {
    room: Room,
    demand: f32,
}

/// **ONE CELL, MEASURED.** The single owner of the vertical demand, so the
/// hoisted sweep and the fresh-pipeline check below cannot measure two different
/// quantities and agree about nothing.
fn measure(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    ov: &OverlayState,
    cell: Cell,
    what: &str,
) -> Reading {
    let (pw, ph) = (
        (cell.w as f32 * cell.dpi) as u32,
        (cell.h as f32 * cell.dpi) as u32,
    );
    p.set_dpi(cell.dpi);
    p.set_size(pw as f32, ph as f32);
    let mut v = content_view(ov);
    v.zoom = cell.zoom;
    p.set_view(&v);
    p.prepare(device, queue, pw, ph).unwrap();

    let geom = p.workspace_geometry(pw);
    let line = p.overlay_hint_line().unwrap_or_else(|| {
        panic!("{what}: the content stage shaped no footer at all, so there is no room to grade")
    });
    let [_cx, cy, _cw, ch] = p.workspace_regions(pw).card;
    assert!(ch > 0.0, "{what}: the card reports no height ({ch})");
    let seat = p.panel_buffer.layout_runs().find_map(|run| {
        (run.line_i == line).then_some((geom.text_top + run.line_top, run.line_height))
    });
    // A CARD TOO SHORT FOR ITS OWN COMPOSITION loses the footer to the layout,
    // and is ledgered — but only after the card's OWN reported geometry says the
    // budget could not have held it, which is the width law's owner for that
    // excuse rather than a second copy of it.
    let Some((top, height)) = seat else {
        assert_the_budget_could_not_hold_it(p, &geom, what);
        return Reading {
            room: Room::Starved,
            demand: f32::INFINITY,
        };
    };
    let demand = (top + height - cy) / ch;
    // THE HALF-PIXEL SLACK is the width law's, deliberately: the two files must
    // agree about what "past the bottom edge" means, or one of them ledgers a
    // cell the other calls fine.
    let room = if top + height > cy + ch + 0.5 {
        Room::Spilled
    } else if demand > 1.0 - CROWDING {
        Room::Crowded
    } else {
        Room::Fits
    };
    Reading { room, demand }
}

/// A crowded cell's ledger entry: its name, and the demand it reached floored to
/// [`NOTICE_STEP`].
fn crowded_entry(label: &str, what: &str, demand: f32) -> String {
    let step = (demand / NOTICE_STEP).floor() * NOTICE_STEP;
    format!("{label} · {what} @{step:.3}")
}

/// Everything one world contributed, in grid order.
struct WorldReadings {
    name: &'static str,
    tag: &'static str,
    readings: Vec<Reading>,
}

impl WorldReadings {
    /// The world's whole vertical answer, as an exact key. Grades AND demands to
    /// the bit: two worlds are the same world to this law only if every cell
    /// agreed, so a group can never hide a member that differed somewhere the
    /// grades happened to match.
    fn signature(&self) -> Vec<(Room, u32)> {
        self.readings
            .iter()
            .map(|r| (r.room, r.demand.to_bits()))
            .collect()
    }
}

/// **THE THREE LEDGERS, COMPARED.** Each is an exact set and each is two-sided.
fn assert_the_ledgers_are_unchanged(spilled: &[String], crowded: &[String], starved: &[String]) {
    assert_eq!(
        starved, STARVED,
        "the set of (row pitch, cell) pairs whose card is too short to lay its footer out at all \
         changed. A pair that is here and not in STARVED is a NEW loss of the Back — fix it. A \
         pair in STARVED that is no longer here has been fixed — delete its entry rather than \
         leave a ledger that grades nothing."
    );
    assert_eq!(
        spilled, SPILLED,
        "the set of (row pitch, cell) pairs whose footer is drawn past the bottom of its own card \
         changed. A pair that is here and not in SPILLED is a NEW spill — fix it. A pair in \
         SPILLED that is no longer here has been fixed — delete its entry. Nothing about this \
         axis is the host's: the vertical chain is exact arithmetic and agrees to the byte across \
         hosts, so a change here is a change in the product."
    );
    assert_eq!(
        crowded, CROWDED,
        "the set of (row pitch, cell) pairs sitting within a hair of the bottom of their card \
         changed — or one of them MOVED, since each entry carries the demand it reached. A pair \
         that is here and not in CROWDED has walked toward its card's edge, which is the thing \
         this ledger exists to say out loud before the day it crosses. An entry whose number \
         ROSE is the same footer with less room than it had. Do NOT resolve this by editing the \
         number to match: that is the whole warning, spent."
    );
}

/// What the sweep accumulated, and the subject of every claim once the loops
/// close.
#[derive(Default)]
struct Sweep {
    spilled: Vec<String>,
    crowded: Vec<String>,
    starved: Vec<String>,
    graded: usize,
    /// The tightest FITTING demand and the loosest CROWDED one — the two cells
    /// [`THRESHOLD_CLEARANCE`] is measured over, carried with their names so a
    /// failure can point at them.
    tightest_fitting: (f32, String),
    loosest_crowded: (f32, String),
}

impl Sweep {
    fn new() -> Self {
        Self {
            tightest_fitting: (0.0, String::new()),
            loosest_crowded: (f32::INFINITY, String::new()),
            ..Default::default()
        }
    }

    fn record(&mut self, label: &str, what: &str, r: Reading) {
        self.graded += 1;
        match r.room {
            Room::Starved => self.starved.push(format!("{label} · {what}")),
            Room::Spilled => self.spilled.push(format!("{label} · {what}")),
            Room::Crowded => {
                self.crowded.push(crowded_entry(label, what, r.demand));
                if r.demand < self.loosest_crowded.0 {
                    self.loosest_crowded = (r.demand, format!("{label} · {what}"));
                }
            }
            Room::Fits => {
                if r.demand > self.tightest_fitting.0 {
                    self.tightest_fitting = (r.demand, format!("{label} · {what}"));
                }
            }
        }
    }
}

/// **THE LAW.** Every world the roster ships is asked how much room its
/// workspace footer has, and the answer is graded exactly rather than banded.
#[test]
fn the_workspace_footers_vertical_room_is_asked_of_every_world_the_roster_ships() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the_workspace_footers_vertical_room...: no wgpu adapter");
        return;
    }
    let kinds = enrolled();
    assert!(
        !kinds.is_empty(),
        "no kind enrolled — an enrolment that matches nothing sweeps nothing"
    );
    // THE AMBIENT VALUES, captured rather than derived. `cfg!(target_os = …)`
    // would report the host that COMPILED this test rather than the branch
    // `menubar::platform_default` actually took.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let world_pin = crate::theme::WorldPin::snapshot();
    let cells = grid();

    // ONE PIPELINE for the whole sweep, checked against fresh ones below.
    let Some((device, queue, mut p)) = headless_dqp(64.0, 64.0) else {
        return;
    };

    let mut worlds: Vec<WorldReadings> = Vec::new();
    for kind in &kinds {
        let ov = card_in_content(*kind);
        for wi in 0..crate::theme::THEMES.len() {
            let world = crate::theme::set_active(wi);
            let mut readings = Vec::with_capacity(cells.len());
            for cell in &cells {
                crate::menubar::set_menu_bar_on(cell.menu_bar);
                let what = cell.describe(*kind);
                readings.push(measure(&device, &queue, &mut p, &ov, *cell, &what));
            }
            worlds.push(WorldReadings {
                name: world.name,
                tag: pitch_tag(world.render_caps.list_style),
                readings,
            });
        }
    }

    // GROUP BY WHAT THEY MEASURED, then label each group with the pitch tags its
    // members carry — never the other way round, which would be assuming the
    // answer this sweep exists to find.
    let mut groups: Vec<(Vec<(Room, u32)>, Vec<usize>)> = Vec::new();
    for (i, w) in worlds.iter().enumerate() {
        let sig = w.signature();
        match groups.iter_mut().find(|(s, _)| *s == sig) {
            Some((_, members)) => members.push(i),
            None => groups.push((sig, vec![i])),
        }
    }
    let mut labelled: Vec<(String, &Vec<usize>)> = groups
        .iter()
        .map(|(_, members)| {
            let mut tags: Vec<&str> = members.iter().map(|i| worlds[*i].tag).collect();
            tags.sort_unstable();
            tags.dedup();
            (tags.join("+"), members)
        })
        .collect();
    labelled.sort_by(|a, b| a.0.cmp(&b.0));

    // **THE LABEL IS THE LEDGER'S KEY, SO IT HAS TO IDENTIFY ITS GROUP.** Two
    // groups under one label means two worlds with the same row pitch measured
    // DIFFERENTLY — something other than the pitch is moving this axis, the key
    // no longer names what it ledgers, and the collapse from twenty worlds to a
    // handful of pitches is hiding a member. Name them and stop.
    for pair in labelled.windows(2) {
        assert_ne!(
            pair[0].0,
            pair[1].0,
            "two vertical groups share the label {:?}: {:?} against {:?}. Worlds with the same \
             row pitch no longer agree about the footer's vertical room, so the pitch is not the \
             only thing moving this axis and these ledgers are keyed by a name that does not \
             identify what it grades",
            pair[0].0,
            pair[0].1.iter().map(|i| worlds[*i].name).collect::<Vec<_>>(),
            pair[1].1.iter().map(|i| worlds[*i].name).collect::<Vec<_>>(),
        );
    }
    // AND THE SWEEP FOUND MORE THAN ONE PITCH. A roster that collapsed to a
    // single group would make every claim below a statement about one
    // configuration wearing the clothes of twenty.
    assert!(
        labelled.len() > 1,
        "every world in the roster measured the same vertical room, so this sweep crossed no \
         pitch at all and its ledgers describe one configuration"
    );

    let mut sweep = Sweep::new();
    // The cells that EARNED a ledger entry, re-measured below against pipelines
    // built for them alone.
    let mut audited: Vec<(usize, usize, String)> = Vec::new();
    for (label, members) in &labelled {
        let rep = members[0];
        for (ci, cell) in cells.iter().enumerate() {
            let kind = kinds[rep / crate::theme::THEMES.len()];
            let what = cell.describe(kind);
            let r = worlds[rep].readings[ci];
            if r.room != Room::Fits {
                audited.push((rep, ci, format!("{label} · {what}")));
            }
            sweep.record(label, &what, r);
        }
    }
    assert_the_ledgers_are_unchanged(&sweep.spilled, &sweep.crowded, &sweep.starved);

    // **THE NOTICE THRESHOLD IS NOT ITSELF A KNIFE'S EDGE.** Measured from both
    // sides and reported, because a threshold nobody measures is the same
    // edge-crowding one level up.
    let below = (1.0 - CROWDING) / sweep.tightest_fitting.0;
    let above = sweep.loosest_crowded.0 / (1.0 - CROWDING);
    assert!(
        below >= THRESHOLD_CLEARANCE,
        "{} fits at {:.4} of its card, only {below:.4}x clear of the {:.2} notice line, under a \
         floor of {THRESHOLD_CLEARANCE:.2}x — this cell is about to become a CROWDED entry \
         without anything having moved it",
        sweep.tightest_fitting.1,
        sweep.tightest_fitting.0,
        1.0 - CROWDING,
    );
    assert!(
        above >= THRESHOLD_CLEARANCE,
        "{} is ledgered crowded at {:.4} of its card, only {above:.4}x past the {:.2} notice \
         line, under a floor of {THRESHOLD_CLEARANCE:.2}x — the ledger is about to lose an entry \
         to the threshold rather than to a fix",
        sweep.loosest_crowded.1,
        sweep.loosest_crowded.0,
        1.0 - CROWDING,
    );

    // **THE HOISTED PIPELINE IS CHECKED, NOT TRUSTED.** Every ledgered cell and
    // the tightest fitting one, measured again against a pipeline that has seen
    // no other geometry.
    let mut rechecked = 0usize;
    let mut audit = audited.clone();
    if let Some((rep, ci)) = tightest_seat(&worlds, &labelled, &cells, &sweep) {
        audit.push((rep, ci, sweep.tightest_fitting.1.clone()));
    }
    for (wi, ci, what) in &audit {
        let cell = cells[*ci];
        crate::theme::set_active(*wi % crate::theme::THEMES.len());
        crate::menubar::set_menu_bar_on(cell.menu_bar);
        let kind = kinds[*wi / crate::theme::THEMES.len()];
        let ov = card_in_content(kind);
        let (pw, ph) = (
            (cell.w as f32 * cell.dpi) as u32,
            (cell.h as f32 * cell.dpi) as u32,
        );
        let Some((d2, q2, mut fresh)) = headless_dqp(pw as f32, ph as f32) else {
            return;
        };
        let again = measure(&d2, &q2, &mut fresh, &ov, cell, what);
        let was = worlds[*wi].readings[*ci];
        assert_eq!(
            (again.room, again.demand.to_bits()),
            (was.room, was.demand.to_bits()),
            "{what}: the sweep's shared pipeline read {:?} at {:.6} of the card and a pipeline \
             built for this cell alone read {:?} at {:.6} — the reuse that makes this sweep \
             affordable is carrying state between cells, so every grade above is suspect",
            was.room,
            was.demand,
            again.room,
            again.demand,
        );
        rechecked += 1;
    }
    assert!(
        rechecked >= sweep.spilled.len() + sweep.crowded.len() + sweep.starved.len(),
        "only {rechecked} cells were re-measured against a fresh pipeline, fewer than the {} that \
         earned a ledger entry — the reuse check stopped covering the readings the ledgers rest on",
        sweep.spilled.len() + sweep.crowded.len() + sweep.starved.len()
    );

    drop(world_pin);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);

    eprintln!(
        "workspace back footer, vertical: {} worlds over {} cells each collapsed to {} row \
         pitches ({}); {} graded, {} spilled, {} crowded, {} starved; tightest fitting {:.4} \
         ({:.4}x clear of the {:.2} notice line) at {}; loosest crowded {:.4} ({:.4}x past it) \
         at {}; {rechecked} cells re-measured against fresh pipelines; ambient world {}, ambient \
         menu bar {ambient_menu_bar}",
        worlds.len(),
        cells.len(),
        labelled.len(),
        labelled
            .iter()
            .map(|(l, m)| format!("{l}×{}", m.len()))
            .collect::<Vec<_>>()
            .join(", "),
        sweep.graded,
        sweep.spilled.len(),
        sweep.crowded.len(),
        sweep.starved.len(),
        sweep.tightest_fitting.0,
        below,
        1.0 - CROWDING,
        sweep.tightest_fitting.1,
        sweep.loosest_crowded.0,
        above,
        sweep.loosest_crowded.1,
        crate::theme::THEMES[world_pin_index()].name,
    );
}

/// The (world, cell) the tightest FITTING reading came from, so the reuse check
/// can re-measure the one cell that decides [`THRESHOLD_CLEARANCE`]'s lower arm.
fn tightest_seat(
    worlds: &[WorldReadings],
    labelled: &[(String, &Vec<usize>)],
    cells: &[Cell],
    sweep: &Sweep,
) -> Option<(usize, usize)> {
    for (_, members) in labelled {
        let rep = members[0];
        for ci in 0..cells.len() {
            let r = worlds[rep].readings[ci];
            if r.room == Room::Fits && r.demand == sweep.tightest_fitting.0 {
                return Some((rep, ci));
            }
        }
    }
    None
}

/// The world index the guard will restore to, read for the report only.
fn world_pin_index() -> usize {
    crate::theme::active_index()
}

/// **THE CELLS WHOSE CARD IS TOO SHORT TO LAY THE FOOTER OUT AT ALL** — the
/// vertical half of the corner the width law's `OVERRUN` records, now asked of
/// every row pitch the roster ships rather than of whichever world happened to
/// be active.
///
/// Every one of them is the app's own enforced MINIMUM window
/// (`app::lifecycle`) above 100% zoom, and the pitch decides how far above: the
/// `pane`/`diagonal` worlds only lose the footer at zoom 2 with the menu bar
/// shown, `rules` loses it at zoom 1.4 on one scale, and `bars` — which buys
/// every row a plate gap — loses it at zoom 1.4 in both scales and at zoom 2
/// with the bar hidden as well.
///
/// Membership is never taken on a cell's name: the width law's
/// `assert_the_budget_could_not_hold_it` must agree, against the card's own
/// reported geometry, that the budget could not have held the line.
const STARVED: &[&str] = &[
    "bars · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=off",
    "bars · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=off",
    "bars · settings at 464x288 logical, zoom=1.4, dpi=1, menu_bar=on",
    "bars · settings at 464x288 logical, zoom=1.4, dpi=2, menu_bar=on",
    "bars · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=on",
    "bars · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=on",
    "diagonal+pane · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=on",
    "diagonal+pane · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=on",
    "rules · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=off",
    "rules · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=off",
    "rules · settings at 464x288 logical, zoom=1.4, dpi=2, menu_bar=on",
    "rules · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=on",
    "rules · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=on",
];

/// **THE CELLS WHOSE FOOTER IS DRAWN PAST THE BOTTOM OF ITS OWN CARD** — the
/// same defect [`STARVED`] holds, one degree earlier: the layout still seats the
/// line, and it seats it outside the room.
///
/// The `diagonal+pane` pair is the corner the width law already ledgers under
/// `OVERRUN`, which is the tell that at the tightest cell the footer runs out of
/// the card in BOTH directions at once. The `bars` and `rules` entries are new
/// to this law and could not have been seen without it: they are the cells where
/// a wider row pitch spends the height budget the `pane` worlds still have.
///
/// It is a ledger and not an exclusion for the width law's reason — the fix is a
/// composition question (elide a cell, wrap the line, or refuse the zoom) owned
/// by whoever owns the card's minimum, not by the key the footer names.
const SPILLED: &[&str] = &[
    "bars · settings at 464x288 logical, zoom=1.4, dpi=1, menu_bar=off",
    "bars · settings at 464x288 logical, zoom=1.4, dpi=2, menu_bar=off",
    "bars · settings at 560x480 logical, zoom=2, dpi=1, menu_bar=on",
    "bars · settings at 560x480 logical, zoom=2, dpi=2, menu_bar=on",
    "diagonal+pane · settings at 464x288 logical, zoom=2, dpi=1, menu_bar=off",
    "diagonal+pane · settings at 464x288 logical, zoom=2, dpi=2, menu_bar=off",
    "rules · settings at 464x288 logical, zoom=1.4, dpi=1, menu_bar=on",
    "rules · settings at 560x480 logical, zoom=2, dpi=1, menu_bar=on",
    "rules · settings at 560x480 logical, zoom=2, dpi=2, menu_bar=on",
];

/// **THE CELLS WITH NO VERTICAL MARGIN LEFT**, and how little each has — the
/// named guard on the knife's edge the width law recorded and could not grade.
///
/// The `diagonal+pane` pair IS that cell: the app's enforced minimum window at
/// zoom 1.4 with the menu bar shown, asking 0.9849 and 0.9824 of its card's
/// height. It fits, on every host, exactly — and one changed line metric from
/// not. It is here so that "it got closer" is a red with a number on it rather
/// than a pass, and so that the day it crosses it moves to [`SPILLED`] from a
/// ledger that was already watching it.
///
/// The `rules` pair is the same corner with the menu bar HIDDEN, and it is
/// tighter still at 0.9942 and 0.9982 — 0.18% of a card away from spilling. It
/// was invisible until this law asked the roster.
///
/// Each entry carries its demand floored to [`NOTICE_STEP`], so a cell already
/// in this ledger cannot quietly walk further in.
const CROWDED: &[&str] = &[
    "diagonal+pane · settings at 464x288 logical, zoom=1.4, dpi=1, menu_bar=on @0.980",
    "diagonal+pane · settings at 464x288 logical, zoom=1.4, dpi=2, menu_bar=on @0.980",
    "rules · settings at 464x288 logical, zoom=1.4, dpi=1, menu_bar=off @0.990",
    "rules · settings at 464x288 logical, zoom=1.4, dpi=2, menu_bar=off @0.995",
];

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
//! # The axes, and the ones that produced the misreadings
//!
//! `workspace_is_wide` is a threshold in LOGICAL px over scaled text, so it
//! MOVES with zoom and with the display face's own metrics — a single quoted
//! width is the threshold at one zoom and nowhere else
//! (`workspace_stage_reach`'s module doc records how that turned into a false
//! defect report). So the sweep crosses width × zoom × scale rather than picking
//! a width, and derives its enrolment from the workspace roster instead of
//! naming Settings.
//!
//! **AND THE MENU BAR, which is the axis a workspace card cannot afford to be
//! blind to.** `menubar::platform_default` is the one platform-forked sticky
//! default in the tree, and the workspace card is sized STRAIGHT off the canvas
//! (`plan_workspace_regions`) less that bar's reserve — so the bar is not
//! decoration here, it is a direct subtraction from the card's own height
//! budget. Both laws below sweep it and capture the AMBIENT value to restore,
//! never `cfg!(target_os = …)`, which reports the host that COMPILED the test
//! rather than the branch the initialiser took.
//!
//! # Where the footer is lost, and which loss is whose
//!
//! Two DIFFERENT degradations live at the same corner — the app's own enforced
//! minimum window above 100% zoom — and this file grades them apart because
//! only one of them is about the Back at all:
//!
//!   * **OVERRUN** (horizontal). The footer is drawn, and is wider than the
//!     card. Ledgered in [`OVERRUN`].
//!   * **STARVATION** (vertical). The card's height budget cannot hold the
//!     footer's line, so the shaper emits it and the layout never reaches it —
//!     `overlay_hint_line()` answers `Some`, `layout_runs()` has nothing for it.
//!     Ledgered in [`STARVED`], and only ever accepted after the CARD'S OWN
//!     REPORTED GEOMETRY says the budget could not have held it.
//!
//! Both ledgers are keyed by the menu-bar arm, because the bar decides which
//! of the two a cell lands in: at 464x288 zoom 2 the footer OVERRUNS with the
//! bar off and STARVES with it on, off one and the same defect.
//!
//! # AND THE HORIZONTAL LEDGER IS GRADED IN BANDS, BECAUSE ITS SUBJECT IS NOT
//! THIS MACHINE'S
//!
//! "Is the footer wider than the card" reads like a fact about awl. Half of it
//! is a fact about the HOST: `⌫` — the Back cell's own glyph, U+232B — is
//! carried by NO face in `assets/fonts`, so its advance comes from whatever the
//! system font DB answers with. That is Apple Symbols here and DejaVu Sans on a
//! Debian/Ubuntu runner, and the same sentence measures **2.92% wider** there.
//!
//! Everything else about the comparison is host-identical, and that was measured
//! rather than assumed: across all 94 laid-out cells the card's own reported
//! width agrees to the byte on both hosts, the vertical demand agrees to the
//! byte, and every cell's shaped width differs by ONE common factor (1.0292).
//! The disagreement is a single scalar on a single axis. But four cells sat 2.4%
//! from their card's right edge, so that scalar decided their boolean — and a
//! ledger of exact cell names then said one thing on a Mac and another on CI.
//!
//! So the horizontal grade is a RATIO with a band around the edge
//! ([`HOST_BAND`]), not a boolean:
//!
//!   * demand ≤ 1 − [`HOST_BAND`] — fits, and no host's fallback face can talk
//!     it over the line. Ledgered nowhere; leaving this band reddens.
//!   * within [`HOST_BAND`] of the edge — **TIGHT**. The outcome here is the
//!     host's to decide, so this law declines to grade it and ledgers the
//!     MEMBERSHIP instead, in [`TIGHT`].
//!   * demand > 1 + [`HOST_BAND`] — **OVERRUN**, by more than any substituted
//!     glyph accounts for. Ledgered in [`OVERRUN`].
//!
//! All three ledgers stay exact and two-sided — a cell that arrives reddens, a
//! cell that leaves reddens — so the bands cost nothing in grading power: a
//! footer that grows moves cells DOWN the grades and reddens on the way. What
//! they buy is that which ledger a cell is in stopped being a property of the
//! machine that ran the test. The band is not taken on faith either: the sweep
//! asserts, and reports, that no cell sits within [`GRADE_HEADROOM`] of
//! changing grade.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};
use crate::overlay::workspace::{BackKey, WorkspaceShape};
use crate::overlay::{OverlayKind, OverlayState};

/// A LOCAL LUMINANCE STEP big enough to be a glyph edge rather than a gradient —
/// the same threshold the foot-hint pixel law uses, for the same reason: card
/// grounds and washes move slowly, type does not.
const GLYPH_STEP: f32 = 24.0;

/// **HOW CLOSE TO ITS CARD'S EDGE A FOOTER MAY SIT BEFORE THIS LAW STOPS
/// GRADING ITS OUTCOME** — the module doc's third grade, sized off the measured
/// spread rather than picked.
///
/// The one host-dependent quantity in the comparison is the shaped width of the
/// footer, and it is host-dependent for one reason: no bundled face carries
/// `⌫` (U+232B), so the Back cell's advance is whatever the system font DB
/// supplies. Measured, the same sentence shapes **2.92%** wider on Ubuntu 24.04
/// against DejaVu Sans than on macOS against Apple Symbols — one common factor
/// across every cell, with the card's own width and the whole vertical axis
/// agreeing to the byte.
///
/// This band is ~3.4× that observed spread, which is deliberate: DejaVu is one
/// substitute out of many, and a host with no system fonts at all substitutes a
/// different width again. It is NOT a tolerance on the product — a cell inside
/// it is ledgered by name in [`TIGHT`] and cannot arrive or leave unnoticed.
const HOST_BAND: f32 = 0.10;

/// **THE BAND'S OWN NON-VACUITY.** A grade whose members crowd its edge is a
/// grade the next runner image re-decides, which is the exact failure
/// [`HOST_BAND`] exists to end — so the sweep requires every cell to stand this
/// factor clear of the nearest boundary it did NOT cross, and reports the
/// tightest it measured.
///
/// 1.05 sits under the measured worst on both hosts (1.086 on this Mac, 1.094
/// against DejaVu), so it is a floor with room rather than a restatement of
/// today's numbers.
const GRADE_HEADROOM: f32 = 1.05;

fn luma(p: [u8; 4]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// THE ENROLLED KINDS, asked of the roster rather than named: every kind that
/// `workspace_shape()` claims and whose own rows live in the CONTENT pane —
/// which is the stage this Back is reached from, and which is asked through
/// `rows_are_primary()`, the one owner, rather than by naming a shape variant.
/// Today that is Settings; a second such member enrols itself.
pub(super) fn enrolled() -> Vec<OverlayKind> {
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
pub(super) fn card_in_content(kind: OverlayKind) -> OverlayState {
    let mut ov = OverlayState::new(
        kind,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_facet_lens(0);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    journey.toggle_detail();
    journey.card().expect("the card is up").clone()
}

/// The card projected the way `App::sync_view` projects it — every workspace
/// field read off the kind's own owners, never written as a literal.
pub(super) fn content_view(ov: &OverlayState) -> ViewState {
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

/// **WHY AN EMPTY FOOTER IS EMPTY, ASKED OF THE CARD ITSELF.** The shaper emits
/// the hint into `panel_buffer` unconditionally; the LAYOUT then stops at the
/// buffer's height, which is exactly `geom.card_h`. So a footer can be planned,
/// be present in the buffer's lines, and still have no run — and where the
/// budget genuinely could not hold it, drawing nothing is the only thing left to
/// do, so asserting against it would be asserting against arithmetic.
///
/// The excuse is granted only against the card's OWN REPORTED GEOMETRY, never
/// against a list of small widths. The SMALLEST composition this stage must
/// stack before its footer is the header lines it planned, the ONE candidate row
/// the flat family's `min_items` floor guarantees, and the blank the shared gap
/// owner reserves — charging the row plan's ACTUAL length instead would let a
/// card excuse a lost footer by planning too many rows. The card's own padding
/// is deliberately left out of the sum, which can only make the bound harder to
/// meet.
pub(super) fn assert_the_budget_could_not_hold_it(
    p: &TextPipeline,
    geom: &crate::render::chrome::OverlayGeom,
    what: &str,
) {
    let floor_rows = geom.header_rows + 1 + crate::render::chrome::overlay_hint_gap_rows(1);
    let need = floor_rows as f32 * p.overlay_lh() + p.overlay_hint_h() + geom.header_gap;
    assert!(
        geom.card_h < need,
        "{what}: the footer is shaped but never laid out, while the card's own {:.1}px \
         height budget holds the {need:.1}px its shortest possible composition needs \
         ({floor_rows} lines at {:.1}px, a {:.1}px hint and a {:.1}px header gap) — this \
         footer was lost by the shaper, not by the canvas",
        geom.card_h,
        p.overlay_lh(),
        p.overlay_hint_h(),
        geom.header_gap,
    );
}

/// **DID TYPE REACH THE FRAME?** Renders the prepared pipeline and counts, over
/// the footer's own shaped band, the canvas columns carrying a glyph edge —
/// `(inked columns, band width, x0, x1)`. The facts the law checks around this
/// are a state oracle; this is the only one that asks the pixels.
#[allow(clippy::too_many_arguments)]
fn footer_ink(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &TextPipeline,
    pw: u32,
    ph: u32,
    left: f32,
    top: f32,
    ink_w: f32,
    height: f32,
) -> (usize, usize, i64, i64) {
    let (texture, tview) = offscreen(device, pw, ph);
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl workspace back footer"),
    });
    p.render(&mut enc, &tview).unwrap();
    queue.submit(Some(enc.finish()));
    let px = read_pixels(device, queue, &texture, pw, ph);
    let lum: Vec<f32> = px.iter().map(|q| luma(*q)).collect();
    let half = (height * 0.5 - 1.0).max(1.0);
    let mid = top + height * 0.5;
    let y0 = ((mid - half) as i64).max(0);
    let y1 = ((mid + half) as i64).min(ph as i64 - 2);
    let x0 = (left as i64).max(0);
    let x1 = ((left + ink_w) as i64).min(pw as i64 - 2);
    let inked = (x0..x1)
        .filter(|x| {
            (y0..y1).any(|y| {
                let i = (y * pw as i64 + x) as usize;
                (lum[i] - lum[i + pw as usize]).abs() > GLYPH_STEP
                    || (lum[i] - lum[i + 1]).abs() > GLYPH_STEP
            })
        })
        .count();
    (inked, (x1 - x0).max(0) as usize, x0, x1)
}

/// One swept cell's coordinates, and the key its failures and both ledgers are
/// reported in. The MENU BAR is part of that key and not a footnote: it is
/// subtracted from the card's height budget, so a cell's classification is not
/// well defined without it.
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

/// Logical windows the sweep crosses: the app's OWN enforced minimum (derived
/// from the same metrics `app::lifecycle` enforces, so a change to either moves
/// this cell with it), a ladder through the staging threshold, and comfortably
/// wide. The threshold's own value is deliberately never written down — it moves
/// with zoom and with the display face, and a law that pinned it would be
/// testing this machine.
pub(super) fn windows() -> Vec<(u32, u32)> {
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

/// Where one cell's footer LANDED — the ledger it earned, or none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Landed {
    /// The card could not lay the line out at all — [`STARVED`].
    Starved,
    /// Past the card's edge by more than a substituted glyph — [`OVERRUN`].
    Overrun,
    /// Within [`HOST_BAND`] of the edge, so the outcome is the host's — [`TIGHT`].
    Tight,
    /// Clear of the edge by more than any host can move it. Ledgered nowhere.
    Fits,
}

/// **THE HORIZONTAL GRADE, AND THE MARGIN IT WAS DECIDED BY** — the module doc's
/// three bands, as one pure function so the boundary arithmetic has one owner
/// and the sweep cannot grade two cells by two rules.
///
/// The second return is how far this cell stood from the boundary it did NOT
/// cross, as the FACTOR its footer would have to change by to be graded
/// differently. A cell inside the band is measured against both edges, because
/// leaving in either direction moves it between ledgers. That number is what
/// [`GRADE_HEADROOM`] floors.
fn grade_the_fit(demand: f32) -> (Landed, f32) {
    if demand > 1.0 + HOST_BAND {
        (Landed::Overrun, demand / (1.0 + HOST_BAND))
    } else if demand > 1.0 - HOST_BAND {
        let to_edges = ((1.0 + HOST_BAND) / demand).min(demand / (1.0 - HOST_BAND));
        (Landed::Tight, to_edges)
    } else {
        (Landed::Fits, (1.0 - HOST_BAND) / demand)
    }
}

/// **THE SHAPED SENTENCE, AND THE TWO CLAIMS THAT ARE ABOUT IT RATHER THAN ITS
/// GEOMETRY.** The footer's own SHAPED line is the subject, because a stage that
/// plans no footer (the staging regime's other stage does exactly that) shapes
/// none, and a sentence read off the card rather than off the frame would agree
/// with itself at every width for free.
fn the_drawn_sentence(
    p: &TextPipeline,
    ov: &OverlayState,
    back: BackKey,
    what: &str,
) -> (usize, String) {
    let line = p.overlay_hint_line().unwrap_or_else(|| {
        panic!(
            "{what}: the content stage shaped no footer at all, so the Back it teaches is \
             unreadable exactly where it is needed most"
        )
    });
    let drawn = p.panel_buffer.lines[line].text().to_string();
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
    (line, drawn)
}

/// **THE PRESENCE FLOOR, SET UNDER THE ROSTER'S TIGHTEST REAL VALUE**, returning
/// the share it measured.
///
/// A handful of inked columns is a floor a nearly invisible footer clears; the
/// sentence is a whole line of type, so the honest question is what SHARE of its
/// own shaped band carries a glyph edge. The tightest cell in this sweep inks 68%
/// of its band and the loosest 83%, so half is a real constraint with room for a
/// shaper's antialiasing to differ across backends — and the measured tightest is
/// reported by the law so the headroom stays visible.
#[allow(clippy::too_many_arguments)]
fn assert_the_footer_is_a_line_of_type(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &TextPipeline,
    pw: u32,
    ph: u32,
    seat: (f32, f32, f32, f32),
    what: &str,
) -> f32 {
    let (left, top, ink_w, height) = seat;
    let (inked, band, x0, x1) = footer_ink(device, queue, p, pw, ph, left, top, ink_w, height);
    assert!(
        band >= 4,
        "{what}: the footer's ink band clamped to {band} canvas columns ({x0}..{x1}) — there \
         is nothing here for a pixel floor to grade"
    );
    assert!(
        inked * 2 >= band,
        "{what}: only {inked} of the footer's own {band} band columns ({x0}..{x1}) carry a \
         glyph edge — the Back cell is planned, shaped and seated, but what reached the frame \
         is not a line of type"
    );
    inked as f32 / band as f32
}

/// What one swept cell contributed to the law.
struct CellOutcome {
    landed: Landed,
    drawn: String,
    /// The share of its card\'s width this footer asked for — the graded
    /// quantity, carried so a failure can print the number it graded.
    demand: f32,
    /// The factor this cell stood clear of its grade's boundary — see
    /// [`grade_the_fit`]. Meaningless, and so infinite, for a starved cell.
    headroom: f32,
    /// The ink share, where the cell was graded for ink at all. A cell past its
    /// card's edge is not: there is no band on the canvas to grade.
    ink_share: Option<f32>,
}

/// **ONE CELL, MEASURED AND GRADED.** Split out of the law because the sweep's
/// five nested axes and its final ledger comparison are two different readings,
/// and a reader chasing "how is a cell graded" should not have to walk past the
/// roster to find out.
fn grade_one_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &TextPipeline,
    ov: &OverlayState,
    back: BackKey,
    (pw, ph): (u32, u32),
    what: &str,
) -> CellOutcome {
    let geom = p.workspace_geometry(pw);
    let (line, drawn) = the_drawn_sentence(p, ov, back, what);
    let run = p.panel_buffer.layout_runs().find_map(|run| {
        (run.line_i == line).then_some((run.line_w, geom.text_top + run.line_top, run.line_height))
    });
    // A CARD TOO SHORT FOR ITS OWN COMPOSITION loses its footer to the layout,
    // and is LEDGERED — but only after the card's own geometry says the budget
    // could not have held it (`assert_the_budget_could_not_hold_it`).
    let Some((ink_w, top, height)) = run else {
        assert_the_budget_could_not_hold_it(p, &geom, what);
        return CellOutcome {
            landed: Landed::Starved,
            drawn,
            demand: 0.0,
            headroom: f32::INFINITY,
            ink_share: None,
        };
    };
    assert!(
        ink_w > 0.0 && height > 0.0,
        "{what}: the footer shaped to nothing ({ink_w}x{height})"
    );
    let [cx, cy, cw, ch] = p.workspace_regions(pw).card;
    // WHERE THE FOOTER STARTS is a seating claim and is asserted outright: a
    // line that begins off its own card is misplaced, not merely too big for
    // the room.
    assert!(
        geom.text_left >= cx && top >= cy,
        "{what}: the footer's ink box starts at ({:.1},{:.1}), outside the card \
         ({cx:.1},{cy:.1} {cw:.1}x{ch:.1})",
        geom.text_left,
        top
    );
    // WHETHER IT FITS is LEDGERED, not asserted — see `OVERRUN` and `TIGHT`.
    //
    // HORIZONTALLY that is graded as a RATIO in three bands (`HOST_BAND`),
    // because the footer's shaped width is partly the host's and a boolean at
    // the card's edge is therefore not this product's to state. VERTICALLY it
    // stays a boolean, because the whole vertical axis was measured identical on
    // both hosts — the line's pitch is a metric the pipeline sets, not one a
    // substituted glyph gets a vote in. Nothing here bands it, and nothing
    // should: a band buys tolerance for a variation this axis cannot have.
    //
    // What this clause CANNOT see is how close a cell that fits is to not
    // fitting, and it reads only the world that happens to be active when it
    // runs — which is one of three row pitches the roster ships. Both are
    // `super::workspace_back_height`'s subject; its ledgers are the wider set,
    // and this clause is the safety net under them.
    let demand = ink_w / (cx + cw - geom.text_left);
    let (mut landed, headroom) = grade_the_fit(demand);
    if top + height > cy + ch + 0.5 {
        landed = Landed::Overrun;
    }
    // A TIGHT CELL IS NOT GRADED FOR FIT — but it is still graded for INK
    // wherever this host's own faces did seat it, because the presence floor is
    // a claim about type reaching the frame and owes nothing to the card's
    // right edge.
    let seated = landed != Landed::Overrun && geom.text_left + ink_w <= cx + cw + 0.5;
    let ink_share = seated.then(|| {
        let seat = (geom.text_left, top, ink_w, height);
        assert_the_footer_is_a_line_of_type(device, queue, p, pw, ph, seat, what)
    });
    CellOutcome {
        landed,
        drawn,
        demand,
        headroom,
        ink_share,
    }
}

/// **THE THREE LEDGERS, COMPARED.** Each is an exact set and each is two-sided:
/// a cell that arrives is a NEW degradation, a cell that has left has been fixed
/// and its entry is stale. Split out so the sweep above reads as a sweep.
fn assert_the_ledgers_are_unchanged(overrun: &[String], tight: &[String], starved: &[String]) {
    // STARVATION IS REPORTED BEFORE OVERRUN, because a card that starves is
    // never graded for width — so a change that pushes cells from one ledger
    // into the other reddens BOTH, and the absent footer is the more useful of
    // the two things to be told about first.
    assert_eq!(
        starved, STARVED,
        "the set of cells whose card is too short to lay its footer out at all changed. A \
         cell that is here and not in STARVED is a NEW loss of the Back — fix it. A cell in \
         STARVED that is no longer here has been fixed — delete its entry rather than leave \
         a ledger that grades nothing."
    );
    assert_eq!(
        overrun, OVERRUN,
        "the set of cells whose footer runs past the card's right edge by more than any \
         substituted glyph accounts for changed. A cell that is here and not in OVERRUN is a \
         NEW overrun — fix it. A cell in OVERRUN that is no longer here has been fixed — \
         delete its entry rather than leave a ledger that grades nothing. (A cell that merely \
         crossed the edge is in TIGHT, not here — see `HOST_BAND`.)"
    );
    assert_eq!(
        tight, TIGHT,
        "the set of cells sitting within a substituted glyph's reach of the card's right edge \
         changed. A cell that is here and not in TIGHT has MOVED toward its card's edge — that \
         is the footer growing, and it is the thing this law is for, whichever side of the edge \
         this particular machine's fonts happened to land it on. A cell in TIGHT that is no \
         longer here has moved away from the edge — delete its entry. Do NOT resolve this by \
         adding the cell: `TIGHT` is not an allow-list for one host's font set, it is the list \
         of places the product has no margin left."
    );
}

/// **EVERYTHING THE SWEEP ACCUMULATED**, and the subject of every claim the law
/// makes once the loops close. Gathered into one owner so the claims can live
/// next to each other and be read as a set: several of them are only worth
/// having because another one is there (invariance needs the threshold crossing,
/// the ledgers need the aggregate presence floor, the bands need the headroom).
#[derive(Default)]
struct Sweep {
    sentences: std::collections::BTreeSet<String>,
    backs: std::collections::BTreeSet<&'static str>,
    overrun: Vec<String>,
    tight: Vec<String>,
    starved: Vec<String>,
    staged: usize,
    wide: usize,
    graded: usize,
    inked_cells: usize,
    /// The tightest INK SHARE any drawn footer reached, reported at the end so a
    /// reader can see how much headroom the presence floor actually had.
    tightest: f32,
    /// THE NARROWEST MARGIN ANY CELL HAD ON ITS OWN GRADE, with the cell that
    /// set it so the failure can name it — the subject of [`GRADE_HEADROOM`].
    closest_call: (f32, String),
}

impl Sweep {
    fn new() -> Self {
        Self {
            tightest: f32::INFINITY,
            closest_call: (f32::INFINITY, String::new()),
            ..Default::default()
        }
    }

    /// File one cell's outcome under the ledger it earned.
    fn record(&mut self, what: String, out: CellOutcome) {
        self.sentences.insert(out.drawn);
        self.graded += 1;
        if out.headroom < self.closest_call.0 {
            self.closest_call = (out.headroom, format!("{what} at demand {:.4}", out.demand));
        }
        match out.landed {
            Landed::Starved => self.starved.push(what),
            Landed::Overrun => self.overrun.push(what),
            Landed::Tight => self.tight.push(what),
            Landed::Fits => {}
        }
        if let Some(share) = out.ink_share {
            self.tightest = self.tightest.min(share);
            self.inked_cells += 1;
        }
    }

    /// **WHAT THE SWEEP PROVED, AND WHAT IT COVERED WHILE PROVING IT.**
    fn assert_and_report(&self, kinds: &[OverlayKind], ambient_menu_bar: bool) {
        let (staged, wide, graded, inked_cells) =
            (self.staged, self.wide, self.graded, self.inked_cells);
        // THE SWEEP CROSSED THE THRESHOLD, and says so. Without this the
        // invariance below is a statement about one regime wearing the clothes
        // of two.
        assert!(
            staged > 0 && wide > 0,
            "the sweep never crossed the staging threshold (staged {staged}, wide {wide}) \
             across {graded} cells over {:?} — one regime went ungraded, so the \
             width-invariance claim is about nothing",
            kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(
            self.sentences.len(),
            1,
            "the footer taught {} different sentences across {graded} cells — the whole point \
             of a Back the action seam derives without a width is that a staged layout and a \
             wide one cannot disagree: {:?}",
            self.sentences.len(),
            self.sentences
        );
        assert_eq!(
            self.backs,
            std::collections::BTreeSet::from([BackKey::Erase.glyph()]),
            "the enrolled roster's content panes must all teach the erase key as their Back — \
             the focus key is the fallback for a live query, and no cell here types one"
        );
        assert_the_ledgers_are_unchanged(&self.overrun, &self.tight, &self.starved);
        // **AND THE BANDS ARE NOT A KNIFE'S EDGE OF THEIR OWN.** The grade above
        // is only worth more than the boolean it replaced if no cell is one
        // runner image away from being graded differently — the same crowding,
        // moved one level up. So the margin is measured over EVERY cell,
        // floored, and printed.
        assert!(
            self.closest_call.0 >= GRADE_HEADROOM,
            "{} is only {:.4}x from being graded differently, under a floor of \
             {GRADE_HEADROOM:.2}x — the three-way grade has developed the same edge-crowding \
             the boolean had, and which ledger this cell lands in is about to become a \
             property of the host again",
            self.closest_call.1,
            self.closest_call.0,
        );
        // **THE PRESENCE FLOOR IN AGGREGATE.** All three ledgers are exact sets,
        // so none can grow quietly — but an exact set says nothing about how
        // much of the sweep is left over to be a law about. This is the
        // statement that the degradations stay a CORNER, and a change that
        // pushed the inked share below two thirds would have made the ledgers
        // the product rather than the exception.
        //
        // The count is deliberately NOT pinned: a TIGHT cell is inked wherever
        // this host's faces seated it, so the exact number is one of the few
        // things here that legitimately differs between machines. It is printed.
        assert!(
            inked_cells * 3 >= graded * 2,
            "only {inked_cells} of {graded} swept cells drew an inked footer inside their card \
             ({} overrun, {} tight, {} starved) — the ledgered corners have grown into the \
             sweep, so the presence claim is now about a minority of the product",
            self.overrun.len(),
            self.tight.len(),
            self.starved.len()
        );
        // WHAT THIS RUN ACTUALLY COVERED, printed rather than assumed — the
        // grade headroom especially, because it is the number that says whether
        // the bands are still doing their job on THIS host's font set.
        eprintln!(
            "workspace back footer: {graded} cells over both menu-bar arms (ambient here is \
             {ambient_menu_bar}), {inked_cells} inked, {} overrun, {} tight, {} starved; \
             tightest ink share {:.3} against a floor of 0.500; closest grade call {:.4}x \
             against a floor of {GRADE_HEADROOM:.2}x, at {}",
            self.overrun.len(),
            self.tight.len(),
            self.starved.len(),
            self.tightest,
            self.closest_call.0,
            self.closest_call.1,
        );
    }
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

    // THE AMBIENT MENU-BAR VALUE, captured rather than derived. A
    // `cfg!(target_os = ...)` here would report the host that COMPILED this
    // test, not the branch `menubar::platform_default` actually took.
    let ambient_menu_bar = crate::menubar::menu_bar_on();

    let mut sweep = Sweep::new();

    for kind in &kinds {
        let ov = card_in_content(*kind);
        let back = ov
            .detail_back()
            .expect("the content pane must have a Back to be invariant about");
        sweep.backs.insert(back.glyph());
        for menu_bar in [false, true] {
            crate::menubar::set_menu_bar_on(menu_bar);
            for (lw, lh) in windows() {
                for zoom in [1.0f32, 1.4, 2.0] {
                    for dpi in [1.0f32, 2.0] {
                        let cell = Cell {
                            w: lw,
                            h: lh,
                            zoom,
                            dpi,
                            menu_bar,
                        };
                        let (pw, ph) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                        let Some((device, queue, mut p)) = headless_dqp(pw as f32, ph as f32)
                        else {
                            return;
                        };
                        p.set_dpi(dpi);
                        p.set_size(pw as f32, ph as f32);
                        let mut v = content_view(&ov);
                        v.zoom = zoom;
                        p.set_view(&v);
                        p.prepare(&device, &queue, pw, ph).unwrap();

                        match p.workspace_is_wide(pw) {
                            true => sweep.wide += 1,
                            false => sweep.staged += 1,
                        }
                        let what = cell.describe(*kind);
                        let out = grade_one_cell(&device, &queue, &p, &ov, back, (pw, ph), &what);
                        sweep.record(what, out);
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    sweep.assert_and_report(&kinds, ambient_menu_bar);
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
///
/// **SIXTEEN OF THE ORIGINAL EIGHTEEN CELLS ARE GONE**, and they left for the
/// composition reason the paragraph above predicted rather than by exclusion:
/// the rows line dropped its `←/→ category` cell when the horizontal keys became
/// the region seam's, so the line the card has to hold is one cell shorter
/// everywhere. What survives is the tightest corner alone — the enforced minimum
/// window at zoom 2, where three cells still do not fit.
///
/// Keyed by the MENU-BAR ARM, because the bar decides whether a cell at this
/// corner overruns or starves: the two 464x288 zoom-2 cells are here only with
/// the bar off, and in [`STARVED`] with it on.
///
/// **AND THE MEMBERSHIP TEST IS A BAND, NOT THE EDGE** ([`HOST_BAND`]): these
/// two demand 1.24 and 1.27 of their card's width on the two hosts measured, so
/// nothing a system font DB can substitute for `⌫` moves them. A cell that
/// merely crossed the edge belongs in [`TIGHT`].
const OVERRUN: &[&str] = &[
    "settings at 464x288 logical, zoom=2, dpi=1, menu_bar=off",
    "settings at 464x288 logical, zoom=2, dpi=2, menu_bar=off",
];

/// **THE CELLS WITH NO MARGIN LEFT** — where the footer comes within
/// [`HOST_BAND`] of its card's right edge, so which side of it the line lands on
/// is decided by the host's own font DB rather than by awl.
///
/// These four are the 560x480 zoom-2 cells, in BOTH menu-bar arms — which is
/// itself the tell that the bar is not the variable. They demand **97.7%** of
/// their card's width when `⌫` (U+232B) is drawn from Apple Symbols and
/// **100.6%** of it when the same glyph is drawn from DejaVu Sans, because no
/// bundled face carries that codepoint at all. Ledgering the boolean pinned one
/// runner image's font set as the product's truth and went red on the other.
///
/// It is a LEDGER OF THE SAME DEFECT [`OVERRUN`] holds, one degree earlier, and
/// not a list of blessed exceptions: a footer that grows arrives here first, and
/// arriving reddens. The composition fix that empties [`OVERRUN`] empties this
/// too.
const TIGHT: &[&str] = &[
    "settings at 560x480 logical, zoom=2, dpi=1, menu_bar=off",
    "settings at 560x480 logical, zoom=2, dpi=2, menu_bar=off",
    "settings at 560x480 logical, zoom=2, dpi=1, menu_bar=on",
    "settings at 560x480 logical, zoom=2, dpi=2, menu_bar=on",
];

/// **THE CELLS WHERE THE CARD IS TOO SHORT TO LAY THE FOOTER OUT AT ALL** — the
/// VERTICAL half of the same corner [`OVERRUN`] records, ledgered the same
/// two-sided way and for the same reason: it is an existing defect this law
/// found and does not own.
///
/// It is the app's own enforced MINIMUM window at zoom 2 WITH THE MENU BAR
/// SHOWN, and nowhere else. The bar's reserve is subtracted straight from the
/// card (`plan_workspace_regions`), which at that cell leaves a 160.8px card
/// against a 54.4px row pitch — under three lines, where the shortest
/// composition this stage can draw is four (the `settings › query` line, the one
/// candidate row the flat family's floor guarantees, the blank the shared gap
/// owner reserves, and the hint). With the bar OFF the same cell has 232.0px and
/// the footer is drawn — running past the card's right edge, which is why it
/// appears in [`OVERRUN`] under `menu_bar=off` and here under `menu_bar=on`. It
/// is ONE defect wearing two faces, not two.
///
/// Membership here is never taken on a cell's name: the law requires the card's
/// own reported budget to be smaller than its own shortest composition before it
/// will accept an empty footer at all.
const STARVED: &[&str] = &[
    "settings at 464x288 logical, zoom=2, dpi=1, menu_bar=on",
    "settings at 464x288 logical, zoom=2, dpi=2, menu_bar=on",
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
    // The AMBIENT value, captured rather than derived — see the module doc.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let (mut graded, mut skipped) = (0usize, 0usize);
    for kind in enrolled() {
        let ov = card_in_content(kind);
        let shipped = ov.foot_hint();
        let was = shipped.replace(
            &format!("{} back", BackKey::Erase.glyph()),
            &format!("{} back", BackKey::Focus.glyph()),
        );
        assert_ne!(
            shipped,
            was,
            "{}: the substitution matched nothing, so this measures one sentence twice",
            kind.as_str()
        );
        for menu_bar in [false, true] {
            crate::menubar::set_menu_bar_on(menu_bar);
            for (lw, lh) in windows() {
                for zoom in [1.0f32, 1.4, 2.0] {
                    let Some((device, queue, mut p)) = headless_dqp(lw as f32, lh as f32) else {
                        return;
                    };
                    let what = format!(
                        "{} at {lw}x{lh} zoom={zoom}, menu_bar={}",
                        kind.as_str(),
                        if menu_bar { "on" } else { "off" }
                    );
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
                                .find_map(|run| (run.line_i == line).then_some(run.line_w)),
                        );
                    }
                    // A card too short to lay its footer out measures NEITHER
                    // sentence — the vertical starvation the headline law
                    // ledgers in `STARVED`. The two must fall out TOGETHER: one
                    // laid out and the other not would be a width compared
                    // against nothing, which is the one way this comparison
                    // could go quietly vacuous.
                    assert_eq!(
                        widths[0].is_none(),
                        widths[1].is_none(),
                        "{what}: one of the two sentences was laid out and the other was not \
                         ({:?} against {:?}) — they share a card and a line, so this is a \
                         defect in the shaper rather than a cell to skip",
                        widths[0],
                        widths[1]
                    );
                    let (Some(shipped_w), Some(was_w)) = (widths[0], widths[1]) else {
                        skipped += 1;
                        continue;
                    };
                    assert!(
                        shipped_w <= was_w,
                        "{what}: `{} back` shapes {shipped_w:.1}px, wider than the `{} back` \
                         it replaced ({was_w:.1}px) — naming the Back must not cost the \
                         footer width it does not have",
                        BackKey::Erase.glyph(),
                        BackKey::Focus.glyph(),
                    );
                    graded += 1;
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    // 47 of this sweep's 48 cells compare two real shaped widths; the floor sits
    // under that rather than at it, so a face whose metrics move a cell over the
    // starvation edge does not redden here.
    assert!(
        graded >= 40,
        "the comparison must actually run, got {graded} (and {skipped} cells whose card was \
         too short to lay either sentence out)"
    );
    // AND THE SKIPPING STAYS BOUNDED BY THE HEADLINE LAW'S OWN LEDGER: this
    // sweep runs at one scale, so it can only ever meet a subset of `STARVED`.
    // A comparison that quietly stopped comparing would show up here first.
    assert!(
        skipped <= STARVED.len(),
        "{skipped} cells could not be compared at all, more than the {} the headline law \
         ledgers as starved — the footer is being lost somewhere that ledger does not know \
         about",
        STARVED.len()
    );
}

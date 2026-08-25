//! THE SUMMONED WORKSPACE'S ACTIVE RAIL ENTRY MUST BE READABLE ON ITS OWN BAND.
//!
//! The rail's fill and label ink resolve as one pair in
//! `chrome::overlay_visual_sel`. Its final glyph buffer has one more ownership
//! constraint: footer fitting must use separate measurement scratch. Reusing the
//! already-final rail buffer replaces every category with the footer sentence;
//! on Potoroo the pixel oracle then reads its light footer ink on the gold active
//! band as 2.30:1 instead of the selected label owner's dark 6.90:1 answer.
//!
//! The headline pixel law uses real frames and geometry; its companion grep law
//! keeps band and ink resolution in the single visual-selection owner.
//! # THE ORACLE: ONE RECT, PHOTOGRAPHED TWICE
//! Every figure below is a comparison of **the same rail entry's own rect, with
//! its mark and without it** — the first frame stands on category 0, the second
//! on category 1, and both are measured at row 0. Nothing is compared to an
//! authored colour: the same Wagtail plate is `[255,255,255]` on this host's
//! Metal and need not be on another backend, and a byte-exact assertion on a
//! rendered pixel has taken a gating CI job red before.
//!
//! That pairing is also what makes the floors mean anything, because a contrast
//! floor over a band is **satisfiable by deleting its own subject**: a plate
//! faded to the card's own colour reports a *better* label-vs-fill ratio than
//! the shipped one, since the label is then simply sitting on the card. So the
//! ratio is held with two PRESENCE floors beside it:
//!
//! * the **BAND** is really there — marked minus unmarked, as a perceptual
//!   distance, on the worlds whose rail draws a plate at all;
//! * the **LABEL** is really there — the same glyphs must lay down a comparable
//!   amount of ink ON the plate as they do OFF it, which is precisely the
//!   quantity the reported defect drove to zero.
//!
//! Measuring the label as a SHARE of its own rect would not have worked, and the
//! attempt is recorded here so it is not retried: a staged-narrow rail is the
//! whole card wide, so the identical three-letter label falls from 5% of its
//! rect to 0.7% of it with nothing wrong at all. Against its own unmarked twin
//! the figure is a ratio near 1 at every canvas.

use super::super::*;
use super::headless_dqp;
use super::pixeldiff::{delta_e, render_frame};
use super::workspace::{workspace_card, workspace_view};

/// **THE LABEL-ON-BAND CONTRAST FLOOR, calibrated from three real readings.**
///
/// * the tightest SHIPPED reading across the swept roster: **3.07:1** (Potoroo,
///   rail UNFOCUSED, wide — the dimmed plate is where every world reads
///   tightest). Every run reports its own figure, so this can be re-read rather
///   than trusted;
/// * the reading the reported defect produces: **1.00:1** — Wagtail's white
///   label on Wagtail's white plate, the ratio of a colour with itself, and the
///   degenerate value of the measure;
/// * the floor between them, **2.6:1**.
///
/// It sits deliberately just below `theme::SELECTED_ROW_INK_CONTRAST_FLOOR`
/// (3.0, the bar the derive owner *chooses* ink against) rather than at it: this
/// is a measurement of RENDERED pixels through a card backing, a texture and a
/// dither, and holding a rendered composite to the exact number the palette
/// maths targets would fail on a rounding step rather than on a defect.
const INK_CONTRAST_FLOOR: f64 = 2.6;

/// **THE LABEL-PRESENCE FLOOR — how much ink the active label lays on its plate,
/// as a fraction of what the SAME label lays on the bare rail.**
///
/// Three readings. Tightest shipped: **0.97** (Potoroo, rail unfocused, wide —
/// and the same figure in BOTH menu-bar arms, to every digit). The defect:
/// **0.00**, exactly — the label contributes no pixel that differs from its plate
/// by any amount at all. Floor: **0.35**, a third of the tightest real reading
/// and unreachable by the defect. Every run reports its own tightest figure, per
/// arm.
///
/// A fraction rather than a count, because the same claim has to hold at three
/// canvas geometries and two pixel densities where the absolute count moves by
/// more than an order of magnitude.
///
/// ⚠️ **A LOW READING HERE IS AS LIKELY TO BE THE GROUND ESTIMATOR AS THE
/// PRODUCT, and the ratio's DENOMINATOR is the half that gets corrupted.** This
/// floor was first calibrated at 0.76 against a mode-based
/// [`column_grounds`] and later read 0.29 on Firetail with the menu bar drawn —
/// neither figure was about the label. Both were the BARE reference rect
/// mistaking the label's own ink for its ground and counting card pixels as
/// marks, which inflates the denominator and can only ever push this ratio DOWN.
/// So the first question a failure here has to answer is whether the NUMERATOR
/// moved: the marked rect's own mark count is in the message, and if it is
/// unchanged while the bare one has grown, the defect is in the measurement.
const LABEL_PRESENCE_FLOOR: f64 = 0.35;

/// **THE BAND-PRESENCE FLOOR — the perceptual distance between the active
/// entry's fill and the same rect's fill when that entry is NOT active.** This
/// is what stops the contrast floor from being satisfiable by fading the plate
/// into the card.
///
/// CIE ΔE rather than a WCAG ratio, because PRESENCE is the question: two light
/// tones a reader separates easily can sit at a contrast ratio of 1.07, and a
/// floor in those units would have to be set so near 1.0 that a plate four bytes
/// from the card would pass it.
///
/// The rail draws its plate at TWO strengths and each gets its own floor,
/// because the reduced one is a declared degradation (`workspace::
/// UNFOCUSED_MARK_ALPHA`, 0.34 — the same rect insisting less, DESIGN.md §5) and
/// holding it to the focused figure would fail the mechanism rather than a bug.
/// Tightest shipped readings: **ΔE 14.66** focused (Tawny, wide) and **ΔE 4.49**
/// dimmed (Bilby, wide). Both floors sit under half of their arm's own tightest
/// real value, and both are far above the ~0 a plate faded into the card gives.
/// Every run reports both figures.
///
/// ΔE ≈ 2.3 is the classic just-noticeable difference, so the DIMMED floor is
/// deliberately below it: the unfocused mark is *meant* to sit at the edge of
/// noticing, and a law demanding more would be arguing with the design rather
/// than guarding it.
const BAND_PRESENCE_FLOOR_FOCUSED: f64 = 6.0;
const BAND_PRESENCE_FLOOR_DIMMED: f64 = 2.0;

/// A pixel counts as MARK rather than as its fill's own anti-aliasing at this
/// luminance departure. The premise measurement of the reported defect cleared
/// it with zero pixels out of 2052.
const MARK_LUMA_STEP: f64 = 0.05;

/// **AND a mark has to be a fraction of the deepest ink the rect itself
/// contains** — the second half of the mark test, and the reason it is not one
/// absolute number.
///
/// A world may print a TEXTURE on the card its rail sits on (Quokka's halftone
/// dot lattice; Wagtail's own stipple on the highlight) and those dots clear an
/// absolute luminance step easily. Measured on Quokka's staged-narrow rail, an
/// absolute-only test counted 2174 "label" pixels in a rect whose label is three
/// letters — the dots, not the label, and the ratio it produced was a statement
/// about how much of the lattice each frame happened to cover.
///
/// So a mark is graded against the rect's OWN deepest ink (its 99.5th percentile
/// of departure, not its single most extreme pixel), which is scale-free, needs
/// no authored constant, and degenerates correctly on the defect: when the label
/// is the colour of its plate there is no deep ink to be a fraction of, and the
/// absolute step above admits nothing.
const MARK_RELATIVE_SHARE: f64 = 0.4;

/// The share of the MARK POPULATION (not of the rect) whose colour is averaged
/// into "the ink": the most-deviating fifth of the pixels that departed at all,
/// which is a glyph's stems rather than its edges.
///
/// Taking a percentile of the RECT instead is the trap this constant exists to
/// avoid — it lands on a stem at one canvas and in pure anti-aliasing at
/// another, and reported eight worlds below a legibility floor they were nowhere
/// near violating.
const INK_CORE_SHARE: f64 = 0.2;

/// The canvases the item names — one comfortably WIDE (both regions drawn) and
/// one below `MIN_PANE_CHARS`, where the workspace STAGES its regions and the
/// rail becomes the whole card — each at BOTH pixel densities, because the plate
/// and the glyph are resolved by different machinery and only a second density
/// asks whether they still land on each other.
///
/// ⚠️ The staging threshold is a LOGICAL width, so a 2x cell has to state twice
/// the device canvas to be the same stage: `2800x1800 @2` is the wide cell and
/// `1400x900 @2` is the NARROW one. Reading `1400x900 @2` as "the wide canvas at
/// 2x" is what a first cut of this sweep did, and it silently graded the narrow
/// stage twice while claiming a density axis it did not have.
const CELLS: &[(u32, u32, f32)] = &[
    (1400, 900, 1.0),
    (700, 900, 1.0),
    (2800, 1800, 2.0),
    (1400, 900, 2.0),
];

fn rel_lum(px: [u8; 4]) -> f64 {
    let f = |v: u8| {
        let c = v as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(px[0]) + 0.7152 * f(px[1]) + 0.0722 * f(px[2])
}

fn contrast(a: [u8; 4], b: [u8; 4]) -> f64 {
    let (la, lb) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// What one rail entry's rect actually contains, read off a rendered frame.
struct RectInk {
    /// The rect's own GROUND — the mean of its per-column background tones,
    /// which is the plate when one is drawn and the card when none is.
    fill: [u8; 4],
    /// The ground local to the ink found below: the mean of the column
    /// backgrounds of the very pixels [`RectInk::ink`] was averaged from. On a
    /// flat rect this equals `fill`; on a rect crossed by a gradient it is the
    /// tone the label is actually sitting on, which is the one a contrast claim
    /// is about.
    local_fill: [u8; 4],
    /// The mean of the [`INK_CORE_SHARE`] most-deviant marks — the label's stems,
    /// when there is a label. Equal to `local_fill` when nothing departed.
    ink: [u8; 4],
    /// How many pixels counted as marks.
    marks: usize,
    /// How many pixels were graded.
    n: usize,
}

/// **THE GROUND UNDER EACH COLUMN IS THE MEDIAN OF A LOCAL POOL OF PIXELS — NOT
/// A MODE, AND NOT A STATISTIC BUILT ON ONE.** Three corrections, each earned by
/// a world that broke the previous cut, and all three recorded because every one
/// of them is a way this rect gets read against a tone it is not sitting on:
///
/// * ONE COLOUR FOR THE WHOLE RECT assumes a flat ground and the roster has not
///   got one. Paperbark's card carries a broad horizontal gradient, so a rect
///   whose label sits in its darker left third reported that label against the
///   lighter tone filling its right two thirds — 1.95:1 for a rail a person reads
///   without effort. So the ground is resolved LOCALLY, per column.
/// * ONE COLUMN ALONE then fails the other way, on a SHORT row: a rail entry at
///   1x is about eighteen usable pixels tall and a cap-height stem fills twelve
///   of them, so a lone column's own statistic IS the stem. Bilby reported its
///   label against itself at 1.13:1. So each column's ground is resolved over a
///   WINDOW of neighbouring columns — wide enough that a stem is a minority of
///   it, narrow enough that a card-wide gradient is flat across it: half a row
///   height each side, which scales with the type and so holds at both densities.
/// * **A MODE NEEDS A REPEATED COLOUR, AND A DITHERED CARD UNDER A WASH HAS
///   NONE.** This is the correction that matters, because the two above were both
///   built ON a per-column mode and the mode itself was the defect. Firetail's
///   card is dithered and its wash runs diagonally, so a 33-pixel column of pure
///   card holds **28 to 30 distinct colours** and the winning "mode" is a count
///   of two or three. That is not a modal colour, it is the tally's TIE-BREAK —
///   and breaking ties on the colour value resolves toward the numerically
///   largest RGB, which on a dark world is the label's own ink. Measured on
///   Firetail's bare rail at 1400x900: seven consecutive columns took
///   `[159,126,124]` (the label) as their ground over the card's `[25,9,13]`, on
///   counts of three against three, and the neighbourhood median went with them —
///   so every card pixel in twenty columns counted as a MARK and the bare
///   reference's ink rose from 158 pixels to 548. Nothing had to change for that
///   but the sub-pixel row of the wash the rect was sampled at, which is exactly
///   what drawing a MENU BAR does to this card.
///
/// So the window's pixels are POOLED and the ground is their median by
/// luminance. A median needs no repeated value, so a dither and a wash cost it
/// nothing; a glyph is a minority of a pool one row tall and thirty-odd columns
/// wide, so type cannot become the ground; a texture lattice is a minority for
/// the same reason; and a filled plate covers the pool entirely, so on a marked
/// rect the median IS the plate, which is the tone a contrast claim there is
/// about.
///
/// The window SLIDES rather than truncating at the rect's edges, so the leftmost
/// column — the one the label starts under — is graded over as many pixels as a
/// middle one instead of over a short window the label can dominate.
///
/// Each pixel's luminance is computed ONCE, alongside its colour, rather than
/// inside the selection's comparator — `rel_lum` is three `powf`s, and a median
/// asks its comparator O(n log n) times over a thousand-pixel pool per column.
fn column_grounds(px: &[[u8; 4]], w: u32, x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<[u8; 4]> {
    let cols: Vec<Vec<([u8; 4], f64)>> = (x0..x1)
        .map(|x| {
            (y0..y1)
                .map(|y| {
                    let c = px[y as usize * w as usize + x as usize];
                    (c, rel_lum(c))
                })
                .collect()
        })
        .collect();
    let win = (((y1 - y0) / 2).max(3)) as usize;
    let span = (2 * win + 1).min(cols.len());
    let mut pool: Vec<([u8; 4], f64)> = Vec::with_capacity(span * (y1 - y0) as usize);
    (0..cols.len())
        .map(|i| {
            let lo = i.saturating_sub(win).min(cols.len() - span);
            pool.clear();
            for col in &cols[lo..lo + span] {
                pool.extend_from_slice(col);
            }
            let mid = pool.len() / 2;
            pool.select_nth_unstable_by(mid, |a, b| a.1.total_cmp(&b.1))
                .1
                .0
        })
        .collect()
}

fn mean_rgb(it: impl Iterator<Item = [u8; 4]>) -> [u8; 4] {
    let mut acc = [0f64; 3];
    let mut n = 0f64;
    for c in it {
        for k in 0..3 {
            acc[k] += c[k] as f64;
        }
        n += 1.0;
    }
    if n == 0.0 {
        return [0, 0, 0, 255];
    }
    [
        (acc[0] / n).round() as u8,
        (acc[1] / n).round() as u8,
        (acc[2] / n).round() as u8,
        255,
    ]
}

/// Read one rail-entry rect out of a rendered frame, inset far enough to clear
/// the plate's own rounded corner and its edge anti-aliasing at both densities.
fn rect_ink(px: &[[u8; 4]], w: u32, h: u32, rect: [f32; 4], dpi: f32) -> RectInk {
    let inset = (2.0 * dpi).round() as i64;
    let x0 = (rect[0].round() as i64 + inset).max(0);
    let y0 = (rect[1].round() as i64 + inset).max(0);
    let x1 = ((rect[0] + rect[2]).round() as i64 - inset).min(w as i64);
    let y1 = ((rect[1] + rect[3]).round() as i64 - inset).min(h as i64);
    if x1 <= x0 || y1 <= y0 {
        let black = [0, 0, 0, 255];
        return RectInk {
            fill: black,
            local_fill: black,
            ink: black,
            marks: 0,
            n: 0,
        };
    }
    let grounds = column_grounds(px, w, x0, y0, x1, y1);
    let fill = mean_rgb(grounds.iter().copied());
    // Every pixel against ITS OWN column's ground.
    let mut departures: Vec<([u8; 4], [u8; 4], f64)> = Vec::new();
    for (i, x) in (x0..x1).enumerate() {
        let g = grounds[i];
        let gl = rel_lum(g);
        for y in y0..y1 {
            let c = px[y as usize * w as usize + x as usize];
            let d = (rel_lum(c) - gl).abs();
            if d > MARK_LUMA_STEP {
                departures.push((c, g, d));
            }
        }
    }
    departures.sort_by(|a, b| b.2.total_cmp(&a.2));
    // The rect's own deepest ink, robustly: the 99.5th percentile of departure
    // rather than the single most extreme sample.
    let deepest = departures
        .get(departures.len() / 200)
        .map(|&(_, _, d)| d)
        .unwrap_or(0.0);
    departures.retain(|&(_, _, d)| d >= deepest * MARK_RELATIVE_SHARE);
    let marks = departures.len();
    let n = ((x1 - x0) * (y1 - y0)) as usize;
    if marks == 0 {
        return RectInk {
            fill,
            local_fill: fill,
            ink: fill,
            marks: 0,
            n,
        };
    }
    let k = ((marks as f64 * INK_CORE_SHARE).round() as usize).clamp(1, marks);
    RectInk {
        fill,
        local_fill: mean_rgb(departures.iter().take(k).map(|&(_, g, _)| g)),
        ink: mean_rgb(departures.iter().take(k).map(|&(c, _, _)| c)),
        marks,
        n,
    }
}

/// Whether THIS world's rail lays a filled plate under its active entry — read
/// off the same no-wildcard branch `chrome::workspace_rail::prepare_rail_mark`
/// takes, so the enrolment is derived from the production decision rather than
/// pinned to a named world (and a fifth list style has to decide here too).
///
/// It is deliberately a question about DATA, not about the frame: a gate keyed
/// on a rendered luminance delta enrols a different set of worlds on a different
/// GPU, which makes the graded set a property of the backend.
fn rail_draws_a_plate() -> bool {
    match crate::render::effective_list_style() {
        theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Diagonal(_) => true,
        // A ruled list refuses a filled band — its rail is marked by rules, so
        // there is no plate for a label to be lost in and nothing to hold a
        // band-presence floor against. The two floors below still apply, and on
        // this arm they read as "the label against the card", which is the right
        // claim when nothing is drawn under it.
        theme::ListStyle::Ruled(_) => false,
    }
}

/// Render the Settings workspace standing on category `lens` and return the
/// frame together with the rail's FIRST entry rect.
fn frame_with_lens(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    lens: usize,
    detail: bool,
    (w, h, dpi): (u32, u32, f32),
) -> (Vec<[u8; 4]>, Option<[f32; 4]>, Option<String>) {
    let ov = workspace_card(lens, detail);
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    p.set_view(&workspace_view(&ov));
    p.prepare(device, queue, w, h).unwrap();
    let rect = p.workspace_rail_probe(w).rows.first().copied().flatten();
    let first_label = p
        .workspace_rail_buffer
        .lines
        .first()
        .map(|line| line.text().to_string());
    (render_frame(p, device, queue, w, h), rect, first_label)
}

/// **THE HEADLINE LAW.** On every world in the roster, at a wide canvas, at the
/// staged-narrow canvas, at two pixel densities, and with focus in EITHER of the
/// workspace's two regions, the Settings rail's active category must be legible
/// on whatever the rail draws under it.
///
/// The subject is the REAL Settings workspace — the same `OverlayState`
/// `overlay::build`'s Settings arm produces, folded into a `ViewState` the way
/// `App::sync_view` does — rendered through a real pipeline on a real device.
/// Enrolment is DERIVED: a cell is graded iff the frame actually resolved that
/// rail rect, which is what silently drops the narrow stage that shows the rows
/// pane instead of the rail, and both the world count and the cell count are
/// asserted so the sweep cannot pass by enrolling nothing.
///
/// BOTH focus states are swept because the rail draws its plate at two presences
/// and a rule that only holds at full strength leaves half of the product's own
/// time unheld — and because the DIMMED state is where the obvious alternative
/// fix (leave the ink alone, it only looks wrong at full strength) fails too.
///
/// **AND BOTH MENU-BAR ARMS.** `menubar::platform_default` is the one
/// platform-forked sticky default in the tree — `false` on macOS, `true`
/// everywhere else — and the workspace card is sized straight off the canvas less
/// that bar's reserve, so the bar moves this rect by most of a row. It is not a
/// cosmetic axis here: the reserve re-samples the card's diagonal wash under the
/// rail, and reading that wash is what [`column_grounds`] exists to do. The
/// AMBIENT value is captured and restored, never `cfg!(target_os = …)`, which
/// reports the host that COMPILED the test rather than the branch the initialiser
/// took.
/// The figures ONE graded cell yields — the unit this law is actually made of:
/// one rail rect, photographed with its mark and without it, and the three
/// claims read off that pair.
struct CellReading {
    /// The rail rect this cell resolved, so the sweep can ask whether an axis
    /// it toggled moved anything.
    rect: [f32; 4],
    /// Marks on the plate as a fraction of marks on the bare rail.
    presence: f64,
    /// The label's ink against the tone it is actually sitting on.
    contrast: f64,
    /// The plate's perceptual distance from the same rect unmarked — `None` on
    /// the worlds whose rail draws no plate for there to be a distance from.
    band: Option<f64>,
}

/// **GRADE ONE CELL, AND ASSERT ITS THREE CLAIMS.** Returns `None` when this
/// cell draws no rail at all — the narrow stage that shows the rows pane instead
/// — which is the DERIVED enrolment: a cell is graded iff the frame actually
/// resolved that rail rect, rather than iff a list of canvases said it would.
fn grade_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    at: &str,
    detail: bool,
    cell: (u32, u32, f32),
) -> Option<CellReading> {
    let (w, h, dpi) = cell;
    // The SAME rect, twice: standing on category 0 (row 0 is the active entry,
    // marked) and on category 1 (row 0 is an ordinary entry, bare). Everything
    // below is one against the other.
    let (marked, rect_a, label_a) = frame_with_lens(device, queue, p, 0, detail, cell);
    let (bare, rect_b, label_b) = frame_with_lens(device, queue, p, 1, detail, cell);
    let (rect, rect_bare) = (rect_a?, rect_b?);
    assert_eq!(
        (label_a.as_deref(), label_b.as_deref()),
        (Some("All"), Some("All")),
        "{at}: the first rail row must keep its `All` label in both selected states; \
         another workspace measurement overwrote it ({label_a:?} vs {label_b:?})"
    );
    assert!(
        rect.iter()
            .zip(rect_bare.iter())
            .all(|(a, b)| (a - b).abs() < 0.5),
        "{at}: the rail's first entry moved between the two frames ({rect:?} vs \
         {rect_bare:?}) — this law's whole oracle is that they are the same rect"
    );

    let m = rect_ink(&marked, w, h, rect, dpi);
    let u = rect_ink(&bare, w, h, rect, dpi);
    assert!(
        m.n > 200,
        "{at}: the rail mark rect {rect:?} graded only {} pixels — this cell cannot \
         say anything about legibility",
        m.n
    );

    // (1) THE LABEL IS THERE — the same glyphs, on the plate and off it. The
    // floor the reported defect drove to exactly zero.
    assert!(
        u.marks > 0,
        "{at}: the UNMARKED reference rect carries no label ink at all, so this \
         cell's presence ratio would be a division by the wrong thing"
    );
    let presence = m.marks as f64 / u.marks as f64;
    assert!(
        presence >= LABEL_PRESENCE_FLOOR,
        "{at}: the active rail entry lays {} mark pixels on its own band where the \
         SAME label lays {} on the bare rail — {presence:.2} of it (floor \
         {LABEL_PRESENCE_FLOOR}). Its category name is not being drawn on its band. \
         Band fill {:?}, most-deviant ink found {:?}.",
        m.marks,
        u.marks,
        m.fill,
        m.ink
    );

    // (2) IT READS. Rendered fill against rendered ink, never against an
    // authored colour.
    let contrast = contrast(m.local_fill, m.ink);
    assert!(
        contrast >= INK_CONTRAST_FLOOR,
        "{at}: the active rail entry's label {:?} on the band tone it is actually \
         sitting on {:?} = {contrast:.2}:1 (floor {INK_CONTRAST_FLOOR}:1) — the \
         category name washes into the mark that is supposed to be pointing at it. \
         The rect's mean ground is {:?}.",
        m.ink,
        m.local_fill,
        m.fill
    );

    // (3) THE BAND IS THERE — so (2) cannot be satisfied by fading the plate
    // into the card. Graded only on the worlds whose rail draws a plate, with
    // the arm named in the message.
    let band = rail_draws_a_plate().then(|| {
        let seen = delta_e(m.fill, u.fill);
        let floor = match detail {
            true => BAND_PRESENCE_FLOOR_DIMMED,
            false => BAND_PRESENCE_FLOOR_FOCUSED,
        };
        assert!(
            seen >= floor,
            "{at}: this rect's fill is {:?} when its entry is active and {:?} when it \
             is not — ΔE {seen:.2} (floor {floor}). This world's rail ({:?}) is \
             supposed to lay a plate under its active category, and a legibility ratio \
             measured over a plate that is not there is a ratio measured over the card",
            m.fill,
            u.fill,
            crate::render::effective_list_style()
        );
        seen
    });

    Some(CellReading {
        rect,
        presence,
        contrast,
        band,
    })
}

/// The tightest reading of one floor and the cell it came from.
type Worst = Option<(String, f64)>;

fn keep_tightest(slot: &mut Worst, at: &str, seen: f64) {
    if slot.as_ref().is_none_or(|&(_, v)| seen < v) {
        *slot = Some((at.to_string(), seen));
    }
}

/// **GRADE ONE MENU-BAR ARM**, over the whole roster × canvases × focus states.
/// Returns the arm's own summary line and every rail rect it resolved.
///
/// The tightest reading of every floor is tracked and reported PER ARM — the
/// three-figure calibration each constant is documented with, so a reader can see
/// what this run actually covered instead of trusting a number written down once,
/// and so an arm cannot borrow the other arm's coverage.
fn grade_menu_bar_arm(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    menu_bar: bool,
    worlds_seen: &mut Vec<&'static str>,
) -> (String, Vec<[f32; 4]>) {
    crate::menubar::set_menu_bar_on(menu_bar);
    let (mut worst_ink, mut worst_label) = (Worst::None, Worst::None);
    let (mut worst_band_focused, mut worst_band_dimmed) = (Worst::None, Worst::None);
    let mut rects: Vec<[f32; 4]> = Vec::new();
    let mut plated = 0usize;

    for th in theme::THEMES.iter() {
        theme::set_active_by_name(th.name).unwrap();
        p.sync_theme();
        for &cell in CELLS {
            let (w, h, dpi) = cell;
            for detail in [false, true] {
                let at = format!(
                    "{} {w}x{h}@{dpi}x {} menu-bar-{}",
                    th.name,
                    if detail {
                        "rail-unfocused"
                    } else {
                        "rail-focused"
                    },
                    if menu_bar { "on" } else { "off" }
                );
                let Some(r) = grade_cell(device, queue, p, &at, detail, cell) else {
                    continue;
                };
                rects.push(r.rect);
                if !worlds_seen.contains(&th.name) {
                    worlds_seen.push(th.name);
                }
                keep_tightest(&mut worst_ink, &at, r.contrast);
                keep_tightest(&mut worst_label, &at, r.presence);
                if let Some(seen) = r.band {
                    plated += 1;
                    let slot = match detail {
                        true => &mut worst_band_dimmed,
                        false => &mut worst_band_focused,
                    };
                    keep_tightest(slot, &at, seen);
                }
            }
        }
    }

    // NON-VACUITY, PER ARM. The sweep is the roster x 4 cells x 2 focus
    // states, less the two NARROW rows-focused stages, which draw no rail at
    // all (the pane takes the card there). The counts below are FLOORS, not
    // equalities, so growing the roster or adding a canvas does not turn this
    // into a bookkeeping test.
    let graded = rects.len();
    assert!(
        graded >= 100 && plated >= 90,
        "with the menu bar {}, the sweep graded {graded} cells ({plated} of them \
         plated) — it is not covering the roster it claims to",
        if menu_bar { "on" } else { "off" }
    );
    (
        format!(
            "menu bar {}: {graded} cells graded, {plated} plated\n    \
             tightest ink contrast   {worst_ink:?}\n    \
             tightest label presence {worst_label:?}\n    \
             tightest band (focused) {worst_band_focused:?}\n    \
             tightest band (dimmed)  {worst_band_dimmed:?}",
            if menu_bar { "ON " } else { "OFF" }
        ),
        rects,
    )
}

#[test]
fn the_active_rail_entrys_label_reads_on_its_own_band_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping rail_ink_law: no wgpu adapter");
        return;
    };
    let saved = theme::active().name;
    let ambient_menu_bar = crate::menubar::menu_bar_on();

    let mut worlds_seen: Vec<&'static str> = Vec::new();
    // ONE SUMMARY PER MENU-BAR ARM, never one total across both. A total is
    // satisfied by a single arm doing all the work and reports the OTHER arm's
    // tightest reading nowhere — which is the shape that let this axis go
    // unmeasured until CI found it.
    let (report_off, rects_off) =
        grade_menu_bar_arm(&device, &queue, &mut p, false, &mut worlds_seen);
    let (report_on, rects_on) = grade_menu_bar_arm(&device, &queue, &mut p, true, &mut worlds_seen);

    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    theme::set_active_by_name(saved).unwrap();
    p.sync_theme();

    // The WORLD count is exact, because the roster IS the enrolment.
    assert_eq!(
        worlds_seen.len(),
        theme::THEMES.len(),
        "the sweep graded {} of {} worlds — it enrolled {worlds_seen:?}",
        worlds_seen.len(),
        theme::THEMES.len()
    );
    // **AND THE TWO ARMS ARE ACTUALLY TWO.** A sweep over a knob that moves
    // nothing is a sweep in name only, and `set_menu_bar_on` is exactly the kind
    // of setter that can be inert. The bar's reserve is subtracted from the card's
    // own height budget, so it MUST move this rail — measured, not assumed.
    // `assert!` rather than `assert_ne!`, which would print both 120-rect vectors
    // and bury the sentence that says what went wrong.
    assert!(
        rects_off != rects_on,
        "the menu-bar arms resolved identical rail geometry at all {} graded cells \
         (both starting {:?}) — the bar's reserve is supposed to come out of the \
         workspace card's own height budget, so either it no longer does or this \
         sweep is toggling nothing",
        rects_off.len(),
        rects_off.first()
    );
    eprintln!(
        "rail_ink_law: {} worlds, both menu-bar arms.\n  {report_off}\n  {report_on}",
        worlds_seen.len()
    );
}

/// **THE PAIR STAYS A PAIR.** The fill an overlay's selected/active row is drawn
/// in, and the ink that has to read on it, are two halves of one decision. They
/// came apart because four chrome sites each re-derived the fill from
/// `theme::highlight_treatment(effective_overlay_selrow_band())` while only some
/// of them asked `overlay_visual_sel` for the ink — so a fifth site could take
/// the fill and invent its own ink, which is precisely what the rail did.
///
/// `chrome::overlay_visual_sel` is now the one owner of both, and this is the
/// grep that keeps it that way: no other file under `render/chrome/` may name
/// `effective_overlay_selrow_band` at all. NO WILDCARD — the allowed set is a
/// single named file, so a new consumer fails here rather than quietly growing a
/// second answer.
#[test]
fn only_the_visual_selection_owner_resolves_the_overlay_selected_band() {
    const OWNER: &str = "overlay_visual_sel.rs";
    const BAND: &str = "effective_overlay_selrow_band";
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render/chrome");
    let mut offenders: Vec<String> = Vec::new();
    let mut owner_seen = false;
    let mut scanned = 0usize;
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("render/chrome is readable") {
            let path = entry.expect("a readable dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            scanned += 1;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("a utf-8 file name")
                .to_string();
            let src = std::fs::read_to_string(&path).expect("a readable source file");
            if !src.contains(BAND) {
                continue;
            }
            if name == OWNER {
                owner_seen = true;
            } else {
                offenders.push(name);
            }
        }
    }
    assert!(
        scanned > 20,
        "the sweep walked only {scanned} files under render/chrome — it is not looking \
         where it thinks it is"
    );
    assert!(
        owner_seen,
        "no file under render/chrome names `{BAND}` — either the owner moved or this law \
         is now measuring nothing"
    );
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "`{BAND}` is the SELECTED BAND's own resolution and belongs to `{OWNER}` alone, \
         beside the ink that has to read on it. These files reach it directly and can \
         take a fill without taking its ink: {offenders:?}"
    );
}

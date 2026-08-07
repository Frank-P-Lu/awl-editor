//! THE CARD-INK VETO — A ONE-DIRECTIONAL ORACLE, AND THE ONE DIRECTION IT ANSWERS.
//!
//! Every pixel law over a crisp picker's frost needs the card's OWN drawing out of the
//! way: the card's rows and its ring are sharp by design, so a measurement of the
//! DOCUMENT that includes them measures the card instead. [`CardInk`] is that exclusion,
//! DERIVED rather than enumerated — render the same picker over an EMPTY document, and a
//! strong local luma step in THAT frame is the card's own drawing, because what the card
//! draws over there is a blur of a blank page and a blur has no step in it.
//!
//! # WHAT IT CANNOT ANSWER — AND WHY THE TYPE HAS NO OTHER METHOD
//!
//! "A blur of a blank page" holds ONLY WHERE THE FROST REACHES. Outside the frost's own
//! shape the empty frame shows the world's live ground at full sharpness — a lava lamp, a
//! dot field, a wallpaper — and every step in that ground is flagged too. The flagged set
//! is therefore a SUPERSET of the card's ink whose surplus is a property of the WORLD, not
//! of the card. Two readings follow, and only one of them is sound:
//!
//! * **AS A VETO — "skip this pixel" — it is safe, in the conservative direction.** A
//!   superset can only shrink a measured set; it can never admit card ink into one. What
//!   it costs is sample size, and that cost is the world's ground rather than anything the
//!   frost did: measured on this roster, the veto removes up to ~14% of a collar just
//!   outside the card on one world and 0% on another at the same geometry. **So every
//!   consumer owes a PRESENCE guard on how many pixels survived the veto** — without one,
//!   a law goes quietly vacuous on a busy-ground world while staying green on a calm one.
//! * **AS AN INCLUSION SET — "this is where the card IS" — it is false, and the error is
//!   not small.** It flags thousands of pixels outside the card's own box, reaching tens of
//!   logical px above the card's top edge — the canvas's very first row, at 1×. The law
//!   below measures both directions and prints every cell's figures.
//! * **INTERSECTING IT WITH A PICKER-OPEN-VERSUS-CLOSED DIFFERENCE DOES NOT RESCUE IT.**
//!   The card casts a soft SHADOW well past its own box, so the intersection selects
//!   ground structure lying under the shadow — thousands of pixels on the worlds that back
//!   their card, and double digits on the world that draws no backing. A shadow is a wash,
//!   not ink, and nothing owes it a frost. **The near-clean world is why a one-world check
//!   cannot find this class**, and the law below reports the quietest member for that
//!   reason.
//!
//! A claim about where the card IS comes from the box a PRODUCTION owner declares —
//! `TextPipeline::overlay_card_rect`, `TextPipeline::overlay_head_band_ink`,
//! `blur::extent::footprint_box` — never from inverting this mask. [`CardInk`] exposes no
//! indexing, no iterator and no count for exactly that reason: both oracles that were
//! built on the inverted reading began by ENUMERATING the flagged set, and the type no
//! longer lets a caller reach it. The one enumeration in the tree is the law in this
//! module, which is what keeps the paragraphs above measured rather than remembered.

use super::super::*;
use super::frost_feather_item312::{DENSE, enrolled_worlds, render_frame, theme_picker};
use super::{headless_dqp, view_md};

/// A local luma step (of 255) that only an EDGE produces — the empty frame's backdrop is
/// a blur, whose successive pixels differ by a fraction of a neighbourhood's range, so
/// this sits in a very wide measured valley rather than on a tuned boundary (this tree's
/// frosted residue peaks near 5 and its sharp residue near 190).
pub(super) const INK_GRADIENT: f32 = 6.0;

/// How far the flagged set is grown, in physical px per DPI unit: a glyph's anti-aliased
/// skirt reaches a pixel or two past the gradient that betrays it.
const INK_DILATE: i64 = 2;

/// The coverage a law over the card's own upright chrome requires of the frost beneath it
/// — the level under which "the frost does not really reach here" begins. Used by this
/// module's law only to say WHERE the veto's premise has failed.
const COVERAGE_FLOOR: f32 = 0.9;

pub(super) fn luma(p: [u8; 4]) -> f32 {
    0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
}

/// The largest of the right/down luma steps at `(x, y)` in a `w`×`h` luma field.
pub(super) fn step(field: &[f32], w: i64, h: i64, x: i64, y: i64) -> f32 {
    let at = |x: i64, y: i64| field[(y * w + x) as usize];
    let here = at(x, y);
    let mut g = 0.0f32;
    if x + 1 < w {
        g = g.max((here - at(x + 1, y)).abs());
    }
    if y + 1 < h {
        g = g.max((here - at(x, y + 1)).abs());
    }
    g
}

/// THE CARD'S OWN INK, DERIVED — and a VETO ONLY. Read this module's header before using
/// it: the flagged set is a superset of the card's drawing, its surplus is the world's own
/// ground, and it does not invert into "where the card is".
///
/// The field is private and there is no accessor but [`CardInk::vetoes`], so the flagged
/// set cannot be enumerated outside this module. That is the point, not an oversight.
pub(super) struct CardInk {
    flags: Vec<bool>,
    w: i64,
    h: i64,
}

impl CardInk {
    /// Derive the veto from a frame of the same picker over an EMPTY document. `dpi`
    /// scales the dilation, so a caller at 2× must pass 2.0 or the skirt goes unswallowed.
    pub(super) fn derive(empty: &[[u8; 4]], w: i64, h: i64, dpi: f32) -> Self {
        let lum: Vec<f32> = empty.iter().map(|p| luma(*p)).collect();
        let dilate = INK_DILATE * dpi.round() as i64;
        let mut flags = vec![false; (w * h) as usize];
        for y in 0..h - 1 {
            for x in 0..w - 1 {
                if step(&lum, w, h, x, y) < INK_GRADIENT {
                    continue;
                }
                for dy in -dilate..=dilate {
                    for dx in -dilate..=dilate {
                        let (xx, yy) = (x + dx, y + dy);
                        if (0..w).contains(&xx) && (0..h).contains(&yy) {
                            flags[(yy * w + xx) as usize] = true;
                        }
                    }
                }
            }
        }
        Self { flags, w, h }
    }

    /// Must a measurement of the DOCUMENT skip this pixel? The only question this oracle
    /// can answer. A `true` means "the card may be drawing here, or the world's ground
    /// is"; it does not mean "the card is here".
    pub(super) fn vetoes(&self, x: i64, y: i64) -> bool {
        if !(0..self.w).contains(&x) || !(0..self.h).contains(&y) {
            return false;
        }
        self.flags[(y * self.w + x) as usize]
    }
}

/// What one cell of the census below measured.
struct Census {
    /// Flagged pixels inside the card's own box, and the box's area: the PRESENCE half.
    inside: (u64, u64),
    /// Flagged pixels outside the card's own box — the inclusion reading's own error.
    outside: u64,
    /// How far the flagged set reaches ABOVE the card's top edge, in LOGICAL px.
    reach: f32,
    /// Flagged AND changed by the picker opening AND under [`COVERAGE_FLOOR`] of frost:
    /// the intersect-with-a-difference rescue, which selects the card's SHADOW.
    shadow: u64,
}

/// THE ONE ENUMERATION OF THE FLAGGED SET IN THE TREE. It lives beside the definition
/// because the field is private here — a consumer cannot write this function, which is the
/// mechanism that keeps the inverted reading out of the callers.
fn census(
    ink: &CardInk,
    frames: (&[[u8; 4]], &[[u8; 4]]),
    card: [f32; 4],
    frost: crate::render::blur::Frost,
    dpi: f32,
) -> Census {
    let (open, closed) = frames;
    let [rx, ry, rw, rh] = card;
    let (mut inside, mut outside, mut shadow) = (0u64, 0u64, 0u64);
    let mut top = ry;
    for y in 0..ink.h {
        for x in 0..ink.w {
            let i = (y * ink.w + x) as usize;
            if !ink.flags[i] {
                continue;
            }
            let (fx, fy) = (x as f32, y as f32);
            if fx >= rx && fx < rx + rw && fy >= ry && fy < ry + rh {
                inside += 1;
                continue;
            }
            outside += 1;
            top = top.min(fy);
            let covered = crate::render::blur::footprint_mask_for(frost, dpi, fx, fy);
            if open[i] != closed[i] && covered < COVERAGE_FLOOR {
                shadow += 1;
            }
        }
    }
    Census {
        inside: (inside, (rw * rh) as u64),
        outside,
        reach: (ry - top) / dpi,
        shadow,
    }
}

/// One cell's three frames — the picker over prose, the picker over an EMPTY document
/// (the veto's own input), and the same prose with no picker at all — plus the card's own
/// box and the frost the frame SENT THE SHADER, both read while the picker's view is still
/// the one set: they are properties of that view, and the third frame closes the picker.
struct Cell {
    open: Vec<[u8; 4]>,
    empty: Vec<[u8; 4]>,
    closed: Vec<[u8; 4]>,
    card: [f32; 4],
    frost: crate::render::blur::Frost,
}

fn cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    (w, h): (u32, u32),
) -> Cell {
    p.set_view(&theme_picker(DENSE));
    let open = render_frame(device, queue, p, w, h);
    let card = p
        .overlay_card_rect()
        .expect("the crisp picker is open, so it has a card box");
    let frost = p.frost_mode().expect("an enrolled world reaches the frost");
    p.set_view(&theme_picker(""));
    let empty = render_frame(device, queue, p, w, h);
    let mut plain = view_md(DENSE, 0, 0);
    plain.overlay_active = false;
    p.set_view(&plain);
    let closed = render_frame(device, queue, p, w, h);
    Cell {
        open,
        empty,
        closed,
        card,
        frost,
    }
}

/// The flagged fraction inside the card's box that keeps every downstream veto from being
/// a veto over nothing. Set well under the roster's measured floor, which the law reports.
const INSIDE_PRESENCE: f32 = 0.05;

/// THE CONTRACT, MEASURED: THE VETO'S TRUES ARE NOT THE CARD'S INK — IN BOTH DIRECTIONS.
///
/// Two clauses, and each refuses one way the module header could rot:
///
/// 1. **PRESENCE.** Inside the card's own box the veto flags a real share of the pixels.
///    Without this the header's "safe as a veto" is satisfied by a mask that flags
///    nothing, and every consumer's exclusion would be an exclusion of nothing.
/// 2. **UNSOUNDNESS OUTSIDE.** In every cell of the sweep the veto also flags hundreds of
///    pixels OUTSIDE that box, reaching tens of logical px above the card's top edge; and
///    on the loud worlds, intersecting it with a picker-open-versus-closed difference
///    still leaves thousands of pixels the frost does not reach. That is what makes an
///    inclusion reading unsound, and it is a measurement rather than a memory.
///
/// If clause 2 ever goes red, the product has not broken: the oracle has become
/// trustworthy in a direction it was not, and the header above is what needs rewriting.
///
/// Swept over the enrolled roster (derived from `blur::footprint_frost_applies`, never
/// named) × 1×/2× × BOTH menu-bar states, and the per-cell floors are taken as a MINIMUM
/// over that sweep. The bar arm is not decoration: its reserve moves the card down the
/// canvas and every figure here moves with it, in opposite directions on the two hosts,
/// so a single-arm run would make this law's own floors a property of the host.
#[test]
fn the_card_ink_veto_flags_the_worlds_own_ground_and_so_cannot_be_inverted() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();
    let worlds = enrolled_worlds();
    assert!(
        !worlds.is_empty(),
        "no world enrols in the footprint frost — this law has no subject"
    );
    let mut floors = (f32::INFINITY, u64::MAX, f32::INFINITY);
    let mut loudest = (0u64, String::new());
    let mut quietest = (u64::MAX, String::new());
    for world in &worlds {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    eprintln!("skipping the card-ink contract law: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    return;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                let f = cell(&device, &queue, &mut p, (w, h));
                let ink = CardInk::derive(&f.empty, w as i64, h as i64, dpi);
                let c = census(&ink, (&f.open, &f.closed), f.card, f.frost, dpi);
                let share = c.inside.0 as f32 / c.inside.1 as f32;
                let label = format!("{world} @ {dpi}x ({w}x{h}) bar {bar}");
                eprintln!(
                    "MEASURED {label}: card {:?} — flagged inside the card's box {}/{} \
                     ({:.1}%), OUTSIDE it {}, reaching {:.0} logical px above its top edge, \
                     of which {} sit under the card's shadow with less than \
                     {COVERAGE_FLOOR} frost",
                    f.card,
                    c.inside.0,
                    c.inside.1,
                    100.0 * share,
                    c.outside,
                    c.reach,
                    c.shadow
                );
                floors = (
                    floors.0.min(share),
                    floors.1.min(c.outside),
                    floors.2.min(c.reach),
                );
                if c.shadow > loudest.0 {
                    loudest = (c.shadow, label.clone());
                }
                if c.shadow < quietest.0 {
                    quietest = (c.shadow, label);
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(entry);
    eprintln!(
        "ROSTER {worlds:?}: tightest flagged share inside the card {:.3}, fewest flagged \
         OUTSIDE it {}, shortest reach above its top edge {:.0} logical px; loudest shadow \
         {} at {}, quietest {} at {}",
        floors.0, floors.1, floors.2, loudest.0, loudest.1, quietest.0, quietest.1
    );
    assert!(
        floors.0 >= INSIDE_PRESENCE,
        "the veto flags only {:.3} of the card's own box at its tightest (floor \
         {INSIDE_PRESENCE}) over {worlds:?} — a veto that flags nearly nothing excludes \
         nearly nothing, and every consumer's card-ink exclusion downstream is then an \
         exclusion of nothing while still reading green",
        floors.0
    );
    assert!(
        floors.1 >= 500 && floors.2 >= 10.0,
        "the veto flags as few as {} pixels OUTSIDE the card's own box, reaching only \
         {:.0} logical px above its top edge, somewhere in {worlds:?}. The header of this \
         module says the flagged set is a SUPERSET of the card's ink because the world's \
         live ground outside the frost is flagged too — if that has stopped being true, \
         the oracle has become trustworthy in a direction it was not, and the contract \
         above is what needs rewriting. Do not relax this to keep the law green",
        floors.1,
        floors.2
    );
    assert!(
        loudest.0 >= 1000,
        "the loudest cell in {worlds:?} leaves only {} flagged pixels under the card's \
         SHADOW with less than {COVERAGE_FLOOR} frost over them ({}). Intersecting the \
         veto with a picker-open-versus-closed difference is the obvious rescue for the \
         inclusion reading, and this figure is why it does not work — a shadow is a wash, \
         not ink. The quietest cell measures {} ({}), two orders of magnitude down, which \
         is why a check on one world alone finds none of this",
        loudest.0,
        loudest.1,
        quietest.0,
        quietest.1
    );
}

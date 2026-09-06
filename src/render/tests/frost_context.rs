//! A POINTER-ANCHORED MENU IS NOT A TAKEOVER OF THE ROOM.
//!
//! The frosted backdrop is the defocus behind a card that has BECOME the subject of the
//! screen: the command palette, go-to, the outline, keybindings, the held HUD. Every one
//! of those is summoned to the middle of the room and answered with the keyboard. A
//! right-click menu is none of them — four rows, under the pointer, gone on the next
//! click — and receding the whole page for it is a value change the size of the window in
//! answer to a gesture the size of a word.
//!
//! # THE ROUTING, AND WHY THE ANSWER IS TWO ANSWERS
//!
//! "Not a takeover" is not the same as "no frost". The predicate keyed on the card being
//! POINTER-ANCHORED (`overlay_declines_takeover`) only decides that the FULL arm is not
//! this card's; where the frost lands then is the footprint arm's own roster question
//! (`blur::footprint_frost_applies`), and it has two answers:
//!
//! * **`None` — no frost at all — on a composition that BACKS ITS ROWS**, with a panel
//!   under the whole card (`ListBacking::Card`) or a plate under each row. Its own surface
//!   already covers its footprint, so a backdrop has nothing left to do. This is the
//!   majority of the roster, and it is the off-switch the whole item is about.
//! * **`Footprint` — the card's own box and a feather, dimmed by nothing — on a
//!   composition that draws NEITHER.** Those are the compositions whose rows would
//!   otherwise interleave with the document glyph-for-glyph; that defect is not a property
//!   of which picker is open, and shipping the off-switch alone would have moved it from
//!   the theme picker onto the context menu.
//!
//! Both are read off the roster's own two backing owners, never a name list — Paperbark is
//! the `Ruled` member with shear 0 that keeps the leaning worlds from standing in for the
//! whole enrolled set.
//!
//! # WHAT EACH LAW BELOW REFUSES
//!
//! The three are a set and none is sufficient alone. The routing law is arithmetic and
//! would be satisfied by a frost that drew nothing anywhere. The page law is pixels and
//! gets strictly HAPPIER the less the frost covers. The footprint law is what refuses
//! that, over the same frames: inside the menu's own footprint, on an enrolled world, no
//! document edge may survive.
//!
//! Swept over the WHOLE world roster (not only the enrolled subset — the `None` answer is
//! half the deliverable) × 1×/2× × both `MENU_BAR_ON` arms, whose reserve moves the card
//! down the canvas. The bar arm is taken from the AMBIENT value and its negation, never
//! from `cfg!`, which inside a test reflects the host that COMPILED it.

use super::super::*;
use super::frost_card_ink::{CardInk, luma, step};
use super::frost_feather::{DENSE, render_frame};
use super::frost_parallelogram::STRONG_GRADIENT;
use super::{headless_dqp, view_md};
use crate::context_menu::{ContextState, ContextTarget};

/// The pointer's own position, in LOGICAL px — far enough into the page that the card is
/// placed by the anchor rather than by the clamp against either canvas edge.
const ANCHOR: (f32, f32) = (300.0, 260.0);

/// A REAL right-click menu over `text`, built through the production row policy
/// (`context_menu::rows`) rather than a synthetic candidate list — the same labels and
/// `enabled` flags `App::on_right_press` summons over a selection, which is the four-row
/// menu the item names. `dpi` scales the anchor, which is a CANVAS position: the card must
/// land at the same LOGICAL place at both scales or the sweep's two arms are two different
/// pictures.
fn context_menu(text: &str, dpi: f32) -> ViewState {
    let state = ContextState {
        has_selection: true,
        link: false,
        heading: false,
        heading_folded: false,
        misspelled: false,
        named_file: true,
    };
    let rows = crate::context_menu::rows(
        ContextTarget::Selection,
        state,
        crate::commands::Platform::Native,
    );
    context_menu_with_rows(&rows, text, dpi)
}

/// Same construction as [`context_menu`], generalized to an arbitrary already-built
/// row list — the seam [`every_anchored_target_declines_the_full_takeover`] drives
/// once per [`ContextTarget`] so the routing is proved for the target the item
/// itself names (Heading), not only Selection.
fn context_menu_with_rows(
    rows: &[crate::context_menu::ContextRow],
    text: &str,
    dpi: f32,
) -> ViewState {
    let mut v = view_md(text, 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Context.title().to_string();
    v.overlay_items = rows.iter().map(|r| r.label.to_string()).collect();
    v.overlay_bindings = vec![String::new(); rows.len()];
    v.overlay_selected = 0;
    v.overlay_context_anchor = Some((ANCHOR.0 * dpi, ANCHOR.1 * dpi));
    v
}

/// THE SAME CARD, SUMMONED TO THE ROOM INSTEAD OF TO THE POINTER — the control arm. It
/// differs from [`context_menu`] in the anchor and nothing else, so anything that moves
/// between the two is the anchor's doing and not the rows'.
fn room_menu(text: &str, dpi: f32) -> ViewState {
    let mut v = context_menu(text, dpi);
    v.overlay_context_anchor = None;
    v
}

/// Does this world forgo the frost entirely — the TRUE 1-BIT exclusion, asked above both
/// arms of `frost_mode` because a Gaussian of a pure-black-or-white page smears every edge
/// into grey. Read off the roster's own cap, so a world that changes backdrop changes its
/// answer here with it.
fn flat_backdrop(world: &str) -> bool {
    crate::theme::THEMES
        .iter()
        .find(|t| t.name == world)
        .is_some_and(|t| t.render_caps.backdrop == crate::theme::Backdrop::Flat)
}

/// Does this world's composition back its rows — the roster's reason a pointer-anchored
/// menu needs no backdrop at all there.
fn backs_its_rows(world: &str) -> bool {
    crate::theme::THEMES
        .iter()
        .find(|t| t.name == world)
        .is_some_and(|t| !crate::render::blur::footprint_frost_applies(t.render_caps.list_style))
}

/// THE SWEEP: the whole roster × 1×/2× × both menu-bar arms. Returns the number of cells
/// that ran, so a law can tell "green" from "no adapter".
fn sweep(
    mut cell: impl FnMut(&wgpu::Device, &wgpu::Queue, &mut TextPipeline, u32, u32, f32, String),
) -> usize {
    let entry = crate::theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();
    let mut ran = 0usize;
    for world in crate::theme::world_names() {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    eprintln!("skipping the context-frost sweep: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    crate::theme::set_active(entry);
                    return ran;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                let label = format!("{world} @ {dpi}x ({w}x{h}) bar {bar}");
                cell(&device, &queue, &mut p, w, h, dpi, label);
                ran += 1;
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(entry);
    ran
}

/// THE ROUTING LAW: A POINTER-ANCHORED MENU NEVER REACHES THE FULL-CANVAS FROST, AND THE
/// ROOM-SUMMONED CARD BESIDE IT STILL DOES.
///
/// Three clauses, and the third is the one that keeps this from being a licence to turn the
/// frost off:
///
/// 1. The menu's `frost_mode` is never [`blur::Frost::Full`], on any world at any scale,
///    and the sidecar's `dim_overlay` agrees with it — they are one predicate, so a report
///    that disagreed with the pass would be a second copy of the rule.
/// 2. WHERE it lands is the roster's own backing question: `Footprint` exactly on the
///    compositions that draw neither a panel nor plates, `None` on the rest. Both arms are
///    required non-empty, because either alone would be true of a routing that answered
///    one thing everywhere.
/// 3. THE CONTROL ARM: the identical card with the anchor removed reaches `Full` on every
///    world whose backdrop is not the 1-bit exclusion. Without it, "the menu is not a
///    takeover" is satisfied by having deleted the takeover.
#[test]
fn a_pointer_anchored_menu_declines_the_full_takeover_and_a_room_summoned_card_keeps_it() {
    let _g = crate::testlock::serial();
    let mut footprinted: Vec<String> = Vec::new();
    let mut unfrosted: Vec<String> = Vec::new();
    let mut room_full: Vec<String> = Vec::new();
    let ran = sweep(|device, queue, p, w, h, dpi, label| {
        let world = crate::theme::active().name;
        p.set_view(&context_menu(DENSE, dpi));
        let _ = render_frame(device, queue, p, w, h);
        let menu = p.frost_mode();
        assert!(
            !p.dims_doc(),
            "{label}: the sidecar reports `dim_overlay` under a POINTER-ANCHORED menu — a \
             footprint dims by nothing and an absent frost dims by less, so the report and \
             the composite pass have parted company"
        );
        assert_ne!(
            menu,
            Some(crate::render::blur::Frost::Full),
            "{label}: a four-row menu summoned under the pointer took the WHOLE-CANVAS \
             frost. The full arm is the defocus behind a card that has become the subject \
             of the screen; this one is dismissed by the next click"
        );
        if backs_its_rows(world) || flat_backdrop(world) {
            assert_eq!(
                menu, None,
                "{label}: this composition draws a panel or plates under its own rows (or \
                 forgoes the frost outright), so its footprint is already covered and the \
                 right answer is no frost at all"
            );
            unfrosted.push(label.clone());
        } else {
            let Some(crate::render::blur::Frost::Footprint(foot)) = menu else {
                panic!(
                    "{label}: this composition draws NOTHING under its rows, so a menu with \
                     no backdrop interleaves with the document glyph-for-glyph — the defect \
                     the footprint arm exists for, moved from the theme picker onto the \
                     right-click menu. Got {menu:?}"
                );
            };
            assert!(
                foot.rect[2] > 0.0 && foot.rect[3] > 0.0,
                "{label}: the footprint is degenerate {:?}",
                foot.rect
            );
            footprinted.push(label.clone());
        }

        // THE CONTROL: the same rows, summoned to the room. Only the anchor differs.
        p.set_view(&room_menu(DENSE, dpi));
        let _ = render_frame(device, queue, p, w, h);
        let room = p.frost_mode();
        if flat_backdrop(world) {
            assert_eq!(
                room, None,
                "{label}: a 1-bit world forgoes the frost either way"
            );
        } else {
            assert_eq!(
                room,
                Some(crate::render::blur::Frost::Full),
                "{label}: the SAME card with its anchor removed no longer takes the room \
                 over. The takeover is what the frost is for, and this law would otherwise \
                 be satisfied by having deleted it"
            );
            assert!(
                p.dims_doc(),
                "{label}: a room-summoned takeover dims the document"
            );
            room_full.push(label);
        }
    });
    if ran == 0 {
        return;
    }
    eprintln!(
        "MEASURED {ran} cells: {} footprinted, {} unfrosted, {} room-summoned takeovers",
        footprinted.len(),
        unfrosted.len(),
        room_full.len()
    );
    assert!(
        !footprinted.is_empty() && !unfrosted.is_empty(),
        "the roster must contain a composition that backs its own rows (answer: no frost) \
         and one that draws nothing under them (answer: a footprint), or one arm of this \
         routing never ran: footprinted {footprinted:?}, unfrosted {unfrosted:?}"
    );
    assert!(
        !room_full.is_empty(),
        "no world reached the full-canvas frost from a room-summoned card — the control arm \
         is empty and clause 3 proves nothing"
    );
}

/// THE ROUTING LAW SWEPT OVER EVERY [`ContextTarget`], NOT SELECTION ALONE — a
/// NO-WILDCARD `match` over [`ContextTarget::ALL`], so a target added to that enum
/// tomorrow is swept by this law the moment it exists rather than when someone
/// remembers to list it here. The item's own report is the Heading target
/// specifically; this is what proves the routing is the ANCHOR's doing (every
/// target reaches the same `context_menu::overlay` door) and not a property only
/// Selection happens to have.
///
/// One dichotomy, one DPI, the ambient menu-bar state — the full world × DPI × bar
/// sweep already lives in the routing law above and stays keyed to Selection; this
/// law adds the target axis instead of multiplying every other axis by it, which
/// would turn a 17s test into one that times out the gate for no new coverage
/// (every target reaches the SAME anchor-only predicate, so no cell here can
/// disagree by world or DPI once the routing law above has already swept those).
///
/// [`ContextTarget::Misspelling`] is the one target whose row list is
/// deliberately empty ("the established spell picker owns this target" —
/// `context_menu::rows`'s own doc) — asserted explicitly below, not skipped by a
/// blanket continue, so a future target that returns rows for the first time is
/// caught rather than silently joining the exemption.
#[test]
fn every_anchored_target_declines_the_full_takeover() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    // Named, not derived: one member of each arm the routing law above already
    // measured (`backs_its_rows`/`flat_backdrop`) — this law's subject is the
    // TARGET axis, and the world roster is already proved by the law above.
    let worlds = ["Magpie", "Wagtail"];
    let state = crate::context_menu::ContextState {
        has_selection: true,
        link: true,
        heading: true,
        heading_folded: false,
        misspelled: true,
        named_file: true,
    };
    let mut swept = Vec::new();
    for world in worlds {
        let Some((device, queue, mut p)) = headless_dqp(1200.0, 900.0) else {
            eprintln!("skipping every_anchored_target_declines_the_full_takeover: no wgpu adapter");
            crate::theme::set_active(entry);
            return;
        };
        crate::theme::set_active_by_name(world).unwrap();
        for target in ContextTarget::ALL {
            let rows = crate::context_menu::rows(target, state, crate::commands::Platform::Native);
            if rows.is_empty() {
                assert_eq!(
                    target,
                    ContextTarget::Misspelling,
                    "{world}/{target:?}: an anchored context menu with no rows is not \
                     `Misspelling` — the one target this law lets through empty. \
                     Either this target needs a state field flipped on above, or it \
                     has quietly joined the spell-picker exemption without a law \
                     saying so."
                );
                continue;
            }
            let v = context_menu_with_rows(&rows, DENSE, 1.0);
            p.set_view(&v);
            let _ = render_frame(&device, &queue, &mut p, 1200, 900);
            let frost = p.frost_mode();
            assert_ne!(
                frost,
                Some(crate::render::blur::Frost::Full),
                "{world}/{target:?}: an anchored context menu ({} row(s)) took the \
                 whole-canvas frost",
                rows.len()
            );
            assert!(
                !p.dims_doc(),
                "{world}/{target:?}: `dim_overlay` is set under an anchored menu"
            );
            swept.push((world, target));
        }
    }
    crate::theme::set_active(entry);
    assert_eq!(
        swept.len(),
        worlds.len() * (ContextTarget::ALL.len() - 1),
        "every target but Misspelling must have run on every named world: {swept:?}"
    );
}

/// The two frames one cell of the pixel laws needs: the SAME menu over dense prose and
/// over an EMPTY document. The card's drawing and the world's ground are bit-identical
/// between them, so their luma residue is the DOCUMENT alone — which is what lets these
/// laws measure the page's own sharpness without an ink veto over the card or the ground.
struct Pair {
    residue: Vec<f32>,
    card: [f32; 4],
    frost: Option<crate::render::blur::Frost>,
    /// The card's own ink, derived off the EMPTY frame. The one place this veto's premise —
    /// "what the card draws over is a blur of a blank page, and a blur has no step in it" —
    /// actually holds is INSIDE the frost, which is the only region that reads it.
    ink: CardInk,
    /// Every row's own ink, GROWN to its scrim ([`TextPipeline::overlay_row_ink_probe`]) —
    /// the production owner of what a `Bars` plate actually occupies. `CardInk` is derived
    /// from a LUMA GRADIENT in the empty frame, and a `Bars` plate/scrim is authored to sit
    /// close to the world's own ground (Firetail's dark plate on its dark blur, Galah's
    /// light plate on its light blur): the gradient at its edge can land under
    /// `INK_GRADIENT`, so the derived veto is structurally blind to it while the plate is
    /// still opaque, real ink. This is the geometric backstop for exactly that blind spot.
    row_ink: Vec<[f32; 4]>,
    w: i64,
    h: i64,
}

impl Pair {
    /// Does a row's own ink (plate/scrim/rule/mark — [`Self::row_ink`]) cover `(x, y)`,
    /// grown by the same anti-aliasing skirt [`CardInk`] dilates by? A pixel this admits is
    /// real card ink whether or not the gradient-derived veto could see it.
    fn vetoes_row_ink(&self, x: i64, y: i64, dpi: f32) -> bool {
        super::frost_card_ink::row_ink_vetoes(&self.row_ink, dpi, x, y)
    }
}

fn pair(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
    dpi: f32,
) -> Pair {
    p.set_view(&context_menu(DENSE, dpi));
    let open = render_frame(device, queue, p, w, h);
    let card = p.overlay_card_rect().expect("the menu has a card box");
    let frost = p.frost_mode();
    let row_ink = p.overlay_row_ink_probe();
    p.set_view(&context_menu("", dpi));
    let empty = render_frame(device, queue, p, w, h);
    let ink = CardInk::derive(&empty, w as i64, h as i64, dpi);
    Pair {
        residue: open
            .iter()
            .zip(empty.iter())
            .map(|(a, b)| luma(*a) - luma(*b))
            .collect(),
        card,
        frost,
        ink,
        row_ink,
        w: w as i64,
        h: h as i64,
    }
}

/// `(pixels measured, pixels carrying a document edge, peak local step)` over the residue,
/// on the region `keep` admits.
///
/// The FIELD is what makes this a document measurement: the two frames share their card and
/// their world's ground exactly, so their residue is the page alone and no veto over either
/// is needed to keep them out of the count. Each law states its own region, and every one
/// below carries a presence guard, because an exclusion inside a region can make any
/// zero-edge claim vacuous.
fn doc_edges(f: &Pair, keep: impl Fn(f32, f32) -> bool) -> (u64, u64, f32) {
    let (mut measured, mut edges, mut peak) = (0u64, 0u64, 0.0f32);
    for y in 0..f.h {
        for x in 0..f.w {
            if !keep(x as f32, y as f32) {
                continue;
            }
            let s = step(&f.residue, f.w, f.h, x, y);
            measured += 1;
            peak = peak.max(s);
            if s >= STRONG_GRADIENT {
                edges += 1;
            }
        }
    }
    (measured, edges, peak)
}

/// THE PAGE LAW: THE DOCUMENT OUTSIDE THE MENU IS STILL SHARP — ON EVERY WORLD.
///
/// This is the item's own figure in pixels, and it is the one that could not be satisfied
/// before the routing change: under the full-canvas frost the whole page is a blur of
/// itself, so the residue between the two frames is smooth everywhere and this count is
/// near zero.
///
/// The region is the canvas beyond the card's own box grown by the frost's SKIRT — the page
/// a reader is still reading while the menu is up. It is derived from
/// `blur::footprint_skirt_px` rather than a chosen pad, so a retuned feather moves the
/// region with it instead of turning this into a reading of the skirt.
#[test]
fn the_document_outside_a_pointer_anchored_menu_keeps_its_own_sharp_edges() {
    let _g = crate::testlock::serial();
    let mut fewest = (u64::MAX, String::new());
    let ran = sweep(|device, queue, p, w, h, dpi, label| {
        let f = pair(device, queue, p, w, h, dpi);
        let skirt = crate::render::blur::footprint_skirt_px(f.frost, dpi);
        let [rx, ry, rw, rh] = f.card;
        let (measured, edges, peak) = doc_edges(&f, |fx, fy| {
            fx < rx - skirt || fx > rx + rw + skirt || fy < ry - skirt || fy > ry + rh + skirt
        });
        eprintln!(
            "MEASURED {label}: frost {:?}, skirt {skirt:.1} — {edges}/{measured} px of the \
             page BEYOND the menu carry a document edge (peak step {peak:.1})",
            f.frost.is_some()
        );
        assert!(
            measured > 100_000,
            "{label}: only {measured} px of page outside the menu's own footprint — a \
             sharpness claim over a region this small is a claim about nothing"
        );
        assert!(
            edges > 2_000 && peak >= STRONG_GRADIENT,
            "{label}: the page outside the menu carries only {edges} document edges (peak \
             step {peak:.1}, threshold {STRONG_GRADIENT}). A pointer-anchored menu must \
             leave the document it was summoned over exactly as sharp as it found it — this \
             count collapses to near zero the moment the menu takes the full-canvas frost"
        );
        if edges < fewest.0 {
            fewest = (edges, label);
        }
    });
    if ran == 0 {
        return;
    }
    eprintln!(
        "ROSTER FEWEST document edges outside the menu: {} at {}",
        fewest.0, fewest.1
    );
}

/// THE FOOTPRINT LAW: WHERE THE MENU *DOES* FROST, THE DOCUMENT UNDER IT IS GONE AS TEXT.
///
/// This is what refuses the page law above, which gets strictly happier the less the frost
/// covers — a routing that turned the frost off outright would satisfy it perfectly on
/// every world. On the compositions that draw nothing under their rows, the menu's own
/// footprint owes the page a backdrop, and 294's headline figure is the proof: behind the
/// card, no glyph edge of the document survives.
///
/// ⚠️ **THE SUBJECT IS THE MASK'S OWN FULL-STRENGTH INTERIOR, NOT THE COVERAGE FLOOR.** At
/// a mask of 0.9 a tenth of the CRISP document composites through, and a document edge of
/// 250 luma still lands a step of 25 at that tenth — above the threshold, and correctly so.
/// The claim "no text survives" belongs to the region the frost fully replaces; the feather's
/// ramp is a different subject and is graded as a ramp, by the laws over the edge profile.
///
/// ⚠️ **AND THE CARD'S OWN INK MUST COME OUT**, even though the residue cancels it. A card
/// glyph composites as `a·ink + (1−a)·backdrop`, so the residue behind it is
/// `(1−a)·(blurA − blurB)` — the card does cancel, but its ALPHA modulates the document's
/// own residue, and a glyph edge is a step in `a`. `CardInk` is the exclusion, used the ONE
/// way its premise holds: this region is exactly where the frost reaches, so the empty
/// frame there is a blur of a blank page and every step in it is the card's. Nothing here
/// erodes as a world's ground gets busier, because inside the frost that ground is blurred
/// too.
#[test]
fn inside_the_menus_own_footprint_no_document_edge_survives() {
    let _g = crate::testlock::serial();
    let mut worst = (0u64, String::new());
    let mut arms = 0usize;
    let ran = sweep(|device, queue, p, w, h, dpi, label| {
        let world = crate::theme::active().name;
        if backs_its_rows(world) || flat_backdrop(world) {
            return; // no frost is owed here; the card's own surface is the backdrop
        }
        let f = pair(device, queue, p, w, h, dpi);
        let Some(frost) = f.frost else {
            panic!("{label}: an enrolled composition reaches the footprint arm");
        };
        let (measured, edges, peak) = doc_edges(&f, |fx, fy| {
            crate::render::blur::footprint_mask_for(frost, dpi, fx, fy) >= 1.0
                && !f.ink.vetoes(fx as i64, fy as i64)
                && !f.vetoes_row_ink(fx as i64, fy as i64, dpi)
        });
        eprintln!(
            "MEASURED {label}: {edges}/{measured} px UNDER the menu's fully-frosted \
             footprint carry a document edge (peak step {peak:.1})"
        );
        assert!(
            measured > 5_000,
            "{label}: only {measured} px of fully-frosted footprint outside the card's own \
             ink — a zero-edge claim over an empty region is satisfied by anything"
        );
        assert_eq!(
            edges, 0,
            "{label}: {edges} of {measured} px under the menu's own footprint still carry a \
             document EDGE (peak step {peak:.1}, threshold {STRONG_GRADIENT}) — the page is \
             drawing as sharp TEXT between the menu's rows, which is the interleaving the \
             footprint arm exists to end"
        );
        arms += 1;
        if measured > worst.0 {
            worst = (measured, label);
        }
    });
    if ran == 0 {
        return;
    }
    assert!(
        arms > 0,
        "no cell reached the footprint arm — this law's whole subject is the compositions \
         that draw nothing under their rows, and none enrolled"
    );
    eprintln!(
        "ROSTER LARGEST frosted footprint under a menu: {} px at {} ({arms} cells)",
        worst.0, worst.1
    );
}

/// THE FORMER TEACHING LINE, forced back on — the counterfactual a card would
/// draw if it still authored a footer, over the same production actions
/// (`OverlayKind::Context::hint_actions`'s retained capability roster:
/// choose/close). Naming the string here rather than importing it keeps this
/// law's subject the CARD's response to a hint of a given shape, independent
/// of whatever `OverlayKind::Context.hint()` answers today — the policy
/// question (does Context carry one) is proven at the roster seam
/// (`overlay::tests::hints::context_menu_draws_no_teaching_footer_...`), not
/// here.
const FORMER_TEACHING_LINE: &str = "type to filter   \u{21B5} choose   esc close";

/// THE CARD HUGS ITS ROWS: dropping the pointer-anchored menu's teaching
/// footer must shrink the card by close to two row pitches (the hint's own
/// line plus its blank separator, `overlay_hint_gap_rows`) — never more
/// (something else moved too) and never less (a residual blank band
/// survived where the footer used to sit). Swept over every
/// [`ContextTarget`] that opens a real card (Misspelling is skipped, checked
/// rather than assumed — `context_menu::rows`'s own documented exemption),
/// both worlds the routing law's two backing arms name, and both DPIs.
///
/// The claim is read directly off two REAL rendered cards — the shipped one
/// and the counterfactual with [`FORMER_TEACHING_LINE`] forced back in — not
/// re-derived from the card-height formula, so a shared bug in that formula
/// cannot make this law agree with itself for the wrong reason.
#[test]
fn context_menu_card_hugs_its_rows_with_no_hint_reserved() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let state = ContextState {
        has_selection: true,
        link: true,
        heading: true,
        heading_folded: false,
        misspelled: true,
        named_file: true,
    };
    let mut swept = 0usize;
    let mut skipped_misspelling = 0usize;
    let (lw, lh_win) = (1200.0f32, 900.0f32);
    for world in ["Magpie", "Wagtail"] {
        for dpi in [1.0f32, 2.0] {
            // Scale the CANVAS with the DPI, same as `sweep()` above, so the
            // LOGICAL window (and the anchor's placement inside it) stays
            // constant across both arms — the anchor is otherwise far
            // enough from either edge to be placed by itself rather than by
            // the clamp, and that must stay true here too.
            let (pw, ph) = ((lw * dpi) as u32, (lh_win * dpi) as u32);
            let Some((device, queue, mut p)) = headless_dqp(pw as f32, ph as f32) else {
                eprintln!(
                    "skipping context_menu_card_hugs_its_rows_with_no_hint_reserved: \
                     no wgpu adapter"
                );
                crate::theme::set_active(entry);
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);
            for target in ContextTarget::ALL {
                let rows =
                    crate::context_menu::rows(target, state, crate::commands::Platform::Native);
                if rows.is_empty() {
                    assert_eq!(
                        target,
                        ContextTarget::Misspelling,
                        "{world}/{target:?}: an anchored menu with no rows is not \
                         Misspelling — this law's own skip has quietly widened"
                    );
                    skipped_misspelling += 1;
                    continue;
                }

                let mut real = context_menu_with_rows(&rows, DENSE, dpi);
                real.overlay_hint = crate::overlay::OverlayKind::Context.hint();
                p.set_view(&real);
                let _ = render_frame(&device, &queue, &mut p, pw, ph);
                let real_card = p.overlay_card_rect().expect("a real anchored card");

                let mut hinted = context_menu_with_rows(&rows, DENSE, dpi);
                hinted.overlay_hint = FORMER_TEACHING_LINE.to_string();
                p.set_view(&hinted);
                let _ = render_frame(&device, &queue, &mut p, pw, ph);
                let hinted_card = p.overlay_card_rect().expect("the counterfactual card");

                let lh = p.overlay_lh();
                let shrink = hinted_card[3] - real_card[3];
                let ctx = format!("{world}/{target:?} dpi={dpi}");
                // The floor is a FULL ROW rather than a fraction tuned to
                // today's dials: the footer's two rows are both deliberately
                // COMPACT (`OVERLAY_HINT_ROW` and `OVERLAY_HINT_GAP_ROW`),
                // and together they clear a row with room to spare at every
                // shipped ratio — so a floor here catches the residual band
                // this law names without re-pinning itself to whatever the
                // separator's own magnitude happens to be.
                assert!(
                    shrink > lh,
                    "{ctx}: dropping the footer must shrink the card by more than a row \
                     pitch ({lh:.1}px), got {shrink:.1}px — a residual blank band would \
                     show up here as a near-zero shrink"
                );
                assert!(
                    shrink < lh * 3.0,
                    "{ctx}: the card shrank by {shrink:.1}px, more than the footer's own \
                     two rows ({:.1}px) can account for — something else moved too",
                    lh * 2.0
                );
                // The two cards' rows themselves are identical (same anchor, same
                // width, same items) — only the footer differs, so their top edges
                // and x/w must agree exactly.
                assert_eq!(
                    (real_card[0], real_card[1], real_card[2]),
                    (hinted_card[0], hinted_card[1], hinted_card[2]),
                    "{ctx}: only the card's own height should differ between the two"
                );
                swept += 1;
            }
            crate::theme::set_active(entry);
        }
    }
    assert!(
        swept > 20,
        "the target × world × dpi sweep must actually run, got {swept}"
    );
    assert!(
        skipped_misspelling > 0,
        "the Misspelling skip must actually be reached, or the check above is vacuous"
    );
}

/// THE BEFORE/AFTER PAIR FOR THE TASTE CALL — a regeneration tool, not a law.
///
/// The final width and the decision itself are taste, and no capture settles them; what a
/// capture can do is put the two states side by side on the world the difference is largest
/// on. Run it explicitly, and read the four PNGs it names:
///
/// ```sh
/// cargo test --bin awl frost_context::gallery -- --ignored --nocapture
/// ```
///
/// It writes into the untracked gallery at 1× on the two compositions that answer
/// the routing differently — one that backs its own rows (no frost) and one that draws
/// nothing under them (a footprint) — with the room-summoned card beside each as the
/// full-takeover reference the change is judged against.
#[test]
#[ignore = "regeneration tool: writes gallery/frost-context/*.png for a human's eye"]
fn gallery() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let dir = std::path::Path::new("gallery/frost-context");
    std::fs::create_dir_all(dir).expect("the gallery directory");
    let (w, h) = (1200u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("no wgpu adapter: no gallery");
        return;
    };
    let names = crate::theme::world_names();
    let bare = names
        .iter()
        .find(|n| !backs_its_rows(n) && !flat_backdrop(n));
    let backed = names
        .iter()
        .find(|n| backs_its_rows(n) && !flat_backdrop(n));
    for world in [bare, backed].into_iter().flatten() {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for (name, v) in [
            ("menu", context_menu(DENSE, 1.0)),
            ("room-takeover", room_menu(DENSE, 1.0)),
        ] {
            p.set_view(&v);
            let px = render_frame(&device, &queue, &mut p, w, h);
            let mut img = image::RgbaImage::new(w, h);
            for (i, q) in px.iter().enumerate() {
                img.put_pixel(i as u32 % w, i as u32 / w, image::Rgba(*q));
            }
            let path = dir.join(format!("{world}-{name}.png"));
            img.save(&path).expect("the gallery frame writes");
            eprintln!(
                "GALLERY {} — frost {:?}",
                path.display(),
                p.frost_mode().is_some()
            );
        }
    }
    crate::theme::set_active(entry);
}

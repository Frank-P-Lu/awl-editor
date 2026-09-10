//! THE CARD'S BACKING MUST SURVIVE A CROSS-WORLD PREVIEW.
//!
//! Open the theme picker on one world and arrow to PREVIEW another (uncommitted): the
//! surrounding page repaints live in the previewed world's colours (that live preview is
//! the whole point of the picker), but the card's own chrome stays pinned to the world it
//! was SUMMONED on (`render::picker_chrome_theme`) so the list itself never
//! recomposes underfoot. The card's OPAQUE/FROSTED BACKING is a separate treatment from
//! that pin — `TextPipeline::frost_mode`'s footprint arm — and it read `theme::active()`
//! (the live, previewed world) rather than the pinned one for its one early exit: **TRUE
//! 1-BIT worlds (`Backdrop::Flat`) forgo the frost entirely**, because a Gaussian defocus
//! of a document that is only ever pure black or pure white mathematically smears every
//! edge into grey. Wagtail is the roster's only `Backdrop::Flat` member today, so opening
//! the picker on any OTHER world and previewing Wagtail flipped `theme::active()` to
//! Wagtail for the span of the preview, tripped that exit, and left the card with no
//! backing at all — the document behind it bled straight through the row text, unreadable.
//! Captured live at `gallery/shared-chrome/theme_kite_preview_wagtail.png`
//! (Kite open, previewing Wagtail) on an unrelated lane's branch, found incidentally.
//!
//! The fix joins the SAME pin the sibling resolvers read rather than a second one:
//! `frost_mode` now reads `picker_chrome_theme().render_caps.backdrop`, exactly like the
//! six existing `effective_*` resolvers read `picker_chrome_theme().render_caps.*` instead
//! of `theme::active().render_caps.*`. `picker_chrome_theme()` falls back to
//! `theme::active()` verbatim whenever nothing is pinned, so this is byte-identical for
//! every other overlay kind and for no overlay at all — only a Theme picker's own
//! preview crossing can ever see the two disagree.
//!
//! # WHAT THIS LAW MEASURES, AND WHY IT IS PIXELS RATHER THAN STATE
//!
//! The sidecar is a state oracle, not an appearance oracle — CLAUDE.md's own tripwire is
//! this exact failure mode one surface over: it once reported `selected_index: 2` while
//! the row rendered fully invisible (Wagtail again). So this law never asks the ViewState
//! whether a card is "open"; it renders the SAME picker over a DENSE document and over an
//! EMPTY one (`frost_footprint`'s own technique — the per-pixel luma difference is the
//! document's own contribution, with the card's sharp-by-design rows excluded by the
//! derived `CardInk` veto) and asks whether a document EDGE survives inside the card's own
//! footprint. A survived edge is undeniably the document showing through as text; its
//! absence, paired with a PRESENCE floor so the law cannot be satisfied by an opaque or
//! flattened card either, is the "backing is opaque" claim the item asks for.
//!
//! # ENROLMENT, DERIVED — NOT NAMED
//!
//! Openers are every world whose composition backs its rows with nothing of its own
//! (`blur::footprint_frost_applies`, the identical predicate `frost_footprint`'s own
//! sweep enrols by): a `Pane` world's permanent card panel already occludes the document
//! regardless of this law's subject, so crossing INTO a flat world from a `Pane` opener
//! would pass whether or not the frost seam was fixed, and proves nothing about the seam
//! under test. Targets are every world whose OWN `Backdrop` is `Flat` — the axis this
//! item's bug lives on, currently Wagtail alone, but read off the roster rather than
//! spelled, so a second flat world enrols itself the day it ships.

use super::super::{TextPipeline, ViewState};
use super::frost_card_ink::{CardInk, luma, row_ink_vetoes, step};
use super::frost_feather::{DENSE, render_frame, theme_picker};
use super::headless_dqp;

/// Same calibration `frost_footprint` measured and named: this tree's frosted residue
/// peaks near 5/255 and its sharp residue near 190/255, a 38x separation, so this sits in
/// a wide valley rather than on a tuned edge.
const STRONG_GRADIENT: f32 = 24.0;

/// The companion floor: without it, an opaque card or a frost dimmed to the flat base
/// would satisfy the smoothness assertion perfectly while deleting the very thing the
/// crisp picker exists to preview.
const PRESENCE_FLOOR: f32 = 12.0;

/// Every world whose crisp-picker composition backs its rows with nothing of its own —
/// the subset this item actually reproduces on, taken from the roster via the identical
/// predicate `frost_footprint::enrolled_worlds` sweeps by
/// (`blur::footprint_frost_applies`), not a name list.
fn footprint_reliant_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| crate::render::blur::footprint_frost_applies(t.render_caps.list_style))
        .map(|t| t.name)
        .collect()
}

/// Every world whose OWN backdrop is the roster's TRUE-1-BIT exclusion — the axis this
/// item's bug lives on. Wagtail is the only member today; derived rather than named.
fn flat_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| t.render_caps.backdrop == crate::theme::Backdrop::Flat)
        .map(|t| t.name)
        .collect()
}

/// Pin the picker's chrome to `opener` — mirroring `OverlayState::new_marked`'s own
/// summon-time call to `pin_picker_chrome` — then arrow the LIVE theme to `target`,
/// exactly the two-step shipped sequence a real preview performs
/// (`OverlayState::new_theme` pins at summon; `actions::preview_move` calls
/// `theme::set_active_by_name` at every arrow step). Building the whole `OverlayState`
/// and its corpus buys this law nothing a direct call doesn't already give it.
fn open_on_and_preview(opener: &str, target: &str) {
    crate::theme::set_active_by_name(opener).unwrap();
    crate::render::pin_picker_chrome();
    crate::theme::set_active_by_name(target).unwrap();
}

/// `frost_feather::theme_picker`, with the selected ROW moved onto `target` — the
/// row a real preview would actually have highlighted (`preview_move` always keeps
/// `selected` on the row it is previewing). `theme_picker` itself hardcodes row 11
/// (its own file's fixed subject), which is a combination a real crossing never
/// produces once the active theme has moved to some OTHER target: a `Diagonal`
/// composition's own per-row travel is a function of which row the frame draws as
/// selected, so leaving it on the wrong row grades a geometry the picker never
/// actually shows during this crossing.
fn crossing_view(text: &str, target: &str) -> ViewState {
    let mut v = theme_picker(text);
    v.overlay_selected = target_roster_index(target);
    v
}

/// `target`'s position in `theme::THEMES` — the unsectioned "All" lens `crossing_view`
/// always shows lists in roster order, with no header lines ahead of any item
/// (`TextPipeline::theme_plan` emits a header only when a row's own section differs
/// from the last, and every section here is `""`), so this doubles as the row's DISPLAY
/// index whenever the whole roster fits with no scroll — true of every crossing this
/// law drives (`overlay_items` is always the unfiltered roster).
fn target_roster_index(target: &str) -> usize {
    crate::theme::THEMES
        .iter()
        .position(|t| t.name == target)
        .expect("target is a real world")
}

/// The vertical band the row marking the CURRENTLY PREVIEWED world occupies — excluded
/// from this law's measurement below, and why: that row draws in the PREVIEWED world's
/// own ink rather than the opener's (real, production behaviour — a "you are here"
/// legibility question of its own, not this item's subject), and Wagtail's ink is
/// literal pure black/white with no anti-aliased middle tone. Its edge against a blur
/// that is itself often near-black (previewing a mostly-black-page world) measures a
/// real, if modest, local step that has nothing to do with whether the document
/// survives as TEXT under the card — `frost_footprint`'s own headline law already
/// proves this ink is clean (`peak_step: 5.0`) on an ordinary (non-`Flat`) world's own
/// row, isolating the residual here to Wagtail's ink rather than to "the current row"
/// in general. `pad` grows the band by the same antialiasing margin `inside` insets the
/// footprint's own faces by. Read off `TextPipeline::overlay_row_geometry` — the same
/// published, per-item rect the capture sidecar's own `layout`/`overlay.window.rows`
/// fields report — rather than re-deriving a row's Y from `text_top` and a pitch by
/// hand, which would drift the moment a header, a docked strip or a facet gap changes
/// what sits above the first item.
fn target_row_band(p: &TextPipeline, idx: usize, pad: f32) -> (f32, f32) {
    let geometry = p
        .overlay_row_geometry()
        .expect("the crisp picker is open, so it publishes a row geometry");
    let row = geometry
        .rows
        .iter()
        .find(|r| r.item == Some(idx))
        .unwrap_or_else(|| panic!("target row {idx} is not in the published row geometry"));
    (row.y - pad, row.y + row.h + pad)
}

/// THE HEADLINE LAW. Pin the picker's chrome to every footprint-reliant opener, arrow
/// the live theme to every `Backdrop::Flat` target, and require that not one document
/// edge survives inside the card's own footprint — paired with a presence floor so the
/// law cannot be satisfied by deleting the very backing it grades.
#[test]
fn previewing_a_true_1bit_world_keeps_the_cards_backing_opaque() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let _pin_restore = crate::render::PickerChromePinRestore::capture();

    let openers = footprint_reliant_worlds();
    let targets = flat_worlds();
    assert!(
        !openers.is_empty(),
        "no world enrols as a footprint-reliant opener — this law has no subject"
    );
    assert!(
        !targets.is_empty(),
        "no world carries Backdrop::Flat — this law's own axis has vanished from the \
         roster; if that is deliberate, this law (and the frost_mode exception it \
         guards) can retire with it"
    );

    let (w, h, dpi) = (1200u32, 800u32, 1.0f32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!(
            "skipping previewing_a_true_1bit_world_keeps_the_cards_backing_opaque: \
             no wgpu adapter"
        );
        return;
    };
    p.set_dpi(dpi);

    let mut cells = 0usize;
    let mut floor_witness = f32::MAX;
    for &target in &targets {
        for &opener in &openers {
            if opener == target {
                continue;
            }
            cells += 1;
            open_on_and_preview(opener, target);
            let label = format!("{opener} open, previewing {target}");

            // A `Diagonal` composition's shear is resolved DURING `render()` (from
            // that frame's own shaped row widths, `overlay_draw.rs`'s
            // `resolve_diagonal_cluster`), so `prepare()` — and the `frost_mode()`
            // read inside it that bakes the backdrop's own mask geometry — sees
            // only the PREVIOUS frame's cluster, `None` on the very first frame a
            // view is set. A live session never shows this: opening the picker and
            // every arrow step is already its own prior render, so a real screen
            // is always at least the second frame before anyone looks at it. One
            // throwaway render settles the cluster onto THIS view before either
            // frame this law actually measures is captured, so the sweep grades
            // the shape a viewer would ever see rather than the single stale frame
            // nothing else in the tree renders only once and stops at.
            p.set_view(&crossing_view(DENSE, target));
            render_frame(&device, &queue, &mut p, w, h);

            // The seam under test, asked directly first: a picker crossing into a
            // flat target from a non-flat, footprint-reliant opener must still take
            // the footprint arm. If the old (buggy) `theme::active()` read were still
            // wired, this would be `None` — the exact defect the pixels below prove.
            let frost = p.frost_mode();
            assert!(
                matches!(frost, Some(crate::render::blur::Frost::Footprint(_))),
                "{label}: expected the footprint frost arm, got {frost:?} — a Flat \
                 preview target must not disable the frost for a picker pinned to a \
                 non-flat, footprint-reliant opener"
            );

            // A: the picker over dense prose. B: the SAME picker over an empty
            // document — identical card (rows are the world roster, not the
            // document), so A - B is the document's own contribution alone.
            let a = render_frame(&device, &queue, &mut p, w, h);
            let rect = p
                .overlay_card_rect()
                .expect("the crisp picker is open, so it has a card box");
            let row_ink = p.overlay_row_ink_probe();
            p.set_view(&crossing_view("", target));
            render_frame(&device, &queue, &mut p, w, h);
            let b = render_frame(&device, &queue, &mut p, w, h);

            let (wi, hi) = (w as i64, h as i64);
            let residue: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(pa, pb)| luma(*pa) - luma(*pb))
                .collect();
            let ink = CardInk::derive(&b, wi, hi, dpi);

            // THE FROST'S OWN SHAPE, NOT THE CARD'S BOX (`frost_footprint`'s own
            // discipline). On a `Diagonal` opener (Mangrove, Magpie) the card leans,
            // and the box's two OFF-RAKE corners are DELIBERATELY unfrosted — the
            // parallelogram the shader actually drew, read straight off `frost_mode`
            // rather than re-derived, is what this law must confine itself to, or it
            // grades the intended unfrosted corners as if they were this item's
            // defect. `pad` keeps the measurement off the mask's own AA seam at the
            // (leaning) face.
            let (frect, shear) = match frost {
                Some(crate::render::blur::Frost::Footprint(f)) => (f.rect, f.shear),
                other => unreachable!("checked above: {other:?}"),
            };
            let [rx, ry, rw, rh] = frect;
            let pad = 4.0 * dpi;
            let lean = |py: f32| shear * (py - (ry + rh * 0.5));
            let inside = |x: i64, y: i64| {
                let (fx, fy) = (x as f32, y as f32);
                fx >= rx + lean(fy) + pad
                    && fx < rx + rw + lean(fy) - pad
                    && fy >= ry + pad
                    && fy < ry + rh - pad
            };
            let (target_top, target_bottom) = target_row_band(&p, target_roster_index(target), pad);
            let mut measured = 0usize;
            let mut edges = 0usize;
            let mut peak_step = 0.0f32;
            let mut peak_amplitude = 0.0f32;
            let mut sample_edges: Vec<(i64, i64, f32)> = Vec::new();
            for y in 0..hi - 1 {
                for x in 0..wi - 1 {
                    if !inside(x, y) {
                        continue;
                    }
                    if (y as f32) >= target_top && (y as f32) < target_bottom {
                        continue;
                    }
                    if ink.vetoes(x, y) || row_ink_vetoes(&row_ink, dpi, x, y) {
                        continue;
                    }
                    measured += 1;
                    let s = step(&residue, wi, hi, x, y);
                    peak_step = peak_step.max(s);
                    peak_amplitude = peak_amplitude.max(residue[(y * wi + x) as usize].abs());
                    if s >= STRONG_GRADIENT {
                        edges += 1;
                        if sample_edges.len() < 20 {
                            sample_edges.push((x, y, s));
                        }
                    }
                }
            }
            eprintln!("SAMPLE EDGES {label}: {sample_edges:?}");
            eprintln!(
                "MEASURED {label}: measured={measured} edges={edges} peak_step={peak_step:.1} \
                 peak_amplitude={peak_amplitude:.1} card={rect:?} \
                 frost_rect={frect:?} shear={shear}"
            );
            assert!(
                measured > 1000,
                "{label}: too few pixels measured ({measured}) inside the card's own \
                 footprint — the region, not the product, is what failed"
            );
            // THE DEFECT ITSELF: not one glyph edge of the document survives inside
            // the card's own footprint.
            assert_eq!(
                edges, 0,
                "{label}: {edges} of {measured} pixels inside the card's own footprint \
                 carry a document EDGE (step >= {STRONG_GRADIENT}, peak {peak_step:.1}) \
                 — the document is drawing straight through the row text instead of \
                 defocused behind it"
            );
            floor_witness = floor_witness.min(peak_amplitude);
        }
    }
    assert!(
        cells > 0,
        "the opener x target sweep ran no cell at all — every opener coincided with \
         every target, which is not a roster this law can mean anything over"
    );
    // THE PRESENCE FLOOR, over the WORST cell measured: the backing must still show
    // the document as a defocus, not delete or flatten it — a law that only checked
    // for zero edges would be satisfied by an opaque card or a frost dimmed to the
    // flat base just as happily as by a real defocus.
    assert!(
        floor_witness >= PRESENCE_FLOOR,
        "the worst cell's backing let only {floor_witness:.1} luma of the document \
         reach through (floor {PRESENCE_FLOOR}) — satisfied by deleting the backing's \
         own subject rather than by defocusing it"
    );

    crate::theme::set_active(entry);
}

/// MUTATION PROOF, IN CODE: the OLD predicate (`theme::active()`, unpinned) and the
/// FIXED one (`picker_chrome_theme()`, pinned to the opener) must actually disagree on
/// at least one enrolled crossing — otherwise the headline law above could be green for
/// a reason that has nothing to do with the chrome pin. Proven by
/// evaluating both readings directly rather than by re-deriving `frost_mode`'s own
/// logic, so a future refactor of that function cannot silently stop testing anything.
#[test]
fn the_old_unpinned_read_would_have_disabled_the_frost_where_the_fixed_one_does_not() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let _pin_restore = crate::render::PickerChromePinRestore::capture();

    let opener = footprint_reliant_worlds()
        .into_iter()
        .find(|w| {
            crate::theme::THEMES
                .iter()
                .find(|t| &t.name == w)
                .is_some_and(|t| t.render_caps.backdrop != crate::theme::Backdrop::Flat)
        })
        .expect("the roster carries at least one non-flat, footprint-reliant opener");
    let target = flat_worlds()
        .into_iter()
        .next()
        .expect("the roster carries at least one Backdrop::Flat world");
    assert_ne!(opener, target);

    open_on_and_preview(opener, target);

    let old_predicate_is_flat =
        crate::theme::active().render_caps.backdrop == crate::theme::Backdrop::Flat;
    let fixed_predicate_is_flat =
        crate::render::picker_chrome_theme().render_caps.backdrop == crate::theme::Backdrop::Flat;

    assert!(
        old_predicate_is_flat,
        "setup: previewing {target} must make theme::active() report Flat, or this \
         crossing does not exercise the bug this law names"
    );
    assert!(
        !fixed_predicate_is_flat,
        "the fixed seam (picker_chrome_theme, pinned to {opener}) must NOT report Flat \
         here — {opener} is not itself a Backdrop::Flat world"
    );

    crate::theme::set_active(entry);
}

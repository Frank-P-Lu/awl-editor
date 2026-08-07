//! THE FOOT BAND vs THE CARD'S OWN TEXT COLUMN.
//!
//! Two laws share one owner here. The first is the KEYBINDINGS FOOTER's
//! no-clip law over every real ledger tip. The second is the law that first
//! one's own sweep earned: the card's width cap is a FIXED LOGICAL CONSTANT
//! with no font term, while the hint band it must hold is shaped in the active
//! world's own chrome face — so the fit is a RATIO between two quantities that
//! were never related, and on five worlds it exceeds 1.
//!
//! **WHICH QUANTITY EACH SIDE MEASURES, because the answer is not symmetric.**
//! The CONTENT side is ADVANCES: `overlay_footer_content_px` reduces the shaped
//! runs by `w = w.max(run.line_w)`, and `line_w` is cosmic-text's advance
//! total. The BUDGET side is neither advances nor extents — it is a constant:
//! `overlay.rs`'s `let text_w = card_w - 2.0 * hpad;` over a `card_w` derived
//! from `CARD_MAX_W: LogicalGrowOnly = LogicalGrowOnly(520.0)`. **There is no
//! font term in the budget at all**, which is the whole mismatch.
//!
//! **THE SHAPED-EXTENT THEORY IS FALSE, MEASURED.** The suspicion was that the
//! budget compares advances against text whose rasterized glyph CELLS are
//! wider. Unioning every hint glyph's real swash placement box
//! (`placement.left + placement.width`, the same convention
//! `caret_glyph_geometry` reads) against the advance total puts the two within
//! ±1.1px on every world, and the ink is usually the NARROWER of the pair
//! (Potoroo's Keybindings hint at scale 1.6: advances 803.2, ink 802.0). So
//! measuring extents would make these numbers worse, not better; the whole
//! overflow already lives in the advances.
//!
//! **WHY IT HIDES AT 1×, AND WHY "2×" IS NOT THE GATE.** `LogicalGrowOnly::px`
//! is `self.0 * scale.max(1.0)`, so below scale 1 the cap holds its DEVICE
//! width while the text shrinks with the scale — at the shipped default (zoom
//! 0.8, dpi 1) that makes the card 25% roomier relative to its own text than
//! it is at any scale ≥ 1. Above 1 the card is exactly proportional, so the
//! fit becomes a pure ratio and stops depending on the scale at all: Potoroo's
//! Keybindings hint measures 1.0121 of its text column at scale 1.0, at 1.6
//! and at 2.0, to four decimals. The reachable-at-shipped-zoom instance is 2×
//! because 0.8 × 2 = 1.6; zoom 1.0 on a 1× display clips identically.
//!
//! **AND THE MENU BAR IS NOT A GATE EITHER.** Both arms measure byte-identical
//! ratios: the bar's reserve moves `card_y`, and nothing in the width budget
//! reads it.
//!
//! **THE ROSTER-SIDE PROPERTY IS NOT ONE FACE.** Restricted to the Keybindings
//! hint, only the two `"Monaspace Xenon"` worlds overflow. Widened to the hint
//! CATALOG, the palette's own `Command` hint overflows on all five worlds whose
//! chrome face is a monospace used for chrome text — `"IBM Plex Mono"`
//! (Tawny), `"JetBrains Mono"` (Mangrove, Wagtail) and `"Monaspace Xenon"`
//! (Potoroo, Firetail) — so a face-name exclusion measured off ONE hint was
//! measuring the wrong axis. The ledger below is keyed by world rather than by
//! face because the deficit is a property of the whole composition: Potoroo
//! and Firetail share a face and differ by 3.8 points of ratio, because
//! `overlay_text_hpad` gives a `Bars` world `BAR_SIDE_INSET + BAR_TEXT_PAD`
//! where a `Pane` world gets `PANE_TEXT_HPAD`.
//!
//! **THE SIBLING RESIDUAL IS THE SAME DEFECT, CONFIRMED TO 0.1px.** The
//! zoom-1.0 Mangrove overflow filed as a separate follow-up — "the plain hint
//! line overflows the card's right edge by ~7.7 logical px, direction-gated" —
//! is this ledger's `Mangrove`/`Command` row: at zoom 1.0, dpi 1 the drawn
//! foot band's right edge sits **7.7px** past `overlay_card_rect`'s. It is not
//! direction-gated — `Tawny`, a plain `Pane` world with no lean at all,
//! measures the same 7.7px — and `Magpie`/`Paperbark` stay clean because their
//! chrome faces are proportional, not because of their shear.
//!
//! **NOT FIXED HERE, ON PURPOSE.** Every repair is a taste call on the card
//! itself: raising the cap (Firetail's `Command` hint needs 1.1137 of its
//! column, so ~575 logical against today's 520 — a >10% wider card on every
//! world at every zoom), letting the hint band elide or wrap, or letting the
//! content-hug measurement bound the cap for every anchor rather than only the
//! right-anchored ones (which today already sizes Cassowary's and Kite's cards
//! to their own hints, and would not help these five — they are AT the cap
//! already). Shortening the hint would make the number pass by making the
//! discoverability affordance worse, so it is not on that list. The question
//! goes to the user with these numbers; the laws below hold the measurement
//! from both ends meanwhile.
//!
//! **HOW THE LEDGER RATCHETS.** It is not an exclusion. Every roster × catalog
//! cell is graded, and a pair overflows if and only if it is ledgered, at the
//! ratio recorded: a NEW overflowing world or hint fails, a ledgered pair that
//! stops overflowing fails (so the fix cannot land unnoticed), and a ledgered
//! ratio that drifts fails. Enrolment comes from the roster and from
//! `workspace_shape()`, never from a name.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// THE MEASURED OVERFLOW LEDGER: `(world, hint kind, hint band ÷ card text
/// column)` for every flat-card cell whose hint does not fit, at every scale
/// ≥ 1. The ratio is the pinned quantity because it is the scale-free one —
/// see the module doc — so one number covers 1×, 1.6× and 2× alike.
const KNOWN_HINT_OVERFLOW: &[(&str, &str, f32)] = &[
    ("Tawny", "Command", 1.0397),
    ("Mangrove", "Command", 1.0397),
    ("Wagtail", "Command", 1.0397),
    ("Potoroo", "Command", 1.0733),
    ("Firetail", "Command", 1.1137),
    ("Potoroo", "Keybindings", 1.0121),
    ("Firetail", "Keybindings", 1.0502),
];

/// Ratios are read off f32 layout arithmetic, so the pin is to four decimals
/// with a hair of room — wide enough not to flake, far tighter than the
/// smallest ledgered deficit (121 points of 10⁻⁴).
const RATIO_TOL: f32 = 0.002;

/// What fraction of its text column this world's hint band is ALLOWED to
/// occupy: 1 for every unledgered pair, the pinned ratio for a ledgered one.
fn allowed_ratio(world: &str, kind: OverlayKind) -> f32 {
    KNOWN_HINT_OVERFLOW
        .iter()
        .find(|(w, k, _)| *w == world && *k == format!("{kind:?}"))
        .map_or(1.0, |(_, _, r)| *r)
}

/// The kinds whose card comes from `overlay_geometry` — the FLAT card, the
/// tightest width budget in the product. A workspace kind is asked by the
/// product's own predicate rather than named: `workspace_shape()` answers
/// `Some` unconditionally for `Settings`, `History` and `Conflict`, whose cards
/// are the wider faceted cap with a rail, so their hint rides a budget this
/// law does not measure and must not pin.
fn flat_card_kinds() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| k.workspace_shape().is_none())
        .filter(|k| !k.hint().is_empty())
        .collect()
}

/// Every `"{chord}  {name}"` tip the REAL ledger could ever hand the footer —
/// the whole catalog, not whichever three usage ranks first today, because
/// the true worst case is a property of the catalog's own longest entry.
fn every_real_tip() -> Vec<String> {
    let names = crate::commands::names();
    let bindings = crate::commands::effective_bindings(&[], &[]);
    names
        .iter()
        .zip(bindings.iter())
        .filter(|(_, chord)| !chord.is_empty())
        .map(|(name, chord)| format!("{chord}  {name}"))
        .collect()
}

/// One flat-card cell's `(hint band px, card text column px)`, both read off
/// the production owners a frame just committed.
fn band_and_column(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cw: u32,
    ch: u32,
    kind: OverlayKind,
    zoom: f32,
    tips: Vec<String>,
) -> (f32, f32) {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.zoom = zoom;
    v.overlay_title = kind.title();
    v.overlay_hint = kind.hint();
    v.overlay_items = vec!["Go to file".into(), "Save".into(), "Undo".into()];
    v.overlay_selected = 0;
    p.set_keybindings_tips(tips);
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).unwrap();
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    (
        p.overlay_footer_content_px(&geom, plan.content_rows()),
        geom.text_w,
    )
}

// `set_keybindings_tips` is the discoverability ledger's own native-only door
// (`chrome/hud.rs`) — a headless/wasm build never populates it, so this law,
// which is entirely about that footer, is native-only too.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_keybindings_footer_never_clips_for_any_real_ledger_tip() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Keybindings footer no-clip law: no wgpu adapter");
        return;
    };
    let tips = every_real_tip();
    assert!(
        tips.len() > 10,
        "the catalog sweep must actually run, got {} tips",
        tips.len()
    );
    let ambient_bar = crate::menubar::menu_bar_on();
    let names = crate::theme::world_names();
    let mut graded = 0usize;
    let mut presence_graded = 0usize;
    for bar in [ambient_bar, !ambient_bar] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for &world in &names {
                crate::theme::set_active_by_name(world).expect("a named world exists");
                p.sync_theme();
                // Every world/tip combination shapes fresh glyphs into the ONE shared
                // atlas; a live frame loop reclaims it every frame (`p.atlas.trim()`),
                // but this tight sweep never presents a frame, so it must trim itself
                // or a long sweep (roster × catalog × dpi × bar) exhausts it
                // (`AtlasFull`) well before any geometry assertion fires.
                p.atlas.trim();
                // The hint band's own known deficit is this world's floor: with the
                // tips present the band is the WIDER of the two, so a tip may never
                // add to an overflow the hint already carries.
                let allow = allowed_ratio(world, OverlayKind::Keybindings);
                for tip in &tips {
                    let (footer_px, text_w) = band_and_column(
                        &mut p,
                        &device,
                        &queue,
                        cw,
                        ch,
                        OverlayKind::Keybindings,
                        0.8, // the shipped default render zoom (what `--screenshot` renders)
                        vec![tip.clone()],
                    );
                    assert!(
                        footer_px > 1.0,
                        "{world} dpi={dpi} bar={bar}: the footer must actually shape glyphs \
                         for tip {tip:?} — a clip floor here would be satisfied by the tip \
                         having vanished"
                    );
                    presence_graded += 1;
                    assert!(
                        footer_px <= text_w * allow + RATIO_TOL * text_w,
                        "{world} dpi={dpi} bar={bar}: Keybindings footer {footer_px:.1}px \
                         clips the card's {text_w:.1}px text column (allowance {allow:.4}) \
                         for tip {tip:?}"
                    );
                    graded += 1;
                }
            }
        }
    }
    p.set_dpi(1.0);
    p.set_keybindings_tips(Vec::new());
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(graded > 500, "the sweep must actually run, got {graded}");
    assert_eq!(
        graded, presence_graded,
        "every graded cell must have passed the presence floor too"
    );
}

/// THE LEDGER IS EXACT, AND THE DEFICIT IS A SCALE-FREE RATIO. Sweeps the
/// roster × the flat-card hint catalog × 1×/2× × both menu-bar arms × the
/// shipped zoom AND zoom 1.0, and asserts three things the module doc argues
/// for: nothing overflows below scale 1 (the grow-only slack is why this hid),
/// the overflow SET at scale ≥ 1 is exactly the ledger, and each ratio is the
/// same number at every scale ≥ 1.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_hint_band_overflow_ledger_is_exact_and_scale_free() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping hint-band overflow ledger law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let names = crate::theme::world_names();
    let kinds = flat_card_kinds();
    assert!(
        kinds.len() > 10 && names.len() > 10,
        "the sweep's own axes must be populated, got {} kinds x {} worlds",
        kinds.len(),
        names.len()
    );
    // (world, kind) -> the ratios seen at every scale >= 1, so the invariance
    // claim is checked against measurements rather than asserted.
    let mut seen: std::collections::BTreeMap<(&str, String), Vec<f32>> =
        std::collections::BTreeMap::new();
    let mut graded = 0usize;
    for bar in [ambient_bar, !ambient_bar] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for zoom in [0.8f32, 1.0] {
                let scale = zoom * dpi;
                for &world in &names {
                    crate::theme::set_active_by_name(world).expect("a named world exists");
                    p.sync_theme();
                    p.atlas.trim();
                    for &kind in &kinds {
                        let (band, column) = band_and_column(
                            &mut p,
                            &device,
                            &queue,
                            cw,
                            ch,
                            kind,
                            zoom,
                            Vec::new(),
                        );
                        assert!(
                            band > 1.0 && column > 1.0,
                            "{world} {kind:?} zoom={zoom} dpi={dpi} bar={bar}: the hint band \
                             must actually shape glyphs into a real column — a ratio floor is \
                             satisfied by an absent hint (band {band:.1}, column {column:.1})"
                        );
                        let ratio = band / column;
                        if scale < 1.0 {
                            assert!(
                                ratio <= 1.0 + RATIO_TOL,
                                "{world} {kind:?} zoom={zoom} dpi={dpi} bar={bar}: nothing may \
                                 overflow below scale 1, where `LogicalGrowOnly` holds the cap's \
                                 device width and the card runs 1/scale roomier than its text — \
                                 got {ratio:.4} ({band:.1}px in {column:.1}px)"
                            );
                        } else {
                            let pinned = allowed_ratio(world, kind);
                            assert!(
                                (ratio - pinned.max(1.0)).abs() <= RATIO_TOL || ratio < 1.0,
                                "{world} {kind:?} zoom={zoom} dpi={dpi} bar={bar} scale={scale}: \
                                 hint band ÷ text column is {ratio:.4}, and the ledger says \
                                 {pinned:.4}. A new overflow, a repaired one, or a drifted \
                                 deficit — all three land here, and all three want the ledger \
                                 re-measured rather than widened"
                            );
                            if ratio > 1.0 + RATIO_TOL {
                                seen.entry((world, format!("{kind:?}")))
                                    .or_default()
                                    .push(ratio);
                            }
                        }
                        graded += 1;
                    }
                }
            }
        }
    }
    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(
        graded > 2000,
        "the roster x catalog x dpi x bar x zoom sweep must actually run, got {graded}"
    );
    // The set, both ways: the ledger holds no pair the product does not
    // overflow on, and the product overflows on no pair the ledger omits.
    let observed: Vec<(&str, String)> = seen.keys().cloned().collect();
    let mut expected: Vec<(&str, String)> = KNOWN_HINT_OVERFLOW
        .iter()
        .map(|(w, k, _)| (*w, (*k).to_string()))
        .collect();
    expected.sort();
    assert_eq!(
        observed, expected,
        "the overflow set moved. Every entry is a live, unfixed clip of the card's own \
         chrome, so a pair leaving this list means the width budget was repaired (say so, \
         and delete the row) and a pair joining it means a world or a hint just started \
         clipping"
    );
    // Each pair's ratio is ONE number across every scale >= 1 — the claim that
    // "2x" is not the gate, checked rather than asserted.
    for ((world, kind), ratios) in &seen {
        let lo = ratios.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = ratios.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            ratios.len() >= 6 && hi - lo <= RATIO_TOL,
            "{world} {kind}: the deficit must be the SAME ratio at every scale >= 1 (that is \
             what makes it a font-versus-fixed-cap mismatch rather than a 2x bug) — saw \
             {} samples spanning {lo:.4}..{hi:.4}",
            ratios.len()
        );
    }
}

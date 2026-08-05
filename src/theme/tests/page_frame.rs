use super::super::*;

/// THE PAGE-FRAME THEME LAW: the frame can never invent a color — its ink is
/// `page_frame_ink` is always the world's `base_content`; assigned weights are
/// positive. The pixel half lives in `render::tests::page_frame`.
#[test]
fn page_frame_ink_is_the_ladder_and_assigned_weights_are_real() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        set_active_by_name(t.name).unwrap();
        assert_eq!(
            derive::page_frame_ink(),
            t.base_content,
            "{}: page_frame_ink must be exactly the world's own base_content",
            t.name
        );
        // An INK-CARET world's caret IS its ink (`primary == base_content`;
        // presence carried by the inverting/filled block, not a hue — Wagtail's
        // pure white, Cassowary's phosphor green), so "never literally primary" is
        // structurally inapplicable there (the frame ink == base_content == primary
        // BY DESIGN); every other world must keep frame-ink and accent distinct.
        if !t.ink_caret() {
            assert_ne!(
                derive::page_frame_ink(),
                t.primary,
                "{}: the page-frame ink must never be literally the accent",
                t.name
            );
        }
        if let model::PageFrame::Line { weight_px } = t.render_caps.page_frame {
            assert!(
                weight_px > 0.0 && weight_px.is_finite(),
                "{}: an assigned page frame must carry a real positive weight (got {weight_px})",
                t.name
            );
        }
    }
    set_active(DEFAULT_THEME);
}

/// THE SPELL-SQUIGGLE PER-WORLD BASELINE DIAL: every world carries
/// [`model::SPELL_UNDERLINE_GAP_DEFAULT`] (byte-identical to the pre-dial
/// hardcoded gap) EXCEPT Bilby, whose report ("the squiggle floats too far
/// below the baseline") earned a tighter, strictly SMALLER override — DATA on
/// `RenderCaps`, never a per-world code path (`render/tests/theme_caps_law.rs`
/// structurally bans a `"Bilby"` string or `.is_one_bit()` read under
/// `src/render/`). No-wildcard over `THEMES`, so a future 19th world defaults
/// through `RenderCaps::DEFAULT` until it consciously opts in too.
#[test]
fn spell_underline_gap_is_the_shared_default_everywhere_except_bilbys_tighter_dial() {
    for t in THEMES.iter() {
        if t.name == "Bilby" {
            assert!(
                t.render_caps.spell_underline_gap < model::SPELL_UNDERLINE_GAP_DEFAULT,
                "Bilby must carry a STRICTLY tighter (smaller) gap than the shared default \
                 ({} vs default {})",
                t.render_caps.spell_underline_gap,
                model::SPELL_UNDERLINE_GAP_DEFAULT
            );
        } else {
            assert_eq!(
                t.render_caps.spell_underline_gap,
                model::SPELL_UNDERLINE_GAP_DEFAULT,
                "{}: every world but Bilby stays on the shared default gap",
                t.name
            );
        }
    }
}

/// WCAG relative-contrast ratio between two opaque colors, gamma-correct Rec.709
/// — a small, deliberate duplication of `render::tests::distinguishability`'s own
/// copy (the accepted shape this codebase already carries for a tiny pure-math
/// helper needed at two test seams; see that file's `redmean`/`srgba_u8_to_linear`
/// precedent doc).
fn wcag_contrast(a: Srgb, b: Srgb) -> f32 {
    fn rel_lum(c: Srgb) -> f32 {
        fn lin(u: u8) -> f32 {
            let s = u as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }
    let (la, lb) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// item 65 NAMED TAIL CONTRAST FLOOR: the bare `theme::faint()` LADDER TOKEN
/// (before any [`super::model::FoldAfford`] lift — see that capability's own
/// doc) must be QUIET but PLAINLY READABLE against the FLAT `base_100` ground —
/// two bounds, on EVERY world:
///
/// (a) READABLE FLOOR — `faint` clears [`FOLD_TAIL_READABLE_FLOOR`] WCAG contrast
///     against `base_100`. The floor (1.8:1) is set just below every world's OWN
///     existing `faint`-vs-`base_100` contrast (empirically probed: 1.97:1 on
///     Mangrove, the global minimum among the 17 non-1-bit worlds, up to 3.20:1
///     on Potoroo) — a REGRESSION guard on the ink `faint` already ships with,
///     not a new design target this round invents; well below WCAG's own 3:1
///     "UI component" floor because `faint` is deliberately the ink ladder's
///     dimmest rung (`Theme::faint`'s own doc: "must barely register"), and well
///     above 1:1 (true invisibility).
///
///     CAVEAT (the Fable item 65 adjustment round): `base_100` is NOT the fold
///     tail's real drawn ground on Mangrove/Firetail — the lamp's edge-glow
///     "soft light-spill under the column" lifts the WHOLE writing column, not
///     only the margin edge (a screenshot pixel probe found the real rendered
///     ground far brighter than `base_100`, e.g. Mangrove `(0x49,0x6D,0x68)` vs
///     `base_100` `(0x11,0x27,0x23)`), which `Theme::background`'s "the page
///     column itself stays flat" doc does not anticipate. This law still holds
///     (it is a real, if incomplete, regression guard on the bare `faint`
///     token, still drawn as-is on every non-lava world and as the tail's
///     un-lifted STARTING point on a lava one) but it is NOT sufficient on its
///     own for the two lava worlds — `capture::tests::folds::
///     fold_afford_ink_clears_the_real_lava_ground_on_every_flagged_world`
///     is the one that proves the ACTUALLY-drawn (possibly lifted) ink against
///     the ACTUALLY-rendered ground, over a real captured PNG.
/// (b) QUIET BOUND — `faint`'s contrast stays STRICTLY BELOW `base_content`'s
///     (the heading's own ink — headings carry no separate color, only size/
///     weight, per the four-role/no-rainbow philosophy) against the SAME ground,
///     so the tail never out-salience the heading it hangs on. EXEMPT on a TRUE
///     1-BIT world ([`Theme::is_one_bit`]): "the ink ladder COLLAPSES to one
///     value in a true 1-bit world — there is nothing else to step through"
///     (Wagtail's own doc comment) is an EXISTING, already-shipped, already
///     law-tested (`wagtail_alone_is_one_bit` + this file's several other
///     `is_one_bit` exemption arms) compensating fact: `faint == base_content`
///     there BY DESIGN, so the two contrasts are trivially EQUAL, never `<`. This
///     mirrors every other ink-ladder law's own declared 1-bit exemption rather
///     than inventing a new one.
#[test]
fn fold_tail_ink_clears_the_readable_floor_and_stays_quieter_than_heading_ink() {
    const FOLD_TAIL_READABLE_FLOOR: f32 = 1.8;
    let _g = crate::testlock::serial();

    for t in THEMES.iter() {
        let faint_c = wcag_contrast(t.faint, t.base_100);
        let content_c = wcag_contrast(t.base_content, t.base_100);
        assert!(
            faint_c >= FOLD_TAIL_READABLE_FLOOR,
            "{}: the fold-tail ink (faint {:?} on base_100 {:?}) is only {faint_c:.2}:1, \
             below the readable floor {FOLD_TAIL_READABLE_FLOOR}:1",
            t.name,
            t.faint,
            t.base_100
        );
        if t.is_one_bit() {
            assert_eq!(
                faint_c, content_c,
                "{}: a true 1-bit world's ink ladder collapses to one value — faint IS \
                 the heading ink here, by design (see Theme::is_one_bit's doc)",
                t.name
            );
        } else {
            assert!(
                faint_c < content_c,
                "{}: the fold-tail ({faint_c:.2}:1) must read QUIETER than the heading ink \
                 ({content_c:.2}:1) it hangs on, else it out-salience the heading it annotates",
                t.name
            );
        }
    }
}

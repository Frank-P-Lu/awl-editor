//! Raw per-face MEASUREMENT primitives the parent module's `roster()` calls
//! once per bundled face — glyph-metric/outline reads through skrifa. No
//! caching and no roster assembly here; that stays the parent module's job.

use glyphon::cosmic_text::skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

/// The TYPICAL-LETTER RATIO used when no bundled/measured face is known for a
/// family (a system fallback face, an `AWL_FONT` override, or a bundled face
/// whose own `OS/2`/`post` tables carry neither x-height nor cap-height). Not
/// load-bearing precision — this is a FALLBACK for the case the real
/// measurement below cannot answer, and the value only ever feeds
/// [`super::super::caret::TextPipeline::caret_cell_vertical`]'s GLYPHLESS
/// synthetic box, never a real glyph's own ink.
pub(crate) const DEFAULT_TYPICAL_LETTER_RATIO: f32 = 0.62;

/// MEASURE one font file's own TYPICAL-LETTER-TO-ASCENT ratio: how
/// tall a "generic" letter's ink sits relative to the font's own ascent, read
/// straight from the face's `OS/2`/`hhea` tables through the SAME skrifa
/// `metrics()` call every other per-face fact here uses.
///
/// Deliberately NOT the bare x-height. A glyphless anchor (space / end-of-line
/// / an empty line / a ligature) has no letter of its own, so ANY single fixed
/// reference is an approximation — but the two obvious single choices both
/// under-serve one of the two glyph classes the caret's own ink-box arm treats
/// as routinely different heights:
///   * x-height alone reproduces the ORIGINAL bug in miniature at the
///     seam for an ASCENDER neighbour (`l`/`h`/`b`/`d`) — x-height sits well
///     below a real ascender's ink top, so the fallback would visibly SHRINK
///     leaving a tall letter for end-of-line;
///   * cap-height alone reintroduces the bug's ORIGINAL direction for an
///     ORDINARY x-height letter (`a`/`m`/`e`) — the literal case the user's
///     `aaa` fixture reports — hanging empty accent space above it again.
///     The MEAN of the two is the balance point: still strictly font-measured (no
///     hand-tuned per-world offset), and it halves the worst-case residual against
///     EITHER class rather than zeroing one at the other's expense. `Size::unscaled()`
///     keeps every quantity in font design units, so the ratio is a pure per-font
///     constant independent of the font size a row happens to be shaped at — the
///     caller multiplies it by that ROW's own (already size/zoom/DPI-scaled)
///     `max_ascent` to get a real pixel height.
///
/// Falls back to [`DEFAULT_TYPICAL_LETTER_RATIO`] when the file won't parse or
/// the face declares NEITHER metric (some symbol/geometric faces don't); a
/// face with only one of the two uses that one alone rather than discarding a
/// real measurement.
pub(super) fn typical_letter_ratio(bytes: &[u8]) -> f32 {
    let Ok(font) = FontRef::new(bytes) else {
        return DEFAULT_TYPICAL_LETTER_RATIO;
    };
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    if m.ascent <= 0.0 {
        return DEFAULT_TYPICAL_LETTER_RATIO;
    }
    let xh = m.x_height.filter(|v| *v > 0.0);
    let ch = m.cap_height.filter(|v| *v > 0.0);
    let px = match (xh, ch) {
        (Some(xh), Some(ch)) => (xh + ch) * 0.5,
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => return DEFAULT_TYPICAL_LETTER_RATIO,
    };
    (px / m.ascent).clamp(0.2, 0.95)
}

/// The ASCENT/DESCENT em fractions used when no bundled/measured face is known
/// for a family (a system fallback face, an `AWL_FONT` override, an unparseable
/// file). Same shape as [`DEFAULT_TYPICAL_LETTER_RATIO`]: a fallback for the
/// case the real measurement cannot answer, never a value a bundled face rides.
pub(crate) const DEFAULT_ASCENT_EM: f32 = 0.8;
pub(crate) const DEFAULT_DESCENT_EM: f32 = 0.2;

/// MEASURE one font file's own ASCENT and DESCENT as fractions of its em
/// square — the two quantities cosmic-text folds into a shaped row's
/// `max_ascent`/`max_descent` (`shape.rs`: `ascent = metrics.ascent /
/// units_per_em`, then `max_ascent = font_size * glyph.ascent`).
///
/// This exists because a row with NO GLYPHS carries neither: cosmic-text emits
/// an empty line's `LayoutLine` with `max_ascent: 0.0, max_descent: 0.0`, so
/// the caret's row lookup has no measured band to size or centre against on an
/// empty line. Multiplying these by that row's own font size reconstructs the
/// exact pair the row WOULD have carried had it held one letter — the same
/// arithmetic, one factor earlier. Both are returned together, never
/// separately: a baseline needs the pair, and splitting them invites one being
/// read against the other's font.
///
/// Read through the SAME skrifa `metrics()` call every other per-face fact
/// here uses. That is not the same crate cosmic-text shapes with (swash), so
/// "these agree with a real shaped row" is a claim about two font stacks and
/// is MEASURED, not assumed: `render::tests::caret_transition`'s
/// `an_empty_line_carries_the_row_metrics_a_letter_would_have_given_it` pins
/// the reconstruction against a really-shaped one-letter row on every bundled
/// display family, and fails if the two stacks ever read a face differently.
pub(super) fn vertical_em_metrics(bytes: &[u8]) -> (f32, f32) {
    let Ok(font) = FontRef::new(bytes) else {
        return (DEFAULT_ASCENT_EM, DEFAULT_DESCENT_EM);
    };
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    if m.units_per_em == 0 || m.ascent <= 0.0 {
        return (DEFAULT_ASCENT_EM, DEFAULT_DESCENT_EM);
    }
    let upem = m.units_per_em as f32;
    // `descent` is negative in the font's own convention (below the baseline);
    // cosmic-text negates it at the same point, so this carries the same sign
    // the caller's `max_descent` has.
    (m.ascent / upem, -m.descent / upem)
}

/// The representative vertical-ink roster [`ink_envelope_em`] reads real
/// OUTLINE bounds for: true ascenders and true descenders in both cases,
/// digits, and the punctuation marks whose own ink runs vertically furthest on
/// an ordinary display face — a paren/bracket/brace routinely clears a
/// lowercase ascender, a comma/semicolon routinely clears a lowercase
/// descender. Covers both letter cases so a face whose capitals reach higher
/// than its lowercase ascenders (routine) is not missed.
const INK_ENVELOPE_PROBE: &str = "AaBbDdFfGgHhJjKkLlMmPpQqTtYy0123456789.,;:!?()[]{}\"'-";

/// The ASCENT/DESCENT em fractions [`ink_envelope_em`] falls back to — same
/// shape and same values as [`DEFAULT_ASCENT_EM`]/[`DEFAULT_DESCENT_EM`],
/// reused rather than duplicated: both are "a plausible letter's vertical
/// reach" in the absence of a real measurement, and inventing a second numeric
/// pair for the same kind of fallback would be two sources for one fact.
pub(crate) const DEFAULT_INK_ASCENT_EM: f32 = DEFAULT_ASCENT_EM;
pub(crate) const DEFAULT_INK_DESCENT_EM: f32 = DEFAULT_DESCENT_EM;

/// MEASURE one font file's own real ink extremes over [`INK_ENVELOPE_PROBE`]:
/// the tallest probed ascender's outline top and the lowest probed
/// descender's outline bottom, each as a fraction of the face's own em
/// square.
///
/// Deliberately NOT [`vertical_em_metrics`]'s `hhea` ascent/descent, and NOT
/// [`typical_letter_ratio`]'s x-height/cap-height mean. `hhea` ascent/descent
/// are LINE-SPACING metrics — generous by design so stacked lines never
/// collide — and on several bundled faces their SUM alone exceeds the app's
/// own configured row height (Literata: ascent 1.177em + descent 0.308em =
/// 1.485em, against a body row of ~1.33em font-size multiples); a caret box
/// sized to them would routinely reach into the row above or below it, which
/// is the exact "never touches an adjacent row" failure the proportional
/// Block caret's envelope law exists to catch. The typical-letter ratio is
/// the RIGHT quantity for the row's GLYPHLESS fallback (a synthetic "how tall
/// is an ordinary letter" reference with no real glyph to measure) but is, by
/// its own design, tuned to the MEAN letter rather than the extremes — the
/// exact under-coverage the Block caret's own envelope must not have.
///
/// A real glyph's drawn ink is reliably shorter than the line-spacing metric
/// that makes room for it, so this reads the face's own OUTLINE bounds
/// (`skrifa::metrics::GlyphMetrics::bounds`, the same call the crate uses
/// internally for its CFF/`gvar` fallback path — an integer glyf box or a
/// `ControlBoundsPen` walk of the real bezier data, never a synthesized
/// approximation) over a roster of genuine ascenders, descenders and
/// punctuation, and takes the MAX across the roster: the envelope a real
/// anchored glyph can never exceed.
///
/// Falls back to [`DEFAULT_INK_ASCENT_EM`]/[`DEFAULT_INK_DESCENT_EM`] when the
/// file won't parse, carries no outline data, or not one probe glyph
/// resolves to real ink.
pub(super) fn ink_envelope_em(bytes: &[u8]) -> (f32, f32) {
    let fallback = (DEFAULT_INK_ASCENT_EM, DEFAULT_INK_DESCENT_EM);
    let Ok(font) = FontRef::new(bytes) else {
        return fallback;
    };
    let charmap = font.charmap();
    let glyph_metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    let upem = font
        .metrics(Size::unscaled(), LocationRef::default())
        .units_per_em;
    if upem == 0 {
        return fallback;
    }
    let upem = upem as f32;
    let (mut top, mut bottom, mut seen) = (0.0f32, 0.0f32, false);
    for ch in INK_ENVELOPE_PROBE.chars() {
        let Some(gid) = charmap.map(ch) else {
            continue;
        };
        let Some(b) = glyph_metrics.bounds(gid) else {
            continue;
        };
        if b.y_max <= b.y_min {
            continue; // an empty outline (this face has no ink for the glyph)
        }
        top = top.max(b.y_max);
        bottom = bottom.max(-b.y_min);
        seen = true;
    }
    if !seen {
        return fallback;
    }
    (top.max(0.0) / upem, bottom.max(0.0) / upem)
}

/// A CJK face's one-em ideographic cell, split above and below the baseline by
/// its own OS/2 typographic ascender/descender. The split is normalised to one
/// em: CJK body glyphs occupy the em square, while hhea's deliberately generous
/// line metrics include room that belongs to the row rather than the glyph cell.
pub(super) fn ideographic_cell_em(bytes: &[u8]) -> Option<(f32, f32)> {
    let font = FontRef::new(bytes).ok()?;
    let os2 = font.os2().ok()?;
    let ascent = os2.s_typo_ascender() as f32;
    let descent = -(os2.s_typo_descender() as f32);
    let total = ascent + descent;
    if ascent <= 0.0 || descent < 0.0 || total <= 0.0 {
        return None;
    }
    Some((ascent / total, descent / total))
}

//! THE TWO-INK LAW, asserted over PIXELS — what actually rendered, not what
//! the source constants say it should have.
//!
//! [`super::Ink`] is enforced from the source end by `ink_color` being the one
//! resolver with a no-wildcard match. That is necessary and not sufficient: it
//! says nothing about what AppKit ultimately drew, and it cannot see a rule, a
//! box, a bezel or a separator — none of which is text and none of which goes
//! through `place`. So this module reads a real capture of the rendered window
//! and answers two questions directly:
//!
//! 1. **How many distinct text inks are on screen?** Asked PER ELEMENT: point
//!    [`element_ink`] at one label's own frame and it reports the colour that
//!    label actually drew in, read off the fully-covered core of its glyphs.
//!    [`distinct_roles`] then collapses those answers, and the law demands the
//!    window gave exactly two.
//! 2. **Did any divider survive?** A rule is not text: it is a long, thin,
//!    horizontal run of near-constant non-background colour with nothing above
//!    or below it. [`divider_rows`] looks for exactly that shape, so an `NSBox`
//!    separator, a 1px filled view or a bordered box all trip it, whatever the
//!    source called itself.
//!
//! **What this module can and cannot be run against.** The About window is a
//! live `NSPanel`; there is no window server in a `--screenshot` run and no
//! main thread in a `cargo test` worker, so awl's headless harness CANNOT
//! render it and this analysis has no way to produce its own input in CI. The
//! honest split, therefore:
//!
//! * The ANALYSIS is unit-tested here against synthetic images, so its own
//!   logic is proven — it really does find a third ink, and really does find a
//!   divider, when one is present.
//! * The RENDER is checked against committed captures of the packaged app
//!   (`tests/fixtures/about/`), taken through `CGWindowListCreateImage` at
//!   native resolution. [`super::tests`] pins their dimensions to
//!   [`super::layout`]'s own arithmetic, so any change to the composition's
//!   geometry fails the fixture test until the capture is retaken — a fixture
//!   that has silently stopped describing the code goes red rather than green.
//! * A pure colour change that leaves geometry alone is the one edit this
//!   coupling does not catch; it is caught by the source-side `Ink` roster
//!   instead. Neither end is claimed to be sufficient alone.

use image::RgbaImage;

/// Two inks are the same ROLE when every channel is within this. Subpixel
/// positioning and gamma jitter a glyph's core by a few levels; two genuinely
/// different label inks are tens of levels apart.
pub const INK_TOLERANCE: i32 = 16;

/// A pixel must differ from the background by at least this on some channel to
/// count as ink at all. Below it a pixel is background, or the faintest edge of
/// an antialiased stem, and carries no role information.
const INK_MIN_DELTA: i32 = 24;

/// Where on the ink→background ramp an element's ink is read, as a fraction of
/// its covered pixels ordered from most to least covered.
///
/// **Why a percentile and not the maximum.** Rendered text is not flat colour:
/// every glyph is a ramp from the background to its ink, and only the fully
/// covered core pixels sit AT the ink. Taking the single most extreme pixel
/// would chase gamma and subpixel outliers; taking a histogram of all colours
/// (the first version of this module) counts each rung of the antialiasing ramp
/// as its own "ink" and reported seven roles for a two-role window. Reading a
/// short way into the sorted-by-coverage list lands squarely in the core and is
/// stable across both appearances.
const INK_CORE_PERCENTILE: f64 = 0.10;

/// An element with fewer covered pixels than this rendered no text worth
/// classifying — the caller is pointed at the wrong frame.
const MIN_COVERED_PIXELS: usize = 40;

fn channel_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> i32 {
    let d = |x: u8, y: u8| (x as i32 - y as i32).abs();
    d(a.0, b.0).max(d(a.1, b.1)).max(d(a.2, b.2))
}

/// The most common colour in a region — the local background. For a text frame
/// that is the window's fill; for the inside of a button it is the bezel, which
/// is exactly what a button label should be measured against.
pub fn background(img: &RgbaImage, region: (u32, u32, u32, u32)) -> (u8, u8, u8) {
    let (x0, y0, x1, y1) = region;
    let mut histogram: std::collections::HashMap<(u8, u8, u8), usize> =
        std::collections::HashMap::new();
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            let p = img.get_pixel(x, y).0;
            *histogram.entry((p[0], p[1], p[2])).or_default() += 1;
        }
    }
    histogram
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or((0, 0, 0))
}

/// The ink ONE text element is set in, read off the rendered pixels of its own
/// frame. `None` when the frame holds no text.
///
/// This is the unit the two-ink law is built from: ask each element what colour
/// it actually drew in, then count how many distinct answers the window gave.
pub fn element_ink(img: &RgbaImage, region: (u32, u32, u32, u32)) -> Option<(u8, u8, u8)> {
    let bg = background(img, region);
    let (x0, y0, x1, y1) = region;
    let mut covered: Vec<((u8, u8, u8), i32)> = Vec::new();
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            let p = img.get_pixel(x, y).0;
            let c = (p[0], p[1], p[2]);
            let d = channel_distance(c, bg);
            if d >= INK_MIN_DELTA {
                covered.push((c, d));
            }
        }
    }
    if covered.len() < MIN_COVERED_PIXELS {
        return None;
    }
    covered.sort_by(|a, b| b.1.cmp(&a.1));
    Some(covered[(covered.len() as f64 * INK_CORE_PERCENTILE) as usize].0)
}

/// Collapse a list of per-element inks into the distinct ROLES they represent,
/// in first-seen order.
pub fn distinct_roles(inks: &[(u8, u8, u8)]) -> Vec<(u8, u8, u8)> {
    let mut roles: Vec<(u8, u8, u8)> = Vec::new();
    for ink in inks {
        if !roles
            .iter()
            .any(|r| channel_distance(*r, *ink) <= INK_TOLERANCE)
        {
            roles.push(*ink);
        }
    }
    roles
}

/// Relative luminance, for ordering roles by how loud they are.
pub fn luminance(c: (u8, u8, u8)) -> f64 {
    0.2126 * c.0 as f64 + 0.7152 * c.1 as f64 + 0.0722 * c.2 as f64
}

/// How wide a horizontal run of non-background pixels has to be, as a fraction
/// of the audited region, before it is a DIVIDER rather than a word. The
/// narrowest rule worth drawing still spans a good part of the column; the
/// widest line of text in this window is broken by inter-word gaps every few
/// characters, so it never produces one unbroken run this long.
const DIVIDER_WIDTH_FRACTION: f64 = 0.25;

/// How TALL a long horizontal band may be and still be a rule. At the 2x
/// capture scale a hairline is 2px and a 2pt rule is 4px; anything taller that
/// spans the column is a filled panel or artwork, not a divider.
const MAX_DIVIDER_THICKNESS: u32 = 6;

/// The rows that look like a drawn rule: one long unbroken horizontal run of
/// non-background pixels, with background directly above and below it.
///
/// Shape-based on purpose. A law that looked for `NSBox` in the source would
/// pass the moment someone drew the same line with a filled `NSView`, a bezel
/// or a border; this one asks what the pixels do.
pub fn divider_rows(
    img: &RgbaImage,
    region: (u32, u32, u32, u32),
    background: (u8, u8, u8),
) -> Vec<u32> {
    let (x0, y0, x1, y1) = region;
    let x1 = x1.min(img.width());
    let y1 = y1.min(img.height());
    let min_run = ((x1 - x0) as f64 * DIVIDER_WIDTH_FRACTION) as u32;
    let is_ink = |x: u32, y: u32| {
        let p = img.get_pixel(x, y).0;
        channel_distance((p[0], p[1], p[2]), background) >= INK_MIN_DELTA
    };
    let longest_run = |y: u32| {
        let (mut best, mut run) = (0u32, 0u32);
        for x in x0..x1 {
            if is_ink(x, y) {
                run += 1;
                best = best.max(run);
            } else {
                run = 0;
            }
        }
        best
    };
    // Walk maximal BANDS of consecutive long-run rows. A rule is a thin band
    // bounded by background; a paragraph of text never produces a long unbroken
    // run at all, and a filled panel produces a band far too tall to be a rule.
    // Banding (rather than testing single rows) is what makes the law catch a
    // 2px rule as readily as a 1px hairline — the first version of this
    // function checked `row above and below are short`, which a 2px rule slips
    // straight through, and the synthetic sweep caught it.
    let mut rows = Vec::new();
    let mut band_start: Option<u32> = None;
    for y in y0..y1 {
        let long = longest_run(y) >= min_run;
        match (long, band_start) {
            (true, None) => band_start = Some(y),
            (false, Some(start)) => {
                if y - start <= MAX_DIVIDER_THICKNESS {
                    rows.push(start);
                }
                band_start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = band_start
        && y1 - start <= MAX_DIVIDER_THICKNESS
    {
        rows.push(start);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: (u8, u8, u8) = (30, 30, 30);
    const SECONDARY: (u8, u8, u8) = (140, 140, 140);
    const THIRD: (u8, u8, u8) = (200, 200, 200);
    const BG: (u8, u8, u8) = (255, 255, 255);

    /// A synthetic "text line" WITH AN ANTIALIASING RAMP — the property that
    /// broke the first version of this module. Each stroke is a solid core
    /// flanked by two partially-covered edge columns, exactly as real glyph
    /// rasterization produces, so a classifier that mistakes a ramp rung for an
    /// ink is caught here rather than on a fixture.
    fn line(img: &mut RgbaImage, y: u32, h: u32, ink: (u8, u8, u8)) {
        let blend = |a: u8, b: u8, t: f64| (a as f64 + (b as f64 - a as f64) * t) as u8;
        for word in 0..5u32 {
            let x0 = 30 + word * 60;
            for stroke in 0..4u32 {
                let x = x0 + stroke * 10;
                for yy in y..(y + h) {
                    // Two ramp columns either side of a solid 4px core.
                    for (dx, t) in [
                        (0u32, 0.35),
                        (1, 0.75),
                        (2, 1.0),
                        (3, 1.0),
                        (4, 1.0),
                        (5, 1.0),
                        (6, 0.75),
                        (7, 0.35),
                    ] {
                        let c = (
                            blend(BG.0, ink.0, t),
                            blend(BG.1, ink.1, t),
                            blend(BG.2, ink.2, t),
                        );
                        img.put_pixel(x + dx, yy, image::Rgba([c.0, c.1, c.2, 255]));
                    }
                }
            }
        }
    }

    fn canvas() -> RgbaImage {
        RgbaImage::from_pixel(400, 400, image::Rgba([BG.0, BG.1, BG.2, 255]))
    }

    #[test]
    fn an_elements_ink_is_its_core_not_a_rung_of_its_antialiasing_ramp() {
        for ink in [BODY, SECONDARY, THIRD] {
            let mut img = canvas();
            line(&mut img, 100, 14, ink);
            let found = element_ink(&img, (0, 95, 400, 120)).expect("the line has text");
            assert!(
                channel_distance(found, ink) <= INK_TOLERANCE,
                "read {found:?} for a line set in {ink:?} — the ramp was \
                 mistaken for the ink"
            );
        }
    }

    #[test]
    fn a_two_role_window_reports_two_roles() {
        let mut img = canvas();
        line(&mut img, 40, 20, BODY);
        line(&mut img, 90, 14, BODY);
        line(&mut img, 160, 12, SECONDARY);
        line(&mut img, 200, 14, BODY);
        let inks: Vec<_> = [(35u32, 65u32), (85, 110), (155, 178), (195, 220)]
            .iter()
            .filter_map(|(a, b)| element_ink(&img, (0, *a, 400, *b)))
            .collect();
        assert_eq!(inks.len(), 4, "every band must classify");
        assert_eq!(
            distinct_roles(&inks).len(),
            2,
            "a body-and-secondary window resolves to two roles, got {inks:?}"
        );
    }

    /// The law must actually SEE a third role, or "exactly two" is unfalsifiable.
    #[test]
    fn a_third_role_is_detected() {
        let mut img = canvas();
        line(&mut img, 40, 20, BODY);
        line(&mut img, 90, 14, SECONDARY);
        line(&mut img, 160, 14, THIRD);
        let inks: Vec<_> = [(35u32, 65u32), (85, 110), (155, 180)]
            .iter()
            .filter_map(|(a, b)| element_ink(&img, (0, *a, 400, *b)))
            .collect();
        assert_eq!(
            distinct_roles(&inks).len(),
            3,
            "three distinct label inks must be reported as three, got {inks:?}"
        );
    }

    /// A button label is read against its BEZEL, not the window fill — so a
    /// bezel is never itself mistaken for an ink role.
    #[test]
    fn a_label_on_a_bezel_reads_against_the_bezel() {
        let mut img = canvas();
        for y in 300..340 {
            for x in 100..300 {
                img.put_pixel(x, y, image::Rgba([236, 236, 236, 255]));
            }
        }
        line(&mut img, 312, 14, BODY);
        assert_eq!(background(&img, (100, 300, 300, 340)), (236, 236, 236));
        let found = element_ink(&img, (100, 300, 300, 340)).expect("the label has text");
        assert!(
            channel_distance(found, BODY) <= INK_TOLERANCE,
            "read {found:?} for a body-ink label on a bezel"
        );
    }

    /// A rule is found by SHAPE, so every way of drawing one trips it.
    #[test]
    fn a_divider_is_detected_however_it_is_drawn() {
        for (label, h, w, ink) in [
            ("1px hairline", 1u32, 200u32, SECONDARY),
            ("2px rule", 2, 300, BODY),
            ("faint separator", 1, 160, (205u8, 205, 205)),
        ] {
            let mut img = canvas();
            line(&mut img, 40, 20, BODY);
            for yy in 150..(150 + h) {
                for xx in 100..(100 + w) {
                    img.put_pixel(xx, yy, image::Rgba([ink.0, ink.1, ink.2, 255]));
                }
            }
            assert!(
                !divider_rows(&img, (0, 0, 400, 400), BG).is_empty(),
                "{label}: a horizontal rule at y=150 must be detected, found none"
            );
        }
    }

    /// …and a line of TEXT must not be mistaken for one, or the divider law
    /// would be unfalsifiable noise.
    #[test]
    fn a_line_of_text_is_not_mistaken_for_a_divider() {
        let mut img = canvas();
        line(&mut img, 150, 12, BODY);
        assert!(
            divider_rows(&img, (0, 0, 400, 400), BG).is_empty(),
            "a row of words is not a rule"
        );
    }
}

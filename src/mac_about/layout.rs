//! The About window's GEOMETRY — pure arithmetic, no AppKit.
//!
//! One centred column with generous vertical rhythm: the shipped app icon at
//! real scale, the name, one product line, the provenance block, the credit,
//! and the two link buttons — separated by air alone, with nothing drawn
//! between them. Every element is horizontally
//! centred and the window's HEIGHT is derived from its contents, so a build
//! that knows fewer facts gets a shorter window rather than a gap where a line
//! would have been.
//!
//! Coordinates are AppKit's: origin bottom-left, y growing upward. The builder
//! places views straight from these frames and computes nothing of its own —
//! which is what makes the composition testable without a window server. The
//! laws below sweep the one axis that actually varies at runtime (how many
//! provenance lines are known, 0…n) and assert the properties a screenshot
//! would otherwise be the only witness for: nothing escapes the content box,
//! nothing overlaps, nothing is off-centre, and nothing has zero extent — the
//! Wagtail class of bug, where state says "it's there" and the pixels say
//! otherwise.

/// Content width. Wide enough for the product line to sit on ONE line at 13pt
/// without wrapping, narrow enough that the window reads as a card rather than
/// a dialog.
pub const WIDTH: f64 = 420.0;

/// Air above the icon. The titled window is `FullSizeContentView` with a
/// transparent titlebar, so the content view runs the full height and the
/// traffic lights float over its top [`TITLEBAR_HEIGHT`] points; this padding
/// is measured from the window's top edge and must clear them with room to
/// spare.
pub const TOP_PADDING: f64 = 54.0;
/// The standard macOS titlebar height the close button occupies, which
/// [`TOP_PADDING`] must clear. Not a layout input — the law's reference value,
/// so nothing in the running app reads it.
#[allow(dead_code)]
pub const TITLEBAR_HEIGHT: f64 = 28.0;

/// The icon's square edge. "Real scale": 128pt is the size Finder's Get Info
/// and the app switcher use, and the shipped `.icns` carries a native
/// representation for it, so it is drawn rather than resampled.
pub const ICON_SIZE: f64 = 128.0;

/// Rhythm between the icon and the name.
pub const GAP_ICON_TITLE: f64 = 22.0;
/// The name's line box, sized for [`TITLE_FONT_SIZE`].
pub const TITLE_HEIGHT: f64 = 40.0;
/// The name's point size — the one loud element in the window. It is a SIZE
/// step, not an ink step: the name is set in the same body ink as the product
/// line, the credit and the buttons (see [`super::Ink`]).
pub const TITLE_FONT_SIZE: f64 = 30.0;

/// The name sits close to its product line: they are one unit.
pub const GAP_TITLE_TAGLINE: f64 = 4.0;
/// The product line's line box.
pub const TAGLINE_HEIGHT: f64 = 18.0;

/// The ONE body point size — the product line, the credit and the button
/// labels all use it. Only the name steps away from it, and only in size.
pub const BODY_FONT_SIZE: f64 = 13.0;

/// THE ONLY THING SEPARATING the identity block above from the provenance
/// block below: air. There is deliberately no rule, no box and no line here —
/// grouping in this window is whitespace and rhythm, nothing drawn. This gap is
/// the largest in the composition precisely because it does that work alone.
pub const GAP_IDENTITY_FACTS: f64 = 40.0;
/// One provenance line's height, at [`FACT_FONT_SIZE`].
pub const FACT_LINE_HEIGHT: f64 = 17.0;
/// The provenance block's point size — small and monospaced. The ONE place the
/// secondary ink is spent (see [`super::Ink`]).
pub const FACT_FONT_SIZE: f64 = 11.0;

/// Air between the provenance block and the credit line. Only spent when
/// there IS a provenance block (see [`layout`]).
pub const GAP_FACTS_ATTRIBUTION: f64 = 16.0;
/// The credit line's height.
pub const ATTRIBUTION_HEIGHT: f64 = 17.0;

/// Air between the credit and the buttons — the largest interior gap, because
/// the buttons are the only interactive thing in the window and everything
/// above them is a statement.
pub const GAP_ATTRIBUTION_BUTTONS: f64 = 30.0;
/// One link button's width.
pub const BUTTON_WIDTH: f64 = 92.0;
/// One link button's height — the standard regular-size push button.
pub const BUTTON_HEIGHT: f64 = 24.0;
/// The gap between the two buttons.
pub const BUTTON_GAP: f64 = 10.0;

/// Air below the buttons.
pub const BOTTOM_PADDING: f64 = 34.0;

/// One placed view, in AppKit content-view coordinates (origin bottom-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Frame {
    /// A frame of size `w`×`h` centred horizontally in the content column,
    /// with its TOP edge at `top`.
    fn centred(w: f64, h: f64, top: f64) -> Self {
        Frame {
            x: (WIDTH - w) / 2.0,
            y: top - h,
            w,
            h,
        }
    }

    /// This frame's top edge. Law-test surface: the builder places frames by
    /// their origin, so only the geometry laws ask where a frame ends.
    #[allow(dead_code)]
    pub fn top(&self) -> f64 {
        self.y + self.h
    }
}

/// Every frame in the window, plus the content size they were laid out for.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// The window's content size, `(width, height)`.
    pub content: (f64, f64),
    pub icon: Frame,
    pub title: Frame,
    pub tagline: Frame,
    /// One frame per provenance line, top to bottom. Empty when nothing about
    /// the build is knowable.
    pub facts: Vec<Frame>,
    pub attribution: Frame,
    /// The two link buttons, left to right: Docs, then GitHub.
    pub buttons: [Frame; 2],
}

impl Layout {
    /// Every frame in top-to-bottom order — what the overlap and bounds laws
    /// sweep, and the only list that must be updated when an element is added.
    /// Law-test surface; the builder walks the named fields directly.
    #[allow(dead_code)]
    pub fn frames(&self) -> Vec<Frame> {
        let mut all = vec![self.icon, self.title, self.tagline];
        all.extend(self.facts.iter().copied());
        all.push(self.attribution);
        all.extend(self.buttons.iter().copied());
        all
    }
}

/// The height the window needs for `fact_count` provenance lines.
///
/// Split out from [`layout`] because the placement pass needs the total before
/// it can convert "distance from the top" into AppKit's bottom-up y — and
/// because a single arithmetic owner is the only way the two can't disagree.
pub fn content_height(fact_count: usize) -> f64 {
    TOP_PADDING
        + ICON_SIZE
        + GAP_ICON_TITLE
        + TITLE_HEIGHT
        + GAP_TITLE_TAGLINE
        + TAGLINE_HEIGHT
        + GAP_IDENTITY_FACTS
        + facts_block_height(fact_count)
        + ATTRIBUTION_HEIGHT
        + GAP_ATTRIBUTION_BUTTONS
        + BUTTON_HEIGHT
        + BOTTOM_PADDING
}

/// The provenance block's total height INCLUDING the gap that follows it.
/// Zero when there are no facts, so a build that knows nothing collapses the
/// block AND its trailing gap rather than leaving a hole above the credit.
fn facts_block_height(fact_count: usize) -> f64 {
    if fact_count == 0 {
        0.0
    } else {
        fact_count as f64 * FACT_LINE_HEIGHT + GAP_FACTS_ATTRIBUTION
    }
}

/// Place every element for a window stating `fact_count` provenance lines.
pub fn layout(fact_count: usize) -> Layout {
    let height = content_height(fact_count);
    // Descends from the window's top edge; each element consumes its own
    // height, each gap consumes itself.
    let mut top = height - TOP_PADDING;

    let icon = Frame::centred(ICON_SIZE, ICON_SIZE, top);
    top = icon.y - GAP_ICON_TITLE;

    let title = Frame::centred(WIDTH, TITLE_HEIGHT, top);
    top = title.y - GAP_TITLE_TAGLINE;

    let tagline = Frame::centred(WIDTH, TAGLINE_HEIGHT, top);
    top = tagline.y - GAP_IDENTITY_FACTS;

    let mut facts = Vec::with_capacity(fact_count);
    for _ in 0..fact_count {
        let line = Frame::centred(WIDTH, FACT_LINE_HEIGHT, top);
        top = line.y;
        facts.push(line);
    }
    if fact_count > 0 {
        top -= GAP_FACTS_ATTRIBUTION;
    }

    let attribution = Frame::centred(WIDTH, ATTRIBUTION_HEIGHT, top);
    top = attribution.y - GAP_ATTRIBUTION_BUTTONS;

    let row_width = BUTTON_WIDTH * 2.0 + BUTTON_GAP;
    let row_left = (WIDTH - row_width) / 2.0;
    let button_y = top - BUTTON_HEIGHT;
    let buttons = [
        Frame {
            x: row_left,
            y: button_y,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        Frame {
            x: row_left + BUTTON_WIDTH + BUTTON_GAP,
            y: button_y,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
    ];

    Layout {
        content: (WIDTH, height),
        icon,
        title,
        tagline,
        facts,
        attribution,
        buttons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runtime axis: how many provenance lines this build happens to know.
    /// 0 (no bundle, no git) through 4 (a future extra fact) — swept by every
    /// law below, because the geometry bug that ships is always in the case the
    /// author did not build for.
    const FACT_COUNTS: std::ops::RangeInclusive<usize> = 0..=4;

    #[test]
    fn nothing_has_zero_extent() {
        for n in FACT_COUNTS {
            for f in layout(n).frames() {
                assert!(
                    f.w > 0.0 && f.h > 0.0,
                    "a {n}-fact window places a frame with no area: {f:?} — a \
                     view with zero extent draws nothing while every other \
                     check still passes"
                );
            }
        }
    }

    #[test]
    fn every_frame_is_inside_the_content_box() {
        for n in FACT_COUNTS {
            let l = layout(n);
            let (w, h) = l.content;
            for f in l.frames() {
                assert!(
                    f.x >= 0.0 && f.x + f.w <= w + f64::EPSILON,
                    "a {n}-fact window places {f:?} outside its {w}pt width"
                );
                assert!(
                    f.y >= 0.0 && f.top() <= h + f64::EPSILON,
                    "a {n}-fact window places {f:?} outside its {h}pt height"
                );
            }
        }
    }

    #[test]
    fn elements_stack_without_overlapping() {
        for n in FACT_COUNTS {
            let frames = layout(n).frames();
            // The two buttons share a row by design; compare everything else
            // pairwise in declaration order, which IS top-to-bottom order.
            for pair in frames.windows(2) {
                let (upper, lower) = (pair[0], pair[1]);
                if (upper.y - lower.y).abs() < f64::EPSILON {
                    continue; // same row (the button pair)
                }
                assert!(
                    upper.y >= lower.top() - f64::EPSILON,
                    "a {n}-fact window overlaps {upper:?} onto {lower:?}"
                );
            }
        }
    }

    #[test]
    fn everything_is_centred_in_the_column() {
        for n in FACT_COUNTS {
            let l = layout(n);
            let mut centred = l.frames();
            // The button ROW is centred as a unit, not each button; check the
            // row's own midpoint instead.
            centred.truncate(centred.len() - 2);
            for f in centred {
                assert!(
                    (f.x + f.w / 2.0 - WIDTH / 2.0).abs() < 0.001,
                    "a {n}-fact window places {f:?} off the column's centre"
                );
            }
            let [docs, github] = l.buttons;
            let row_mid = (docs.x + github.x + github.w) / 2.0;
            assert!(
                (row_mid - WIDTH / 2.0).abs() < 0.001,
                "the button row is off centre: {docs:?} {github:?}"
            );
            assert!(
                (github.x - (docs.x + docs.w) - BUTTON_GAP).abs() < 0.001,
                "the buttons do not keep their declared gap"
            );
        }
    }

    #[test]
    fn the_icon_clears_the_traffic_lights() {
        for n in FACT_COUNTS {
            let l = layout(n);
            let air = l.content.1 - l.icon.top();
            assert!(
                air >= TITLEBAR_HEIGHT,
                "a {n}-fact window leaves only {air}pt above the icon; the close \
                 button occupies the top {TITLEBAR_HEIGHT}pt of a \
                 FullSizeContentView window and would sit on the artwork"
            );
        }
    }

    #[test]
    fn a_known_fact_costs_exactly_one_line_of_height() {
        for n in 1..4 {
            assert!(
                (content_height(n + 1) - content_height(n) - FACT_LINE_HEIGHT).abs() < 0.001,
                "adding a provenance line to a {n}-line window must grow it by \
                 exactly one line"
            );
        }
        // The empty block collapses its trailing gap too, so a knowledge-free
        // build is a genuinely shorter window, not one with a hole in it.
        assert!(
            (content_height(1) - content_height(0) - FACT_LINE_HEIGHT - GAP_FACTS_ATTRIBUTION)
                .abs()
                < 0.001,
            "the first provenance line must bring the block's gap with it"
        );
    }

    #[test]
    fn the_fact_block_reads_top_to_bottom() {
        let l = layout(3);
        for pair in l.facts.windows(2) {
            assert!(
                pair[0].y > pair[1].y,
                "provenance lines must descend in order: {:?}",
                l.facts
            );
        }
        assert!(
            l.facts[0].top() <= l.tagline.y - GAP_IDENTITY_FACTS + 0.001,
            "the fact block must sit below the identity block, separated by air alone"
        );
        assert!(
            l.attribution.top() <= l.facts.last().unwrap().y,
            "the credit must sit below the fact block"
        );
    }
}

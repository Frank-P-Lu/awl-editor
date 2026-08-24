//! CHROME'S PIXEL SPACE — the laws that enforce it.
//!
//! awl draws in device pixels and nothing else; "logical" means only
//! "multiplied by `zoom * dpi` on its way in". The text and caret families
//! already passed through one boundary — `Metrics::with_dpi` — and chrome was
//! never enrolled, so every hand-authored chrome constant was an independent
//! coin flip and its padding rendered at half its tuned physical size on every
//! retina display.
//!
//! Four claims, deliberately separated so no one of them can carry another:
//!
//!   1. THE DRAWN RESULT. The palette's own glyphs, measured in the rendered
//!      PNG, sit at twice the offset on a 2x panel that they sit at on a 1x
//!      one. This is the headline and the only one that grades pixels; a law
//!      that graded a computed length would stay green through exactly the
//!      mistake it is named for.
//!   2. THE RATIO FAMILY DOES NOT MOVE. A dimensionless dial must not be
//!      caught up in the multiply.
//!   3. `overlay_lh`'s THREE TERMS SCALE TOGETHER, on the shipping `Bars`
//!      worlds where the raw theme-authored gap really did drift.
//!   4. THE DECLARATION SWEEP. Every authored constant in the swept sources
//!      states its unit family in its TYPE, and no migrated length escapes the
//!      owner through a bare field access. A suffix is not a type; this is the
//!      law that makes the type mean something for a constant nobody has
//!      written yet. Its scope has been the defect three times, so it is now
//!      widened to `src/render.rs` itself — where the exclusions a length may
//!      claim are DERIVED (from `Metrics::with_dpi`'s own body) or TYPED (a
//!      `Millis` is not a length and the compiler knows it) rather than
//!      granted by a reader, and nothing is left over.

use super::super::*;
use super::{headless_dqp, view};

/// The palette the drawn claim is measured on. A takeover card with a query
/// line, a lens strip and real candidate rows — the surface every world
/// composes and the one the item measured by hand.
fn palette_view() -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = (0..12).map(|i| format!("Command {i}")).collect();
    v.overlay_hint = "type to filter".into();
    v
}

/// The DRAWN horizontal inset of the card's own row text: the distance from the
/// card's left edge to the leftmost glyph ink of a candidate row, in device px.
///
/// The measured quantity is pixels and nothing else. Geometry picks the SCAN
/// WINDOW (which rectangle of the canvas holds the card) and never contributes
/// to the answer. Each row is compared against its OWN modal colour inside that
/// window rather than against one sampled fill, so a world whose ground is a
/// gradient, a dither or a bare `Diagonal` canvas is graded on the same terms
/// as a flat card: whatever a row is mostly made of is its ground, and the
/// first pixel that is emphatically not is its first mark.
///
/// **THE SCAN BAND IS THE PLANNED ROWS, NOT A FRACTION OF THE CARD.** A
/// `ListBacking::BarePlates` world (every `Bars`, `Diagonal` and `Ruled`
/// world) paints NO card plate, so the world's own ground shows through the
/// card everywhere a row plate does not cover — and a fixed
/// `0.30..0.95 * card_h` slice reaches past the last candidate row into that
/// bare ground, below which sit only the hint separator and the hint. There
/// the "first pixel emphatically not the modal colour" is a feature of the
/// CANVAS: on Firetail (a `Bars` world over `Background::Lava`) a blob edge
/// crossing one pixel inside `card_x` reported an inset of 1px, and whether
/// it did depended on where the card sat over a canvas-anchored ground — so
/// the same law read 21px with the menu bar hidden and 1px with it shown,
/// off the same geometry. Scanning the rows the plan actually places makes
/// the measured quantity the one the name claims.
fn drawn_row_text_inset(
    frame: &[[u8; 4]],
    w: u32,
    card: [f32; 4],
    bands: &[(u32, u32)],
) -> Option<f32> {
    let x0 = card[0].max(0.0) as u32;
    let x1 = ((card[0] + card[2]) as u32).min(w);
    if x1 <= x0 + 8 || bands.is_empty() {
        return None;
    }
    let mut best: Option<u32> = None;
    for y in bands.iter().flat_map(|&(a, b)| a..b) {
        let row = &frame[(y * w) as usize..][..w as usize];
        let mut counts: std::collections::HashMap<[u8; 4], u32> = std::collections::HashMap::new();
        for x in x0..x1 {
            *counts.entry(row[x as usize]).or_default() += 1;
        }
        let (ground, n) = counts.iter().max_by_key(|(_, c)| **c)?;
        // A row that is not mostly ONE thing has no usable ground; skip it
        // rather than report a false edge.
        if *n * 3 < (x1 - x0) {
            continue;
        }
        let ground = *ground;
        let ink = |x: u32| -> bool {
            let px = row[x as usize];
            (0..3)
                .map(|c| (px[c] as i32 - ground[c] as i32).abs())
                .sum::<i32>()
                > 60
        };
        // THE RIM IS NOT THE TEXT. A bordered world (Wagtail) draws its card
        // edge AT `card_x`, so the first non-ground pixel of the row is the
        // border itself and the inset would read zero on every panel. Step over
        // the leading run that touches the edge, then take the first mark
        // INSIDE — which is the row's own glyph on a bordered world and the
        // glyph directly on an unbordered one.
        let mut x = x0;
        while x < x1 && ink(x) {
            x += 1;
        }
        while x < x1 && !ink(x) {
            x += 1;
        }
        if x < x1 {
            best = Some(best.map_or(x, |b| b.min(x)));
        }
    }
    best.map(|x| x as f32 - card[0])
}

/// Render the palette on `world` at `dpi`, on a canvas that is the SAME LOGICAL
/// size at both tiers, and return `(card_x, drawn text inset, row pitch)`.
fn drawn_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    world: &str,
    dpi: f32,
) -> Option<(f32, f32, f32)> {
    const LOGICAL: (f32, f32) = (1200.0, 800.0);
    let (w, h) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
    theme::set_active_by_name(world).unwrap();
    p.sync_theme();
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    p.set_view(&palette_view());
    p.prepare(device, queue, w, h).ok()?;
    let card = p.overlay_card_rect()?;
    // The scan band, read off the ONE planner rather than a fraction of the
    // card: the interior 20%..80% of every planned row that carries an ITEM.
    // Trimming each slot's own ends keeps a plate's rounded corner and its
    // antialiased edge out of a measurement about glyph ink.
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let bands: Vec<(u32, u32)> = plan
        .rows()
        .iter()
        .filter(|r| r.item.is_some())
        .map(|r| {
            (
                (r.top + r.height * 0.2).max(0.0) as u32,
                ((r.top + r.height * 0.8) as u32).min(h.saturating_sub(1)),
            )
        })
        .filter(|(a, b)| b > a)
        .collect();
    let frame = super::pixeldiff::render_frame(p, device, queue, w, h);
    let inset = drawn_row_text_inset(&frame, w, card, &bands)?;
    Some((card[0], inset, p.overlay_lh()))
}

/// **CLAIM 1 — THE DRAWN RESULT.** On every world, the row text a summoned card
/// actually DRAWS sits at twice the inset from the card's own edge on a 2x
/// panel that it sits at on a 1x one — and the card's own left edge sits at
/// twice the offset too.
///
/// This is the whole defect stated as an outcome, in pixels. Before the
/// migration that inset was a physical `12.0` (or `BAR_SIDE_INSET +
/// BAR_TEXT_PAD`) at BOTH tiers while the glyphs beside it doubled, so the
/// padding-to-text ratio halved on every retina display — measured by hand on
/// the shipping default at 0.3438 of a line height at dpi 1 against 0.1719 at
/// dpi 2, exactly one half.
///
/// The two halves are asserted SEPARATELY on purpose. A law that graded only
/// the card's placement would stay green through an unscaled pad, and a law
/// that graded only the pad would stay green through an unscaled placement;
/// each is the other's blind spot.
///
/// **SWEPT OVER THE MENU BAR, because that is the axis this law ran one side
/// of for its whole life.** `crate::menubar::MENU_BAR_ON` initialises to
/// `false` on macOS and `true` on every other platform, so the drawn bar — and
/// the vertical reserve it takes off the top of every card's budget — was
/// present in CI's Linux job and absent from every run on the authoring host.
/// The state a law never enters is the state it cannot grade.
#[test]
fn the_cards_drawn_row_text_holds_its_inset_on_a_two_x_panel() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the chrome pixel-space drawn law: no wgpu adapter");
        return;
    };
    // The AMBIENT value, never `cfg!(target_os = ...)`: a `cfg!` inside a test
    // reports the host that COMPILED it, not the branch `MENU_BAR_ON`'s
    // initialiser actually took, so a restore written that way restores the
    // wrong value under any forcing of that initialiser.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let mut graded = [0usize; 2];
    let mut worst = 0.0f32;
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for world in theme::THEMES.iter().map(|t| t.name) {
            let cell = format!("{world} menu_bar={bar}");
            let Some((cx1, pad1, lh1)) = drawn_cell(&device, &queue, &mut p, world, 1.0) else {
                continue;
            };
            let Some((cx2, pad2, lh2)) = drawn_cell(&device, &queue, &mut p, world, 2.0) else {
                continue;
            };
            // NON-VACUITY: a zero pad would make `0 == 2 * 0` pass on anything, and
            // the row pitch must genuinely have doubled or the fixture is not at 2x.
            assert!(
                pad1 >= 4.0,
                "{cell}: the drawn row text is only {pad1}px inside its own card at \
                 dpi 1 — comparing zero against twice zero would pass on anything"
            );
            assert!(
                (lh2 - 2.0 * lh1).abs() < 0.01,
                "{cell}: the row pitch is {lh1} at dpi 1 and {lh2} at dpi 2; the \
                 text half of this comparison is not at the scale it claims"
            );
            for (what, a, b) in [("card edge", cx1, cx2), ("row-text inset", pad1, pad2)] {
                let want = 2.0 * a;
                // One device pixel for the rasterized edge, plus a proportional
                // term: a card edge on a fractional pixel resolves with partial
                // coverage and can move the first qualifying byte by one.
                let tol = 1.5 + 0.01 * want;
                worst = worst.max((b - want).abs());
                assert!(
                    (b - want).abs() <= tol,
                    "{cell}: the drawn {what} measures {a}px on a 1x panel and {b}px \
                     on a 2x one, where a chrome length that passed the pixel-space \
                     owner would put it at {want} (tolerance {tol:.2}). A constant \
                     left in device pixels renders at half its tuned size on every \
                     retina display."
                );
            }
            graded[usize::from(bar)] += 1;
        }
    }
    // PER-MENU-BAR-STATE, not an aggregate: an aggregate floor is satisfied by
    // one state grading the whole roster while the other grades nothing, which
    // is precisely the coverage hole this sweep exists to close.
    for (bar, n) in graded.iter().enumerate() {
        assert!(
            *n >= 15,
            "the drawn law must grade the world roster with menu_bar={}, got {n} \
             (both states: {graded:?})",
            bar == 1
        );
    }
    eprintln!(
        "chrome pixel space: {graded:?} worlds drawn-graded (menu_bar off/on), \
         worst error {worst:.2}px"
    );
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}

/// **CLAIM 2 — THE RATIO FAMILY DOES NOT SCALE.** A dimensionless dial is not a
/// length, and the law must not force one through the multiply.
///
/// Stated against the dials themselves rather than against a remembered number:
/// the hint row and the query beat are FRACTIONS OF THE ROW PITCH, so at every
/// scale each must still be its own authored fraction of `overlay_lh` (to
/// within the rounding both owners apply), and the overlay's type scale must
/// still be exactly itself. A ratio pushed through the pixel-space owner would
/// square itself here — doubling its fraction on a 2x panel.
#[test]
fn the_ratio_family_holds_its_value_at_every_scale() {
    let _g = crate::testlock::serial();
    let Some((_d, _q, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the chrome ratio law: no wgpu adapter");
        return;
    };
    let mut cells = 0usize;
    for (zoom, dpi) in [
        (1.0f32, 1.0f32),
        (1.0, 2.0),
        (2.0, 1.0),
        (0.7, 2.0),
        (1.3, 2.0),
    ] {
        p.set_dpi(dpi);
        let mut v = palette_view();
        v.zoom = zoom;
        p.set_view(&v);
        let lh = p.overlay_lh();
        assert!(lh > 0.0, "an inert row pitch would make every ratio 0/0");
        let cell = format!("zoom {zoom} dpi {dpi} (lh {lh})");
        // Both owners `.round()` their result, so the tolerance is the rounding
        // and nothing else — deliberately tighter than one part in the dial.
        for (name, got, want) in [
            (
                "hint row",
                p.overlay_hint_h(),
                lh * chrome::OVERLAY_HINT_ROW.0,
            ),
            (
                "query beat",
                p.overlay_header_gap(),
                lh * chrome::OVERLAY_QUERY_BEAT.0,
            ),
        ] {
            assert!(
                (got - want).abs() <= 0.51,
                "{cell}: {name} is a RATIO of the row pitch and must stay one: \
                 drew {got}, the authored fraction of this cell's own pitch is \
                 {want}. A dimensionless dial enrolled in the pixel-space owner \
                 would square itself here."
            );
        }
        let type_scale = p.overlay_char_width() / p.metrics.char_width;
        assert!(
            (type_scale - chrome::OVERLAY_UI_SCALE).abs() < 1e-5,
            "{cell}: the overlay type scale is a RATIO and must not move: got \
             {type_scale}, authored {}",
            chrome::OVERLAY_UI_SCALE
        );
        cells += 1;
    }
    assert_eq!(cells, 5, "the ratio sweep must visit every cell");
    p.set_dpi(1.0);
}

/// **CLAIM 3 — `overlay_lh`'s THREE TERMS SCALE TOGETHER.** The row pitch is a
/// dpi-scaled line height PLUS the overlay's extra leading PLUS `Bars`'s own
/// `BarConfig::gap`, and the last two were raw. So on the three shipping
/// `Bars` worlds the one quantity the tree treats as logical drifted out of
/// proportion across displays — the gap's SHARE of the row pitch halved on
/// every retina panel.
///
/// Graded as the gap's share of the pitch, which is what a reader sees, and
/// swept over every world that actually authors a gap. The `Pane` complement is
/// asserted too: a world with no gap must still report a share of exactly zero,
/// so the law cannot pass by finding nothing.
#[test]
fn the_row_pitchs_three_terms_hold_one_proportion_across_dpi() {
    let _g = crate::testlock::serial();
    let Some((_d, _q, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the row-pitch law: no wgpu adapter");
        return;
    };
    let mut with_gap = 0usize;
    let mut without_gap = 0usize;
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let mut shares = Vec::new();
        for (zoom, dpi) in [(1.0f32, 1.0f32), (1.0, 2.0), (1.3, 2.0)] {
            p.set_dpi(dpi);
            let mut v = palette_view();
            v.zoom = zoom;
            p.set_view(&v);
            let lh = p.overlay_lh();
            let text = lh - p.overlay_leading() - p.overlay_row_gap();
            assert!(
                text > 0.0,
                "{world}: the row pitch's text term went non-positive at zoom \
                 {zoom} dpi {dpi} — the three terms no longer sum to it"
            );
            shares.push(p.overlay_row_gap() / lh);
        }
        let authored = shares[0] > 0.0;
        if authored {
            with_gap += 1;
        } else {
            without_gap += 1;
        }
        for (i, s) in shares.iter().enumerate().skip(1) {
            assert!(
                (shares[0] - s).abs() < 0.002,
                "{world}: the gap's share of the row pitch is {} at dpi 1 and {s} \
                 at cell {i}. A raw gap summed with a scaled line height halves \
                 its own share on every retina display, which is the drift this \
                 law is named for.",
                shares[0]
            );
        }
    }
    assert!(
        with_gap >= 3 && without_gap >= 3,
        "the sweep must see both arms — worlds that author a gap ({with_gap}) \
         and worlds that do not ({without_gap})"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}

// ---------------------------------------------------------------------------
// CLAIM 4 — the declaration sweep
// ---------------------------------------------------------------------------

/// The unit families a chrome constant may be authored in. `Logical` is the
/// DEFAULT and the only one a new length should ever need.
const UNIT_TYPES: &[&str] = &["Logical", "Physical", "LogicalGrowOnly", "Chars", "Rows"];

/// Families that are declared in the TYPE and are deliberately NOT lengths.
/// This is the by-KIND escape from the length sweep: a `Millis` carries no
/// [`Metrics::px`], so the compiler — not a table a reader has to maintain —
/// is what stops an animation duration from being multiplied by the pixel
/// scale. A non-length that can be typed belongs here rather than in
/// [`DIMENSIONLESS`].
const NON_LENGTH_TYPES: &[&str] = &["Millis"];

/// Families that are declared in the TYPE and scale as an AREA — the SQUARE of
/// the display factor — rather than as a length or not at all. Kept apart from
/// [`NON_LENGTH_TYPES`]: a `Millis` never meets the pixel scale at all, but an
/// `Area` genuinely does, just through its own quadratic door (`Area::px2`)
/// rather than a length's linear one, so lumping it with a true non-length
/// would hide that it is still scale-dependent.
const AREA_TYPES: &[&str] = &["Area"];

/// The constants [`Metrics::with_dpi`] resolves ITSELF, read out of that
/// function's own body rather than listed by name.
///
/// These are the base text/caret metrics: `with_dpi` multiplies each by
/// `s = zoom * dpi` and stores the result on [`Metrics`], which is the single
/// source of truth every consumer reads. They are excluded from the length
/// sweep BECAUSE THE OWNER ALREADY MULTIPLIES THEM — declaring one `Logical`
/// and then passing it through [`Metrics::px`] as well would apply DPI twice,
/// which is invisible at the `--capture-dpi 1` every capture defaults to.
///
/// **The enrolment is DERIVED, which is the whole point.** A hand-written list
/// would keep excusing `FONT_SIZE` on the day somebody deletes
/// `font_size: FONT_SIZE * s` from the owner; this parse stops naming it that
/// same day, and the constant becomes an offender until it declares a family.
fn metrics_resolved_constants(render_src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in render_src.lines() {
        if line.contains("pub fn with_dpi(") {
            inside = true;
            continue;
        }
        if inside {
            // The owner's body ends at its own closing brace, at method indent.
            if line == "    }" {
                break;
            }
            // `field: NAME * s,` / `field: crate::path::NAME * s,` — the shape
            // of a base metric being resolved, and nothing else in the body.
            if let Some((_, rhs)) = line.split_once(':') {
                let rhs = rhs.trim();
                if let Some(name) = rhs.strip_suffix("* s,") {
                    let name = name.trim().rsplit("::").next().unwrap_or("").trim();
                    if !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                    {
                        out.push(name.to_string());
                    }
                }
            }
        }
    }
    out
}

/// The constants under the swept files (`src/render/chrome/`, plus the writing
/// column's own `render/geometry.rs` / `render/geometry/**` / `render/scroll.rs`)
/// that are NOT lengths, each with the reason it carries no unit. Enumerated by name
/// with no wildcard, so a new bare `f32` constant fails this law rather than joining
/// the list silently.
const DIMENSIONLESS: &[(&str, &str)] = &[
    (
        "QUOTE_MARK_SCALE",
        "a MULTIPLE of the body font size, which Metrics::with_dpi already \
         resolves — the pull-quote mark is shaped at that product, never padded by it",
    ),
    (
        "IMAGE_REVEAL_DIM_ALPHA",
        "an alpha on the revealed image's quad, not a distance",
    ),
    (
        "DEGENERATE_CELL_FRAC",
        "a fraction of metrics.char_width, not a length of its own",
    ),
    (
        "CW",
        "a geometry/tests.rs alias for CHAR_WIDTH — a base text metric already \
         scaled through Metrics::with_dpi, not an authored chrome pad",
    ),
    (
        "ADAPTIVE_DPI",
        "a fixed test-only DPI multiplier (always 1.0, see its own doc) — a \
         scale factor, not a length",
    ),
    (
        "ADAPTIVE_LEFT_PAD",
        "PAGE_MIN_PAD resolved AT ADAPTIVE_DPI, extracted once for repeated \
         test arithmetic — not a second authored pad",
    ),
    ("OVERLAY_UI_SCALE", "a ratio: the whole-menu type scale"),
    (
        "PLACARD_CALIBRATION_TITLE",
        "a ratio: the frozen title rung",
    ),
    (
        "PLACARD_HEIGHT_PER_SCALE",
        "a ratio: placard height per unit of window short side",
    ),
    (
        "PLACARD_SIZE_STEP",
        "a ratio: the geometric atlas-safety ladder's step",
    ),
    (
        "TRAVEL_MAX_BAND_FRACTION",
        "a fraction of the card's own side territory",
    ),
    (
        "WORKSPACE_MARGIN_FRAC",
        "a fraction of the smaller window dimension",
    ),
    (
        "TIMELINE_MAX_FRAC",
        "a fraction of the workspace's interior",
    ),
    (
        "UNFOCUSED_MARK_ALPHA",
        "an alpha fraction of the focused marker's",
    ),
    ("OUTLINE_EDGE_FADE_ALPHA", "an alpha fraction"),
    (
        "DROP_WIDTH_SLACK",
        "a slack factor on an estimated content width",
    ),
    (
        "PAGE_MIN_MARGIN_FRAC",
        "a fraction of the window width, taken as the max against the \
         already-scaled PAGE_MIN_MARGIN_PX",
    ),
    (
        "COPY_PULSE_LIFT_L",
        "an HSL LIGHTNESS delta added to the theme's own lightness — a colour \
         channel, not a distance",
    ),
    (
        "COPY_PULSE_LIFT_ALPHA",
        "an ALPHA delta on the 0..255 channel scale, added to the theme's own \
         alpha and clamped",
    ),
    (
        "CARET_MORPH_SETTLE_SHOW",
        "a settle FRACTION compared against the caret animation's own progress \
         in 0..1",
    ),
    (
        "DRAG_SCROLL_MIN_RATE",
        "a RATE in device px/sec, not a length — the overshoot-to-rate curve's \
         floor the instant the dead zone clears",
    ),
    (
        "DRAG_SCROLL_MAX_RATE",
        "a RATE in device px/sec, not a length — the overshoot-to-rate curve's \
         cap, reached at DRAG_SCROLL_RAMP_PX overshoot",
    ),
];

/// The `NAME: TYPE ...` tail of a constant declaration, whatever visibility it
/// carries. `pub(super) const BAR_SIDE_INSET` is the ordinary shape in this
/// directory, and a sweep that only matched a bare `const` would silently skip
/// most of what it is meant to grade.
fn const_decl(line: &str) -> Option<&str> {
    let mut t = line.trim_start();
    if let Some(rest) = t.strip_prefix("pub") {
        t = match rest.strip_prefix('(') {
            Some(vis) => vis.split_once(')')?.1.trim_start(),
            None => rest.trim_start(),
        };
    }
    t.strip_prefix("const ")
}

/// WIDENED PAST `src/render/chrome/` ALONE. `TEXT_TOP` and `TEXT_LEFT` both lived
/// untyped in `render/geometry.rs` / `render.rs` for their whole lives, and the
/// reason is structural: this sweep never looked outside `chrome/`. `render/
/// geometry.rs`, `render/geometry/**` and `render/scroll.rs` are the writing
/// column's OWN pixel-space files — the direct neighbourhood both constants
/// actually lived in — added here so a new one authored there fails this law by
/// name instead of surviving on the same technicality.
///
/// WIDENED AGAIN, to `src/menubar.rs`. A law's SCOPE has now been the defect twice:
/// widening past `chrome/` caught three instances on its first run, and `menubar.rs`
/// — the menu bar's own pure layout math, whose pads are added to device-scaled
/// glyph positions in `chrome/menubar.rs` and `chrome/menubar/dropdown.rs` — sat
/// outside the sweep for its whole life. Adding the one path is what made the
/// compiler and this law ENUMERATE the work rather than a reader having to guess it.
/// It is not a `chrome/` file only because the module is shared with the native
/// menu roster; every length in it is a chrome length.
///
/// WIDENED A THIRD TIME, to `src/render.rs` — the file that declares the newtypes
/// and, for its whole life, the largest population of untyped ones. Folding it in
/// was held back once on purpose, because it declares FOUR families this sweep had
/// never met and guessing at any of them would have encoded the guess as a passing
/// check: the base text/caret metrics that [`Metrics::with_dpi`] resolves itself,
/// animation durations in milliseconds, raw alpha/lightness channel values, and a
/// pad measured in CHARACTER cells. Each now has a mechanism rather than a
/// judgement — [`metrics_resolved_constants`] derives the first from the owner's own
/// body, [`NON_LENGTH_TYPES`] gives the second a type, [`DIMENSIONLESS`] records the
/// third with its reason, and the fourth is a [`Chars`]. What is left over is a
/// measured DEFECT with its own closed ledger, [`DPI_BLIND_PENDING`], not a
/// classification.
///
/// WIDENED A FIFTH TIME, to `src/render/layers.rs`, `src/render/rects.rs` and
/// `src/render/rects/**` — the writing column's own DECORATION files, and the last
/// neighbourhood in which a bare `f32` length was still authored without this law
/// noticing. Five of the seven lengths that round found live in these three
/// paths: the inline image's corner, the caption scrim's two pads, and a minimum
/// decoration width three builders had each spelled as its own `2.0 *
/// metrics.zoom`. What the widening ENUMERATED beyond them is four constants
/// that are innocent and now say so: two dimensionless (a font-size multiple and
/// an alpha) and two `Physical` device-grid tolerances, each carrying the
/// `FLUSH_EPS` reason rather than a reader's assurance.
///
/// ⚠️ WHAT IS DELIBERATELY STILL OUTSIDE, with its count, because a guess encoded
/// as a passing check is worse than a gap: `src/caret.rs` (32 constants, in
/// families this sweep has never met — spring stiffness in 1/s², damping,
/// velocities in px/s, plus durations and fractions), `src/render/spans/colors.rs`
/// (17, all HSL channel values and two vertical fractions),
/// `src/render/spans/conceal.rs` (3) and `src/render/layers/fold_chevron.rs` (6,
/// of which five want `Chars` and one wants `Millis` — mechanical, and the next
/// widening's obvious first step). The READ-SITE law below reaches every one of
/// those files today; it is only the DECLARATION that is ungraded there.
///
/// WIDENED A FOURTH TIME, to `src/render/caret_body.rs` — the caret's own
/// minimum-visible-body floor, one directory out from every sweep above and the
/// same untyped-`f32` shape this file exists to close. Its two length constants
/// recover `Metrics::scale` at the call site rather than reading `Metrics`
/// directly and pass it through `Logical::px`, the same recovery `CARET_INK_PAD`
/// already used. Its third constant is an AREA, met here for the first time: an
/// area scales as the SQUARE of the display factor, so a length family would
/// silently under-scale it by one factor of `scale` — it gets its own by-kind
/// family, `Area`, with a `.px2` door rather than `Logical`'s linear `.px`.
fn chrome_sources() -> Vec<(String, String)> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut stack = vec![manifest.join("src/render/chrome")];
    let single_files = [
        "src/render.rs",
        "src/render/geometry.rs",
        "src/render/scroll.rs",
        "src/menubar.rs",
        "src/render/caret_body.rs",
        "src/render/layers.rs",
        "src/render/rects.rs",
    ];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("chrome dir readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(manifest)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                out.push((
                    rel,
                    std::fs::read_to_string(&path).expect("source readable"),
                ));
            }
        }
    }
    // `render/geometry/` is a real directory (`column.rs`, `page.rs`, `tests.rs`) but
    // `render.rs`, `render/geometry.rs` and `render/scroll.rs` are FILES beside their
    // own directories, not inside a walk of them — named explicitly rather than
    // re-deriving the dir walk for a three-file exception.
    for f in single_files {
        let path = manifest.join(f);
        out.push((
            f.to_string(),
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{f} readable: {e}")),
        ));
    }
    for dir in ["src/render/geometry", "src/render/rects"] {
        for entry in std::fs::read_dir(manifest.join(dir)).expect("swept dir readable") {
            let path = entry.expect("dir entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(manifest)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                out.push((
                    rel,
                    std::fs::read_to_string(&path).expect("source readable"),
                ));
            }
        }
    }
    assert!(out.len() > 20, "the chrome source sweep found nothing");
    out.sort();
    out
}

/// EVERY PRODUCT `.rs` FILE UNDER `src/`, test files excluded.
///
/// Two of these laws grade READ SITES rather than declarations, and a read site
/// is a claim about what the shipping code does — a test that multiplies one of
/// these constants by a scale in order to describe the defect is not a
/// counter-example to it. Directories named `tests` and files named `tests.rs`
/// are dropped for that reason and no other.
fn product_sources() -> Vec<(String, String)> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out: Vec<(String, String)> = Vec::new();
    let mut stack = vec![manifest.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src readable") {
            let path = entry.expect("dir entry").path();
            let rel = path
                .strip_prefix(manifest)
                .unwrap_or(&path)
                .display()
                .to_string();
            if path.is_dir() {
                if !rel.ends_with("/tests") {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") && !rel.ends_with("tests.rs") {
                out.push((
                    rel,
                    std::fs::read_to_string(&path).expect("source readable"),
                ));
            }
        }
    }
    assert!(
        out.len() > 60,
        "the product-source scan found only {} files",
        out.len()
    );
    out.sort();
    out
}

/// Per-constant classification tally for
/// [`every_authored_chrome_constant_declares_its_unit_family`], pulled into its
/// own owner so the law itself reads as a list of assertions rather than a
/// scan wearing some assertions at the end.
#[derive(Default)]
struct DeclarationTally {
    offenders: Vec<String>,
    typed: usize,
    non_length_typed: usize,
    area_typed: usize,
    dimensionless: usize,
    owner_resolved: usize,
}

fn tally_declarations(sources: &[(String, String)], resolved: &[String]) -> DeclarationTally {
    let mut t = DeclarationTally::default();
    for (path, src) in sources {
        for (i, line) in src.lines().enumerate() {
            let Some(rest) = const_decl(line) else {
                continue;
            };
            let Some((name, ty)) = rest.split_once(':') else {
                continue;
            };
            let name = name.trim();
            let ty = ty
                .split(['=', ';'])
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches('>')
                .to_string();
            // Only NUMERIC constants are in scope: a `&str` separator or a `u8`
            // level is not a length and never could be.
            if !(ty == "f32"
                || UNIT_TYPES.contains(&ty.as_str())
                || NON_LENGTH_TYPES.contains(&ty.as_str())
                || AREA_TYPES.contains(&ty.as_str()))
            {
                continue;
            }
            if UNIT_TYPES.contains(&ty.as_str()) {
                t.typed += 1;
            } else if NON_LENGTH_TYPES.contains(&ty.as_str()) {
                t.non_length_typed += 1;
            } else if AREA_TYPES.contains(&ty.as_str()) {
                t.area_typed += 1;
            } else if resolved.iter().any(|n| n == name) {
                t.owner_resolved += 1;
            } else if DIMENSIONLESS.iter().any(|(n, _)| *n == name) {
                t.dimensionless += 1;
            } else {
                t.offenders.push(format!(
                    "{path}:{}: `{name}: f32` is an untyped chrome constant. \
                     Chrome's default pixel space is LOGICAL — declare it \
                     `Logical` (or `Physical` with a reason, `LogicalGrowOnly`, \
                     `Chars`, `Rows`, `Millis`), let `Metrics::with_dpi` resolve \
                     it, or add it to this law's DIMENSIONLESS table with the \
                     reason it is not a length.",
                    i + 1
                ));
            }
        }
    }
    t
}

/// **CLAIM 4a — EVERY AUTHORED CHROME CONSTANT DECLARES ITS UNIT FAMILY.**
///
/// The `_LOGICAL` suffixes this round retired were the right instinct with an
/// unenforceable mechanism: a suffix is not a type, and nothing stopped the
/// next constant from being a bare `f32` multiplied by nothing. This is the
/// enforcement. A new chrome length authored as a bare `f32` fails HERE, by
/// name, and the only way past is to state which of the four families it is in
/// — or to record, with a reason, that it is not a length at all.
///
/// **FOUR MECHANISMS, and each one names what enrolled it**, because the sweep
/// now covers `render.rs` and a bare `f32` there can be innocent for a reason no
/// chrome pad ever had:
///
///   * a UNIT TYPE, or a `Millis` — the by-kind exclusion, enforced by the
///     compiler rather than by this file;
///   * an `Area` — the by-kind exclusion for a quantity that scales as the
///     SQUARE of the display factor, so a length's linear door would silently
///     under-scale it;
///   * resolved by [`Metrics::with_dpi`] itself, DERIVED from that function's
///     own body ([`metrics_resolved_constants`]) so the exclusion expires the
///     moment the owner stops multiplying it;
///   * [`DIMENSIONLESS`], the reasoned table for a ratio or a colour channel
///     that no type currently expresses.
///
/// There is no fifth door. The DPI-blind ledger this law used to carry — the
/// sixteen writing-column lengths whose read sites multiplied them by
/// `metrics.zoom` alone, so each held its device size as the panel got denser —
/// is gone because every entry was given a family, and a bare `f32` in the swept
/// sources now fails HERE with nowhere to be parked. The OUTCOME half of that
/// repair is graded where a reader can see it, in `writing_column_decor_dpi.rs`:
/// a declaration sweep grades the constant, and only a geometry law grades the
/// factor its read site hands it.
#[test]
fn every_authored_chrome_constant_declares_its_unit_family() {
    let sources = chrome_sources();
    let render_src = sources
        .iter()
        .find(|(p, _)| p == "src/render.rs")
        .map(|(_, s)| s.clone())
        .expect("src/render.rs is in the swept set");
    let resolved = metrics_resolved_constants(&render_src);
    // NON-VACUITY OF THE DERIVATION, before anything is excused by it: the owner
    // really does resolve a family of base metrics, and a parse that silently
    // matched nothing would excuse nothing rather than everything — but it would
    // also mean the mechanism this law advertises does not exist.
    assert!(
        resolved.len() >= 10,
        "Metrics::with_dpi's body yielded only {} resolved constants ({resolved:?}) \
         — the derivation that excuses the base metrics is not reading the owner",
        resolved.len()
    );
    let t = tally_declarations(&sources, &resolved);
    assert!(
        t.offenders.is_empty(),
        "chrome constants authored outside the pixel space:\n{}",
        t.offenders.join("\n")
    );
    // Non-vacuity: the sweep must actually be finding constants of every kind it
    // claims to sort, and each floor is named so a green run says what it graded.
    assert!(
        t.typed >= 25,
        "the sweep found only {} unit-typed chrome constants — it is not \
         reading the sources it thinks it is",
        t.typed
    );
    assert!(
        t.non_length_typed >= 3 && t.owner_resolved >= 10 && t.area_typed >= 1,
        "the by-kind exclusions must all be populated: {} Millis-typed, {} \
         Area-typed, {} resolved by Metrics::with_dpi",
        t.non_length_typed,
        t.area_typed,
        t.owner_resolved
    );
    assert_eq!(
        t.dimensionless,
        DIMENSIONLESS.len(),
        "every entry in the DIMENSIONLESS table must still name a live chrome \
         constant; a stale entry silently excuses a name nobody wrote"
    );
    eprintln!(
        "declaration sweep: {} unit-typed, {} Millis, {} Area, {} resolved by \
         Metrics::with_dpi, {} dimensionless",
        t.typed, t.non_length_typed, t.area_typed, t.owner_resolved, t.dimensionless
    );
}

/// **CLAIM 4d — A LENGTH IS NEVER RESOLVED AGAINST ZOOM ALONE.**
///
/// The newtype stops a length reaching a draw call UNMULTIPLIED. It does not,
/// on its own, stop a caller handing [`Logical::px`] the WRONG factor — the
/// method takes any `f32`, because the pure geometry policies are called with a
/// bare `dpi` or a bare `scale` and have no `Metrics` to ask. So the second half
/// of the guarantee has to be a law, and this is it: the factor is
/// `Metrics::scale` (`zoom * dpi`) or the `dpi` a policy was handed, never
/// `zoom` by itself.
///
/// `zoom` alone is the defect the writing column's decorations carried for
/// their whole lives — a length that tracks the user's type size and ignores the
/// panel's density, so it halves against the text it was tuned beside on every
/// retina display. This law is what stops a repaired length being given a unit
/// family while keeping the factor that made it wrong.
#[test]
fn no_length_is_resolved_against_zoom_alone() {
    let mut offenders = Vec::new();
    let mut resolutions = 0usize;
    for (path, src) in product_sources() {
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("///") {
                continue;
            }
            for door in [".px(", ".px_grow_only(", ".px_physical("] {
                let mut rest = line;
                while let Some(at) = rest.find(door) {
                    let arg = &rest[at + door.len()..];
                    let arg = arg.split(')').next().unwrap_or("");
                    // `Metrics::px(SOME_LOGICAL)` and `Logical::px(scale)` share
                    // one spelling. Only the SCALE-taking form is in scope, and an
                    // argument that starts upper-case (or is a path to one) is the
                    // other form.
                    let last = arg.rsplit("::").next().unwrap_or(arg).trim();
                    let takes_scale = last
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
                    if takes_scale {
                        resolutions += 1;
                        let words: Vec<&str> = arg
                            .split(|c: char| !c.is_alphanumeric() && c != '_')
                            .collect();
                        if words.contains(&"zoom") && !words.contains(&"scale") {
                            offenders.push(format!("{path}:{}: {}", i + 1, line.trim()));
                        }
                    }
                    rest = &rest[at + door.len()..];
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a logical length resolved against ZOOM alone — it will hold its device \
         size as the display gets denser, which is the halving this file's drawn \
         claim exists to catch:\n{}",
        offenders.join("\n")
    );
    // NON-VACUITY: the scan must have found real resolution sites to grade. A
    // path-shape change that stopped matching `.px(` would otherwise report a
    // clean sweep of nothing.
    assert!(
        resolutions >= 30,
        "the scan graded only {resolutions} scale-taking resolutions — it is not \
         finding the door it thinks it is"
    );
    eprintln!("zoom-alone sweep: {resolutions} scale-taking resolutions graded");
}

/// The `zoom` multiplies that are NOT a length being resolved, each with the
/// reason — the whole exception list for [`nothing_is_multiplied_by_zoom_alone`].
///
/// Deliberately keyed by file AND needle and graded for staleness, so an entry
/// cannot outlive the line it excuses. There are two and they are the same
/// quantity: the zoom PERCENTAGE a readout prints. Nothing else in the product
/// multiplies by `zoom` without `dpi` beside it.
const ZOOM_PERCENT_READOUTS: &[(&str, &str, &str)] = &[
    (
        "src/render/chrome/readout.rs",
        "(zoom * 100.0)",
        "the zoom readout's own PERCENTAGE — a number printed as text, not a length",
    ),
    (
        "src/render/chrome/debug_text.rs",
        "(m.zoom * 100.0)",
        "the debug panel's zoom percentage, same quantity as the readout's",
    ),
];

/// **CLAIM 4e — NOTHING IS MULTIPLIED BY ZOOM ALONE, DOOR OR NO DOOR.**
///
/// [`no_length_is_resolved_against_zoom_alone`] grades the argument handed to
/// `Logical::px`, and that is exactly why it could not see this round's seven: a
/// bare `f32` never reaches a door at all. `CORNER_RADIUS * m.zoom`,
/// `CAPTION_SCRIM_PAD_X * zoom`, `IMAGE_CORNER_PX * zoom`,
/// `spell_underline_gap * m.zoom`, `thickness * zoom`, `2.0 * m.zoom` — every
/// one of them was a multiplication statement with no door in it, in files the
/// declaration sweep did not read, and the two laws between them graded neither
/// end.
///
/// So this arm asks the question with no door in it either: **`metrics.zoom` is
/// never a multiplier on its own.** `zoom * dpi` is the scale's own derivation
/// and is allowed by having `dpi` in the same expression; everything else needs
/// `scale`. The enrolment is every product source and every `*`, which is the
/// widest form the question has — a new file cannot be outside it, which is what
/// five separate scope defects on the DECLARATION side have earned.
/// Whether `line` uses an identifier `zoom` as an operand of a BINARY `*`.
///
/// A plain `line.contains('*')` beside the word `zoom` is not this question, and
/// the difference is seven false positives on the first run: `*ctx.zoom` is a
/// DEREFERENCE, and `(w as f32 * 0.5, h as f32 * 0.5, m.zoom)` has a multiply on
/// the same line as a `zoom` that is not in it. So the star has to be adjacent to
/// this expression, and it has to be binary — a prefix `*` is followed
/// immediately by its operand, a binary one carries a space or sits between two
/// operand characters.
fn multiplies_by_zoom(line: &str) -> bool {
    let b = line.as_bytes();
    let path_char = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b':');
    let mut i = 0usize;
    while let Some(found) = line[i..].find("zoom") {
        let s = i + found;
        let e = s + 4;
        i = e;
        // A whole identifier, not the tail of `some_zoomish`.
        if s > 0 && (b[s - 1].is_ascii_alphanumeric() || b[s - 1] == b'_') {
            continue;
        }
        if e < b.len() && (b[e].is_ascii_alphanumeric() || b[e] == b'_') {
            continue;
        }
        // `zoom *` / `zoom*` — the star follows the expression.
        let mut f = e;
        while f < b.len() && (b[f] == b' ' || b[f] == b')') {
            f += 1;
        }
        if f < b.len() && b[f] == b'*' && !(f + 1 < b.len() && b[f + 1] == b'*') {
            return true;
        }
        // `... * <path>.zoom` — walk back off the path, then look for a BINARY star.
        let mut r = s;
        while r > 0 && path_char(b[r - 1]) {
            r -= 1;
        }
        while r > 0 && b[r - 1] == b' ' {
            r -= 1;
        }
        if r > 0 && b[r - 1] == b'*' {
            let star = r - 1;
            let after_is_space = star + 1 < b.len() && b[star + 1] == b' ';
            let before_is_operand = star > 0
                && (b[star - 1].is_ascii_alphanumeric()
                    || b[star - 1] == b'_'
                    || b[star - 1] == b')');
            if after_is_space || before_is_operand {
                return true;
            }
        }
    }
    false
}

#[test]
fn nothing_is_multiplied_by_zoom_alone() {
    let mut offenders = Vec::new();
    let mut graded = 0usize;
    let mut excused = vec![0usize; ZOOM_PERCENT_READOUTS.len()];
    for (path, src) in product_sources() {
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("#[") {
                continue;
            }
            if !multiplies_by_zoom(line) {
                continue;
            }
            let words: Vec<&str> = line
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .collect();
            graded += 1;
            // `scale` or `dpi` in the same expression means the multiply either IS
            // the scale's derivation or already carries the density.
            if words.contains(&"scale") || words.contains(&"dpi") {
                continue;
            }
            match ZOOM_PERCENT_READOUTS
                .iter()
                .position(|(f, needle, _)| *f == path && line.contains(needle))
            {
                Some(at) => excused[at] += 1,
                None => offenders.push(format!("{path}:{}: {}", i + 1, line.trim())),
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "`zoom` used as a multiplier with no `dpi` or `scale` beside it. The user's \
         type size is not a display factor: a quantity scaled this way holds its \
         DEVICE size as the panel gets denser, which is the halving this file's \
         drawn claim exists to catch. Resolve it through `Metrics::px` (or hand \
         `Metrics::scale` to the policy), or record it above with the reason it is \
         not a length:\n{}",
        offenders.join("\n")
    );
    for (at, (path, needle, reason)) in ZOOM_PERCENT_READOUTS.iter().enumerate() {
        assert_eq!(
            excused[at], 1,
            "the exception for `{needle}` in {path} matched {} lines, not one — a \
             stale entry excuses a line nobody wrote, and a duplicated one hides a \
             second site (recorded reason: {reason})",
            excused[at]
        );
    }
    // NON-VACUITY, and the shape that has failed elsewhere: a needle that stops
    // matching reports a clean sweep of nothing. `zoom` appears in a multiply in
    // the scale derivation, the wheel, the lava helper and the readouts at
    // minimum, so a single-digit count means the scan is broken, not the tree.
    assert!(
        graded >= 6,
        "the scan found only {graded} lines multiplying by `zoom` — it is not \
         reading the product it thinks it is"
    );
    eprintln!("zoom-multiply sweep: {graded} multiplying lines graded, 2 excused by reason");
}

/// **CLAIM 4b — A MIGRATED LENGTH MAY NOT ESCAPE THE OWNER BY FIELD ACCESS.**
///
/// The newtype's whole force is that it carries no arithmetic, so a `Logical`
/// cannot reach a draw call without passing `Metrics::px`. `.0` is the one hole
/// in that, and it is legitimate exactly twice: for `Chars` and `Rows`, which
/// multiply a base that is ALREADY scaled and would double if they passed the
/// owner as well.
#[test]
fn no_migrated_chrome_length_reaches_a_draw_call_through_a_bare_field_access() {
    let sources = chrome_sources();
    // The names declared in each family, gathered from the sources themselves so
    // this cannot go stale against a rename.
    let mut scaled: Vec<String> = Vec::new();
    for (_, src) in &sources {
        for line in src.lines() {
            let Some(rest) = const_decl(line) else {
                continue;
            };
            let Some((name, ty)) = rest.split_once(':') else {
                continue;
            };
            let ty = ty.split(['=', ';']).next().unwrap_or("").trim();
            // `Physical` is deliberately NOT swept: `px_physical` is the
            // identity, so a bare `.0` on one loses nothing — and one of them is
            // read inside a `const` initializer, where no method call is legal.
            if matches!(ty, "Logical" | "LogicalGrowOnly") {
                scaled.push(name.trim().to_string());
            }
        }
    }
    assert!(
        scaled.len() >= 25,
        "the family sweep found only {} scaled chrome constants",
        scaled.len()
    );
    let mut offenders = Vec::new();
    for (path, src) in &sources {
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("const ") {
                continue;
            }
            for name in &scaled {
                if line.contains(&format!("{name}.0")) {
                    offenders.push(format!(
                        "{path}:{}: `{name}.0` takes a migrated chrome length \
                         out of the pixel space by hand. Resolve it through \
                         `Metrics::px` / `px_grow_only` / `px_physical`, which \
                         is the whole reason the newtype carries no arithmetic.",
                        i + 1
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "chrome lengths escaping the owner:\n{}",
        offenders.join("\n")
    );
}

//! CHROME'S PIXEL SPACE (item 242) — the laws that did not exist.
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
//!   4. THE DECLARATION SWEEP. Every authored constant under
//!      `src/render/chrome/` states its unit family in its TYPE, and no
//!      migrated length escapes the owner through a bare field access. A
//!      suffix is not a type; this is the law that makes the type mean
//!      something for a constant nobody has written yet.

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
fn drawn_row_text_inset(frame: &[[u8; 4]], w: u32, card: [f32; 4]) -> Option<f32> {
    let x0 = card[0].max(0.0) as u32;
    let x1 = ((card[0] + card[2]) as u32).min(w);
    let y0 = (card[1] + card[3] * 0.30).max(0.0) as u32;
    let y1 = (card[1] + card[3] * 0.95) as u32;
    if x1 <= x0 + 8 || y1 <= y0 {
        return None;
    }
    let mut best: Option<u32> = None;
    for y in y0..y1 {
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
    let frame = super::pixeldiff::render_frame(p, device, queue, w, h);
    let inset = drawn_row_text_inset(&frame, w, card)?;
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
#[test]
fn the_cards_drawn_row_text_holds_its_inset_on_a_two_x_panel() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the chrome pixel-space drawn law: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    let mut worst = 0.0f32;
    for world in theme::THEMES.iter().map(|t| t.name) {
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
            "{world}: the drawn row text is only {pad1}px inside its own card at \
             dpi 1 — comparing zero against twice zero would pass on anything"
        );
        assert!(
            (lh2 - 2.0 * lh1).abs() < 0.01,
            "{world}: the row pitch is {lh1} at dpi 1 and {lh2} at dpi 2; the \
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
                "{world}: the drawn {what} measures {a}px on a 1x panel and {b}px \
                 on a 2x one, where a chrome length that passed the pixel-space \
                 owner would put it at {want} (tolerance {tol:.2}). A constant \
                 left in device pixels renders at half its tuned size on every \
                 retina display."
            );
        }
        graded += 1;
    }
    assert!(
        graded >= 15,
        "the drawn law must grade the world roster, got {graded}"
    );
    eprintln!("chrome pixel space: {graded} worlds drawn-graded, worst error {worst:.2}px");
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
/// dpi-scaled line height PLUS the overlay's extra leading PLUS the theme's own
/// `ListStyle::Bars { gap }`, and the last two were raw. So on the three
/// shipping `Bars` worlds the one quantity the tree treats as logical drifted
/// out of proportion across displays — the gap's SHARE of the row pitch halved
/// on every retina panel.
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

/// The constants under `src/render/chrome/` that are NOT lengths, each with the
/// reason it carries no unit. Enumerated by name with no wildcard, so a new bare
/// `f32` constant fails this law rather than joining the list silently.
const DIMENSIONLESS: &[(&str, &str)] = &[
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

fn chrome_sources() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render/chrome");
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("chrome dir readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let rel = path
                    .strip_prefix(env!("CARGO_MANIFEST_DIR"))
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

/// **CLAIM 4a — EVERY AUTHORED CHROME CONSTANT DECLARES ITS UNIT FAMILY.**
///
/// The `_LOGICAL` suffixes this round retired were the right instinct with an
/// unenforceable mechanism: a suffix is not a type, and nothing stopped the
/// next constant from being a bare `f32` multiplied by nothing. This is the
/// enforcement. A new chrome length authored as a bare `f32` fails HERE, by
/// name, and the only way past is to state which of the four families it is in
/// — or to record, with a reason, that it is not a length at all.
#[test]
fn every_authored_chrome_constant_declares_its_unit_family() {
    let mut offenders: Vec<String> = Vec::new();
    let mut typed = 0usize;
    let mut dimensionless = 0usize;
    for (path, src) in chrome_sources() {
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
            if !(ty == "f32" || UNIT_TYPES.contains(&ty.as_str())) {
                continue;
            }
            if UNIT_TYPES.contains(&ty.as_str()) {
                typed += 1;
            } else if DIMENSIONLESS.iter().any(|(n, _)| *n == name) {
                dimensionless += 1;
            } else {
                offenders.push(format!(
                    "{path}:{}: `{name}: f32` is an untyped chrome constant. \
                     Chrome's default pixel space is LOGICAL — declare it \
                     `Logical` (or `Physical` with a reason, `LogicalGrowOnly`, \
                     `Chars`, `Rows`), or add it to this law's DIMENSIONLESS \
                     table with the reason it is not a length.",
                    i + 1
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "chrome constants authored outside the pixel space:\n{}",
        offenders.join("\n")
    );
    // Non-vacuity: the sweep must actually be finding constants of both kinds.
    assert!(
        typed >= 25,
        "the sweep found only {typed} unit-typed chrome constants — it is not \
         reading the sources it thinks it is"
    );
    assert_eq!(
        dimensionless,
        DIMENSIONLESS.len(),
        "every entry in the DIMENSIONLESS table must still name a live chrome \
         constant; a stale entry silently excuses a name nobody wrote"
    );
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

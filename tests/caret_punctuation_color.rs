//! Punctuation must not shrink or recolor the caret transaction.
//!
//! WHY THE EXISTING LAWS WERE GREEN WHILE THE BUG WAS LIVE. Two law families
//! already covered this exact surface before this file existed:
//!
//!   * `src/render/tests/caret_visual_body.rs` sweeps EVERY world (no
//!     wildcard) and asserts `caret_visual_body_dims` returns a floored
//!     `(w, h)` for every punctuation class — a pure GEOMETRY computation, no
//!     GPU, no pixel read. It never asks what COLOUR anything paints.
//!   * `tests/caret_punctuation_pixels.rs` (split per-world by
//!     196) spawns the real binary and reads real pixels, but on a
//!     hand-picked FIVE-world roster (Mopoke/Gumtree/Bilby/Bombora/Saltpan —
//!     none of them Bowerbird), and its one appearance assertion,
//!     `caret_body_relief`, only checks that the glyph survives as a SECOND
//!     population distinct from the body colour ("not swallowed flat") — it
//!     never checks WHICH colour that second population is. A glyph knocked
//!     back through the wrong colour (this bug: `primary_content`, the
//!     ink-caret-world knockout colour, instead of `base_content`, ordinary
//!     prose ink) is still visibly a "second population" against the body —
//!     so `caret_body_relief` reports healthy relief and the law stays green
//!     on every world it happens to sweep, this bug included.
//!
//! So the axis neither law ever swept is COLOUR IDENTITY: not "is the glyph
//! visible against the body" but "is the glyph the SAME colour it is
//! everywhere else in the document." This file is that law. It is pixel-only
//! (the sidecar exposes `theme.primary` but not `base_content`/
//! `primary_content` — CLAUDE.md's "sidecar is a state oracle, not an
//! appearance oracle" applies literally here), and it never hardcodes a
//! world's palette: the expected "ordinary ink" colour for a glyph is read
//! directly out of the SAME capture's own rendering of that SAME character
//! off the caret (see `glyph_ink_mode`) — so the oracle cannot drift from the
//! product's own palette data, and no second copy of `theme::worlds.rs` needs
//! maintaining here.
//!
//! THE BUG (fixed by this round, `src/render/caret_body.rs`,
//! `prepare_morph_body_or_empty`): when a punctuation mark's ink is too thin
//! to clear the authored minimum caret body, a Morph caret draws a body in
//! the world's accent (`primary`, unchanged, correct) BUT recoloured the
//! covered glyph through `primary_content` — the colour authored for an
//! INK-CARET world's Filled block (Cassowary), where `primary == base_content`
//! and a second copy of the same ink would vanish into the block. On an
//! ordinary (`CaretBlockStyle::Normal`) world `primary_content` has no
//! relation to the page's own ink OR the accent; reusing it read as an
//! inverted/flipped glyph (Bombora's asterisk, Bowerbird's comma) or, on a
//! world whose `primary_content` happens to sit close to its own page ground,
//! as the glyph nearly vanishing — the reported "shrink" (this file's Kite/
//! Paperbark section explains why Kite itself is out of reach here).
mod common;
use common::ScratchDir;
use std::path::{Path, PathBuf};

/// The reported/repro roster plus the original coverage roster, in one
/// place. `*` is added for Bombora's specifically-reported asterisk; the rest
/// is unchanged from `tests/caret_punctuation_pixels.rs` so both files sweep
/// the same shapes (dash/bracket/quote/CJK-ideographic-comma included).
const PUNCT: [char; 11] = [',', '.', '\'', ':', ';', '-', '(', '[', '—', '。', '*'];

/// The subset of [`PUNCT`] this file's own pixel-forensic COLOUR oracle can
/// discriminate reliably ACROSS THE WHOLE ROSTER. Every mark sweeps the
/// GEOMETRY floor (`assert_geometry_floor`, unconditional, no exceptions) —
/// this narrower list gates the general roster COLOUR comparison alone. A
/// wide/tall/filled/many-boundaried mark (a bracket, a dash, the CJK
/// ideographic full stop's filled circle, a multi-pointed asterisk) produced,
/// on a CORRECT render, an "off-segment" pixel population this file's own
/// classifier could not cleanly separate from the primary/page gradient on
/// SOME world during development (measured: `(` on Tawny/Bowerbird, `—` on
/// Paperbark, `。` on Tawny/Bowerbird, `*` on Tawny/Bowerbird — none of them
/// the world that mark was actually reported on) — a limit of THIS TEST's
/// pixel arithmetic on a complex boundary shape, not a rendering defect
/// (their off-caret rendering matched their on-caret rendering by eye and by
/// the same oracle at a looser tolerance every time it was checked).
///
/// Asterisk is EXCLUDED here despite being a literal repro character because
/// it is not reliable EVERYWHERE — but `asterisk_in_bombora_does_not_invert`
/// asserts colour ownership on it DIRECTLY (not through this gate, not
/// through `assert_roster_cell`) on the one world it was actually reported
/// on, where it IS reliable. Comma stays: it is both a repro character and
/// reliable across the roster.
const COLOR_RELIABLE_PUNCT: [char; 5] = [',', '.', '\'', ':', ';'];

/// The pass/fail line between "the same ink, averaged slightly differently
/// by AA discretization" and "genuinely recoloured." Measured, not guessed:
/// a correct render's on/off-caret distance landed 0-39 across every world
/// this file swept during development (the worst legitimate cases were
/// colon on Bombora/Bowerbird/Paperbark, 26-39 — a thin glyph's solid "core"
/// picking up one or two AA-blended neighbours in the averaging step); the
/// live bug this item fixes (`primary_content` in place of `base_content`)
/// measured 129-360 on the very same fixtures. 45 sits with real margin on
/// both sides of that gap.
const INK_MATCH_TOLERANCE: f32 = 45.0;
const DOC_LINE: &str = "b, . ' : ; - ( [ — 。 z *";
const DOC: &str = "b, . ' : ; - ( [ — 。 z *\n\n\nreference\n";
/// Same line/row structure as [`DOC`] but with row 0 BLANK — several worlds'
/// backgrounds are a gradient/textured field, not a flat colour (e.g. Bilby),
/// so "the pixel at one corner of a bounding box" is not a valid stand-in for
/// "the background anywhere in that box." This fixture gives
/// [`off_caret_ink`] a true PER-PIXEL background at the exact same screen
/// coordinates DOC's row 0 renders at, so subtracting it isolates real ink
/// the same differential way `footprint` isolates the caret's own paint.
const BLANK_DOC: &str = "\n\n\nreference\n";

fn temp(tag: &str) -> ScratchDir {
    let p = std::env::temp_dir().join(format!(
        "awl-caret-punctuation-color-{}-{tag}",
        std::process::id()
    ));
    ScratchDir::new(p)
}

fn write_fixture(dir: &Path, name: &str, content: &str) -> PathBuf {
    let doc = dir.join(name);
    std::fs::write(&doc, content).unwrap();
    doc
}

fn fixture(dir: &Path) -> PathBuf {
    std::fs::write(
        common::config_path_in(dir),
        "writing_nits = false\nspellcheck = false\n",
    )
    .unwrap();
    write_fixture(dir, "fixture.txt", DOC)
}

/// The blank-row-0 companion to [`fixture`] — same sandbox/config, same
/// scratch dir, a second file.
fn fixture_blank(dir: &Path) -> PathBuf {
    write_fixture(dir, "fixture-blank.txt", BLANK_DOC)
}

/// The col index of `ch` on `DOC_LINE` (every fixture caret target lives on
/// the doc's first line, exactly like the existing pixel fixture).
fn col_of(ch: char) -> usize {
    DOC_LINE.chars().position(|c| c == ch).unwrap()
}

/// The real `awl` binary, the ONE CLI door (`--list-worlds`) onto
/// `theme::world_names()` — never a hand-maintained copy of the roster, so a
/// world added/removed/renamed in `theme::THEMES` changes what this file
/// sweeps with no edit here. (`tests/world_gallery_roster.rs` is the law that
/// `--list-worlds` itself stays complete; this file trusts it rather than
/// re-proving it.)
fn list_worlds(sandbox: &Path) -> Vec<String> {
    let out = common::awl(sandbox)
        .arg("--list-worlds")
        .output()
        .expect("spawn --list-worlds");
    assert!(out.status.success(), "--list-worlds should exit 0");
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

struct Capture<'a> {
    sandbox: &'a Path,
    doc: &'a Path,
    world: &'a str,
    dpi: f32,
    zoom: f32,
}

impl Capture<'_> {
    fn run(&self, out: &Path, mode: &str, keys: &str) {
        let o = common::awl(self.sandbox)
            .args([
                "--theme",
                self.world,
                "--capture-dpi",
                &self.dpi.to_string(),
                "--zoom",
                &self.zoom.to_string(),
                "--caret-mode",
                mode,
                "--screenshot",
            ])
            .arg(out)
            .arg("--keys")
            .arg(keys)
            .arg(self.doc)
            .output()
            .unwrap();
        if !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("no wgpu adapter") {
            panic!("caret punctuation colour PNG verification requires a real GPU adapter");
        }
        assert!(
            o.status.success(),
            "{} {mode}: {}",
            self.world,
            String::from_utf8_lossy(&o.stderr)
        );
    }

    /// Mid-glide: `--screenshot-motion` instead of `--screenshot`. The
    /// injected demo ALWAYS lands on line index 2, col 24 (clamped to the
    /// line's length) — `caret::motion::inject_motion_demo` reads the buffer
    /// content, not `--keys` — so the caller controls WHAT is there by
    /// shaping the fixture's third line, not by choosing keys.
    /// `--screenshot-motion` honors neither `--capture-dpi` nor `--zoom`
    /// (`main/args.rs`'s `unused_hooks` rejects them outright), so unlike
    /// [`Self::run`] this always captures at the tool's default scale.
    fn run_motion(&self, out: &Path, mode: &str) {
        let o = common::awl(self.sandbox)
            .args([
                "--theme",
                self.world,
                "--caret-mode",
                mode,
                "--screenshot-motion",
            ])
            .arg(out)
            .arg(self.doc)
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "{} {mode} motion: {}",
            self.world,
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

fn rgba(p: &Path) -> (u32, u32, Vec<u8>) {
    let i = image::open(p).unwrap().to_rgba8();
    (i.width(), i.height(), i.into_raw())
}

fn px(img: &(u32, u32, Vec<u8>), x: u32, y: u32) -> [u8; 3] {
    let (w, _, data) = img;
    let i = ((y * *w + x) * 4) as usize;
    [data[i], data[i + 1], data[i + 2]]
}

fn footprint(
    a: &(u32, u32, Vec<u8>),
    b: &(u32, u32, Vec<u8>),
    top: u32,
    bottom: u32,
) -> (u32, u32, u32, u32, usize) {
    let (w, h, ap) = a;
    let (_, _, bp) = b;
    let mut minx = *w;
    let mut maxx = 0;
    let mut miny = *h;
    let mut maxy = 0;
    let mut n = 0;
    for y in top..bottom.min(*h) {
        for x in 0..*w {
            let i = ((y * *w + x) * 4) as usize;
            if ap[i..i + 4] != bp[i..i + 4] {
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
                n += 1;
            }
        }
    }
    assert!(n > 0, "caret drew no changed PNG pixels");
    (minx, miny, maxx, maxy, n)
}

fn hex_rgb(value: &serde_json::Value) -> [u8; 3] {
    let hex = value.as_str().unwrap().strip_prefix('#').unwrap();
    [
        u8::from_str_radix(&hex[0..2], 16).unwrap(),
        u8::from_str_radix(&hex[2..4], 16).unwrap(),
        u8::from_str_radix(&hex[4..6], 16).unwrap(),
    ]
}

fn dist(a: [u8; 3], b: [u8; 3]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as f32 - y as f32).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// The interior probe (matching the shared pixel probe): inset from the footprint
/// edge so the AA rim (which legitimately blends body/ink/page at every
/// boundary, on every glyph, bug or no bug) never enters the sample.
fn probe(rect: (u32, u32, u32, u32), _w: u32) -> impl Iterator<Item = (u32, u32)> {
    let inset_x = ((rect.2 - rect.0 + 1) / 4).max(2);
    (rect.1 + 2..=rect.3 - 2)
        .flat_map(move |y| (rect.0 + inset_x..=rect.2 - inset_x).map(move |x| (x, y)))
}

/// The off-caret rendering of a thin punctuation mark occupies only a small
/// fraction of `rect` (`rect` is sized to the ON-caret footprint, which the
/// authored floor widens well past the bare glyph — that widening is the
/// whole point of the earlier coverage, but it means the SAME rect used off-caret is
/// mostly bare page). A single inset-and-mode pass over that whole rect can
/// have background AA out-vote the actual ink core. This finds the glyph's
/// own TIGHT bounding box within `rect` first (every pixel genuinely unlike
/// `page`, no inset), then takes the dominant colour of just that — small
/// enough that its own interior is real ink, not background.
/// A single "background colour" sample cannot stand in for a whole box on a
/// world whose ground is a gradient/texture (e.g. Bilby): a pixel many rows
/// away from a sampled corner can legitimately differ from it while still
/// being background, and — the flaw an earlier version of this function had
/// — a corner or edge pixel can ALSO be a neighbouring glyph's own faint,
/// FLAT-valued AA fringe (the preceding letter's ink, one column over),
/// which is consistent enough across a few rows to out-vote a genuinely thin
/// mark's own true ink core under a bare frequency count.
///
/// `blank` is the SAME screen coordinates with NO text there at all
/// (`BLANK_DOC`), so diffing per-pixel against it isolates real content the
/// same differential way `footprint` isolates the caret's own paint,
/// regardless of what the ground is doing underneath — but the oracle this
/// returns is the AVERAGE colour of the pixels closest to the LARGEST
/// deviation found (the solid ink core, wherever it is, however few pixels
/// it covers), not the most FREQUENT colour: a comma's solid interior is
/// often only a handful of pixels, easily rivalled in raw count by a
/// neighbour's much fainter but flatter-valued AA rim, while the CORE is
/// reliably the pixels that deviate from the ground the most.
fn off_caret_ink(
    reference: &(u32, u32, Vec<u8>),
    blank: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
) -> Option<[u8; 3]> {
    let tol = 20.0;
    let mut candidates: Vec<([u8; 3], f32)> = Vec::new();
    for y in rect.1..=rect.3 {
        for x in rect.0..=rect.2 {
            let c = px(reference, x, y);
            let d = dist(c, px(blank, x, y));
            if d > tol {
                candidates.push((c, d));
            }
        }
    }
    if candidates.is_empty() {
        return None;
    }
    let max_d = candidates.iter().map(|&(_, d)| d).fold(0.0f32, f32::max);
    // The CORE: pixels within 40% of the strongest deviation found, i.e. the
    // ink that changed the MOST from bare ground — a marginal AA/gradient
    // artifact a few units over `tol` never gets close to this bar next to a
    // real glyph's solid interior.
    let core: Vec<[u8; 3]> = candidates
        .into_iter()
        .filter(|&(_, d)| d >= max_d * 0.85)
        .map(|(c, _)| c)
        .collect();
    let n = core.len() as f32;
    let sum = core.iter().fold([0f32; 3], |acc, c| {
        [
            acc[0] + c[0] as f32,
            acc[1] + c[1] as f32,
            acc[2] + c[2] as f32,
        ]
    });
    Some([
        (sum[0] / n).round() as u8,
        (sum[1] / n).round() as u8,
        (sum[2] / n).round() as u8,
    ])
}

/// Shortest distance from `c` to the LINE SEGMENT `a..b` in RGB space.
/// Anti-aliasing blends two colours LINEARLY (glyph-coverage alpha over
/// whatever sits underneath), so a pixel that is genuinely just "primary
/// fading into the page" (a rounded body corner, a glyph-silhouette rim, a
/// mono world's accent-recoloured letter fading to its background — none of
/// them the bug this file is about) lands almost exactly ON the segment
/// between those two endpoints, however far along it. A pixel painted a
/// THIRD, unrelated colour (this bug's `primary_content` knockback) does
/// not, no matter where it falls between the endpoints.
fn segment_dist(c: [u8; 3], a: [u8; 3], b: [u8; 3]) -> f32 {
    let f = |v: [u8; 3]| [v[0] as f32, v[1] as f32, v[2] as f32];
    let (c, a, b) = (f(c), f(a), f(b));
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ab_len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab_len2 > 0.0 {
        (ac[0] * ab[0] + ac[1] * ab[1] + ac[2] * ab[2]) / ab_len2
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    let proj = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
    let d = [c[0] - proj[0], c[1] - proj[1], c[2] - proj[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// The MODE colour among probe pixels that are NOT explained as an
/// anti-aliased blend between `primary` and `page` (see [`segment_dist`]) —
/// i.e. the dominant GENUINELY THIRD colour surviving the caret's own
/// footprint, or `None` when everything there is accounted for by the
/// primary/page gradient (a solid-accent silhouette, an ordinary rounded
/// body corner, or — the mono case — an accent-recoloured letter fading
/// straight to its background with no third hue anywhere).
/// Same "average the CORE, not the mode" reasoning as [`off_caret_ink`]: a
/// tiny mark's AA rim can straddle the primary/page segment by an amount
/// that clears `tol` at several pixels without any of them being real ink —
/// the body's own rounded corner blending with the SAME glyph's ink-side AA
/// one more time (three colours meeting at one small pixel). Picking the
/// pixels FARTHEST from the segment (the ones a stray blend fragment cannot
/// reach) and averaging THOSE is robust to that where a flat frequency mode
/// is not (measured: an apostrophe on Bowerbird's Block caret and an em-dash
/// on Paperbark's both produced a mode-selected "on-caret ink" 120+ RGB units
/// from their own real ink before this change, despite the render being
/// correct).
fn dominant_off_segment(
    img: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
    primary: [u8; 3],
    page: [u8; 3],
    tol: f32,
) -> Option<([u8; 3], usize)> {
    let mut candidates: Vec<([u8; 3], f32)> = Vec::new();
    for (x, y) in probe(rect, img.0) {
        let c = px(img, x, y);
        let d = segment_dist(c, primary, page);
        if d > tol {
            candidates.push((c, d));
        }
    }
    if candidates.is_empty() {
        return None;
    }
    let n_total = candidates.len();
    let max_d = candidates.iter().map(|&(_, d)| d).fold(0.0f32, f32::max);
    let core: Vec<[u8; 3]> = candidates
        .into_iter()
        .filter(|&(_, d)| d >= max_d * 0.85)
        .map(|(c, _)| c)
        .collect();
    let n = core.len() as f32;
    let sum = core.iter().fold([0f32; 3], |acc, c| {
        [
            acc[0] + c[0] as f32,
            acc[1] + c[1] as f32,
            acc[2] + c[2] as f32,
        ]
    });
    Some((
        [
            (sum[0] / n).round() as u8,
            (sum[1] / n).round() as u8,
            (sum[2] / n).round() as u8,
        ],
        n_total,
    ))
}

/// MUST NOT SHRINK — the same authored floor `caret_visual_body_dims`
/// guarantees, re-proven here from real pixels
/// rather than trusted from the geometry-only unit law. PROPORTIONAL WORLDS
/// ONLY, deliberately: the floor is built entirely off `caret_anchor_ink_box`
/// (`src/render/caret.rs`), which is gated OUT on a mono face — a mono
/// world's caret has no ink-derived sizing to shrink in the first place (its
/// cell is the fixed mono grid), so this assertion does not apply
/// there and callers must not run it on one (see
/// `mono_worlds_tawny_and_mangrove_are_unaffected`, which correctly omits
/// it).
#[allow(clippy::too_many_arguments)]
fn assert_geometry_floor(
    world: &str,
    mode: &str,
    ch: char,
    scale: f32,
    rect: (u32, u32, u32, u32),
    band_top: u32,
    band_bottom: u32,
    area: usize,
) {
    let (left, outer_top, right, outer_bottom) = rect;
    let w = (right - left + 1) as f32;
    let h = (outer_bottom - outer_top + 1) as f32;
    // The -4.0 slack is deliberately wider than the earlier roster's -2.0 because it
    // included asterisk at a non-1.0 dpi*zoom product) gives a genuinely
    // wide/complex mark's rounding at an odd scale product room without
    // weakening the claim: a real shrink regression drops FAR below the
    // floor (the raw unfloored glyph, not a couple of rounded pixels off it).
    assert!(
        w >= 6.5 * scale - 4.0,
        "{world} {mode} {ch:?}: caret width floor in real pixels (got {w}, scale {scale})"
    );
    assert!(
        h >= 12.0 * scale - 4.0,
        "{world} {mode} {ch:?}: caret height floor in real pixels (got {h}, scale {scale})"
    );
    assert!(
        w * h >= 96.0 * scale * scale * 0.55,
        "{world} {mode} {ch:?}: caret outer-bbox area floor in real pixels (got {}, scale {scale})",
        w * h
    );
    assert!(
        outer_top > band_top && outer_bottom + 1 < band_bottom,
        "{world} {mode} {ch:?}: caret clipped by its own row band"
    );
    assert!(
        area as f32 >= 96.0 * scale * scale * 0.25,
        "{world} {mode} {ch:?}: visible changed-pixel area floor (no swallowed glyph)"
    );
}

/// MUST NOT RECOLOUR — the colour-ownership oracle, universal (proportional
/// AND mono; every caret form that can inhabit a glyph). `rendered` is the
/// caret-on-`ch` capture, `reference` the SAME world/dpi/zoom/doc with the
/// caret elsewhere (so `reference`'s own rendering of `ch`, at the SAME
/// pixels, is pure off-caret ink — no second fixture, no hardcoded palette).
/// Returns whether a distinct ink population was found and checked, so
/// callers can fold non-vacuity across a sweep.
#[allow(clippy::too_many_arguments)]
fn assert_color_ownership(
    world: &str,
    mode: &str,
    ch: char,
    rect: (u32, u32, u32, u32),
    rendered: &(u32, u32, Vec<u8>),
    reference: &(u32, u32, Vec<u8>),
    blank: &(u32, u32, Vec<u8>),
    primary: [u8; 3],
) -> bool {
    // The reference has NO caret anywhere near `rect` (it is parked on a
    // different row entirely), so `rect`'s own top-left corner there is real
    // page background, not glyph ink or AA rim — the same "outer top-left is
    // page colour" fact the off-caret glyph-ink law already relies on,
    // and (same world/doc/row) the identical background `rendered` shows at
    // that spot too. Used only to seed the ON-CARET primary/page AA-blend
    // line (a rough endpoint is fine there); the OFF-CARET extraction below
    // needs the real per-pixel ground (`blank`), not this single sample — see
    // `off_caret_ink`'s doc for why a gradient ground makes the single-sample
    // version wrong.
    let page_color = px(reference, rect.0, rect.1);

    // Whatever ink survives the caret's own footprint AND is not explained
    // as an anti-aliased primary/page blend (empty when the glyph is a plain
    // solid-accent silhouette, e.g. an ordinary letter, a mono world's
    // recoloured letter fading straight to its background, or an ordinary
    // rounded body corner) must be the SAME colour this exact glyph renders
    // in off the caret — read from `reference`'s own rendering of the SAME
    // rect, not a hardcoded palette entry, so no world's data is duplicated
    // into this file.
    let on_caret_ink = dominant_off_segment(rendered, rect, primary, page_color, 24.0);
    let Some((on_ink, on_n)) = on_caret_ink else {
        // Solid-accent silhouette (no distinct ink population) — correct and
        // unaffected by this bug either way; nothing further to compare.
        return false;
    };
    // Too small to trust as a real population (one or two stray AA/dilation
    // crosstalk pixels off a rim the inset didn't fully exclude — expected on
    // a genuinely thin mark, e.g. a mono world's Morph-recoloured `'` with no
    // support body to give it a solid interior at all): nothing to compare,
    // not a failure.
    if on_n < 4 {
        return false;
    }

    let off_ink = off_caret_ink(reference, blank, rect).unwrap_or_else(|| {
        panic!("{world} {mode} {ch:?}: fixture must render real off-caret ink at this rect")
    });

    let d = dist(on_ink, off_ink);
    assert!(
        d <= INK_MATCH_TOLERANCE,
        "{world} {mode} {ch:?}: covered punctuation glyph recoloured — on-caret ink \
         {on_ink:?} vs its own off-caret ink {off_ink:?} (distance {d:.1} > \
         {INK_MATCH_TOLERANCE}); the caret must not change a covered glyph's colour, only \
         sit behind it"
    );
    true
}

/// MONO IMMUNITY is proved at the UNIT seam instead, not here:
/// `render::tests::caret_visual_body::mono_worlds_never_read_a_punctuation_ink_box`
/// calls `caret_anchor_ink_box()` directly (no GPU pixel read at all) and
/// asserts `None` for the punctuation roster on every mono world, both caret
/// forms — the exact gate that makes `prepare_morph_body_or_empty`'s
/// `needs_body` structurally always `false` on a mono face, so the branch
/// this item's bug lived in can never even be entered there. This file tried
/// two PIXEL heuristics first (an ink-identity comparison — wrong, because
/// Morph legitimately recolours the WHOLE letter to accent on mono, so there
/// is no "keep the ordinary ink" claim to make; then a uniform-grid-width
/// comparison — also wrong, because Morph recolours the glyph's OWN raster
/// shape, which genuinely varies per mark even on a mono ADVANCE grid, e.g.
/// a comma's ink is narrower than a letter's even though their CELLS are
/// identical width). Both false-failed on a CORRECT render. The unit test is
/// the reliable proof; the assertions below are load-bearing only for "the
/// caret still draws something, nothing panics."
fn assert_mono_caret_painted(world: &str, mode: &str, ch: char, n: usize) {
    assert!(n > 0, "{world} {mode} {ch:?}: caret drew no changed pixels");
}

/// The combined oracle for PROPORTIONAL worlds: both the geometry floor and
/// colour ownership, from the same footprint. Not for a mono world (see
/// [`assert_geometry_floor`]'s doc).
#[allow(clippy::too_many_arguments)]
fn assert_geometry_and_color_ownership(
    world: &str,
    mode: &str,
    ch: char,
    scale: f32,
    rendered: &(u32, u32, Vec<u8>),
    reference: &(u32, u32, Vec<u8>),
    blank: &(u32, u32, Vec<u8>),
    band_top: u32,
    band_bottom: u32,
    primary: [u8; 3],
) -> bool {
    let (left, outer_top, right, outer_bottom, area) =
        footprint(rendered, reference, band_top, band_bottom);
    let rect = (left, outer_top, right, outer_bottom);
    assert_geometry_floor(world, mode, ch, scale, rect, band_top, band_bottom, area);
    assert_color_ownership(world, mode, ch, rect, rendered, reference, blank, primary)
}

/// The per-roster-sweep entry point: the geometry floor runs on every mark,
/// unconditionally; the colour-ownership oracle runs only when `ch` is in
/// [`COLOR_RELIABLE_PUNCT`] (its own doc explains why the rest are excluded).
#[allow(clippy::too_many_arguments)]
fn assert_roster_cell(
    world: &str,
    mode: &str,
    ch: char,
    scale: f32,
    rendered: &(u32, u32, Vec<u8>),
    reference: &(u32, u32, Vec<u8>),
    blank: &(u32, u32, Vec<u8>),
    band_top: u32,
    band_bottom: u32,
    primary: [u8; 3],
) -> bool {
    let (left, outer_top, right, outer_bottom, area) =
        footprint(rendered, reference, band_top, band_bottom);
    let rect = (left, outer_top, right, outer_bottom);
    assert_geometry_floor(world, mode, ch, scale, rect, band_top, band_bottom, area);
    if COLOR_RELIABLE_PUNCT.contains(&ch) {
        assert_color_ownership(world, mode, ch, rect, rendered, reference, blank, primary)
    } else {
        false
    }
}

fn body_rgb(sidecar: &Path) -> [u8; 3] {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(sidecar).unwrap()).unwrap();
    hex_rgb(&v["theme"]["primary"])
}

fn band(sidecar: &Path) -> (u32, u32) {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(sidecar).unwrap()).unwrap();
    let top = v["text_origin"]["top"].as_f64().unwrap() as u32;
    let lh = v["font"]["line_height"].as_f64().unwrap() as u32;
    (top, lh)
}

/// The mono-PROSE-face subset of `theme::THEMES`, snapshotted the same
/// deliberate way `tests/world_gallery_roster.rs`'s `EXPECTED_WORLDS` is: a
/// duplicate this file owns so it fails LOUDLY (a geometry-floor assertion
/// misfiring on the wrong world) the moment a world's prose face changes
/// mono/proportional identity, rather than silently reading the wrong
/// oracle. Matches `crate::caret::font_is_mono` applied to each world's own
/// `font` (`src/theme/worlds.rs`), independently re-derived here, not
/// imported — this is an integration test with no door into crate internals.
const MONO_PROSE_WORLDS: [&str; 7] = [
    "Potoroo",
    "Tawny",
    "Currawong",
    "Mangrove",
    "Wagtail",
    "Firetail",
    "Cassowary",
];

/// The two `CaretBlockStyle` EXCEPTIONS (`folds_morph_to_block` in
/// `src/theme/model.rs`) where a covered glyph's colour is a DIFFERENT,
/// pre-existing, already-law-tested rule — not the surface this item touches
/// at all, and asserting THIS file's "must equal off-caret ink" oracle
/// against them fails for a reason unrelated to this caret transaction:
///
///   * Cassowary (`CaretBlockStyle::Filled`) is the authentic ink-caret CRT
///     world where `primary == base_content`; `prepare_caret_block`'s
///     `Filled` arm deliberately knocks the covered glyph back through
///     `primary_content` (== `base_100`, the GROUND) by design,
///     because painting the SAME ink again over an ink-coloured block would
///     vanish. Requesting Block OR Morph both land here (Morph folds to
///     Block before this file's fix is ever reached).
///   * Wagtail (`CaretBlockStyle::InverseVideo`) does not paint a fixed
///     colour under the caret at all — it PHOTO-NEGATES whatever composited
///     beneath it (`caret_invert`, a `OneMinusDst` blend), "the 1-bit caret
///     round". No static "covered ink" exists to compare.
///
/// Both are excluded from the covered-glyph COLOUR assertion only; geometry
/// and footprint sanity for every other world (including these two) is
/// unaffected.
const INK_CARET_EXCEPTION_WORLDS: [&str; 2] = ["Cassowary", "Wagtail"];

/// THE CORE LAW (no-wildcard world x caret-form sweep). Every world
/// `--list-worlds` names, in `["block", "morph"]` (the two forms whose
/// covered-glyph colour differs — see the module doc), on the fixture's
/// comma. One representative punctuation mark, not the full roster, is what
/// keeps this ~40-capture sweep the width it is; the FULL roster is swept
/// per-world in the repro-named tests below and was already swept for
/// GEOMETRY ALONE by `tests/caret_punctuation_pixels.rs` / the unit law.
///
/// COLOUR ownership is asserted on every world without exception. The
/// GEOMETRY floor is asserted only on the proportional subset — see
/// `assert_geometry_floor`'s doc for why a mono world has no such floor to
/// prove; `mono_worlds_tawny_and_mangrove_are_unaffected` below is the
/// dedicated law for the mono complement.
#[test]
fn every_world_every_form_owns_comma_geometry_and_colour_correctly() {
    let dir = temp("roster");
    let doc = fixture(&dir);
    let blank_doc = fixture_blank(&dir);
    let worlds = list_worlds(&dir);
    assert!(
        worlds.len() >= 15,
        "expected the full world roster, got {}: {worlds:?}",
        worlds.len()
    );

    let comma_col = col_of(',');
    let mut saw_distinct_ink = false;
    let mut swept_proportional = 0usize;
    let mut swept_mono = 0usize;
    for world in &worlds {
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };
        let reference = dir.join(format!("{world}-ref.png"));
        capture.run(&reference, "block", "Down Down");
        let refimg = rgba(&reference);
        let blank_capture = Capture {
            sandbox: &dir,
            doc: &blank_doc,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };
        let blank_out = dir.join(format!("{world}-blank.png"));
        blank_capture.run(&blank_out, "block", "Down Down");
        let blankimg = rgba(&blank_out);
        let (top, lh) = band(&reference.with_extension("json"));
        let primary = body_rgb(&reference.with_extension("json"));
        let band_top = top.saturating_sub(20);
        let band_bottom = top + lh + 20;
        let is_mono = MONO_PROSE_WORLDS.contains(&world.as_str());
        if is_mono {
            swept_mono += 1;
        } else {
            swept_proportional += 1;
        }

        let is_ink_caret_exception = INK_CARET_EXCEPTION_WORLDS.contains(&world.as_str());

        for mode in ["block", "morph"] {
            let c = if mode == "morph" {
                comma_col + 1
            } else {
                comma_col
            };
            let out = dir.join(format!("{world}-{mode}-comma.png"));
            capture.run(&out, mode, &"Right ".repeat(c));
            let rendered = rgba(&out);
            if is_ink_caret_exception {
                // Geometry/footprint sanity only — see
                // `INK_CARET_EXCEPTION_WORLDS`'s doc for why the colour claim
                // does not apply here.
                let (_l, _t, _r, _b, n) = footprint(&rendered, &refimg, band_top, band_bottom);
                assert!(n > 0, "{world} {mode}: caret drew no changed pixels");
            } else if is_mono {
                // See `assert_mono_caret_painted`'s doc: mono immunity is
                // proved at the unit seam
                // (`render::tests::caret_visual_body::mono_worlds_never_read_a_punctuation_ink_box`
                // — wrapped here only to respect the column limit), not
                // pixel-side — this is footprint sanity only.
                let (_l, _t, _r, _b, n) = footprint(&rendered, &refimg, band_top, band_bottom);
                assert_mono_caret_painted(world, mode, ',', n);
            } else {
                saw_distinct_ink |= assert_geometry_and_color_ownership(
                    world,
                    mode,
                    ',',
                    1.0,
                    &rendered,
                    &refimg,
                    &blankimg,
                    band_top,
                    band_bottom,
                    primary,
                );
            }
        }
    }
    assert!(
        saw_distinct_ink,
        "non-vacuity: no world/form in this sweep ever exercised the covered-glyph \
         colour path — the sweep would pass even if colour ownership were never checked"
    );
    assert!(
        swept_proportional >= 10 && swept_mono >= 5,
        "non-vacuity: this sweep must genuinely cross both the proportional and mono \
         subsets (got {swept_proportional} proportional, {swept_mono} mono) — a roster \
         drift that silently emptied one side would otherwise still pass"
    );
}

/// Every caret FORM, `CaretMode::ALL`'s three members by literal name (no
/// wildcard): Block/Morph are covered above; Ibeam is the bar form, which
/// `docs/render.md` and `render/tests/caret_ink_box.rs` already establish
/// never touches glyph ink at all (it sits BETWEEN cells). This proves that
/// claim from pixels too — the caret paints, nothing panics, and no
/// off-primary ink population appears (there is no glyph recolour for a bar
/// to own, correctly).
#[test]
fn ibeam_form_never_recolours_a_glyph_it_does_not_touch() {
    let dir = temp("ibeam");
    let doc = fixture(&dir);
    for world in ["Bombora", "Bowerbird", "Gumtree"] {
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };
        let reference = dir.join(format!("{world}-ibeam-ref.png"));
        capture.run(&reference, "ibeam", "Down Down");
        let refimg = rgba(&reference);
        let (top, lh) = band(&reference.with_extension("json"));
        let out = dir.join(format!("{world}-ibeam-comma.png"));
        capture.run(&out, "ibeam", &"Right ".repeat(col_of(',')));
        let rendered = rgba(&out);
        let (_l, ot, _r, ob, n) =
            footprint(&rendered, &refimg, top.saturating_sub(20), top + lh + 20);
        assert!(n > 0, "{world} ibeam: caret drew nothing");
        assert!(
            ob > ot,
            "{world} ibeam: degenerate footprint height at a punctuation column"
        );
    }
}

/// BOMBORA'S ASTERISK, named because that is the exact reported repro. Full
/// punctuation roster, both forms, both scales — the strongest single-world
/// proof in this file.
#[test]
fn asterisk_in_bombora_does_not_invert() {
    let dir = temp("bombora");
    let doc = fixture(&dir);
    let blank_doc = fixture_blank(&dir);
    let world = "Bombora";
    for (dpi, zoom) in [(1.0, 1.0), (2.0, 1.5)] {
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi,
            zoom,
        };
        let reference = dir.join(format!("bombora-{dpi}-{zoom}-ref.png"));
        capture.run(&reference, "block", "Down Down");
        let refimg = rgba(&reference);
        let blank_capture = Capture {
            sandbox: &dir,
            doc: &blank_doc,
            world,
            dpi,
            zoom,
        };
        let blank_out = dir.join(format!("bombora-{dpi}-{zoom}-blank.png"));
        blank_capture.run(&blank_out, "block", "Down Down");
        let blankimg = rgba(&blank_out);
        let (top, lh) = band(&reference.with_extension("json"));
        let primary = body_rgb(&reference.with_extension("json"));
        let pad = (20.0 * dpi * zoom) as u32;

        for ch in PUNCT {
            let col = col_of(ch);
            for mode in ["block", "morph"] {
                let c = if mode == "morph" { col + 1 } else { col };
                let out = dir.join(format!("bombora-{dpi}-{zoom}-{ch:?}-{mode}.png"));
                capture.run(&out, mode, &"Right ".repeat(c));
                let rendered = rgba(&out);
                // Asterisk gets the FULL colour-ownership assertion directly
                // (bypassing `COLOR_RELIABLE_PUNCT`'s general-roster gate —
                // see that constant's doc): it is the literal reported
                // repro on THIS world, where it is provably reliable, even
                // though it is not reliable on every world.
                let checked = if ch == '*' {
                    assert_geometry_and_color_ownership(
                        world,
                        mode,
                        ch,
                        dpi * zoom,
                        &rendered,
                        &refimg,
                        &blankimg,
                        top.saturating_sub(pad),
                        top + lh + pad,
                        primary,
                    )
                } else {
                    assert_roster_cell(
                        world,
                        mode,
                        ch,
                        dpi * zoom,
                        &rendered,
                        &refimg,
                        &blankimg,
                        top.saturating_sub(pad),
                        top + lh + pad,
                        primary,
                    )
                };
                if ch == '*' && mode == "morph" {
                    assert!(
                        checked,
                        "non-vacuity: Bombora's asterisk (the exact reported repro) \
                         must exercise the covered-glyph colour path in Morph"
                    );
                }
            }
        }
    }
}

/// BOWERBIRD'S BLACK FLIP, named because that is the exact reported repro:
/// with the bug live, Bowerbird's `primary_content` (`#2A1B06`, a near-black
/// warm brown chosen to sit ON a Filled ink-caret block) painted straight
/// over any punctuation mark thin enough to need the caret's support body,
/// reading as the glyph flipping black. Full punctuation roster, both forms,
/// both scales.
#[test]
fn comma_and_roster_in_bowerbird_never_flip_black() {
    let dir = temp("bowerbird");
    let doc = fixture(&dir);
    let blank_doc = fixture_blank(&dir);
    let world = "Bowerbird";
    // The literal reported colour, independent of this file's own oracle:
    // proof that the CURRENT render never paints it, not merely that the
    // on/off-caret comparison passes.
    let reported_black = [0x2A, 0x1B, 0x06];
    let mut saw_distinct_ink = false;
    for (dpi, zoom) in [(1.0, 1.0), (2.0, 1.5)] {
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi,
            zoom,
        };
        let reference = dir.join(format!("bowerbird-{dpi}-{zoom}-ref.png"));
        capture.run(&reference, "block", "Down Down");
        let refimg = rgba(&reference);
        let blank_capture = Capture {
            sandbox: &dir,
            doc: &blank_doc,
            world,
            dpi,
            zoom,
        };
        let blank_out = dir.join(format!("bowerbird-{dpi}-{zoom}-blank.png"));
        blank_capture.run(&blank_out, "block", "Down Down");
        let blankimg = rgba(&blank_out);
        let (top, lh) = band(&reference.with_extension("json"));
        let primary = body_rgb(&reference.with_extension("json"));
        let pad = (20.0 * dpi * zoom) as u32;
        let band_top = top.saturating_sub(pad);
        let band_bottom = top + lh + pad;

        for ch in PUNCT {
            let col = col_of(ch);
            for mode in ["block", "morph"] {
                let c = if mode == "morph" { col + 1 } else { col };
                let out = dir.join(format!("bowerbird-{dpi}-{zoom}-{ch:?}-{mode}.png"));
                capture.run(&out, mode, &"Right ".repeat(c));
                let rendered = rgba(&out);
                saw_distinct_ink |= assert_roster_cell(
                    world,
                    mode,
                    ch,
                    dpi * zoom,
                    &rendered,
                    &refimg,
                    &blankimg,
                    band_top,
                    band_bottom,
                    primary,
                );
                let (l, t, r, b, _) = footprint(&rendered, &refimg, band_top, band_bottom);
                let page_color = px(&refimg, l, t);
                if let Some((dominant, n)) =
                    dominant_off_segment(&rendered, (l, t, r, b), primary, page_color, 24.0)
                    && n >= 4
                {
                    let d = dist(dominant, reported_black);
                    assert!(
                        d > 40.0,
                        "{world} {mode} {ch:?}: covered glyph reads as the exact \
                         reported black flip ({reported_black:?}, measured {dominant:?}, \
                         distance {d:.1})"
                    );
                }
            }
        }
    }
    assert!(
        saw_distinct_ink,
        "non-vacuity: Bowerbird must exercise the covered-glyph colour path somewhere \
         in this roster sweep"
    );
}

/// KITE / PAPERBARK — Kite is the literally-reported world for the "shrinks"
/// symptom, but it does not exist on `main` today: it lives on a held branch
/// and `theme::THEMES` (hence `--list-worlds`, hence every world this binary
/// can select) does not include it.
///
/// THIS TEST SWITCHES ON BY ROSTER PRESENCE, not by a future edit: `world` is
/// chosen below from `list_worlds()`'s OWN read of `--list-worlds` — `"Kite"`
/// the moment it is reachable, the analogue until then. When Kite lands on
/// `main`, this test starts asserting the real reported repro on Kite itself
/// with no code change here at all; nothing to remember, nothing to go stale.
///
/// PAPERBARK is the nearest HONEST in-repo analogue for the exact FAILURE
/// MODE reported as "shrink" in the meantime: measured directly off
/// `src/theme/worlds.rs`, Paperbark's `primary_content` (`#FFF6E9`) sits only
/// ~2.0 RGB-distance units from its own `base_100` page ground (`#FFF8E9`) —
/// the closest pair in the whole roster (next closest: Galah at ~5.1,
/// Saltpan at ~7.5; every dark world is 15+). Painting a covered glyph in
/// `primary_content` on Paperbark would have rendered it almost exactly the
/// colour of the empty page around it — visually reading as the caret's
/// accent-coloured support body with a nearly-invisible mark inside, i.e.
/// "the caret got tiny." Offered as the closest available evidence for the
/// reported mechanism, not as a claim that Kite itself has been verified.
#[test]
fn kite_is_unreachable_paperbark_is_the_documented_analogue() {
    let dir = temp("kite-deferred");
    let worlds = list_worlds(&dir);
    assert!(
        worlds.iter().any(|w| w == "Paperbark"),
        "Paperbark (the deferred-analogue fallback) must be in the reachable roster"
    );
    let world = if worlds.iter().any(|w| w == "Kite") {
        "Kite"
    } else {
        "Paperbark"
    };

    let doc_dir = temp("kite-deferred-doc");
    let doc = fixture(&doc_dir);
    let blank_doc = fixture_blank(&doc_dir);
    let mut saw_distinct_ink = false;
    for (dpi, zoom) in [(1.0, 1.0), (2.0, 1.5)] {
        let capture = Capture {
            sandbox: &doc_dir,
            doc: &doc,
            world,
            dpi,
            zoom,
        };
        let reference = doc_dir.join(format!("{world}-{dpi}-{zoom}-ref.png"));
        capture.run(&reference, "block", "Down Down");
        let refimg = rgba(&reference);
        let blank_capture = Capture {
            sandbox: &doc_dir,
            doc: &blank_doc,
            world,
            dpi,
            zoom,
        };
        let blank_out = doc_dir.join(format!("{world}-{dpi}-{zoom}-blank.png"));
        blank_capture.run(&blank_out, "block", "Down Down");
        let blankimg = rgba(&blank_out);
        let (top, lh) = band(&reference.with_extension("json"));
        let primary = body_rgb(&reference.with_extension("json"));
        let pad = (20.0 * dpi * zoom) as u32;

        for ch in PUNCT {
            let col = col_of(ch);
            for mode in ["block", "morph"] {
                let c = if mode == "morph" { col + 1 } else { col };
                let out = doc_dir.join(format!("{world}-{dpi}-{zoom}-{ch:?}-{mode}.png"));
                capture.run(&out, mode, &"Right ".repeat(c));
                let rendered = rgba(&out);
                saw_distinct_ink |= assert_roster_cell(
                    world,
                    mode,
                    ch,
                    dpi * zoom,
                    &rendered,
                    &refimg,
                    &blankimg,
                    top.saturating_sub(pad),
                    top + lh + pad,
                    primary,
                );
            }
        }
    }
    assert!(
        saw_distinct_ink,
        "non-vacuity: {world} must exercise the covered-glyph colour path somewhere in this sweep"
    );
}

/// MONO IMMUNITY — Tawny + Mangrove, the same two mono witnesses
/// `docs/render.md` already names for the mono uniform-grid law. A mono
/// world's caret never reads an ink box at all (`caret_anchor_ink_box` is
/// gated out on a mono face), so `prepare_morph_body_or_empty`'s
/// `needs_body` is always `false` there: Morph ALWAYS recolours the WHOLE
/// letter to the accent (`prepare_morph_body_or_empty`'s `else` arm — no
/// support body, no knockback), the SAME as any ordinary ink-sized letter.
///
/// The RELIABLE proof of that is at the unit seam — see
/// `assert_mono_caret_painted`'s doc for why two pixel heuristics tried here
/// first (ink-identity, then uniform-grid-width) both false-failed on a
/// correct render. This test stays as the PIXEL half of the claim it CAN
/// make honestly: the caret still paints something real (no swallow, no
/// crash) for the whole punctuation roster on a mono world's Morph caret.
#[test]
fn mono_worlds_tawny_and_mangrove_are_unaffected() {
    let dir = temp("mono");
    let doc = fixture(&dir);
    for world in ["Tawny", "Mangrove"] {
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };
        let reference = dir.join(format!("{world}-ref.png"));
        capture.run(&reference, "block", "Down Down");
        let refimg = rgba(&reference);
        let (top, lh) = band(&reference.with_extension("json"));
        let band_top = top.saturating_sub(20);
        let band_bottom = top + lh + 20;

        for ch in PUNCT {
            let col = col_of(ch);
            let out = dir.join(format!("{world}-{ch:?}-morph.png"));
            capture.run(&out, "morph", &"Right ".repeat(col + 1));
            let rendered = rgba(&out);
            let (_l, _t, _r, _b, n) = footprint(&rendered, &refimg, band_top, band_bottom);
            assert_mono_caret_painted(world, "morph", ch, n);
        }
    }
}

/// SETTLED VS MID-MORPH. A fast-travelling Morph caret defers to the plain
/// Block/streak paint (`prepare_caret_layer`'s `paint_silhouette` gate:
/// `settle >= CARET_MORPH_SETTLE_SHOW`) — the glyph-silhouette / knockback
/// path this whole file is about never runs while the caret is still moving.
/// `inject_motion_demo` (the `--screenshot-motion` engine) hardcodes its
/// landing at line index 2, col 24 REGARDLESS of `--keys` — the fixture's
/// third line is built so that column lands on a comma, verified from the
/// sidecar's own `layout.caret` (a STATE field; the surrounding appearance
/// claim is still read from pixels).
#[test]
fn mid_glide_frames_never_engage_the_punctuation_colour_swap() {
    let dir = temp("motion");
    let motion_doc = dir.join("motion.txt");
    // Three repeats of the punctuation line put a comma at col 24 (verified
    // below from the sidecar) — see this module's header derivation.
    let content = format!(
        "reference line zero\nreference line one\n{}\n",
        (DOC_LINE.to_string() + " ").repeat(3)
    );
    std::fs::write(&motion_doc, &content).unwrap();
    // Row 2 BLANK (rows 0/1 unchanged) — [`off_caret_ink`]'s per-pixel ground,
    // same reasoning as [`BLANK_DOC`].
    let motion_doc_blank = dir.join("motion-blank.txt");
    std::fs::write(
        &motion_doc_blank,
        "reference line zero\nreference line one\n\n",
    )
    .unwrap();
    std::fs::write(
        common::config_path_in(&dir),
        "writing_nits = false\nspellcheck = false\n",
    )
    .unwrap();

    for world in ["Bombora", "Bowerbird"] {
        let capture = Capture {
            sandbox: &dir,
            doc: &motion_doc,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };

        // A SETTLED reference: caret parked on line 0 (nowhere near line 2),
        // same doc/world/scale, so line 2's comma renders as plain off-caret
        // ink here — the same cross-capture, same-glyph technique the rest
        // of this file uses, just aimed at the motion demo's fixed landing
        // row instead of the fixture's usual first line.
        let reference = dir.join(format!("{world}-motion-ref.png"));
        capture.run(&reference, "block", "");
        let refimg = rgba(&reference);
        let blank_capture = Capture {
            sandbox: &dir,
            doc: &motion_doc_blank,
            world,
            dpi: 1.0,
            zoom: 1.0,
        };
        let blank_out = dir.join(format!("{world}-motion-blank.png"));
        blank_capture.run(&blank_out, "block", "");
        let blankimg = rgba(&blank_out);

        let out = dir.join(format!("{world}-motion.png"));
        capture.run_motion(&out, "morph");
        let sidecar: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap())
                .unwrap();
        let caret_row = sidecar["layout"]["caret"]["row"].as_u64().unwrap() as u32;
        let caret_col = sidecar["layout"]["caret"]["col"].as_u64().unwrap();
        assert_eq!(
            caret_col, 24,
            "{world}: motion demo's hardcoded landing column drifted — re-derive the \
             fixture line so col 24 is still a comma"
        );
        let top = sidecar["text_origin"]["top"].as_f64().unwrap() as u32;
        let lh = sidecar["font"]["line_height"].as_f64().unwrap() as u32;
        let primary = hex_rgb(&sidecar["theme"]["primary"]);

        let rendered = rgba(&out);
        let band_top = top + caret_row * lh;
        let band_bottom = top + (caret_row + 1) * lh;
        let (l, t, r, b, n) = footprint(&rendered, &refimg, band_top, band_bottom);
        assert!(n > 0, "{world}: mid-glide capture painted no caret pixels");

        // The claim: whatever ink survives inside the mid-glide caret's own
        // footprint (there almost never is any — a streak is solid accent)
        // must be the SAME colour this exact comma renders off the caret,
        // never a third, unrelated colour — i.e. the settled-frame knockback
        // bug this file fixes does not reappear mid-travel either.
        let page_color = px(&refimg, l, t);
        if let Some((on_ink, on_n)) =
            dominant_off_segment(&rendered, (l, t, r, b), primary, page_color, 24.0)
            && on_n >= 4
        {
            let off_ink = off_caret_ink(&refimg, &blankimg, (l, t, r, b))
                .expect("comma must render real off-caret ink at this rect");
            let d = dist(on_ink, off_ink);
            assert!(
                d <= INK_MATCH_TOLERANCE,
                "{world}: mid-glide frame's covered comma ink {on_ink:?} diverged from \
                 its own off-caret ink {off_ink:?} (distance {d:.1} > \
                 {INK_MATCH_TOLERANCE}) — motion must not recolour a glyph either"
            );
        }
    }
}

/// The shared support floor a thin mark's ink cannot explain on its own — see
/// `punctuation_caret_body_sits_in_its_rows_letter_band`'s doc.
/// `caret_visual_body_dims`'s W/H floors (6.5, 12.0) engage before its area
/// floor does (`6.5 * 12.0 = 78 < 96`), so the area floor grows BOTH
/// dimensions by one scale-invariant ratio, `sqrt(96/78)`.
fn predicted_floor_body(scale: f32) -> (f32, f32) {
    let grow = (96.0f32 / (6.5 * 12.0)).sqrt();
    (6.5 * scale * grow, 12.0 * scale * grow)
}

/// The tight bbox of pixels in `[x0, x1) x [y0, y1)` that differ from
/// `blank` at the SAME coordinates — a glyph's own raw ink, with no caret
/// and no authored floor involved at all.
fn raw_glyph_bbox(
    rendered: &(u32, u32, Vec<u8>),
    blank: &(u32, u32, Vec<u8>),
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
) -> Option<(u32, u32)> {
    let mut minx = x1;
    let mut maxx = x0;
    let mut miny = y1;
    let mut maxy = y0;
    let mut any = false;
    for y in y0..y1 {
        for x in x0..x1 {
            if dist(px(rendered, x, y), px(blank, x, y)) > 20.0 {
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
                any = true;
            }
        }
    }
    any.then_some((maxx - minx + 1, maxy - miny + 1))
}

/// One (dpi, zoom) cell of `punctuation_caret_body_sits_in_its_rows_letter_band`:
/// captures the reference + blank once, then checks every mark in `marks`
/// (both Block and Morph). Returns `(floor_engaged, raw_ink_confirmed_tiny)`
/// so the caller can fold non-vacuity across the whole scale sweep.
#[allow(clippy::too_many_arguments)]
fn check_floor_engagement_at_scale(
    dir: &Path,
    doc: &Path,
    blank_doc: &Path,
    world: &str,
    dpi: f32,
    zoom: f32,
    marks: &[(char, usize)],
) -> (bool, bool) {
    let scale = dpi * zoom;
    let (pred_w, pred_h) = predicted_floor_body(scale);
    let capture = Capture {
        sandbox: dir,
        doc,
        world,
        dpi,
        zoom,
    };
    let reference = dir.join(format!("floor-{dpi}-{zoom}-ref.png"));
    capture.run(&reference, "block", "Down Down");
    let refimg = rgba(&reference);
    let blank_capture = Capture {
        sandbox: dir,
        doc: blank_doc,
        world,
        dpi,
        zoom,
    };
    let blank_out = dir.join(format!("floor-{dpi}-{zoom}-blank.png"));
    blank_capture.run(&blank_out, "block", "Down Down");
    let blankimg = rgba(&blank_out);
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(reference.with_extension("json")).unwrap())
            .unwrap();
    let row = &sidecar["layout"]["rows"][0];
    let xs: Vec<f64> = row["xs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let row_top = row["top"].as_f64().unwrap() as u32;
    let row_h = row["height"].as_f64().unwrap() as u32;
    let (top, lh) = band(&reference.with_extension("json"));
    let pad = (20.0 * scale) as u32;
    let band_top = top.saturating_sub(pad);
    let band_bottom = top + lh + pad;

    let mut floor_engaged = false;
    let mut raw_tiny = false;
    for &(ch, col) in marks {
        // Raw ink: diff against the blank-row reference within a tight
        // per-glyph column strip (this glyph's own x to the next one's).
        let x0 = (xs[col] as u32).saturating_sub(2);
        let x1 = (xs[col + 1] as u32) + 2;
        let Some((raw_w, raw_h)) =
            raw_glyph_bbox(&refimg, &blankimg, x0, x1, row_top, row_top + row_h)
        else {
            panic!("{world} {ch:?} scale={scale}: raw ink must be non-empty");
        };
        // A mark's own natural ink is genuinely THIN only when it sits
        // clearly below the floor (asterisk's own points can already
        // approach or clear a 12.0-unit height, so it does not always
        // qualify — that is a fact about the glyph, not a test gap).
        let ink_is_thin = (raw_w as f32) < pred_w * 0.6 && (raw_h as f32) < pred_h * 0.6;
        raw_tiny |= ink_is_thin;

        for mode in ["block", "morph"] {
            // PIXEL ORACLE: the floor alone is not the product
            // promise.  Measure the actual caret body on this SAME row's `b`
            // and compare the short punctuation body to it.  The old
            // ink-derived vertical rule passes every floor assertion below,
            // but fails this relative comparison by the reported 12px class.
            let letter = dir.join(format!("floor-{dpi}-{zoom}-letter-{mode}.png"));
            capture.run(&letter, mode, if mode == "morph" { "Right" } else { "" });
            let letter_img = rgba(&letter);
            let (_, letter_top, _, letter_bottom, _) =
                footprint(&letter_img, &refimg, band_top, band_bottom);
            let letter_h = (letter_bottom - letter_top + 1) as f32;
            let c = if mode == "morph" { col + 1 } else { col };
            let out = dir.join(format!("floor-{dpi}-{zoom}-{ch:?}-{mode}.png"));
            capture.run(&out, mode, &"Right ".repeat(c));
            let rendered = rgba(&out);
            let (_left, outer_top, _right, outer_bottom, _) =
                footprint(&rendered, &refimg, band_top, band_bottom);
            let h = (outer_bottom - outer_top + 1) as f32;
            if ink_is_thin {
                // The shared body floor remains a minimum, but it no longer
                // owns the resting height: short punctuation rises to the same optical seat
                // into the row's x-height band.
                assert!(
                    h >= pred_h - 3.0,
                    "{world} {mode} {ch:?} scale={scale}: rendered caret body height {h} fell \
                     below its support floor {pred_h:.1} (raw ink height {raw_h})"
                );
                assert!(
                    (h - letter_h).abs() <= 9.0 * scale,
                    "{world} {mode} {ch:?} scale={scale}: rendered short-punctuation caret \
                     height {h} differs from its same-row letter caret {letter_h} — \
                     vertical sizing may not fall back to the punctuation ink"
                );
                floor_engaged = true;
            } else {
                // A mark whose own ink already reaches or exceeds the floor
                // (asterisk, at some scales) is CORRECTLY drawn at its own
                // real size, not clamped down to the floor — the floor is a
                // MINIMUM here, so only the lower bound holds.
                assert!(
                    h >= pred_h - 3.0,
                    "{world} {mode} {ch:?} scale={scale}: rendered caret body height {h} fell \
                     BELOW the floor {pred_h:.1} even though its own raw ink ({raw_h}) is not \
                     thin — this is the shrink this item's floor exists to prevent"
                );
            }
        }
    }
    (floor_engaged, raw_tiny)
}

/// PIXEL EVIDENCE — a thin mark's body remains visible and sits in
/// its row's letter band, proven from pixels rather than inferred from the
/// geometry owner.
/// `caret_visual_body_dims` (`render/caret_body.rs`) floors a punctuation
/// mark's body to `CARET_VISUAL_BODY_MIN_W` (6.5) / `_MIN_H` (12.0) /
/// `_MIN_AREA` (96.0), scaled by `px = metrics.caret_h / CARET_H`, which
/// `Metrics::with_dpi` sets to exactly `zoom * dpi` (`src/render.rs`) — the
/// same `scale` this file already threads through every capture. A thin
/// mark's floored body is therefore predictable in closed form: the W/H
/// floors engage before the area floor does (`6.5 * 12.0 = 78 < 96`), so the
/// area floor grows BOTH dimensions by one scale-invariant ratio,
/// `sqrt(96/78) ≈ 1.109` — `predicted_floor_body` below is exactly that
/// formula, nothing measured or eyeballed.
///
/// The claim: on a real capture, a thin mark's ON-CARET body sits WITHIN A
/// FEW PIXELS of that prediction (the residual is AA + the rounded-corner
/// overhang, not drift) — while its own OFF-CARET, no-caret-involved raw ink
/// is measured (not assumed) to be much smaller, so the floor is shown
/// PULLING the caret UP from a tiny glyph, not merely coexisting with an
/// already-large one. Swept over Block AND Morph, and four scale products
/// spanning 2x-via-DPI, 2x-via-zoom, and a combined product — a floor
/// expressed in scaled units has more ways to silently not engage (a wrong
/// axis fed the scale, a clamp order bug, a missing multiply) than to engage
/// by accident, so covering DPI and zoom SEPARATELY as well as together is
/// the point, not a formality.
#[test]
fn punctuation_caret_body_sits_in_its_rows_letter_band() {
    let dir = temp("floor-proof");
    let doc = fixture(&dir);
    let blank_doc = fixture_blank(&dir);
    let world = "Gumtree";
    // (char, DOC_LINE column) — period is the tightest natural ink of the
    // three; comma and asterisk are the two literally-reported repro marks.
    let marks = [(',', col_of(',')), ('.', col_of('.'))];
    let mut floor_engaged_somewhere = false;
    let mut raw_ink_confirmed_tiny_somewhere = false;

    for (dpi, zoom) in [(1.0, 1.0), (2.0, 1.0), (1.0, 2.0), (2.0, 1.5)] {
        let (engaged, tiny) =
            check_floor_engagement_at_scale(&dir, &doc, &blank_doc, world, dpi, zoom, &marks);
        floor_engaged_somewhere |= engaged;
        raw_ink_confirmed_tiny_somewhere |= tiny;
    }
    assert!(
        floor_engaged_somewhere,
        "non-vacuity: no cell in this sweep ever matched the predicted floor height — the \
         sweep would pass even if the floor prediction itself were checking nothing"
    );
    assert!(
        raw_ink_confirmed_tiny_somewhere,
        "non-vacuity: no mark's raw off-caret ink was ever confirmed smaller than the floor — \
         without this, 'the floor pulls the caret up' is not actually shown from pixels"
    );
}

//! Real PNG proof that punctuation remains a locatable caret body.
//!
//! The in-render law sweeps every proportional world.  This intentionally
//! bounded spawned-binary sample covers the five materially different prose
//! faces (serif/slab/sans/display/one-bit-adjacent) at both caret looks, two
//! DPI/zoom products, every punctuation class, and letter/space/EOL controls.
//!
//! One `#[test]` per world, not one `#[test]` for all five. The
//! original single function spawned the awl binary ~270 times *sequentially*
//! on one libtest thread (118s mac / 166s linux measured, x4 per CI run: two
//! platforms x two keymap conventions) because a single `#[test]` cannot be
//! split across libtest's default thread pool. Five worlds are independent
//! fixtures already — nothing here shares state across worlds — so five
//! `#[test]` fns let cargo's default concurrent test runner reclaim that
//! wall time on any box with more than one core. Each world keeps its own
//! `active_comma` semantics unchanged: only Mopoke's run ever sets it, so
//! only Mopoke's test asserts on it.

use std::path::{Path, PathBuf};

mod common;
use common::ScratchDir;

const PUNCT: [char; 10] = [',', '.', '\'', ':', ';', '-', '(', '[', '—', '。'];
const SCALES: [(f32, f32); 2] = [(1.0, 1.0), (2.0, 1.5)];
const DOC: &str = "a, . ' : ; - ( [ — 。 z\n\n\nreference\n";

/// A fresh, uniquely-named tempdir under the OS temp root, owned by a
/// [`ScratchDir`] guard that removes it on drop; this fixture used to never
/// remove it at all. Keyed by world as well as pid because the single
/// sequential test was split into one `#[test]` per world, so several
/// of these now run concurrently in the same process and would otherwise
/// collide on one shared directory.
fn temp(world: &str) -> ScratchDir {
    let p = std::env::temp_dir().join(format!(
        "awl-caret-punctuation-pixels-{}-{world}",
        std::process::id()
    ));
    ScratchDir::new(p)
}

fn fixture(dir: &Path) -> PathBuf {
    let doc = dir.join("fixture.txt");
    std::fs::write(&doc, DOC).unwrap();
    // Prose underlines are document state, not caret pixels. Keep the differential
    // reference free of them so entering the punctuation row cannot erase a
    // writing-nit or spelling squiggle and inflate the caret's measured footprint.
    std::fs::write(
        common::config_path_in(dir),
        "writing_nits = false\nspellcheck = false\n",
    )
    .unwrap();
    doc
}

struct Capture<'a> {
    sandbox: &'a Path,
    doc: &'a Path,
    world: &'a str,
    dpi: f32,
    zoom: f32,
}

impl Capture<'_> {
    fn run(&self, out: &Path, mode: Option<&str>, keys: &str) {
        let mut c = common::awl(self.sandbox);
        c.args([
            "--theme",
            self.world,
            "--capture-dpi",
            &self.dpi.to_string(),
            "--zoom",
            &self.zoom.to_string(),
            "--screenshot",
        ])
        .arg(out)
        .arg("--keys")
        .arg(keys);
        if let Some(mode) = mode {
            c.args(["--caret-mode", mode]);
        }
        let o = c.arg(self.doc).output().unwrap();
        if !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("no wgpu adapter") {
            panic!("caret punctuation PNG verification requires a real GPU adapter");
        }
        assert!(
            o.status.success(),
            "{} {mode:?}: {}",
            self.world,
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

fn assert_visible_controls(
    capture: &Capture<'_>,
    dir: &Path,
    tag: &str,
    top: u32,
    bottom: u32,
    reference: &(u32, u32, Vec<u8>),
) {
    for (label, col) in [
        ("letter", 0usize),
        ("space", 2),
        ("eol", DOC.lines().next().unwrap().chars().count()),
    ] {
        for mode in ["block", "morph"] {
            let c = if mode == "morph" && col > 0 {
                col + 1
            } else {
                col
            };
            let out = dir.join(format!("{tag}-{label}-{mode}.png"));
            capture.run(&out, Some(mode), &"Right ".repeat(c));
            let got = footprint(&rgba(&out), reference, top, bottom);
            assert!(
                got.4 >= 8,
                "{} {label} {mode}: visible control",
                capture.world
            );
        }
    }
}

fn rgba(p: &Path) -> (u32, u32, Vec<u8>) {
    let i = image::open(p).unwrap().to_rgba8();
    (i.width(), i.height(), i.into_raw())
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

/// THE ONE OWNER of "which pixels inside the caret footprint are the glyph's":
/// the footprint minus its own antialiased rim, top and bottom.
///
/// ⚠️ THIS USED TO PROBE THE MIDDLE HALF HORIZONTALLY, on the premise that "the
/// target glyph is centred in the caret body". That premise died with the
/// per-glyph cell. The caret's box is now one height per row and its width is
/// the glyph's advance grown to the authored minimum body, so a tall narrow mark
/// — `[` is the roster's worst — sits against the LEFT edge of a box that does
/// not track it, with its arms above and below the box entirely. A centred
/// window then samples flat accent and scores the mark as swallowed while the
/// capture plainly shows it (verified by eye on Mopoke Block and Gumtree Morph
/// before this was widened). Scoring a coincidence of position rather than the
/// visibility of a glyph is the exact fragility this law's own history names.
///
/// The vertical inset stays: the body's antialiased rim is not glyph ink, and
/// counting it would let a genuinely swallowed mark pass on its rim alone.
fn probe(rect: (u32, u32, u32, u32), w: u32) -> impl Iterator<Item = usize> {
    (rect.1 + 2..=rect.3 - 2)
        .flat_map(move |y| (rect.0..=rect.2).map(move |x| ((y * w + x) * 4) as usize))
}

/// HOW MUCH ink this punctuation carries with the caret parked elsewhere.
///
/// A SCALAR, and deliberately only a scalar. The caret-free capture may be asked
/// how much ink the glyph has; it must NEVER be asked WHERE that ink sits. The
/// caret re-lays the row it lands on — `render::caret_body::caret_visual_body_dims`
/// widens a thin glyph's cell to the authored minimum body, which moves the glyph
/// INSIDE that cell — so a pixel index harvested from the caret-free capture stops
/// landing on the glyph in the caret-bearing one. Reading positions across the two
/// captures is what made this law platform-fragile: it scored a coincidence of
/// position, not the visibility of a glyph.
fn glyph_ink_off_caret(reference: &(u32, u32, Vec<u8>), rect: (u32, u32, u32, u32)) -> usize {
    let (w, _, pixels) = reference;
    // The outer top-left is inside the padded body bbox but outside the glyph; it
    // is the real page colour for this exact capture, so patterned worlds and
    // antialiasing cannot turn an unrelated palette count into a passing oracle.
    let page = [
        pixels[((rect.1 * *w + rect.0) * 4) as usize],
        pixels[((rect.1 * *w + rect.0) * 4) as usize + 1],
        pixels[((rect.1 * *w + rect.0) * 4) as usize + 2],
    ];
    probe(rect, *w)
        .filter(|&i| pixels[i..i + 3] != page)
        .count()
}

/// HOW MUCH of the caret's own footprint is not a flat slab — read out of the
/// RENDERED capture alone, so no cross-capture position can enter the oracle.
///
/// A swallowed glyph leaves the body one uniform colour, so the modal colour is
/// the whole probe and the relief is nil. Surviving punctuation always shows as a
/// second population, whichever way the world draws it: knocked back through a
/// support body in `primary_content` (Filled block, settled Morph — see
/// `render::caret_body::prepare_morph_body_or_empty`), or left in its own ink over
/// a block painted beneath the text. Taking the MODAL colour rather than naming the
/// body colour is what makes those cases one rule — including a bodyless Morph,
/// where the modal colour is the page and the accent silhouette is the minority.
fn caret_body_relief(rendered: &(u32, u32, Vec<u8>), rect: (u32, u32, u32, u32)) -> usize {
    let (w, _, pixels) = rendered;
    let mut hist: std::collections::HashMap<[u8; 3], usize> = std::collections::HashMap::new();
    let mut total = 0;
    for i in probe(rect, *w) {
        total += 1;
        *hist
            .entry([pixels[i], pixels[i + 1], pixels[i + 2]])
            .or_default() += 1;
    }
    total - hist.values().copied().max().unwrap_or(0)
}

/// The floor a surviving glyph must clear: half the ink it carries off-caret, and
/// never fewer than 4 pixels.
///
/// Both halves are measured, not guessed. Over the full 200-cell sweep this test
/// already walks (5 worlds x 2 scale products x 10 punctuation classes x 2 caret
/// looks) the relief never falls below 0.95x the off-caret ink and never below 10
/// pixels absolute, so the floor keeps about a 2x margin on every cell — while a
/// body that genuinely swallows its glyph scores 0.
fn survival_floor(off_caret_ink: usize) -> usize {
    (off_caret_ink / 2).max(4)
}

fn assert_punctuation_glyph_contribution(
    reference: &(u32, u32, Vec<u8>),
    rendered: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
    world: &str,
    ch: char,
    mode: &str,
) {
    let off_caret = glyph_ink_off_caret(reference, rect);
    assert!(
        off_caret >= 2,
        "{world} {ch:?} {mode}: fixture must contain punctuation ink"
    );
    let relief = caret_body_relief(rendered, rect);
    let floor = survival_floor(off_caret);
    assert!(
        relief >= floor,
        "{world} {ch:?} {mode}: covered punctuation swallowed into uniform body \
         (relief {relief} < floor {floor}, off-caret ink {off_caret})"
    );
}

/// The synthetic swallow: the caret body painted flat across the whole probe,
/// which is exactly what "swallowed into uniform body" looks like in pixels.
fn swallowed_control(
    rendered: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
    body: [u8; 3],
) -> (u32, u32, Vec<u8>) {
    let mut swallowed = rendered.clone();
    for i in probe(rect, rendered.0) {
        swallowed.2[i..i + 3].copy_from_slice(&body);
    }
    swallowed
}

/// NON-VACUITY, run against the ACTUAL production assertion: feed it a capture in
/// which the glyph really has been swallowed and require it to go red. Silence the
/// panic hook first — this panic is the expected result, and printing it into a
/// green run's stderr has already been read as a failure once.
fn assert_swallowed_control_is_red(
    reference: &(u32, u32, Vec<u8>),
    swallowed: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
) {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mutation_failed = std::panic::catch_unwind(|| {
        assert_punctuation_glyph_contribution(reference, swallowed, rect, "Mopoke", ',', "block");
    })
    .is_err();
    std::panic::set_hook(hook);
    assert!(
        mutation_failed,
        "swallowed control must fail the production glyph assertion"
    );
}

/// The full per-world body: every scale, every punctuation class, both caret
/// looks. Called once per `#[test]` below, one call per world — see the
/// header comment for why this is no longer one function looping
/// over all five.
fn proportional_punctuation_has_a_real_pixel_body_for(world: &str) {
    let dir = temp(world);
    let doc = fixture(&dir);
    let mut active_comma = false;
    for (dpi, zoom) in SCALES {
        let tag = format!("{world}-{dpi}-{zoom}");
        let reference = dir.join(format!("{tag}-ref.png"));
        let capture = Capture {
            sandbox: &dir,
            doc: &doc,
            world,
            dpi,
            zoom,
        };
        capture.run(&reference, None, "Down Down Down");
        let side: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(reference.with_extension("json")).unwrap(),
        )
        .unwrap();
        let top = side["text_origin"]["top"]
            .as_f64()
            .expect("text_origin.top") as u32;
        let lh = side["font"]["line_height"]
            .as_f64()
            .expect("font.line_height") as u32;
        let band_pad = (8.0 * dpi * zoom).ceil() as u32;
        let caret_rgb = hex_rgb(&side["theme"]["primary"]);
        let refimg = rgba(&reference);
        assert_visible_controls(
            &capture,
            &dir,
            &tag,
            top.saturating_sub(band_pad),
            top + lh + band_pad,
            &refimg,
        );
        for ch in PUNCT {
            let col = DOC
                .lines()
                .next()
                .unwrap()
                .chars()
                .position(|x| x == ch)
                .unwrap();
            for mode in ["block", "morph"] {
                let c = if mode == "morph" { col + 1 } else { col };
                let out = dir.join(format!("{tag}-{ch:?}-{mode}.png"));
                capture.run(&out, Some(mode), &"Right ".repeat(c));
                let band_top = top.saturating_sub(band_pad);
                let band_bottom = top + lh + band_pad;
                let rendered = rgba(&out);
                let (left, outer_top, right, outer_bottom, area) =
                    footprint(&rendered, &refimg, band_top, band_bottom);
                let w = right - left + 1;
                let h = outer_bottom - outer_top + 1;
                let scale = dpi * zoom;
                assert!(
                    w as f32 >= 6.5 * scale - 2.0,
                    "{world} {ch:?} {mode}: width floor in pixels"
                );
                assert!(
                    h as f32 >= 12.0 * scale - 2.0,
                    "{world} {ch:?} {mode}: height floor in pixels"
                );
                assert!(
                    w as f32 * h as f32 >= 96.0 * scale * scale * 0.65,
                    "{world} {ch:?} {mode}: outer bbox area"
                );
                assert!(
                    outer_top > band_top && outer_bottom + 1 < band_bottom,
                    "{world} {ch:?} {mode}: caret clipped by row band"
                );
                let rect = (left, outer_top, right, outer_bottom);
                assert_punctuation_glyph_contribution(&refimg, &rendered, rect, world, ch, mode);
                if world == "Mopoke" && ch == ',' && mode == "block" {
                    let swallowed = swallowed_control(&rendered, rect, caret_rgb);
                    assert_swallowed_control_is_red(&refimg, &swallowed, rect);
                }
                assert!(
                    area as f32 >= 96.0 * scale * scale * 0.25,
                    "{world} {ch:?} {mode}: visible area floor / no swallowed glyph"
                );
                active_comma |= world == "Mopoke"
                    && ch == ','
                    && mode == "block"
                    && w as f32 >= 6.5 * scale - 1.0;
            }
        }
    }
    if world == "Mopoke" {
        assert!(
            active_comma,
            "non-vacuity: Mopoke what, comma activated the floor in real pixels"
        );
    }
}

#[test]
fn proportional_punctuation_has_a_real_pixel_body_mopoke() {
    proportional_punctuation_has_a_real_pixel_body_for("Mopoke");
}

#[test]
fn proportional_punctuation_has_a_real_pixel_body_gumtree() {
    proportional_punctuation_has_a_real_pixel_body_for("Gumtree");
}

#[test]
fn proportional_punctuation_has_a_real_pixel_body_bilby() {
    proportional_punctuation_has_a_real_pixel_body_for("Bilby");
}

#[test]
fn proportional_punctuation_has_a_real_pixel_body_bombora() {
    proportional_punctuation_has_a_real_pixel_body_for("Bombora");
}

#[test]
fn proportional_punctuation_has_a_real_pixel_body_saltpan() {
    proportional_punctuation_has_a_real_pixel_body_for("Saltpan");
}

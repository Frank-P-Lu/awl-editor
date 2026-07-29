//! tests/caret_mono_grid_pixels.rs — QUEUE ITEM 97: THE MONO CARET GRID, IN
//! PIXELS, ON EVERY WORLD.
//!
//! THE BUG. `caret::font_is_mono` was a literal three-name match — "IBM Plex
//! Mono" | "JetBrains Mono" | "Monaspace Xenon". **Iosevka**, a genuinely
//! fixed-pitch face and the display face of BOTH Currawong and Cassowary, was
//! never in it, so those two worlds took the PROPORTIONAL arm of
//! `render::caret::caret_anchor_ink_box`: the caret sized itself to each glyph's
//! own raster ink instead of holding the uniform cell every other mono world
//! keeps. Measured on this exact fixture at zoom 1 BEFORE the fix, Currawong's
//! block spanned y18..43 on `l` and y23..48 on `g` — a 5px top wobble letter to
//! letter — while Tawny/Mangrove/Potoroo/Firetail held a fixed y20 top on all
//! three. Item 91's vision smoke saw it as "Currawong's caret hugs the g".
//!
//! WHAT THIS ASSERTS, and why in pixels. The sidecar is a STATE oracle, not an
//! appearance oracle (CLAUDE.md), and "the caret holds a grid" is an appearance
//! claim: it is about where accent ink lands on screen. So every number below is
//! arithmetic over the capture PNG. The caret's drawn footprint is isolated by
//! DIFFING each capture against a REFERENCE capture of the same world and same
//! document with the caret parked four lines away — everything else on row 0
//! renders identically, so the changed pixels in row 0's band ARE the caret
//! (block quad + the glyph it recolours). That works on every world including
//! the ones a colour-keyed probe cannot read: Wagtail is 1-bit, where the accent
//! IS the text ink, and Cassowary's CRT phosphor likewise.
//!
//! THE ROSTER IS NOT HARDCODED HERE. Which worlds are mono-faced is read back
//! OUT OF THE PRODUCT: with no `--caret-mode` override the sidecar's `caret_mode`
//! reports the FONT-DERIVED default — `block` on a mono display face, `morph` on
//! a proportional one (`caret::default_mode`) — so the split this test sweeps is
//! the very predicate under test, applied by the real binary to the real font
//! files. A world added or re-faced joins the correct arm automatically, and the
//! in-crate roster laws (`render::tests::facepitch`) are what make an
//! unregistered face fail rather than drift.
//!
//! FIXTURE. `log` — an ASCENDER (`l`), an X-HEIGHT letter (`o`) and a DESCENDER
//! (`g`), the three letter classes the item names, and a real English word so no
//! spell nit underlines the row (a misspelling's squiggle is suppressed on the
//! caret's own line, which would leak into the diff).
//!
//! HERMETICITY (item 93). Every child goes through `common::awl`, which PINS
//! `$AWL_CONFIG` inside the test's own sandbox — never `env_remove`. `--theme`
//! and `--zoom 1.0` are explicit on every capture; `--caret-mode` is explicit on
//! every MEASURED capture and deliberately omitted on the reference capture,
//! whose whole job is to report the un-overridden default. With the config
//! pinned to an absent file there is no sticky `caret_mode` for it to inherit.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};

mod common;
use common::ScratchDir;

/// ASCENDER + X-HEIGHT + DESCENDER on row 0, then blank rows so the reference
/// capture can park the caret far from the band being measured.
const DOC: &str = "log\n\n\n\nnote\n";

/// Columns measured on row 0, by the `Right` presses that reach them, paired
/// with the letter that sits there.
const COLUMNS: [(&str, char); 3] = [("", 'l'), ("Right", 'o'), ("Right Right", 'g')];

/// Keys that park the caret on row 4 — off the measured band entirely — for the
/// reference capture. `Down` is `NamedKey::ArrowDown` => `Action::NextLine`, a
/// static arm on every keymap flavour (unlike `C-n`, which the Linux convention
/// rebinds — the trap `tests/bullet_blank_line_nit_pixels.rs` documents).
const PARK_KEYS: &str = "Down Down Down Down";

/// How many captures run at once. Each is a separate process holding a wgpu
/// adapter; six keeps the sweep to ~20s wall without thrashing the GPU.
const PARALLEL: usize = 6;

/// A fresh, uniquely-named tempdir under the OS temp root, owned by a
/// [`ScratchDir`] guard that removes it on drop (queue item 168: the prior
/// end-of-function `remove_dir_all` never ran when this sweep's own asserts
/// failed).
fn tmp_dir(tag: &str) -> ScratchDir {
    let dir =
        std::env::temp_dir().join(format!("awl-item97-caretgrid-{tag}-{}", std::process::id()));
    ScratchDir::new(dir)
}

/// One capture job: a world, the keys to replay, and an OPTIONAL explicit caret
/// mode (`None` = let the font-derived default resolve, which is what the
/// reference capture reads back).
struct Job {
    out: PathBuf,
    theme: String,
    keys: String,
    caret_mode: Option<&'static str>,
}

fn spawn(job: &Job, sandbox: &Path, doc: &Path) -> Child {
    let mut cmd: Command = common::awl(sandbox);
    cmd.arg("--theme")
        .arg(&job.theme)
        .arg("--zoom")
        .arg("1.0")
        .arg("--screenshot")
        .arg(&job.out)
        .arg("--keys")
        .arg(&job.keys);
    if let Some(mode) = job.caret_mode {
        cmd.arg("--caret-mode").arg(mode);
    }
    cmd.arg(doc)
        .env_remove("AWL_CJK_FORCE")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    cmd.spawn()
        .expect("failed to spawn the awl binary under CARGO_BIN_EXE_awl")
}

/// Run every job with bounded parallelism. Returns `false` (skip the whole test)
/// iff a child reported no wgpu adapter — the suite-wide headless tolerance.
fn run_all(jobs: &[Job], sandbox: &Path, doc: &Path) -> bool {
    for chunk in jobs.chunks(PARALLEL) {
        let children: Vec<Child> = chunk.iter().map(|j| spawn(j, sandbox, doc)).collect();
        for (child, job) in children.into_iter().zip(chunk) {
            let out = child.wait_with_output().expect("capture child ran");
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            if !out.status.success() && err.contains("no wgpu adapter for headless capture") {
                return false;
            }
            assert!(
                out.status.success(),
                "awl capture failed for {} ({:?}): {}\n{err}",
                job.theme,
                job.keys,
                out.status
            );
        }
    }
    true
}

fn sidecar(png: &Path) -> serde_json::Value {
    let json = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    serde_json::from_str(&json).expect("sidecar parses")
}

struct Image {
    w: u32,
    h: u32,
    rgba: Vec<u8>,
}

impl Image {
    fn px(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.w + x) * 4) as usize;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}

fn decode(png: &Path) -> Image {
    let img = image::open(png)
        .unwrap_or_else(|e| panic!("decode {}: {e}", png.display()))
        .to_rgba8();
    Image {
        w: img.width(),
        h: img.height(),
        rgba: img.into_raw(),
    }
}

/// The caret's drawn footprint on row 0: the bounding box of pixels that DIFFER
/// from the reference capture inside row 0's band, as `(left, right, top,
/// bottom)` inclusive. `None` when nothing changed (which would itself be a bug —
/// a caret that draws no ink).
///
/// The box is one pixel looser than the block quad at each edge, because the
/// glyph the block recolours is antialiased against it; every assertion below is
/// therefore a COMPARISON between boxes measured the same way, with a 1px
/// tolerance, never an absolute pixel claim.
fn caret_box(cap: &Image, refr: &Image, band: (u32, u32)) -> Option<(u32, u32, u32, u32)> {
    let (mut l, mut r, mut t, mut b) = (u32::MAX, 0u32, u32::MAX, 0u32);
    let mut any = false;
    for y in band.0..band.1.min(cap.h) {
        for x in 0..cap.w {
            if cap.px(x, y) != refr.px(x, y) {
                any = true;
                l = l.min(x);
                r = r.max(x);
                t = t.min(y);
                b = b.max(y);
            }
        }
    }
    any.then_some((l, r, t, b))
}

fn i(v: u32) -> i64 {
    v as i64
}

/// THE LAW. On every MONO-faced world the caret holds a uniform cell grid across
/// an ascender, an x-height letter and a descender — same top, same width, a
/// constant column pitch — while every PROPORTIONAL world keeps its per-letter
/// ink box. Both arms sweep the FULL shipped world roster, split by the product's
/// own font-derived caret default.
#[test]
fn caret_cell_is_glyph_independent_on_every_mono_world() {
    let sandbox = tmp_dir("sweep");
    let doc = sandbox.join("log.txt");
    std::fs::write(&doc, DOC).unwrap();

    // The world roster, straight from the binary — never a copied name list.
    let listed = common::awl(&sandbox)
        .arg("--list-worlds")
        .output()
        .expect("awl --list-worlds runs");
    assert!(
        listed.status.success(),
        "--list-worlds failed: {}",
        listed.status
    );
    let worlds: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert!(
        worlds.len() >= 18,
        "expected the full world roster, got {worlds:?}"
    );

    // Every capture up front, so the whole sweep runs at once.
    let mut jobs: Vec<Job> = Vec::new();
    for world in &worlds {
        jobs.push(Job {
            out: sandbox.join(format!("{world}-ref.png")),
            theme: world.clone(),
            keys: PARK_KEYS.to_string(),
            caret_mode: None,
        });
        for (n, (keys, _)) in COLUMNS.iter().enumerate() {
            jobs.push(Job {
                out: sandbox.join(format!("{world}-{n}.png")),
                theme: world.clone(),
                keys: (*keys).to_string(),
                caret_mode: Some("block"),
            });
        }
    }
    if !run_all(&jobs, &sandbox, &doc) {
        eprintln!("skipping caret_cell_is_glyph_independent_on_every_mono_world: no wgpu adapter");
        return;
    }

    let mut mono_worlds: Vec<&str> = Vec::new();
    let mut proportional_worlds: Vec<&str> = Vec::new();
    for world in &worlds {
        let reference = sandbox.join(format!("{world}-ref.png"));
        let side = sidecar(&reference);

        // Row 0's band: the caret can hang a little above the row top and a
        // dipper's block drops below the row bottom, so the band is the row plus
        // a margin — still nowhere near row 4, where the reference caret sits.
        let top = side["text_origin"]["top"]
            .as_f64()
            .expect("text_origin.top") as u32;
        let lh = side["font"]["line_height"]
            .as_f64()
            .expect("font.line_height") as u32;
        let band = (top.saturating_sub(8), top + lh + 8);

        // WHAT THE PREDICATE CLAIMS, read out of the product: with no
        // --caret-mode override the sidecar reports the font-derived default —
        // Block on a face `caret::font_is_mono` calls mono, Morph otherwise.
        let auto = side["caret_mode"].as_str().expect("caret_mode").to_string();
        assert!(
            auto == "block" || auto == "morph",
            "{world}: the font-derived default must be block or morph, got {auto:?}"
        );
        let claimed_mono = auto == "block";

        let refr = decode(&reference);
        let boxes: Vec<(u32, u32, u32, u32)> = (0..COLUMNS.len())
            .map(|n| {
                let cap = decode(&sandbox.join(format!("{world}-{n}.png")));
                caret_box(&cap, &refr, band).unwrap_or_else(|| {
                    panic!("{world}: the caret drew NO ink at column {n} in row 0's band")
                })
            })
            .collect();
        let tops: Vec<u32> = boxes.iter().map(|b| b.2).collect();
        let lefts: Vec<u32> = boxes.iter().map(|b| b.0).collect();
        let widths: Vec<i64> = boxes.iter().map(|b| i(b.1) - i(b.0) + 1).collect();
        let bottoms: Vec<u32> = boxes.iter().map(|b| b.3).collect();
        let letters: Vec<char> = COLUMNS.iter().map(|(_, c)| *c).collect();
        let face = side["theme"]["font_family"]
            .as_str()
            .unwrap_or("?")
            .to_string();
        let what = format!(
            "{world} ({face}) {letters:?} tops={tops:?} lefts={lefts:?} widths={widths:?} bottoms={bottoms:?}"
        );

        // WHETHER THE FACE REALLY IS MONOSPACED, measured from the SAME pixels
        // and INDEPENDENT of the predicate under test: the caret is drawn AT the
        // column it is on — either the fixed cell (mono arm) or that glyph's own
        // ink (proportional arm) — so the STEP between consecutive caret left
        // edges follows the face's own advances either way. A fixed-pitch face
        // steps by one constant; a proportional one cannot (`l` is narrow, `o` is
        // not). This is the oracle — without it the split below would be derived
        // from the very predicate it is meant to test, and a misclassified world
        // would merely change arms instead of failing.
        //
        // The two populations are far apart, not adjacent: every shipped mono
        // face measures a pitch spread of 0-1px (antialias), every proportional
        // one 4-9px. The assert pins that gap so a marginal reading is a failure
        // rather than a coin flip.
        let pitch: Vec<i64> = lefts.windows(2).map(|w| i(w[1]) - i(w[0])).collect();
        let pitch_spread = pitch.iter().max().unwrap() - pitch.iter().min().unwrap();
        assert!(
            pitch_spread <= 1 || pitch_spread >= 4,
            "ambiguous advance measurement (spread {pitch_spread}): {what}"
        );
        let really_mono = pitch_spread <= 1;

        // THE CORE LAW OF ITEM 97: the predicate agrees with the font. Iosevka is
        // a fixed-pitch face and the retired name list did not know it, so
        // Currawong and Cassowary claimed "proportional" while their advances
        // marched in lockstep — this line is what that failure looks like now.
        assert_eq!(
            claimed_mono, really_mono,
            "font_is_mono disagrees with the face's own measured advances \
             (claims mono={claimed_mono}, pitch={pitch:?}): {what}"
        );

        if really_mono {
            mono_worlds.push(world);

            // THE GRID, edge by edge. The caret's TOP does not move with the
            // letter — the exact property Currawong and Cassowary lost.
            let (tmin, tmax) = (*tops.iter().min().unwrap(), *tops.iter().max().unwrap());
            assert!(
                i(tmax) - i(tmin) <= 1,
                "mono world caret top must not move with the glyph: {what}"
            );
            // Same drawn WIDTH on every letter.
            let (wmin, wmax) = (widths.iter().min().unwrap(), widths.iter().max().unwrap());
            assert!(
                wmax - wmin <= 1,
                "mono world caret width must not move with the glyph: {what}"
            );
            // A CONSTANT, FORWARD COLUMN PITCH — the cells sit on one grid.
            assert!(
                *pitch.iter().min().unwrap() > 0,
                "mono world caret cells must advance forward (pitch={pitch:?}): {what}"
            );
            // The BOTTOM is the one declared exception: `caret_cell_vertical`
            // drops it for a real dipper (CARET_DESCENDER_PAD) so a `g` stays
            // inside its block, and holds it fixed for everything else.
            assert!(
                (i(bottoms[1]) - i(bottoms[0])).abs() <= 1,
                "the two non-dippers must share a bottom: {what}"
            );
            assert!(
                i(bottoms[2]) >= i(bottoms[1]),
                "the descender may only DROP the bottom, never raise it: {what}"
            );
        } else {
            proportional_worlds.push(world);

            // PROPORTIONAL WORLDS ARE UNCHANGED: the caret still sizes to each
            // glyph's own ink, so its top genuinely moves between an ascender and
            // an x-height letter. A predicate that over-reached — calling a
            // near-gridded face (the bundled duospace iA Writer Quattro S) mono —
            // would flatten this and fail here.
            let (tmin, tmax) = (*tops.iter().min().unwrap(), *tops.iter().max().unwrap());
            assert!(
                i(tmax) - i(tmin) >= 3,
                "proportional world caret must still hug each glyph's ink: {what}"
            );
        }
    }

    // NON-VACUITY + THE NAMED REGRESSION. Both arms must be populated, and the
    // two worlds the retired name list missed must be in the MONO arm — this
    // test's whole reason for existing.
    assert!(
        mono_worlds.len() >= 7,
        "expected every mono-faced world in the grid arm, got {mono_worlds:?}"
    );
    assert!(
        proportional_worlds.len() >= 11,
        "expected the proportional worlds in the ink arm, got {proportional_worlds:?}"
    );
    for regained in ["Currawong", "Cassowary"] {
        assert!(
            mono_worlds.contains(&regained),
            "{regained} shapes in Iosevka, a fixed-pitch face — it must hold the mono \
             caret grid (mono arm was {mono_worlds:?})"
        );
    }
}

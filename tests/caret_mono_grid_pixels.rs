//! Actual native PNG proof of glyph-adaptive blocks across the full world roster.
//!
//! The approved block policy fits shaped ink in both axes on mono and proportional
//! faces alike. Diff each letter capture against the same frame with the caret
//! parked away: an ascender must be taller than the x-height letter, a descender
//! must extend below it, and every body must remain visible and advance forward.
//! Both 1x and 2x DPI use the real spawned binary and hermetic capture sandbox.
//!
//! Fixed-pitch font classification remains owned by the in-process facepitch laws;
//! adaptive ink bearings cannot classify pitch from caret left edges anymore.

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
/// [`ScratchDir`] guard that removes it on drop; the prior
/// end-of-function `remove_dir_all` never ran when this sweep's own asserts
/// failed).
fn tmp_dir(tag: &str) -> ScratchDir {
    let dir =
        std::env::temp_dir().join(format!("awl-caret-mono-grid-{tag}-{}", std::process::id()));
    ScratchDir::new(dir)
}

/// One actual native capture at an explicit DPI. The reference parks its default
/// Block caret away from the measured row; the three probes anchor on l/o/g.
struct Job {
    out: PathBuf,
    theme: String,
    keys: String,
    caret_mode: Option<&'static str>,
    dpi: f32,
}

fn spawn(job: &Job, sandbox: &Path, doc: &Path) -> Child {
    let mut cmd: Command = common::awl(sandbox);
    cmd.arg("--theme")
        .arg(&job.theme)
        .arg("--zoom")
        .arg("1.0")
        .arg("--capture-dpi")
        .arg(job.dpi.to_string())
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

/// These are comparisons of actual changed pixels, not sidecar state or a
/// restatement of the production geometry calculation. The fixed-face envelope
/// retired by the adaptive policy fails the ascender/x-height relation.
fn assert_adaptive_ink(boxes: &[(u32, u32, u32, u32)], what: &str) {
    let heights: Vec<i64> = boxes.iter().map(|b| i(b.3) - i(b.2) + 1).collect();
    assert!(
        heights.iter().all(|h| *h > 1),
        "every body must be visible: {what}"
    );
    assert!(
        heights[0] > heights[1] + 1,
        "the x-height letter must have a shorter block than the ascender: {what}"
    );
    assert!(
        boxes[2].3 > boxes[1].3,
        "the descender's complete ink must extend below the x-height letter: {what}"
    );
    let centres: Vec<u32> = boxes.iter().map(|b| b.0 + b.1).collect();
    assert!(
        centres.windows(2).all(|pair| pair[1] > pair[0]),
        "successive visible blocks must advance forward: {what}"
    );
}

/// The world roster, straight from the binary — never a copied name list, so a
/// world added or re-faced joins this sweep automatically.
fn listed_worlds(sandbox: &std::path::Path) -> Vec<String> {
    // The world roster, straight from the binary — never a copied name list.
    let listed = common::awl(sandbox)
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

    worlds
}

#[test]
fn blocks_fit_each_letters_ink_on_every_world_and_dpi() {
    let sandbox = tmp_dir("sweep");
    let doc = sandbox.join("log.txt");
    std::fs::write(&doc, DOC).unwrap();
    let worlds = listed_worlds(&sandbox);
    let mut jobs = Vec::new();
    for world in &worlds {
        for dpi in [1.0, 2.0] {
            jobs.push(Job {
                out: sandbox.join(format!("{world}-{dpi}-ref.png")),
                theme: world.clone(),
                keys: PARK_KEYS.to_string(),
                caret_mode: None,
                dpi,
            });
            for (n, (keys, _)) in COLUMNS.iter().enumerate() {
                jobs.push(Job {
                    out: sandbox.join(format!("{world}-{dpi}-{n}.png")),
                    theme: world.clone(),
                    keys: (*keys).to_string(),
                    caret_mode: Some("block"),
                    dpi,
                });
            }
        }
    }
    if !run_all(&jobs, &sandbox, &doc) {
        eprintln!("skipping native adaptive caret pixel law: no wgpu adapter");
        return;
    }
    let mut checked = 0;
    for world in &worlds {
        for dpi in [1.0, 2.0] {
            let reference = sandbox.join(format!("{world}-{dpi}-ref.png"));
            let side = sidecar(&reference);
            let top = side["text_origin"]["top"].as_f64().unwrap() as u32;
            let lh = side["font"]["line_height"].as_f64().unwrap() as u32;
            let pad = (8.0 * dpi) as u32;
            let band = (top.saturating_sub(pad), top + lh + pad);
            let refr = decode(&reference);
            let boxes: Vec<_> = (0..COLUMNS.len())
                .map(|n| {
                    let cap = decode(&sandbox.join(format!("{world}-{dpi}-{n}.png")));
                    caret_box(&cap, &refr, band).unwrap_or_else(|| {
                        panic!("{world} dpi={dpi}: the caret drew no ink at column {n}")
                    })
                })
                .collect();
            assert_adaptive_ink(&boxes, &format!("{world} dpi={dpi} boxes={boxes:?}"));
            checked += 1;
        }
    }
    assert_eq!(checked, worlds.len() * 2, "every world at both DPI sizes");
}

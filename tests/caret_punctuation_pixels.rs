//! ITEM 126 — real PNG proof that punctuation remains a locatable caret body.
//!
//! The in-render law sweeps every proportional world.  This intentionally
//! bounded spawned-binary sample covers the five materially different prose
//! faces (serif/slab/sans/display/one-bit-adjacent) at both caret looks, two
//! DPI/zoom products, every punctuation class, and letter/space/EOL controls.

use std::path::{Path, PathBuf};

mod common;

const WORLDS: [&str; 5] = ["Mopoke", "Gumtree", "Bilby", "Bombora", "Saltpan"];
const PUNCT: [char; 10] = [',', '.', '\'', ':', ';', '-', '(', '[', '—', '。'];
const SCALES: [(f32, f32); 2] = [(1.0, 1.0), (2.0, 1.5)];
const DOC: &str = "a, . ' : ; - ( [ — 。 z\n\n\nreference\n";

fn temp() -> PathBuf {
    let p = std::env::temp_dir().join(format!("awl-item126-pixels-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
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
            panic!("item 126 PNG verification requires a real GPU adapter");
        }
        assert!(
            o.status.success(),
            "{} {mode:?}: {}",
            self.world,
            String::from_utf8_lossy(&o.stderr)
        );
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

#[test]
fn proportional_punctuation_has_a_real_pixel_body() {
    let dir = temp();
    let doc = dir.join("fixture.txt");
    std::fs::write(&doc, DOC).unwrap();
    let mut active_comma = false;
    for world in WORLDS {
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
            let top = side["text_origin"]["top"].as_u64().unwrap() as u32;
            let lh = side["font"]["line_height"].as_f64().unwrap() as u32;
            let refimg = rgba(&reference);
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
                    let got = footprint(&rgba(&out), &refimg, top.saturating_sub(8), top + lh + 8);
                    assert!(got.4 >= 8, "{world} {label} {mode}: visible control");
                }
            }
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
                    let band_top = top.saturating_sub(8);
                    let band_bottom = top + lh + 8;
                    let (left, outer_top, right, outer_bottom, area) =
                        footprint(&rgba(&out), &refimg, band_top, band_bottom);
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
    }
    assert!(
        active_comma,
        "non-vacuity: Mopoke what, comma activated the floor in real pixels"
    );
}

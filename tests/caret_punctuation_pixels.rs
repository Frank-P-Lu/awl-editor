//! ITEM 126 — real PNG proof that punctuation remains a locatable caret body.
//!
//! The in-render law sweeps every proportional world.  This intentionally
//! bounded spawned-binary sample covers the five materially different prose
//! faces (serif/slab/sans/display/one-bit-adjacent) at both caret looks, two
//! DPI/zoom products, every punctuation class, and letter/space/EOL controls.

use std::collections::BTreeMap;
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

fn dominant_rgb(image: &(u32, u32, Vec<u8>), rect: (u32, u32, u32, u32)) -> [u8; 3] {
    let (w, _, pixels) = image;
    let mut colors = BTreeMap::new();
    for y in rect.1..=rect.3 {
        for x in rect.0..=rect.2 {
            let i = ((y * *w + x) * 4) as usize;
            *colors
                .entry([pixels[i], pixels[i + 1], pixels[i + 2]])
                .or_insert(0usize) += 1;
        }
    }
    colors
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap()
        .0
}

fn glyph_mask(reference: &(u32, u32, Vec<u8>), rect: (u32, u32, u32, u32)) -> Vec<usize> {
    let (w, _, pixels) = reference;
    // The outer top-left is inside the padded body bbox but outside the glyph; it
    // is the real page colour for this exact capture, so patterned worlds and
    // antialiasing cannot turn an unrelated palette count into a passing oracle.
    let page = [
        pixels[((rect.1 * *w + rect.0) * 4) as usize],
        pixels[((rect.1 * *w + rect.0) * 4) as usize + 1],
        pixels[((rect.1 * *w + rect.0) * 4) as usize + 2],
    ];
    let mut mask = Vec::new();
    for y in rect.1..=rect.3 {
        for x in rect.0..=rect.2 {
            let i = ((y * *w + x) * 4) as usize;
            if pixels[i..i + 3] != page {
                mask.push(i);
            }
        }
    }
    mask
}

fn glyph_contribution_pixels(
    rendered: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
    mask: &[usize],
) -> usize {
    let body = dominant_rgb(rendered, rect);
    mask.iter()
        .filter(|&&i| rendered.2[i..i + 3] != body)
        .count()
}

fn assert_punctuation_glyph_contribution(
    reference: &(u32, u32, Vec<u8>),
    rendered: &(u32, u32, Vec<u8>),
    rect: (u32, u32, u32, u32),
    world: &str,
    ch: char,
    mode: &str,
) {
    let mask = glyph_mask(reference, rect);
    assert!(
        mask.len() >= 2,
        "{world} {ch:?} {mode}: fixture must contain punctuation ink"
    );
    assert!(
        glyph_contribution_pixels(rendered, rect, &mask) >= 2,
        "{world} {ch:?} {mode}: covered punctuation swallowed into uniform body"
    );
    if world == "Mopoke" && ch == ',' && mode == "block" {
        // Mutation proof: the oracle rejects the exact regression it names,
        // rather than merely observing palette variation.
        let mut erased = rendered.clone();
        let body = dominant_rgb(&erased, rect);
        for &i in &mask {
            erased.2[i..i + 3].copy_from_slice(&body);
        }
        assert_eq!(
            glyph_contribution_pixels(&erased, rect, &mask),
            0,
            "mutation proof must erase the glyph/knockout contribution"
        );
    }
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
                    assert_punctuation_glyph_contribution(
                        &refimg, &rendered, rect, world, ch, mode,
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

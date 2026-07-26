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

fn cap(
    out: &Path,
    doc: &Path,
    world: &str,
    mode: Option<&str>,
    dpi: f32,
    zoom: f32,
    keys: &str,
    sandbox: &Path,
) {
    let mut c = common::awl(sandbox);
    c.args([
        "--theme",
        world,
        "--capture-dpi",
        &dpi.to_string(),
        "--zoom",
        &zoom.to_string(),
        "--screenshot",
    ])
    .arg(out)
    .arg("--keys")
    .arg(keys);
    if let Some(mode) = mode {
        c.args(["--caret-mode", mode]);
    }
    let o = c.arg(doc).output().unwrap();
    if !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("no wgpu adapter") {
        return;
    }
    assert!(
        o.status.success(),
        "{world} {mode:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
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
) -> (u32, u32, usize) {
    let (w, h, ap) = a;
    let (_, _, bp) = b;
    let mut minx = *w;
    let mut maxx = 0;
    let mut n = 0;
    for y in top..bottom.min(*h) {
        for x in 0..*w {
            let i = ((y * *w + x) * 4) as usize;
            if ap[i..i + 4] != bp[i..i + 4] {
                minx = minx.min(x);
                maxx = maxx.max(x);
                n += 1;
            }
        }
    }
    assert!(n > 0, "caret drew no changed PNG pixels");
    (maxx - minx + 1, bottom.min(*h) - top, n)
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
            cap(
                &reference,
                &doc,
                world,
                None,
                dpi,
                zoom,
                "Down Down Down",
                &dir,
            );
            if !reference.exists() {
                eprintln!("skipping item126 pixels: no adapter");
                return;
            }
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
                    cap(
                        &out,
                        &doc,
                        world,
                        Some(mode),
                        dpi,
                        zoom,
                        &"Right ".repeat(c),
                        &dir,
                    );
                    let got = footprint(&rgba(&out), &refimg, top.saturating_sub(8), top + lh + 8);
                    assert!(got.2 >= 8, "{world} {label} {mode}: visible control");
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
                    cap(
                        &out,
                        &doc,
                        world,
                        Some(mode),
                        dpi,
                        zoom,
                        &"Right ".repeat(c),
                        &dir,
                    );
                    let (w, _, area) =
                        footprint(&rgba(&out), &refimg, top.saturating_sub(8), top + lh + 8);
                    let scale = dpi * zoom;
                    assert!(
                        w as f32 >= 6.5 * scale - 2.0,
                        "{world} {ch:?} {mode}: width floor in pixels"
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

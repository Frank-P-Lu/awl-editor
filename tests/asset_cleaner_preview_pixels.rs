//! tests/asset_cleaner_preview_pixels.rs — THE ASSET CLEANER'S LIVE PREVIEW,
//! PROVED IN PIXELS.
//!
//! Item's own Verify clause: seed distinct solid-color orphans, drive
//! selection with `--keys`, and assert by pixel arithmetic that the preview
//! region wears the selected orphan's colour and follows a selection move
//! (swept first/middle/last, not one hand-picked row); the can't-decode
//! state gets its own capture law.
//!
//! Spawns the REAL `awl` binary (`CARGO_BIN_EXE_awl`, mirroring
//! `tests/bullet_blank_line_nit_pixels.rs` / `tests/frost_rail_pixels.rs`)
//! against a REAL on-disk project: `assets::scan`'s TRASH/decode halves both
//! read real files off disk regardless of the `crate::fs` seam
//! (`image::ImageReader::open` never routes through it), so an in-memory
//! fixture cannot stand in here.
//!
//! `overlay.asset_preview` (schema `/211`) gives the exact panel rect to
//! sample — see `CAPTURE.md`'s own entry and
//! `render/chrome/asset_preview.rs::asset_preview_report`. The SELECTION →
//! PREVIEW wiring itself is already mutation-proved at two purer seams
//! (`overlay::tests::assets::selected_asset_path_follows_the_highlight_first_middle_last`
//! and
//! `render::tests::asset_preview::asset_preview_decodes_once_per_selection_never_once_per_frame`,
//! both driven red on a real reverted fix during development); this suite's
//! job is the one thing only a real capture can prove — that the wiring
//! actually reaches drawn pixels.

use std::path::{Path, PathBuf};

mod common;
use common::ScratchDir;

/// Three fixture colours, chosen far apart per channel so a loose decode /
/// downscale / sRGB round-trip cannot blur one into another, plus a name
/// (`assets/<name>`) whose ALPHABETICAL order fixes which picker row each
/// lands on — `a-red.png` < `b-green.png` < `c-blue.png`, so the sorted
/// orphan roster (`assets::scan` sorts by `rel`) puts them at rows 0/1/2:
/// first/middle/last, not one hand-picked row.
const FIXTURES: [(&str, [u8; 3]); 3] = [
    ("a-red.png", [214, 38, 38]),
    ("b-green.png", [36, 176, 58]),
    ("c-blue.png", [36, 92, 214]),
];

/// Sorts LAST (`z-` beats every fixture's `a-`/`b-`/`c-` prefix), so it never
/// disturbs the first/middle/last colour rows above — its own row is tested
/// separately.
const BROKEN_SVG: &str = "z-broken.svg";

fn tmp_dir(tag: &str) -> ScratchDir {
    let dir = std::env::temp_dir().join(format!(
        "awl-asset-cleaner-preview-pixels-{tag}-{}",
        std::process::id()
    ));
    ScratchDir::new(dir)
}

/// Build the sandbox project: one real markdown doc (so `build_index` has a
/// corpus and the scan has something to open), an `assets/` directory
/// holding the three solid-colour PNGs plus one `.svg` (a real,
/// product-real can't-decode case — `IMAGE_EXTS` includes `svg`, but
/// `image_cache::decode_upload` is PNG-only, per `assets.rs`'s own module
/// doc) — none referenced by the doc, so all four are genuine orphans.
/// Returns the doc's path (the positional `file` argument, whose PARENT
/// becomes the project root with no explicit `--root`).
fn seed_project(dir: &Path) -> PathBuf {
    let doc = dir.join("doc.md");
    std::fs::write(&doc, "nothing here references an image\n").unwrap();
    let assets = dir.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    for (name, [r, g, b]) in FIXTURES {
        let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([r, g, b, 255]));
        img.save(assets.join(name)).unwrap();
    }
    // Not a real SVG -- irrelevant, since the decoder never looks past the
    // extension gate (`assets::IMAGE_EXTS`) and then simply fails to decode
    // PNG-format bytes it is not.
    std::fs::write(assets.join(BROKEN_SVG), b"not a decodable raster").unwrap();
    doc
}

/// Spawn `awl --screenshot OUT doc.md --keys "s-p c l e a n RET<Down x N>"` —
/// the exact command-palette chord `run::tests::goto_project::
/// replay_keys_asset_cleaner_lists_only_the_orphans_from_the_scan` already
/// proves reaches the Asset Cleaner (`s-p` opens the palette; typing
/// `clean` narrows to "Clean unused assets…"; `RET` accepts it) — then
/// `Down` N times to land the highlight on row `n`. Returns `None` on a
/// headless box with no GPU adapter (mirrors every other pixel suite here).
fn capture(
    dir: &Path,
    doc: &Path,
    out: &Path,
    row: usize,
) -> Option<(image::RgbaImage, serde_json::Value)> {
    let mut keys = "s-p c l e a n RET".to_string();
    for _ in 0..row {
        keys.push_str(" Down");
    }
    let output = common::awl(dir)
        // The chord literal above is Mac-authored (`s-p` == Super-P for the
        // command palette); pin the child to that convention so the gate's
        // own `AWL_CONVENTION_FORCE=linux` sweep for the OUTER test process
        // does not leak in and flip Cmd-slot bindings to require Control
        // instead (`tests/hermetic_canary.rs`/`tests/seed_data_slot.rs` pin
        // the same way for the same reason).
        .env("AWL_CONVENTION_FORCE", "mac")
        .arg("--screenshot")
        .arg(out)
        .arg(doc)
        .arg("--keys")
        .arg(&keys)
        .output()
        .expect("failed to spawn the awl binary under CARGO_BIN_EXE_awl");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no wgpu adapter for headless capture") {
            return None;
        }
        panic!("awl capture failed: {}\n{stderr}", output.status);
    }
    let png = image::open(out)
        .unwrap_or_else(|e| panic!("decode {}: {e}", out.display()))
        .to_rgba8();
    let json = std::fs::read_to_string(out.with_extension("json")).expect("sidecar exists");
    let sidecar: serde_json::Value = serde_json::from_str(&json).expect("sidecar parses");
    Some((png, sidecar))
}

/// The panel's own rect from `overlay.asset_preview` (schema `/211`), or a
/// clear panic naming what was there instead -- a `null` here means the
/// picker didn't open or the canvas had no room, either of which is this
/// law's own setup failing, not a passing "nothing to check".
fn preview_rect(sidecar: &serde_json::Value) -> (f64, f64, f64, f64) {
    let r = &sidecar["overlay"]["asset_preview"];
    (
        r["x"]
            .as_f64()
            .unwrap_or_else(|| panic!("overlay.asset_preview missing/null in {sidecar}")),
        r["y"].as_f64().unwrap(),
        r["w"].as_f64().unwrap(),
        r["h"].as_f64().unwrap(),
    )
}

/// The pixel at the panel rect's own centre — always inside the contain-fit
/// thumbnail for a reasonably sized fixture/box (`render/chrome/
/// asset_preview.rs::prepare_asset_preview` centres the image inside the
/// panel on both axes), or inside the can't-decode text block otherwise.
fn center_pixel(png: &image::RgbaImage, rect: (f64, f64, f64, f64)) -> [u8; 3] {
    let (x, y, w, h) = rect;
    let cx = ((x + w / 2.0).round() as i64).clamp(0, png.width() as i64 - 1) as u32;
    let cy = ((y + h / 2.0).round() as i64).clamp(0, png.height() as i64 - 1) as u32;
    let p = png.get_pixel(cx, cy);
    [p[0], p[1], p[2]]
}

fn close(a: [u8; 3], b: [u8; 3], tol: i32) -> bool {
    (0..3).all(|i| (a[i] as i32 - b[i] as i32).abs() <= tol)
}

/// THE HEADLINE LAW: the preview panel wears the SELECTED orphan's own
/// colour, swept over first/middle/last so a bug that only shows up
/// mid-list (e.g. an off-by-one between `overlay.selected_index` and the
/// row the preview decodes) cannot hide behind a single-row fixture.
#[test]
fn preview_wears_the_selected_orphans_colour_first_middle_last() {
    let dir = tmp_dir("colour");
    let doc = seed_project(&dir);
    let mut sampled = Vec::new();
    for (row, (name, expected)) in FIXTURES.iter().enumerate() {
        let out = dir.join(format!("row{row}.png"));
        let Some((png, sidecar)) = capture(&dir, &doc, &out, row) else {
            eprintln!(
                "skipping preview_wears_the_selected_orphans_colour_first_middle_last: \
                 no wgpu adapter"
            );
            return;
        };
        let sel = sidecar["overlay"]["selected_index"].as_u64().unwrap() as usize;
        assert_eq!(
            sidecar["overlay"]["items"][sel].as_str(),
            Some(*name),
            "row {row} selects {name}: {sidecar}"
        );
        let rect = preview_rect(&sidecar);
        assert!(
            rect.2 > 8.0 && rect.3 > 8.0,
            "row {row}: the preview panel reports a real rect, not a sliver: {rect:?}"
        );
        let px = center_pixel(&png, rect);
        assert!(
            close(px, *expected, 24),
            "row {row} ({name}): preview centre pixel {px:?} is not within tolerance of the \
             fixture's own colour {expected:?} -- captured {}",
            out.display()
        );
        sampled.push(px);
    }
    // NON-VACUITY: three genuinely distinct fixture colours must read as three
    // genuinely distinct sampled pixels -- a law that could pass with all three
    // rows drawing the SAME (stale, or first-row-only) preview would not
    // actually be checking that the panel follows the highlight.
    assert_ne!(
        sampled[0], sampled[1],
        "row 0 and row 1 read the same pixel"
    );
    assert_ne!(
        sampled[1], sampled[2],
        "row 1 and row 2 read the same pixel"
    );
    assert_ne!(
        sampled[0], sampled[2],
        "row 0 and row 2 read the same pixel"
    );
}

/// THE CAN'T-DECODE LAW: an orphan the decoder cannot open still gets an
/// HONEST panel -- drawn (a real rect, real ink), never a blank that reads
/// as a bug, and never mistaken for a decoded image (its centre pixel does
/// not match any of the fixture colours, ruling out a stray/garbage texture
/// read).
#[test]
fn cant_decode_orphan_draws_an_honest_panel_never_a_blank() {
    let dir = tmp_dir("missing");
    let doc = seed_project(&dir);
    let out = dir.join("row-missing.png");
    // The broken `.svg` sorts last, at row 3 (after the three PNGs).
    let Some((png, sidecar)) = capture(&dir, &doc, &out, FIXTURES.len()) else {
        eprintln!(
            "skipping cant_decode_orphan_draws_an_honest_panel_never_a_blank: no wgpu adapter"
        );
        return;
    };
    let sel = sidecar["overlay"]["selected_index"].as_u64().unwrap() as usize;
    assert_eq!(
        sidecar["overlay"]["items"][sel].as_str(),
        Some(BROKEN_SVG),
        "row 3 selects the can't-decode fixture: {sidecar}"
    );
    let rect = preview_rect(&sidecar);
    assert!(
        rect.2 > 8.0 && rect.3 > 8.0,
        "the can't-decode row still draws a real panel rect: {rect:?}"
    );
    let (x, y, w, h) = rect;
    let corner = {
        let cx = (x + 4.0).round() as u32;
        let cy = (y + 4.0).round() as u32;
        let p = png.get_pixel(cx.min(png.width() - 1), cy.min(png.height() - 1));
        [p[0], p[1], p[2]]
    };
    // NOT a decoded image: nowhere near any of the three fixture colours.
    for (name, expected) in FIXTURES {
        assert!(
            !close(corner, expected, 24),
            "the can't-decode panel's own corner pixel {corner:?} reads as fixture \
             '{name}' ({expected:?}) -- it should never have decoded anything: captured {}",
            out.display()
        );
    }
    // NOT A BLANK: scan a horizontal band through the panel's own vertical
    // centre (where the can't-decode statement's middle line sits) and
    // require genuine pixel VARIATION against the corner's background
    // reading -- text ink drawn over a flat panel, proven without needing to
    // know this world's exact ink colour (a differential oracle, the same
    // shape `docs/render.md`'s query-field law and `mark_field` use).
    let mid_y = (y + h / 2.0).round() as u32;
    let x0 = x.round() as u32;
    let x1 = (x + w).round() as u32;
    let mut max_delta = 0i32;
    for px in x0..x1.min(png.width()) {
        let p = png.get_pixel(px, mid_y.min(png.height() - 1));
        let d = (0..3)
            .map(|i| ((p[i] as i32) - (corner[i] as i32)).abs())
            .max()
            .unwrap_or(0);
        max_delta = max_delta.max(d);
    }
    assert!(
        max_delta > 20,
        "the can't-decode panel reads as a flat, textless blank (max channel delta \
         against its own background corner: {max_delta}) -- an orphan that fails to \
         decode is the one this feature must show HONESTLY, never as an absence: \
         captured {}",
        out.display()
    );
}

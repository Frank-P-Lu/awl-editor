//! Real-pixel law: the Insert-link field's ghost text ("Paste or type a URL")
//! must be genuinely PRESENT (not washed to invisible) and genuinely DIMMER
//! than a real typed URL — swept over the WHOLE world roster, never one named
//! world.
//!
//! THE DEFECT this guards: before this item, the query line never carried the
//! typed URL at all (`OverlayState::query` stayed empty for the life of the
//! card — the URL lived only in a permanently-"selected", permanently-
//! duplicated candidate row below), so there was nothing to seat ghost text
//! against and no caret ever sat where a person actually reads. The fix
//! mirrors `link_edit.input` into `query` on every keystroke (the same seam
//! `RenameEdit` already proved) and shows `OverlayKind::field_placeholder`'s
//! dim text there while it's empty. This law pins the fix at the pixel level:
//! a future edit that drops the placeholder, or paints it in full `ink`
//! (reading as though something had already been typed), goes red here.

use super::{headless_dqp, view};
use crate::capture::{CaptureOpts, OverlayInfo, capture_with};
use crate::testscratch::ScratchDir;
use std::collections::HashMap;

const PLACEHOLDER: &str = "Paste or type a URL";
const TYPED_URL: &str = "https://example.com/a/long/path";

fn insert_link_opts(query: &str) -> CaptureOpts {
    let kind = crate::overlay::OverlayKind::InsertLink;
    CaptureOpts {
        overlay: Some(OverlayInfo {
            active: true,
            mode: kind.as_str(),
            title: kind.title().to_string(),
            align: crate::render::effective_card_anchor(),
            chrome_theme: None,
            query: query.to_string(),
            // Hold the caret at the same cell in the empty and typed frames.
            // The pixel diff can then isolate the field ink without admitting
            // the accent-coloured caret as a false dominant cluster.
            query_caret: 0,
            query_selection: None,
            // The card's one row is the fixed click-to-commit label, never the
            // URL itself (`OverlayState::new_link_edit`'s own doc) — this law
            // only samples the FIELD line above it, so the exact text here
            // does not matter beyond being non-empty and realistic.
            items: vec!["\u{21B5}  insert link".to_string()],
            empty: None,
            bindings: vec![String::new()],
            ranges: Vec::new(),
            match_highlights: Vec::new(),
            git: Vec::new(),
            selected_index: 0,
            hint: "esc cancel".to_string(),
            files_location: None,
            files_surface: false,
            files_query_focused: false,
            settings_focus: None,
            browse_dir: None,
            return_to: None,
            spell_target: None,
            table_dims: None,
            context_anchor: None,
            asset_preview: None,
            capture: None,
            notice: String::new(),
            lens: None,
            lens_strip: Vec::new(),
            sections: Vec::new(),
            preview_id: None,
            preview_view: None,
            workspace: false,
            detail_focus: false,
            diff_scroll: 0,
            show_hidden: false,
        }),
        ..CaptureOpts::default()
    }
}

/// Render the Insert-link card through the REAL headless-capture path
/// (`capture::capture_with`, the exact function `--screenshot` calls) and
/// decode the PNG back to an `RgbaImage`. Caller holds `testlock::serial()`.
fn capture_insert_link(
    dir: &std::path::Path,
    world: &str,
    query: &str,
    tag: &str,
) -> image::RgbaImage {
    assert!(
        crate::theme::set_active_by_name(world).is_some(),
        "unknown world {world:?}"
    );
    let buf = crate::buffer::Buffer::from_str("hello world\n");
    let opts = insert_link_opts(query);
    let png = dir.join(format!("{world}_{tag}.png"));
    capture_with(&png, &buf, &opts).expect("insert-link capture renders");
    image::open(&png)
        .expect("decode insert-link png")
        .to_rgba8()
}

/// The query header's real ink box. This follows rotated/raked header
/// compositions too; the differential sampler below removes the unchanged
/// title prefix and retains only the value that differs between frames.
fn field_region(world: &str) -> (f32, f32, f32, f32) {
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    crate::theme::set_active_by_name(world);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::InsertLink.title().to_string();
    v.overlay_query_placeholder = Some(PLACEHOLDER.to_string());
    v.overlay_items = vec!["\u{21B5}  insert link".to_string()];
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);
    let [x, y, w, h] = p
        .overlay_head_band_ink(&geom, &plan)
        .expect("Insert link has shaped header ink");
    (x, y, w, h)
}

/// The single most-common pixel color over the region — the row's own
/// background (glyph ink covers a small minority of a text row's area).
fn region_mode_color(img: &image::RgbaImage, x0: i64, y0: i64, x1: i64, y1: i64) -> [u8; 4] {
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in y0.max(0)..y1.min(img.height() as i64) {
        for x in x0.max(0)..x1.min(img.width() as i64) {
            let p = img.get_pixel(x as u32, y as u32).0;
            *counts.entry(p).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}

/// Dominant non-ground ink in `source` at pixels that differ from `other`.
/// The unchanged title prefix is therefore excluded even when it shares the
/// placeholder's muted role.
fn dominant_changed_ink(
    source: &image::RgbaImage,
    other: &image::RgbaImage,
    region: (f32, f32, f32, f32),
    bg: [u8; 4],
) -> ([u8; 4], usize) {
    let (x, y, w, h) = region;
    let (x0, y0, x1, y1) = (x as i64, y as i64, (x + w) as i64, (y + h) as i64);
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    let mut n = 0usize;
    for py in y0.max(0)..y1.min(source.height() as i64) {
        for px in x0.max(0)..x1.min(source.width() as i64) {
            let p = source.get_pixel(px as u32, py as u32).0;
            if p == other.get_pixel(px as u32, py as u32).0 {
                continue;
            }
            let d = p[0]
                .abs_diff(bg[0])
                .max(p[1].abs_diff(bg[1]))
                .max(p[2].abs_diff(bg[2]));
            if d > 24 {
                n += 1;
                *counts.entry(p).or_insert(0) += 1;
            }
        }
    }
    let dom = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(color, _)| color)
        .unwrap_or([0, 0, 0, 0]);
    (dom, n)
}

fn chan_dist(a: [u8; 4], b: [u8; 4]) -> u8 {
    a[0].abs_diff(b[0])
        .max(a[1].abs_diff(b[1]))
        .max(a[2].abs_diff(b[2]))
}

/// THE LAW: over every world in `THEMES` (derived from the roster, never a
/// named subset), the empty field's ghost text is PRESENT (a real,
/// well-populated ink cluster, not zero or a stray anti-aliased pixel) and
/// reads MEASURABLY CLOSER to that world's own `muted` than to its
/// `base_content` — proving it renders dim, not as if something had already
/// been typed. A companion capture with a real typed URL proves the SAME
/// field paints in a visibly different ink once there is real content,
/// so the law cannot pass by the placeholder and the real value looking
/// identical.
#[test]
fn insert_link_placeholder_is_present_and_dimmer_than_typed_text_across_worlds() {
    let _g = crate::testlock::serial();
    if headless_dqp(1200.0, 800.0).is_none() {
        eprintln!(
            "skipping insert_link_placeholder_is_present_and_dimmer_than_typed_text_across_worlds: \
             no wgpu adapter"
        );
        return;
    }
    let orig_theme = crate::theme::active_index();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-insert-link-field_{}", std::process::id())),
    );

    const MIN_INK_PIXELS: usize = 8;
    const CLOSER_MARGIN: i32 = 4;

    for world in crate::theme::THEMES.iter().map(|t| t.name) {
        let region = field_region(world);
        assert!(region.3 > 0.0, "{world}: field region has no height");

        let empty_img = capture_insert_link(&dir, world, "", "empty");
        let (ex0, ey0, ex1, ey1) = (
            region.0 as i64,
            region.1 as i64,
            (region.0 + region.2) as i64,
            (region.1 + region.3) as i64,
        );
        let typed_img = capture_insert_link(&dir, world, TYPED_URL, "typed");
        let bg = region_mode_color(&empty_img, ex0, ey0, ex1, ey1);
        let (ph_ink, ph_n) = dominant_changed_ink(&empty_img, &typed_img, region, bg);
        assert!(
            ph_n >= MIN_INK_PIXELS,
            "{world}: placeholder {PLACEHOLDER:?} found only {ph_n} ink pixels in the field \
             region (bg={bg:?}) — the ghost text reads empty"
        );

        crate::theme::set_active_by_name(world);
        let muted = crate::theme::muted();
        let ink = crate::theme::base_content();
        // A ONE-BIT world (today: Wagtail) authors the SAME `muted`/
        // `base_content` value by construction — its whole identity is
        // quantizing every ink down to two colors, so "dim vs full" has no
        // color to ride there at all. Derived from the roster (are the two
        // role colors actually distinct on THIS world?), never a named
        // exception, so a future one-bit world is exempted the same way
        // without a second line here.
        let world_distinguishes_dim_ink = muted.r != ink.r || muted.g != ink.g || muted.b != ink.b;
        if world_distinguishes_dim_ink {
            let dist_to_muted = chan_dist(ph_ink, [muted.r, muted.g, muted.b, 255]) as i32;
            let dist_to_ink = chan_dist(ph_ink, [ink.r, ink.g, ink.b, 255]) as i32;
            assert!(
                dist_to_muted + CLOSER_MARGIN < dist_to_ink,
                "{world}: placeholder ink {ph_ink:?} reads {dist_to_muted} from muted {muted:?} \
                 but {dist_to_ink} from full content ink {ink:?} — it is not rendering dim"
            );
        }

        let (dx0, dy0, dx1, dy1) = (ex0, ey0, ex1, ey1);
        let bg_typed = region_mode_color(&typed_img, dx0, dy0, dx1, dy1);
        let (typed_ink, typed_n) = dominant_changed_ink(&typed_img, &empty_img, region, bg_typed);
        assert!(
            typed_n >= MIN_INK_PIXELS,
            "{world}: typed URL found only {typed_n} ink pixels in the field region \
             (bg={bg_typed:?}) — the field never shows the real value"
        );
        let ph_vs_typed = chan_dist(ph_ink, typed_ink) as i32;
        assert!(
            !world_distinguishes_dim_ink || ph_vs_typed > 0,
            "{world}: placeholder ink {ph_ink:?} and typed-text ink {typed_ink:?} are the SAME \
             color — the ghost text and a real value would be indistinguishable"
        );
    }

    crate::theme::set_active(orig_theme);
}

/// GEOMETRY LAW: the field's own card stays within the canvas — never a
/// negative width, never a right edge past the window — at a NARROW width
/// (the composition brief's own "clamping on small windows" policy this item
/// preserves rather than reinvents) and at `--capture-dpi 2` (CLAUDE.md's own
/// tripwire: a chrome quantity tuned only at dpi 1 has shipped wrong on every
/// Retina display before). Swept over the same world roster, both DPIs, at a
/// canvas narrow enough that "Link destination › Paste or type a URL" cannot
/// possibly fit unclipped-and-uncllamped — this is a fit/clamp law, not a
/// no-wrap promise.
#[test]
fn insert_link_field_card_stays_within_a_narrow_canvas_at_dpi_1_and_2() {
    let _g = crate::testlock::serial();
    const NARROW_W: f32 = 380.0;
    const NARROW_H: f32 = 600.0;
    if headless_dqp(NARROW_W, NARROW_H).is_none() {
        eprintln!(
            "skipping insert_link_field_card_stays_within_a_narrow_canvas_at_dpi_1_and_2: \
             no wgpu adapter"
        );
        return;
    }
    let orig_theme = crate::theme::active_index();
    for world in crate::theme::THEMES.iter().map(|t| t.name) {
        for dpi in [1.0f32, 2.0] {
            let Some((device, queue, mut p)) = headless_dqp(NARROW_W, NARROW_H) else {
                continue;
            };
            crate::theme::set_active_by_name(world);
            p.set_dpi(dpi);
            let mut v = view("hello world\n", 0, 0);
            v.overlay_active = true;
            v.overlay_title = crate::overlay::OverlayKind::InsertLink.title().to_string();
            v.overlay_items = vec!["\u{21B5}  insert link".to_string()];
            let (device_w, device_h) = ((NARROW_W * dpi) as u32, (NARROW_H * dpi) as u32);
            p.set_view(&v);
            p.prepare(&device, &queue, device_w, device_h).unwrap();
            let [card_x, _card_y, card_w, card_h] = p
                .overlay_card_rect()
                .expect("the overlay card must be open");
            assert!(
                card_w > 0.0 && card_h > 0.0,
                "{world} @dpi{dpi}: card has non-positive extent [{card_w}x{card_h}]"
            );
            assert!(
                card_x >= -0.01 && card_x + card_w <= device_w as f32 + 0.01,
                "{world} @dpi{dpi}: card x-span [{card_x}, {}] escapes the {device_w}px-wide \
                 canvas — the small-window clamp policy this item preserves has a hole",
                card_x + card_w
            );
            let hpad = p.overlay_text_hpad();
            assert!(
                card_w - 2.0 * hpad > 0.0,
                "{world} @dpi{dpi}: the field's own text column has non-positive width \
                 after its horizontal pad — the label has nowhere to draw"
            );
        }
    }
    crate::theme::set_active(orig_theme);
}

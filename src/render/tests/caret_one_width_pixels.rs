//! THE CARET'S HORIZONTAL PRESENCE, IN RENDERED PIXELS — the width sibling of
//! `caret_one_height_pixels.rs`. That file's ratio floor pins VERTICAL
//! presence against the row's own rendered ink; nothing pinned the horizontal
//! body against the anchored glyph's own rendered ink width, so a resting
//! cell that hugged the bare raster box exactly (ratio 1.00, zero margin)
//! would have passed every existing law. Same method: every quantity below is
//! a rendered pixel compared to another rendered pixel, isolated by
//! re-rendering the identical prepared state with the caret pipelines
//! emptied, never a rendered pixel compared to an authored constant.
//!
//! Each fixture holds exactly ONE glyph (rather than a shared row of several,
//! as the height law uses) so the glyph's own rendered ink width can be read
//! directly off a text-vs-empty-document frame diff with no other glyph's
//! ink to exclude.
//!
//! SWEPT: the full proportional-display roster × 1x/2x DPI × three anchors —
//! an x-height letter (`a`), an ascender (`l`), and a wide letter (`m`) — the
//! same class spread `caret_one_height_pixels.rs` uses. Mono is absent: its
//! block width is the row-scaled cell / real advance, never ink-aligned, so
//! it carries none of `CARET_INK_PAD_W`'s margin by construction.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

const W: u32 = 400;
const H: u32 = 200;

const ANCHORS: [&str; 3] = ["a", "l", "m"];

/// The horizontal extent of a set of differing pixels, in device pixels.
struct HBounds {
    left: i32,
    right: i32,
    count: usize,
}

impl HBounds {
    fn width(&self) -> i32 {
        self.right - self.left + 1
    }
}

fn diff_hbounds(a: &[[u8; 4]], b: &[[u8; 4]]) -> HBounds {
    let (mut left, mut right, mut count) = (i32::MAX, i32::MIN, 0usize);
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let i = (y as usize) * (W as usize) + x as usize;
            if a[i] != b[i] {
                left = left.min(x);
                right = right.max(x);
                count += 1;
            }
        }
    }
    HBounds { left, right, count }
}

fn prepare(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue, text: &str) {
    p.set_view(&view(text, 0, 0));
    p.settle_caret();
    p.prepare(device, queue, W, H).unwrap();
}

/// The caret's own drawn pixels, isolated as a with/without-caret frame diff —
/// the identical technique `caret_one_height_pixels.rs` uses, applied to the
/// horizontal axis.
fn caret_hbounds(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) -> HBounds {
    let with_caret = pixeldiff::render_frame(p, device, queue, W, H);
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let without_caret = pixeldiff::render_frame(p, device, queue, W, H);
    diff_hbounds(&with_caret, &without_caret)
}

/// One (world × DPI × anchor) cell: the caret's own rendered width against the
/// SAME single glyph's own rendered ink width, both frame-diff measured.
/// Returns the ratio so the caller can report the roster's tightest.
fn assert_width_presence(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    world: &str,
    dpi: f32,
    anchor: &str,
) -> f32 {
    // The glyph's own rendered ink, caret suppressed: a document holding only
    // this one character against a genuinely empty one, so the horizontal
    // diff is that glyph's ink and nothing else's.
    prepare(p, device, queue, anchor);
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let glyph_frame = pixeldiff::render_frame(p, device, queue, W, H);
    prepare(p, device, queue, "");
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let empty_frame = pixeldiff::render_frame(p, device, queue, W, H);
    let ink = diff_hbounds(&glyph_frame, &empty_frame);
    assert!(
        ink.count > 0 && ink.width() > 1,
        "{world} d{dpi} {anchor}: the fixture glyph must render real ink \
         ({} px wide, {} pixels) or the presence floor below has no oracle",
        ink.width(),
        ink.count
    );

    // The caret's own drawn pixels on the same anchor, settled, Block form.
    prepare(p, device, queue, anchor);
    let caret = caret_hbounds(p, device, queue);
    assert!(
        caret.count > 0,
        "{world} d{dpi} {anchor}: the caret drew NOTHING — there is no width \
         to compare"
    );
    assert!(
        caret.left > 0 && caret.right < W as i32 - 1,
        "{world} d{dpi} {anchor}: the caret's box ({}..{}) touches the canvas \
         edge, so this cell has no subject to measure — widen the canvas \
         rather than reading the numbers below",
        caret.left,
        caret.right
    );

    let ratio = caret.width() as f32 / ink.width() as f32;
    assert!(
        ratio >= 1.05,
        "{world} d{dpi} {anchor}: the caret ({}px) must stand WIDER than the \
         glyph's own rendered ink ({}px) by a real margin: ratio {ratio:.3}",
        caret.width(),
        ink.width()
    );
    ratio
}

/// THE LAW. Every proportional world × both DPIs × three anchor classes: the
/// caret's own drawn width must be a real fraction WIDER than the anchored
/// glyph's own rendered ink — never the bare hug a revert to `CARET_INK_PAD_W
/// == 0` would produce (ratio ~1.00, the exact failure this floor exists to
/// catch).
#[test]
fn caret_stands_wider_than_its_anchored_glyphs_own_ink_in_real_pixels() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping the caret-width presence pixel law: no wgpu adapter");
        return;
    };
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);

    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    let mut worst_ratio = f32::MAX;

    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();
            for &anchor in &ANCHORS {
                let ratio = assert_width_presence(&mut p, &device, &queue, t.name, dpi, anchor);
                worst_ratio = worst_ratio.min(ratio);
                checked += 1;
            }
        }
    }

    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    assert!(
        checked >= 66,
        "every proportional-display world is swept at both DPIs and all three \
         anchors (got {checked})"
    );
    eprintln!(
        "caret horizontal presence: {checked} (world × DPI × anchor) cells; \
         tightest caret-to-glyph-ink width ratio {worst_ratio:.3}"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

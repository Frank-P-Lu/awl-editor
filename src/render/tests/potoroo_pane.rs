//! ITEM 123 — Potoroo's rust-striped Frame can meet the ordinary `base_300`
//! command-pane face, leaving no reliably locatable card edge. `Elevation::Recessed`
//! makes the Pane-family face one existing value step darker (`base_200`) without
//! adding a rim, so this file proves the outcome in real rendered pixels rather
//! than trusting an instance count or sidecar state.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

static ELEVATION_OVERRIDE: std::sync::Mutex<Option<theme::Elevation>> = std::sync::Mutex::new(None);

pub(in crate::render) fn elevation_override() -> Option<theme::Elevation> {
    *ELEVATION_OVERRIDE.lock().unwrap_or_else(|e| e.into_inner())
}

fn set_elevation_override(value: Option<theme::Elevation>) {
    *ELEVATION_OVERRIDE.lock().unwrap_or_else(|e| e.into_inner()) = value;
}

fn redmean(a: theme::Srgb, b: theme::Srgb) -> f32 {
    let rbar = (a.r as f32 + b.r as f32) * 0.5;
    let dr = a.r as f32 - b.r as f32;
    let dg = a.g as f32 - b.g as f32;
    let db = a.b as f32 - b.b as f32;
    ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
        .sqrt()
}

fn avg(px: &[[u8; 4]], w: u32, x: i32, y: i32) -> theme::Srgb {
    let mut sum = [0u32; 3];
    let mut n = 0u32;
    for yy in (y - 1)..=(y + 1) {
        for xx in (x - 1)..=(x + 1) {
            assert!(
                xx >= 0 && xx < w as i32 && yy >= 0,
                "sample stays on-canvas"
            );
            let p = px[(yy as u32 * w + xx as u32) as usize];
            for c in 0..3 {
                sum[c] += p[c] as u32;
            }
            n += 1;
        }
    }
    theme::Srgb::rgb((sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8)
}

fn pane_view(selected: usize) -> ViewState {
    let mut v = view("quiet Room text stays behind this command surface\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "Commands";
    v.overlay_items = vec![
        "Open…".into(),
        "Save".into(),
        "Theme…".into(),
        "Settings".into(),
    ];
    v.overlay_bindings = vec!["⌘O".into(), "⌘S".into(), "⌘T".into(), "".into()];
    v.overlay_selected = selected;
    v
}

fn changed_in_rect(a: &[[u8; 4]], b: &[[u8; 4]], width: u32, rect: [f32; 4]) -> (usize, usize) {
    let [x, y, w, h] = rect;
    let (x0, y0) = ((x + 3.0).ceil() as u32, (y + 3.0).ceil() as u32);
    let (x1, y1) = ((x + w - 3.0).floor() as u32, (y + h - 3.0).floor() as u32);
    let mut changed = 0;
    let mut total = 0;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let i = (yy * width + xx) as usize;
            changed += usize::from(a[i] != b[i]);
            total += 1;
        }
    }
    (changed, total)
}

/// MATCHED PARENT-vs-CANDIDATE arithmetic at the real framebuffer: force the
/// pre-item-123 Flat value, then Recessed, with geometry/text held identical.
/// Both split surfaces must materially change; closing Commands makes the two
/// frames byte-identical because elevation owns summoned Pane pixels only.
#[test]
fn matched_flat_to_recessed_changes_header_and_body_but_not_the_closed_room() {
    let _g = crate::testlock::serial();
    let _pin = theme::WorldPin::world("Potoroo").expect("Potoroo ships");
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping matched Potoroo Pane pixels: no wgpu adapter");
        return;
    };

    let open = pane_view(1);
    set_elevation_override(Some(theme::Elevation::Flat));
    p.set_view(&open);
    p.prepare(&device, &queue, w, h).unwrap();
    let fills = p.overlay_pane_fills_probe();
    assert_eq!(
        fills.len(),
        2,
        "fixture has a header/query and body surface"
    );
    let parent = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    set_elevation_override(Some(theme::Elevation::Recessed));
    p.set_view(&open);
    p.prepare(&device, &queue, w, h).unwrap();
    let candidate = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

    for (name, rect) in [("header/query", fills[0]), ("candidate body", fills[1])] {
        let (changed, total) = changed_in_rect(&parent, &candidate, w, rect);
        assert!(
            changed * 10 >= total * 6,
            "{name}: Recessed must materially repaint the whole Pane face \
             (changed {changed}/{total} pixels)"
        );
    }

    let closed = view("quiet Room text stays behind this command surface\n", 0, 0);
    set_elevation_override(Some(theme::Elevation::Flat));
    p.set_view(&closed);
    p.prepare(&device, &queue, w, h).unwrap();
    let closed_parent = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    set_elevation_override(Some(theme::Elevation::Recessed));
    p.set_view(&closed);
    p.prepare(&device, &queue, w, h).unwrap();
    let closed_candidate = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    assert_eq!(
        closed_parent, closed_candidate,
        "closing Commands restores the byte-identical Potoroo Room"
    );
    set_elevation_override(None);
}

/// The actual visible edge, not card geometry: each side samples one pixel
/// inside a clean fill margin and one pixel immediately beyond it. This covers
/// narrow/wide logical windows, 1×/2× physical scale, and both split surfaces;
/// Potoroo's moving stripe phase cannot hide a side behind an equal value.
#[test]
fn potoroo_pane_edges_separate_from_the_real_striped_frame_on_all_sides() {
    let _g = crate::testlock::serial();
    let _pin = theme::WorldPin::world("Potoroo").expect("Potoroo ships");
    for (logical_w, logical_h, dpi) in [(640u32, 700u32, 1.0), (1200, 800, 1.0), (1200, 800, 2.0)] {
        let (w, h) = (
            (logical_w as f32 * dpi) as u32,
            (logical_h as f32 * dpi) as u32,
        );
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping Potoroo Pane pixels: no wgpu adapter");
            return;
        };
        p.set_dpi(dpi);
        p.set_view(&pane_view(1));
        p.prepare(&device, &queue, w, h).unwrap();
        let fills = p.overlay_pane_fills_probe();
        assert_eq!(
            fills.len(),
            2,
            "{logical_w}@{dpi}: Potoroo is a split Pane world"
        );
        let px = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
        let upper = fills[0];
        let lower = fills[1];
        let authored = theme::base_200();
        for (name, fill) in [("header", upper), ("body", lower)] {
            // Sampled just inside each surface's OWN TOP edge, which is card
            // ground on both by construction (the upper surface's query line
            // starts a full pad below its top; the lower surface's first row
            // starts a beat's tail below its own). The surface's vertical CENTRE
            // is not ground on either: item 219 moved the seam up to hug the
            // query bar, so the lower surface's midpoint now falls on the
            // selected row's own band.
            let face = avg(
                &px,
                w,
                (fill[0] + fill[2] * 0.5).round() as i32,
                (fill[1] + 6.0 * dpi).round() as i32,
            );
            assert!(
                redmean(face, authored) < 8.0,
                "{logical_w}x{logical_h}@{dpi} {name}: drawn Pane face {face:?} \
                 must adopt the authored Recessed rung {authored:?}"
            );
        }
        let inset = (4.0 * dpi).ceil() as i32;
        let sides = [
            (upper[0] + upper[2] * 0.5, upper[1], 0, -1, "top"),
            (upper[0], upper[1] + upper[3] * 0.5, -1, 0, "left"),
            (
                lower[0] + lower[2],
                lower[1] + lower[3] * 0.5,
                1,
                0,
                "right",
            ),
            (
                lower[0] + lower[2] * 0.5,
                lower[1] + lower[3],
                0,
                1,
                "bottom",
            ),
        ];
        for (x, y, dx, dy, side) in sides {
            let (x, y) = (x.round() as i32, y.round() as i32);
            let inside = avg(&px, w, x - dx * inset, y - dy * inset);
            let outside = avg(&px, w, x + dx * inset, y + dy * inset);
            let d = redmean(inside, outside);
            assert!(
                d >= 24.0,
                "{logical_w}x{logical_h}@{dpi} {side}: Pane {inside:?} vs \
                 Frame {outside:?} must separate (redmean {d:.1})"
            );
        }
    }
}

/// The data-only repair changes no Room token and no other world's Pane face.
/// This is deliberately a complete roster sweep, while the edge law above is
/// the appearance oracle for Potoroo itself.
#[test]
fn recessed_pane_face_is_potoroo_only_and_room_tokens_are_unchanged() {
    let _g = crate::testlock::serial();
    let _pin = theme::WorldPin::snapshot();
    for th in theme::THEMES.iter() {
        theme::set_active_by_name(th.name).unwrap();
        let face = theme::pane_surface(th.render_caps.elevation);
        if th.name == "Potoroo" {
            assert_eq!(th.render_caps.elevation, theme::Elevation::Recessed);
            assert_eq!(
                face, th.base_200,
                "Potoroo's Pane face is the existing recess rung"
            );
            assert_ne!(face, th.base_300, "the repair is a real value separation");
            assert_eq!(
                th.base_100.hex(),
                "#1f0400",
                "Potoroo Room ground stays byte-identical"
            );
            assert_eq!(
                th.base_content.hex(),
                "#f0e6de",
                "Potoroo Room ink stays byte-identical"
            );
        } else {
            assert_eq!(
                face, th.base_300,
                "{}: its Pane face remains the pre-item-123 focused rung",
                th.name
            );
        }
    }
}

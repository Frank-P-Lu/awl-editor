//! ITEM 94 — THE SETTINGS RANGE ROW'S RAIL: geometry, hit target, and real
//! pixels. The rail is the first drawn CONTROL in a picker row, so it gets the
//! same treatment every other drawn affordance does — one geometry owner shared
//! by the draw and the hit-test (asserted here by driving BOTH), and appearance
//! claims made by arithmetic over the rendered pixels, never by state.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::overlay::{OverlayKind, OverlayState};
use crate::render::rowlayout;

/// The probe values every rail test renders from: an off-default zoom, so a test
/// that accidentally assumed 100 % (mid-band on nothing, rail head on nothing)
/// fails loudly.
fn values(zoom: f32) -> crate::settings::SettingsValues {
    crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    }
}

/// A REAL Settings overlay at `zoom` — built exactly as `overlay::build`'s
/// Settings arm builds it (names + value cells + rail cells), so these tests
/// render the production row shape.
fn settings_state(zoom: f32) -> OverlayState {
    let vals = values(zoom);
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    // Select the Zoom row (the rail row) — the same thing arrowing onto it does.
    let zoom_row = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .expect("the Settings corpus has a Zoom row");
    ov.selected = zoom_row;
    ov
}

/// Fold a Settings overlay into a `ViewState` the way `App::sync_view` does.
fn settings_view(ov: &OverlayState) -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Settings.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v
}

/// PURE GEOMETRY (no GPU) — the rail's own arithmetic: the track spans exactly
/// its authored length ending a gap short of the value text, the thumb rides the
/// fraction, the hit band is GENEROUSLY wider than the drawn thumb in both axes,
/// and a pointer x maps back to the fraction it came from.
#[test]
fn the_rail_geometry_round_trips_and_its_hit_target_is_generous() {
    let lh = 24.0f32;
    let text_right = 900.0f32;
    let value_w = 40.0f32;
    let row_top = 200.0f32;
    for &frac in &[0.0f32, 0.2, 0.5, 0.87, 1.0] {
        let rail = rowlayout::rail_geom(text_right, value_w, 600.0, row_top, lh, frac)
            .expect("a wide row seats a rail");
        // The track ends a gap short of the value text, and never touches it.
        assert!(rail.x1 < text_right - value_w, "the track must clear the value text");
        assert!(rail.x0 < rail.x1);
        // The thumb rides the fraction along the track, inside it at both ends.
        let cx = rail.thumb[0] + rail.thumb[2] * 0.5;
        assert!(
            (cx - (rail.x0 + frac * (rail.x1 - rail.x0))).abs() < 0.01,
            "the thumb must sit at frac {frac} of the track"
        );
        // px -> fraction is the exact inverse at the thumb's own centre.
        assert!(
            (rowlayout::rail_frac_at(cx, rail.x0, rail.x1) - frac).abs() < 1e-4,
            "a press on the thumb resolves to the fraction it is drawn at"
        );
        // THE GENEROUS TARGET: the hit band is far wider than the visually small
        // thumb, spans the WHOLE row height, and contains the whole track.
        assert!(
            rail.hit[2] > rail.thumb[2] * 4.0,
            "the hit band ({}) must be much wider than the thumb ({})",
            rail.hit[2],
            rail.thumb[2]
        );
        assert!(rail.hit[3] >= rail.thumb[3], "the hit band spans the row, not the thumb");
        assert!((rail.hit[3] - lh).abs() < 0.01, "the hit band is the whole row band");
        assert!(rail.hit[0] < rail.x0 && rail.hit[0] + rail.hit[2] > rail.x1);
        // A press anywhere in the band resolves ON the track (never off the ends).
        for px in [rail.hit[0], rail.hit[0] + rail.hit[2], rail.x0 - 3.0, rail.x1 + 3.0] {
            let f = rowlayout::rail_frac_at(px, rail.x0, rail.x1);
            assert!((0.0..=1.0).contains(&f), "px {px} resolved off the rail: {f}");
        }
        // Containment agrees with the band on every corner + just outside it.
        assert!(rowlayout::rail_hit(&rail, cx, row_top + lh * 0.5));
        assert!(!rowlayout::rail_hit(&rail, rail.hit[0] - 1.0, row_top + lh * 0.5));
        assert!(!rowlayout::rail_hit(&rail, cx, row_top - 1.0));
        assert!(!rowlayout::rail_hit(&rail, cx, row_top + lh + 1.0));
    }
    // THE SECONDARY COLUMN'S YIELD RULE: a row with no room for the rail beside
    // its label gets NO rail rather than one painted over the name.
    assert!(rowlayout::rail_geom(text_right, value_w, 10.0, row_top, lh, 0.5).is_none());
    // The rail scales with the row (zoom): a taller row gets a longer track.
    let small = rowlayout::rail_geom(text_right, value_w, 600.0, row_top, 12.0, 0.5).unwrap();
    let big = rowlayout::rail_geom(text_right, value_w, 600.0, row_top, 36.0, 0.5).unwrap();
    assert!(big.track[2] > small.track[2] * 2.0, "the rail scales with the row height");
}

/// THE DRAWN RAIL AND THE CLICKABLE RAIL ARE ONE RECTANGLE — driven through the
/// REAL pipeline: the hit-test finds the Zoom row's rail exactly where the draw
/// path puts it, resolves the fraction the pointer landed on, and refuses every
/// pixel of the row's LABEL side (where a click must only select).
#[test]
fn a_rails_hit_target_is_where_it_is_drawn_and_the_label_is_not_part_of_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_rails_hit_target_is_where_it_is_drawn: no wgpu adapter");
        return;
    };
    let ov = settings_state(1.0);
    let v = settings_view(&ov);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let zoom_item = ov.selected;
    let (x0, x1) = p
        .overlay_range_scale(zoom_item)
        .expect("the Zoom row draws a rail on a 1200px card");
    assert!(x1 > x0, "the track has a positive length");

    // Find the row band by sweeping y at the track's midpoint — the ONLY y's that
    // answer are that row's own band (every other row carries no rail).
    let mid_x = (x0 + x1) * 0.5;
    let hits: Vec<f32> = (0..800)
        .map(|y| y as f32)
        .filter(|&y| p.overlay_range_at(mid_x, y).is_some())
        .collect();
    assert!(!hits.is_empty(), "the drawn rail must be hit-testable");
    let (top, bottom) = (hits[0], hits[hits.len() - 1]);
    let lh = p.overlay_lh();
    assert!(
        (bottom - top - lh).abs() <= 2.0,
        "the hit band is exactly the row band ({top}..{bottom} vs lh {lh})"
    );
    let row_y = (top + bottom) * 0.5;

    // The resolved item is the Zoom row, and the fraction tracks the pointer.
    for (px, want) in [(x0, 0.0f32), (mid_x, 0.5), (x1, 1.0)] {
        let (item, frac) = p.overlay_range_at(px, row_y).expect("on the rail");
        assert_eq!(item, zoom_item, "the rail belongs to the row it is drawn on");
        assert!((frac - want).abs() < 0.02, "px {px} -> frac {frac}, wanted {want}");
    }
    // GENEROUS: a press well past each visible end still lands on the rail (and
    // clamps to that end) — the thumb is small, the target is not.
    for (px, want) in [(x0 - 6.0, 0.0f32), (x1 + 6.0, 1.0)] {
        let (_, frac) = p.overlay_range_at(px, row_y).expect("the padded band still hits");
        assert!((frac - want).abs() < 1e-3, "a press past the end clamps to it");
    }

    // THE LABEL SIDE IS NOT THE RAIL: every pixel from the card's left edge up to
    // the hit band's own start answers `None`, so a click there can only select.
    let [cx, _cy, _cw, _ch] = p.overlay_card_rect().expect("a settings card");
    let mut x = cx + 2.0;
    while x < x0 - 20.0 {
        assert!(
            p.overlay_range_at(x, row_y).is_none(),
            "x={x} on the label side must NOT be part of the rail's target"
        );
        x += 4.0;
    }
    // …and NO other row has a rail at all (only range rows draw one).
    let mut y = 0.0f32;
    while y < 800.0 {
        if !(top..=bottom).contains(&y) {
            assert!(
                p.overlay_range_at(mid_x, y).is_none(),
                "y={y} is not the Zoom row, so it must carry no rail"
            );
        }
        y += 3.0;
    }
}

/// THE THUMB TRACKS THE VALUE — real pixels: rendering the same card at the band
/// FLOOR and at the band CEILING puts the thumb's ink at opposite ends of the
/// track (and the value cell agrees), so the drawn control genuinely reports the
/// setting rather than sitting decoratively still.
#[test]
fn the_thumb_moves_across_the_track_with_the_value_real_pixels() {
    let _g = crate::testlock::serial();
    let w = 1200u32;
    let h = 800u32;
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_thumb_moves_across_the_track_with_the_value: no wgpu adapter");
        return;
    };
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();

    // The x of the thumb's ink, measured on the rendered frame within the rail's
    // own row band — the DRAWN position, not the computed one.
    let thumb_x = |p: &mut TextPipeline, zoom: f32| -> (f32, f32, f32) {
        let ov = settings_state(zoom);
        let v = settings_view(&ov);
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let (x0, x1) = p.overlay_range_scale(ov.selected).expect("a rail");
        let mid = (x0 + x1) * 0.5;
        let ys: Vec<f32> = (0..h)
            .map(|y| y as f32)
            .filter(|&y| p.overlay_range_at(mid, y).is_some())
            .collect();
        let row_y = (ys[0] + ys[ys.len() - 1]) * 0.5;
        let pixels = pixeldiff::render_frame(p, &device, &queue, w, h);
        // Scan the track's own y for the column whose ink is FARTHEST from the
        // card ground: the thumb is the tallest, most present mark on the rail.
        let bg = theme::base_300().rgba_bytes();
        let mut best = (0.0f32, -1i64);
        let mut x = x0 - 12.0;
        while x <= x1 + 12.0 {
            let idx = ((row_y as usize) * (w as usize)) + (x as usize);
            let c = pixels[idx];
            let d: i64 = (0..3).map(|k| (c[k] as i64 - bg[k] as i64).abs()).sum();
            if d > best.1 {
                best = (x, d);
            }
            x += 1.0;
        }
        assert!(best.1 > 0, "the rail must paint SOMETHING at zoom {zoom}");
        (best.0, x0, x1)
    };

    let (low_x, x0, x1) = thumb_x(&mut p, spec.min);
    let (high_x, _, _) = thumb_x(&mut p, spec.max);
    assert!(
        low_x < x0 + (x1 - x0) * 0.25,
        "at the band FLOOR the thumb sits at the rail's head ({low_x} in {x0}..{x1})"
    );
    assert!(
        high_x > x0 + (x1 - x0) * 0.75,
        "at the band CEILING the thumb sits at the rail's tail ({high_x} in {x0}..{x1})"
    );
    assert!(high_x > low_x + (x1 - x0) * 0.5, "the thumb genuinely travelled");
}

/// LEGIBILITY IN BOTH LIGHT AND DARK — real pixels: on a light world and a dark
/// world, both selected and unselected, the rail's TRACK and its THUMB each paint
/// ink genuinely distinct from what they sit on, and the thumb leads the track
/// (figure over furniture). Never amber: the rail's ink is the theme's own muted
/// ladder, so this also witnesses the one-accent law on the new surface.
#[test]
fn the_rail_reads_against_its_ground_in_light_and_dark_worlds_real_pixels() {
    let _g = crate::testlock::serial();
    let w = 1200u32;
    let h = 800u32;
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_rail_reads_against_its_ground_in_light_and_dark: no wgpu adapter");
        return;
    };
    let saved = theme::active().name.to_string();
    // One LIGHT world and one DARK world, both `Pane` (a card behind the rows) so
    // the rail's ground is the card/band rather than the live page.
    for world in ["Bilby", "Bombora"] {
        if theme::set_active_by_name(world).is_none() {
            continue;
        }
        p.sync_theme();
        for &selected in &[true, false] {
            let mut ov = settings_state(1.4);
            let zoom_item = ov.selected;
            if !selected {
                // Move the highlight OFF the rail row (row 0 is never Zoom).
                ov.selected = 0;
            }
            let v = settings_view(&ov);
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let (x0, x1) = p.overlay_range_scale(zoom_item).expect("a rail");
            let mid = (x0 + x1) * 0.5;
            let ys: Vec<f32> = (0..h)
                .map(|y| y as f32)
                .filter(|&y| p.overlay_range_at(mid, y).is_some())
                .collect();
            assert!(!ys.is_empty(), "{world}: the rail must be present");
            let row_y = ((ys[0] + ys[ys.len() - 1]) * 0.5) as i64;
            let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let at = |x: i64, y: i64| -> [u8; 4] { pixels[(y as usize) * (w as usize) + x as usize] };
            // GROUND: a pixel on the same row, on the rail's own y, just OUTSIDE
            // the track (past its padded end) — the card or the selected band,
            // whichever this row sits on. Sampled, never assumed.
            let ground = at((x1 + 14.0) as i64, row_y);
            let dist = |c: [u8; 4]| -> i64 {
                (0..3).map(|k| (c[k] as i64 - ground[k] as i64).abs()).sum()
            };
            // The strongest ink anywhere along the track's y IS the thumb; the
            // weakest non-ground ink is the track.
            let mut thumb = (0i64, [0u8; 4]);
            let mut track = (i64::MAX, [0u8; 4]);
            let mut x = x0 as i64;
            while x <= x1 as i64 {
                let c = at(x, row_y);
                let d = dist(c);
                if d > thumb.0 {
                    thumb = (d, c);
                }
                if d > 0 && d < track.0 {
                    track = (d, c);
                }
                x += 1;
            }
            let ctx = format!("{world} (selected={selected})");
            assert!(
                thumb.0 >= 24,
                "{ctx}: the THUMB must read against its ground (distance {} of ink {:?} vs ground {:?})",
                thumb.0,
                thumb.1,
                ground
            );
            assert!(
                track.0 != i64::MAX && track.0 >= 4,
                "{ctx}: the TRACK must paint something distinct from its ground"
            );
            assert!(
                thumb.0 > track.0,
                "{ctx}: the thumb must lead the track by value (thumb {} vs track {})",
                thumb.0,
                track.0
            );
            // THE ONE-ACCENT LAW: nothing on the rail may paint the caret's amber.
            let accent = theme::primary().rgba_bytes();
            let near_accent = |c: [u8; 4]| -> bool {
                (0..3).map(|k| (c[k] as i64 - accent[k] as i64).abs()).sum::<i64>() < 24
            };
            assert!(!near_accent(thumb.1), "{ctx}: the thumb must not be the accent");
            assert!(!near_accent(track.1), "{ctx}: the track must not be the accent");
        }
    }
    theme::set_active_by_name(&saved);
    p.sync_theme();
}

/// NO RAIL ANYWHERE ELSE: a card with no range rows (the command palette's shape)
/// hit-tests as railless over its whole area — the rail machinery is inert for
/// every other picker, so those cards render exactly as they always did.
#[test]
fn a_card_with_no_range_rows_carries_no_rail_at_all() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_card_with_no_range_rows_carries_no_rail_at_all: no wgpu adapter");
        return;
    };
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = vec!["Save".into(), "Go to file…".into(), "Switch theme…".into()];
    v.overlay_bindings = vec!["\u{2318}S".into(), "\u{2318}O".into(), String::new()];
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let mut y = 0.0f32;
    while y < 800.0 {
        let mut x = 0.0f32;
        while x < 1200.0 {
            assert!(p.overlay_range_at(x, y).is_none(), "a plain picker has no rail at ({x},{y})");
            x += 17.0;
        }
        y += 11.0;
    }
}

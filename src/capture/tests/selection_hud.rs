//! End-to-end laws for the held HUD's raw-buffer Selection group.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

fn read_sidecar(png: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap()).unwrap()
}

fn capture_selection(
    dir: &std::path::Path,
    name: &str,
    buffer: &Buffer,
    selection: Option<((usize, usize), (usize, usize))>,
    preview_text: Option<String>,
) -> (Vec<u8>, serde_json::Value) {
    let png = dir.join(format!("{name}.png"));
    capture_with(
        &png,
        buffer,
        &CaptureOpts {
            selection,
            preview_text,
            ..CaptureOpts::default()
        },
    )
    .expect("selection HUD capture");
    (std::fs::read(&png).unwrap(), read_sidecar(&png))
}

/// The sidecar is the end-to-end state oracle: the selection travels through
/// CaptureOpts -> ViewState -> pipeline -> HudReport -> JSON. These cases sweep
/// the axes most likely to tempt a renderer-derived answer: direction,
/// graphemes, concealed source, a fold-filtered page and a History transcript.
#[test]
fn hud_selection_sidecar_reports_raw_buffer_text_across_every_substitute_axis() {
    if !adapter_available() {
        eprintln!("skipping selection HUD sidecar sweep: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_selection_hud_{}", std::process::id())),
    );
    crate::hud::set_held(true);

    let unicode =
        Buffer::from_str("head\ncafe\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦 tail\nlast\n");
    let (_, reversed) = capture_selection(
        &dir,
        "unicode-reversed",
        &unicode,
        Some(((2, 0), (1, 0))),
        None,
    );
    assert_eq!(
        reversed["hud"]["selection"],
        serde_json::json!({ "words": 3, "characters": 12 }),
        "multiline Unicode and reversed direction resolve to the same raw region"
    );

    let concealed = Buffer::from_str("**bold** and [link](url)\n");
    let (_, concealed_json) = capture_selection(
        &dir,
        "concealed-markdown",
        &concealed,
        Some(((0, 0), (0, 24))),
        None,
    );
    assert_eq!(
        concealed_json["hud"]["selection"],
        serde_json::json!({ "words": 3, "characters": 24 }),
        "concealed punctuation remains selected buffer text"
    );

    let folded_text =
        "# A\nvisible\nhidden α\nhidden 👨\u{200d}👩\u{200d}👧\u{200d}👦\n# B\ntail\n";
    let mut folded = Buffer::from_str(folded_text);
    assert_eq!(folded.toggle_fold_at_cursor(), Some(0));
    let emoji_end = "hidden 👨\u{200d}👩\u{200d}👧\u{200d}👦".chars().count();
    let (_, folded_json) = capture_selection(
        &dir,
        "folded-source",
        &folded,
        Some(((2, 0), (3, emoji_end))),
        None,
    );
    assert_eq!(
        folded_json["hud"]["selection"],
        serde_json::json!({ "words": 4, "characters": 17 }),
        "fold-filtered lines are still raw selected document text"
    );
    assert!(
        !folded_json["text"].as_str().unwrap().contains("hidden"),
        "non-vacuity: the shaped page really omitted the selected hidden lines"
    );

    let history =
        Buffer::from_str("raw one\nselected e\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦\nraw end\n");
    let selection = Some((
        (1, 0),
        (
            1,
            "selected e\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦"
                .chars()
                .count(),
        ),
    ));
    let transcript = "# diff\n\n~~other~~ ==text==\n";
    assert_eq!(
        crate::card::figures::SelectionFigures::of(transcript, selection),
        None,
        "non-vacuity: the same coordinates select nothing in the substitute"
    );
    let (_, history_json) = capture_selection(
        &dir,
        "history-substitute",
        &history,
        selection,
        Some(transcript.to_string()),
    );
    assert_eq!(
        history_json["hud"]["selection"],
        serde_json::json!({ "words": 3, "characters": 12 }),
        "History shapes the transcript but counts the raw document selection"
    );
    assert_eq!(history_json["text"], serde_json::json!(transcript));

    crate::hud::set_held(false);
}

/// A caret-only `Some` is semantically no selection. The old ordinary HUD and
/// the explicit caret-only route must produce identical card pixels, while the
/// sidecar reports null through both inputs.
#[test]
fn ordinary_no_selection_and_caret_only_hud_are_png_byte_identical() {
    if !adapter_available() {
        eprintln!("skipping selection HUD identity law: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_selection_hud_identity_{}", std::process::id())),
    );
    crate::hud::set_held(true);
    let buffer = Buffer::from_str("# Ordinary\n\nno selection here\n");
    let (plain_png, plain) = capture_selection(&dir, "plain", &buffer, None, None);
    let (caret_png, caret) =
        capture_selection(&dir, "caret", &buffer, Some(((2, 3), (2, 3))), None);
    assert_eq!(plain_png, caret_png, "caret-only adds no HUD/card pixels");
    assert_eq!(plain["hud"]["selection"], serde_json::Value::Null);
    assert_eq!(caret["hud"]["selection"], serde_json::Value::Null);
    crate::hud::set_held(false);
}

fn grade_card_pixels(img: &image::RgbaImage, dpi: f32, world: &str) {
    let [r, g, b, _] = crate::theme::base_300().rgba_bytes();
    let dist = |p: &image::Rgba<u8>| {
        (p[0] as i32 - r as i32).abs()
            + (p[1] as i32 - g as i32).abs()
            + (p[2] as i32 - b as i32).abs()
    };
    let (w, h) = img.dimensions();
    let (x0, x1) = (w / 5, w * 4 / 5);
    let (y0, y1) = (h / 20, h * 19 / 20);
    let mut card = 0usize;
    let mut ink = 0usize;
    for y in y0..y1 {
        for x in x0..x1 {
            let d = dist(img.get_pixel(x, y));
            card += usize::from(d <= 18);
            ink += usize::from(d >= 70);
        }
    }
    let scale = (dpi * dpi) as usize;
    assert!(
        card >= 4_000 * scale,
        "{world} @ {dpi}x: HUD card fill presence only {card} pixels"
    );
    assert!(
        ink >= 350 * scale,
        "{world} @ {dpi}x: HUD readable ink presence only {ink} pixels"
    );
}

/// Every authored world at both capture densities. The sidecar proves which
/// Selection figures belong to the card; PNG arithmetic separately proves a
/// real central card surface and readable contrasting ink survived rendering.
#[test]
fn selected_hud_is_present_and_legible_in_every_world_at_1x_and_2x() {
    if !adapter_available() {
        eprintln!("skipping selection HUD world/DPI audit: no wgpu adapter");
        return;
    }
    let _guard = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_selection_hud_worlds_{}", std::process::id())),
    );
    let buffer = Buffer::from_str(
        "# Selection audit\n\nselected cafe\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦\nsecond line\n",
    );
    crate::hud::set_held(true);
    for world in crate::theme::THEMES.iter() {
        crate::theme::set_active_by_name(world.name).unwrap();
        for dpi in [1.0, 2.0] {
            let png = dir.join(format!("{}-{dpi}x.png", world.name));
            capture_with(
                &png,
                &buffer,
                &CaptureOpts {
                    selection: Some(((2, 0), (3, 11))),
                    dpi: Some(dpi),
                    canvas: Some(((1200.0 * dpi) as u32, (800.0 * dpi) as u32)),
                    ..CaptureOpts::default()
                },
            )
            .expect("world/DPI selection HUD capture");
            let json = read_sidecar(&png);
            assert_eq!(
                json["hud"]["selection"],
                serde_json::json!({ "words": 5, "characters": 27 }),
                "{} @ {dpi}x: exact Selection figures",
                world.name
            );
            grade_card_pixels(
                &image::open(&png).expect("decode HUD PNG").to_rgba8(),
                dpi,
                world.name,
            );
        }
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::hud::set_held(false);
}

/// Regeneration tool for the five affordance-locating vision-smoke shots:
/// which figures belong to Selection, and can every caption/value be read?
#[test]
#[ignore = "regeneration tool: writes gallery/selection-hud/*.png"]
fn gallery_selection_hud_vision_smoke() {
    if !adapter_available() {
        eprintln!("no wgpu adapter: no selection HUD gallery");
        return;
    }
    let _guard = crate::testlock::serial();
    let dir = std::path::Path::new("gallery/selection-hud");
    std::fs::create_dir_all(dir).expect("selection HUD gallery directory");
    let buffer = Buffer::from_str(
        "# Selection audit\n\nselected cafe\u{301} 👨\u{200d}👩\u{200d}👧\u{200d}👦\nsecond line\n",
    );
    crate::hud::set_held(true);
    for (world, dpi) in [
        ("Saltpan", 1.0),
        ("Currawong", 2.0),
        ("Firetail", 1.0),
        ("Wagtail", 2.0),
        ("Mulga", 1.0),
    ] {
        crate::theme::set_active_by_name(world).unwrap();
        capture_with(
            &dir.join(format!("{world}-{dpi}x.png")),
            &buffer,
            &CaptureOpts {
                selection: Some(((2, 0), (3, 11))),
                dpi: Some(dpi),
                canvas: Some(((1200.0 * dpi) as u32, (800.0 * dpi) as u32)),
                ..CaptureOpts::default()
            },
        )
        .expect("selection HUD gallery capture");
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::hud::set_held(false);
}

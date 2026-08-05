use super::super::*;
use crate::testscratch::ScratchDir;

/// USER-BUG LAW: changing the page measure must only reveal/occlude ONE
/// already-authored lava backdrop. The pixels that remain exposed at both
/// widths are therefore byte-identical, while pixels well inside the page
/// stay one flat color. This renders through the real shader/uniform path;
/// the pure `lava::tests` sibling alone cannot catch a bad upload or mask.
#[test]
fn lava_backdrop_pixels_are_page_width_invariant_and_page_interior_is_flat() {
    let _g = crate::testlock::serial();
    let old_theme = crate::theme::active_index();
    let old_measure = crate::page::measure();
    let old_page = crate::page::page_on();
    crate::theme::set_active_by_name("Mangrove").unwrap();
    crate::page::set_page_on(true);

    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-lava-width-law-{}", std::process::id())),
    );
    let render = |measure, stem: &str| {
        crate::page::set_measure(measure);
        let out = dir.join(format!("{stem}.png"));
        let opts = CaptureOpts {
            canvas: Some((1200, 800)),
            ..CaptureOpts::default()
        };
        capture_screenshot(
            out.clone(),
            None,
            opts,
            Vec::new(),
            crate::keymap::KeymapState::new(),
            Some(dir.to_path_buf()),
            Some(dir.to_path_buf()),
            dir.to_path_buf(),
            Config::empty(),
            false, // permissive (the legacy default)
        )
        .expect("lava width-law capture succeeds");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap())
                .unwrap();
        let left = json["page"]["column"]["left"].as_f64().unwrap() as u32;
        let width = json["page"]["column"]["width"].as_f64().unwrap() as u32;
        (image::open(out).unwrap().to_rgba8(), left, left + width)
    };
    let (narrow, narrow_l, narrow_r) = render(40, "narrow");
    let (wide, wide_l, wide_r) = render(70, "wide");

    let left_full = narrow_l
        .min(wide_l)
        .saturating_sub(crate::lava::MARGIN_GAP_PX as u32);
    let right_full = (narrow_r.max(wide_r) + crate::lava::MARGIN_GAP_PX as u32).min(1200);
    let mut compared = 0usize;
    for y in 80..720 {
        for x in (0..left_full).chain(right_full..1200) {
            assert_eq!(
                narrow.get_pixel(x, y),
                wide.get_pixel(x, y),
                "common exposed backdrop changed at ({x},{y})"
            );
            compared += 1;
        }
    }
    assert!(
        compared > 50_000,
        "width law sampled a substantial common margin"
    );

    let x0 = narrow_l.max(wide_l) + 64;
    let x1 = narrow_r.min(wide_r).saturating_sub(64);
    for (label, frame) in [("narrow", &narrow), ("wide", &wide)] {
        let flat = *frame.get_pixel(600, 650);
        for y in 600..720 {
            for x in x0..x1 {
                assert_eq!(
                    *frame.get_pixel(x, y),
                    flat,
                    "{label} page is not flat at ({x},{y})"
                );
            }
        }
    }

    crate::theme::set_active(old_theme);
    crate::page::set_measure(old_measure);
    crate::page::set_page_on(old_page);
}

#[test]
fn workspace_defaults_to_root_parent_when_unset() {
    let root = PathBuf::from("/home/me/work/repos/some-repo");
    assert_eq!(
        resolve_workspace(&None, &root),
        PathBuf::from("/home/me/work/repos")
    );
}

#[test]
fn explicit_workspace_overrides_the_default() {
    let root = PathBuf::from("/home/me/work/repos/some-repo");
    let ws = PathBuf::from("/elsewhere/projects");
    assert_eq!(resolve_workspace(&Some(ws.clone()), &root), ws);
}

#[test]
fn workspace_falls_back_to_root_when_no_parent() {
    let root = PathBuf::from("/");
    assert_eq!(resolve_workspace(&None, &root), root);
}

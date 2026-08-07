//! `menubar`'s own unit tests — the pure bar-height/DPI arithmetic, the sliver-fix
//! bleed math, the global toggle/dropdown state, and the title/drop hit-testing.
//! Carved out of `menubar.rs` as a sibling module so the production file stays
//! inside its size mark — `code-health.py`'s `production()` exempts a file named
//! `tests.rs` precisely so carving an inline `mod tests` out of an oversized module
//! is a real remedy (the same move `readout.rs` / `readout/tests.rs` already use).

use super::*;

/// **THE BAR'S HEIGHT HAS ONE OWNER, AND THIS IS WHAT REFUSES A SECOND ONE.**
/// `render/geometry.rs`'s `TextPipeline::menubar_reserve` is the door; [`bar_height`]
/// is the arithmetic behind it. The reserve and the DRAWN strip used to spell
/// `bar_height(metrics.line_height * LABEL, metrics.scale)` independently and agreed
/// only because both authors remembered to — the same shape `TEXT_TOP +
/// menubar_reserve()` had at six call sites, where a real bug survived at half of
/// them; a probe that tripled the reserve drove a wedge straight between these two.
///
/// So the CONSTRUCTION is that `chrome/menubar.rs` draws at `self.menubar_reserve()`,
/// and this is the sweep that keeps it that way. **NO WILDCARD:** every non-test
/// caller of `bar_height(` outside this module must appear in
/// [`BAR_HEIGHT_CALLERS`] with a reason, so a new consumer fails here by name
/// rather than becoming a second owner in silence. Non-vacuous in both directions —
/// a stale entry that no longer calls it fails too, because an allow-list nobody
/// prunes is how a closed bypass reads as still-open.
const BAR_HEIGHT_CALLERS: &[(&str, &str)] = &[(
    "src/render/geometry.rs",
    "TextPipeline::menubar_reserve — THE door. It owns the `menu_bar_on()` gate and \
     the LABEL-scaled line height, and every consumer (the document inset, the \
     caret, the hit-test, every card's height budget, the capture sidecar, and the \
     DRAWN strip in chrome/menubar.rs) reads its answer rather than this fn",
)];

#[test]
fn bar_height_has_exactly_one_non_test_caller() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut hits: Vec<(String, usize)> = Vec::new();
    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("src dir readable")
            .flatten()
            .collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // Test code is not a consumer, and `menubar.rs` is the DEFINITION plus
            // the doc comments that name it. Same exemptions `code-health.py`'s
            // `production()` applies, for the same reason.
            if !name.ends_with(".rs")
                || name == "tests.rs"
                || name.ends_with("_test.rs")
                || path.components().any(|c| c.as_os_str() == "tests")
                || path == root.join("src/menubar.rs")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("source readable");
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            for (i, line) in text.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with("//") || !line.contains("bar_height(") {
                    continue;
                }
                hits.push((rel.clone(), i + 1));
            }
        }
    }
    // The sweep must be reading something at all — an empty walk would satisfy the
    // no-stray assertion below while proving nothing.
    assert!(
        !hits.is_empty(),
        "the bar_height caller sweep found no calls anywhere under src/ — it is not \
         reading the sources it thinks it is"
    );
    let stray: Vec<_> = hits
        .iter()
        .filter(|(f, _)| !BAR_HEIGHT_CALLERS.iter().any(|(name, _)| name == f))
        .map(|(f, l)| format!("  {f}:{l}"))
        .collect();
    assert!(
        stray.is_empty(),
        "the menu bar's height has ONE owner — `TextPipeline::menubar_reserve` — and \
         these call `menubar::bar_height` directly, which is how the reserve and the \
         drawn strip came to be two independent spellings of one number. Read \
         `self.menubar_reserve()` instead, or add the file to BAR_HEIGHT_CALLERS with \
         the reason it must own a second copy:\n{}",
        stray.join("\n")
    );
    for (file, reason) in BAR_HEIGHT_CALLERS {
        assert!(
            hits.iter().any(|(f, _)| f == file),
            "{file} is allow-listed ({reason}) but no longer calls bar_height — \
             remove the entry rather than leaving a closed bypass looking open"
        );
    }
}

/// THE FORCING KNOB'S PARSER, swept over every input shape rather than the one
/// value this process happens to have launched with. `on`/`off` force; a typo, a
/// capitalisation, an empty string and an unset variable must all be INERT —
/// a knob that silently forced on `ON` would make the gate's two arms depend on
/// how the caller spelled them.
#[test]
fn classify_force_reads_on_and_off_and_leaves_everything_else_alone() {
    assert_eq!(classify_force(Some("on")), Some(true));
    assert_eq!(classify_force(Some("off")), Some(false));
    for inert in ["", "ON", "Off", "1", "0", "true", "false", "yes", " on"] {
        assert_eq!(
            classify_force(Some(inert)),
            None,
            "{inert:?} must leave the platform default alone"
        );
    }
    assert_eq!(classify_force(None), None, "unset is a total no-op");
}

/// THE KNOB REALLY OWNS THE DEFAULT — non-vacuous exactly in the gate arms that
/// set it. Under `AWL_MENU_BAR_FORCE=on|off`, `menu_bar_default()` must be the
/// FORCED value: a `menu_bar_default` that read `cfg!(target_os = …)` and
/// ignored the knob passes unforced on every host and fails here in both arms,
/// which is the whole point of running them.
///
/// The unforced arm deliberately claims less than an equality against
/// `platform_default(cfg!(…))` would: asserting a `cfg!`-derived value against the
/// identical `cfg!` is the tautology `Config::menu_bar_on` used to be. It asserts
/// the answer is one of the two NAMED consts, which a third default would break.
#[test]
fn menu_bar_default_honours_the_forcing_and_otherwise_reads_a_named_const() {
    let forced = std::env::var("AWL_MENU_BAR_FORCE").ok();
    match classify_force(forced.as_deref()) {
        Some(want) => assert_eq!(
            menu_bar_default(),
            want,
            "AWL_MENU_BAR_FORCE={forced:?} must own the default"
        ),
        None => assert!(
            menu_bar_default() == MENU_BAR_DEFAULT_MACOS
                || menu_bar_default() == MENU_BAR_DEFAULT_OTHER,
            "unforced, the default must be one of the two named platform consts"
        ),
    }
}

/// `bar_height` IS DPI-INVARIANT AT MATCHED LOGICAL GEOMETRY, WITH A PRESENCE
/// FLOOR. This is the PURE half — the row-count-style arithmetic, swept without a
/// device. The live-pipeline half (whether `TextPipeline::menubar_reserve` / the
/// card's height budget move together with this fn) needs a `TextPipeline` from
/// `src/render/tests/mod.rs::headless_pipeline`, which this module does not reach.
///
/// `line_height` here stands in for the caller's already-scaled
/// `Metrics::line_height`; sweeping `scale` while holding the LOGICAL line height
/// fixed and dividing the pad term back out proves `BAR_PAD_Y` moves WITH it
/// rather than getting added in raw. BOTH SIDES: (1) the device-px answer must
/// actually MOVE as scale changes — ruling out a scale-blind fn that would
/// trivially look invariant — and (2) the recovered LOGICAL answer must be
/// identical at every tier AND equal to the authored `line_height +
/// 2*BAR_PAD_Y.0` — a presence floor, since `Logical(0.0)` is perfectly
/// DPI-invariant too and would pass side (2) alone.
#[test]
#[allow(clippy::assertions_on_constants)] // the constant IS the subject under test
fn bar_height_is_dpi_invariant_at_matched_logical_geometry_with_a_presence_floor() {
    assert!(
        BAR_PAD_Y.0 > 0.0,
        "presence floor: BAR_PAD_Y must not be zeroed"
    );
    const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];
    let logical_line_height = 20.0f32;
    let baseline = bar_height(logical_line_height, 1.0);

    let mut logical_answers = Vec::with_capacity(TIERS.len());
    for &scale in &TIERS {
        let physical_line_height = logical_line_height * scale;
        let h = bar_height(physical_line_height, scale);
        // SIDE ONE: the fn is not scale-blind — its device-px answer actually
        // moves once scale departs from 1.0.
        if scale != 1.0 {
            assert_ne!(
                h, baseline,
                "scale {scale}: bar_height must not be scale-blind"
            );
        }
        logical_answers.push(h / scale);
    }
    // SIDE TWO: every tier recovers the SAME logical answer, and it is the
    // authored constant — not just "some" invariant value (the trap a deleted
    // pad would also satisfy).
    let want_logical = logical_line_height + 2.0 * BAR_PAD_Y.0;
    for (&scale, &got) in TIERS.iter().zip(logical_answers.iter()) {
        assert!(
            (got - want_logical).abs() < 1e-4,
            "scale {scale}: recovered logical bar height {got} != authored \
             line_height + 2*BAR_PAD_Y ({want_logical}) — BAR_PAD_Y is not \
             scaling with line_height"
        );
    }
}

/// THE SLIVER FIX, pure: a rect flush on all three canvas-touching sides (the
/// bar's own ground strip) bleeds top/left/right by `EDGE_BLEED_PX.0`, and its
/// BOTTOM (never a canvas edge for the bar) is untouched.
#[test]
fn bleed_extends_every_flush_edge_and_leaves_the_bottom_alone() {
    let rect = [0.0, 0.0, 1200.0, 32.0];
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], -EDGE_BLEED_PX.0, "left bleeds past x=0");
    assert_eq!(bled[1], -EDGE_BLEED_PX.0, "top bleeds past y=0");
    assert_eq!(
        bled[2],
        1200.0 + 2.0 * EDGE_BLEED_PX.0,
        "width bleeds on both flush sides"
    );
    assert_eq!(
        bled[3],
        32.0 + EDGE_BLEED_PX.0,
        "height bleeds on the top side only"
    );
    // The bottom edge (y + h) moves by exactly the top bleed, i.e. the BOTTOM
    // itself (a non-flush edge) never moved: bled_y + bled_h == rect_y + rect_h.
    assert_eq!(
        bled[1] + bled[3],
        rect[1] + rect[3],
        "the bottom edge itself is unmoved"
    );
}

#[test]
fn bleed_leaves_interior_left_and_right_edges_untouched() {
    let rect = [400.0, 0.0, 80.0, 32.0]; // nowhere near x=0 or x=1200
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], 400.0, "left edge is interior, untouched");
    assert_eq!(bled[2], 80.0, "width is untouched (no side bled)");
    assert_eq!(
        bled[1], -EDGE_BLEED_PX.0,
        "top still bleeds — it's always flush for the bar"
    );
    assert_eq!(bled[3], 32.0 + EDGE_BLEED_PX.0);
}

#[test]
fn bleed_is_independent_per_side() {
    let rect = [1100.0, 0.0, 100.0, 32.0]; // right edge exactly at canvas_w=1200
    let bled = bleed_to_canvas_edges(rect, 1200.0);
    assert_eq!(bled[0], 1100.0, "left edge is interior, untouched");
    assert_eq!(
        bled[2],
        100.0 + EDGE_BLEED_PX.0,
        "right bleeds (flush to canvas_w)"
    );
    assert_eq!(bled[1], -EDGE_BLEED_PX.0);
}

/// A rect NOT touching the canvas top at all (hypothetical future caller) is
/// left exactly alone on every side — the fix only ever touches an edge that is
/// ACTUALLY flush with the canvas boundary, never a rect drawn purely elsewhere.
#[test]
fn bleed_is_a_total_no_op_off_every_canvas_edge() {
    let rect = [200.0, 50.0, 300.0, 40.0];
    assert_eq!(bleed_to_canvas_edges(rect, 1200.0), rect);
}

#[test]
fn globals_toggle_and_open_close() {
    let _g = crate::testlock::serial();
    let ambient = menu_bar_on(); // not `cfg!`: that reflects the host, not the initializer
    set_menu_bar_on(true);
    assert!(menu_bar_on());
    assert_eq!(toggle_open(2), Some(2));
    assert_eq!(open_menu(), Some(2));
    assert_eq!(toggle_open(2), None);
    assert_eq!(open_menu(), None);
    set_open(Some(1));
    assert_eq!(toggle_open(3), Some(3));
    set_open(Some(0));
    set_menu_bar_on(false);
    assert!(!menu_bar_on());
    assert_eq!(open_menu(), None, "a hidden bar holds no open dropdown");
    set_open(Some(0));
    assert!(toggle(), "toggle from off -> on");
    set_open(Some(0));
    assert!(!toggle(), "toggle from on -> off closes the dropdown");
    assert_eq!(open_menu(), None);
    set_menu_bar_on(ambient);
}

#[test]
fn boxes_from_extents_abut_at_midpoints() {
    let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)], 1.0);
    assert_eq!(boxes.len(), 3);
    assert_eq!(boxes[0].band_left, 20.0 - TITLE_PAD_X.px(1.0));
    assert_eq!(boxes[0].text_left, 20.0);
    assert_eq!(boxes[0].text_right, 50.0);
    assert_eq!(boxes[0].band_right, (50.0 + 70.0) / 2.0);
    assert_eq!(boxes[1].band_left, boxes[0].band_right, "bands abut");
    assert_eq!(boxes[1].band_right, (96.0 + 110.0) / 2.0);
    assert_eq!(boxes[2].band_left, boxes[1].band_right);
    assert_eq!(boxes[2].band_right, 146.0 + TITLE_PAD_X.px(1.0));
}

#[test]
fn title_at_maps_x_across_the_whole_bar() {
    let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)], 1.0);
    let bar_h = bar_height(20.0, 1.0);
    assert_eq!(
        title_at(&boxes, bar_h, boxes[0].text_left + 1.0, 4.0),
        Some(0)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[1].text_left + 1.0, 4.0),
        Some(1)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[2].band_right - 1.0, 4.0),
        Some(2)
    );
    assert_eq!(
        title_at(&boxes, bar_h, boxes[0].text_left, bar_h + 1.0),
        None
    );
    assert_eq!(title_at(&boxes, bar_h, 0.0, 4.0), None);
    assert_eq!(
        title_at(&boxes, bar_h, boxes[2].band_right + 5.0, 4.0),
        None
    );
}

#[test]
fn drop_rows_stack_uniform_slots_marking_separators() {
    let (rows, total) = drop_rows(&[false, false, true, false], 22.0);
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].top, 0.0);
    assert_eq!(rows[1].top, 22.0);
    assert_eq!(rows[2].top, 44.0);
    assert!(rows[2].separator, "the third row is the separator");
    assert_eq!(rows[3].top, 66.0);
    assert_eq!(total, 4.0 * 22.0);
}

/// SWEPT ACROSS THE DPI TIERS, not asserted at 1x alone. The hit-test resolves the
/// card pad itself, so a pad that scaled on the DRAWN side and not here (or the
/// reverse) puts every clickable row a few device px off its ink — the class of bug
/// that is invisible on the dev host, where the bar does not even draw by default.
/// Row 0's ink begins at [`drop_inner_origin`], the one owner both sides read.
#[test]
fn drop_item_at_hits_clickable_rows_only_at_every_dpi_tier() {
    for scale in [1.0f32, 1.5, 2.0, 3.0] {
        let anchor = TitleBox {
            band_left: 40.0,
            text_left: 52.0,
            text_right: 84.0,
            band_right: 90.0,
        };
        let bar_h = bar_height(20.0 * scale, scale);
        let row_h = 22.0 * scale;
        let (rows, total) = drop_rows(&[false, true, false], row_h);
        let rect = drop_rect(&anchor, bar_h, 120.0 * scale, total, scale);
        assert_eq!(rect[0], 40.0, "the dropdown left-aligns under its title");
        assert_eq!(rect[1], bar_h, "it hangs just below the bar");
        assert_eq!(rect[2], 120.0 * scale + 2.0 * DROP_PAD_X.px(scale));
        let (inner_x, inner_y) = drop_inner_origin(rect, scale);
        assert_eq!(inner_x, rect[0] + DROP_PAD_X.px(scale));
        let (x, y) = (rect[0] + 5.0, inner_y + 1.0);
        assert_eq!(drop_item_at(rect, &rows, x, y, scale), Some(0), "s={scale}");
        // The separator row (index 1) is never a hit.
        let sep_y = inner_y + rows[1].top + 1.0;
        assert_eq!(
            drop_item_at(rect, &rows, x, sep_y, scale),
            None,
            "s={scale}"
        );
        let third_y = inner_y + rows[2].top + 1.0;
        assert_eq!(
            drop_item_at(rect, &rows, x, third_y, scale),
            Some(2),
            "s={scale}"
        );
        assert_eq!(drop_item_at(rect, &rows, rect[0] - 1.0, y, scale), None);
        assert_eq!(
            drop_item_at(rect, &rows, x, rect[1] + 1.0, scale),
            None,
            "s={scale}: the pad's own band is above row 0, never a hit"
        );
    }
}

/// **THE FOUR LOGICAL PADS HOLD THEIR LOGICAL SIZE AT EVERY DPI TIER, WITH A
/// PRESENCE FLOOR EACH.** Modelled on `bar_height`'s own law, and for the same
/// reason: invariance ALONE is satisfiable by deleting the pad, since `0 x dpi` is
/// perfectly invariant. So every pad is graded on three sides — it is present
/// (> 0), the device answer MOVES with scale (ruling out a scale-blind reader), and
/// the recovered logical answer is the AUTHORED number at every tier.
///
/// Each pad is measured through the fn that actually resolves it, never through
/// `.px` directly: a law that multiplied the constant itself would pass over a
/// caller that forgot to.
/// One pad's row in the sweep: its NAME (so a failure says which), its AUTHORED
/// logical value, and the fn that OBSERVES its device value at a given scale
/// through the resolver a real caller goes through.
type PadProbe = (&'static str, f32, fn(f32) -> f32);

#[test]
fn the_logical_pads_hold_their_logical_size_at_every_dpi_tier() {
    const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];
    let probes: [PadProbe; 4] = [
        // The bar's left inset, as the drawn origin resolves it.
        ("BAR_INSET_X", BAR_INSET_X.0, |s| BAR_INSET_X.px(s)),
        // The LAST title's outer band, measured as the overhang past its own ink.
        ("TITLE_PAD_X", TITLE_PAD_X.0, |s| {
            let ink = (100.0 * s, 140.0 * s);
            boxes_from_extents(&[ink], s)[0].band_right - ink.1
        }),
        // The card's own horizontal padding, from the rect it sizes.
        ("DROP_PAD_X", DROP_PAD_X.0, |s| {
            let anchor = TitleBox {
                band_left: 0.0,
                text_left: 0.0,
                text_right: 0.0,
                band_right: 0.0,
            };
            (drop_rect(&anchor, 0.0, 120.0 * s, 0.0, s)[2] - 120.0 * s) * 0.5
        }),
        // The card's vertical padding, from where row 0's ink actually begins.
        ("DROP_PAD_Y", DROP_PAD_Y.0, |s| {
            drop_inner_origin([0.0, 0.0, 0.0, 0.0], s).1
        }),
    ];
    for (name, authored, observe) in probes {
        assert!(
            authored > 0.0,
            "presence floor: {name} must not be zeroed — invariance alone is \
             satisfied by a deleted pad"
        );
        let baseline = observe(1.0);
        for scale in TIERS {
            let device = observe(scale);
            if scale != 1.0 {
                assert_ne!(
                    device, baseline,
                    "{name} at scale {scale}: its reader is scale-BLIND — the \
                     device answer never moved off its 1x value"
                );
            }
            let recovered = device / scale;
            assert!(
                (recovered - authored).abs() < 1e-4,
                "{name} at scale {scale}: recovered logical {recovered} != \
                 authored {authored} — the pad is not scaling with the ink it \
                 is added to"
            );
        }
    }
}

/// **THE ANNOTATED PHYSICAL EXCEPTION EARNS ITS DECLARATION.** `EDGE_BLEED_PX` is
/// `Physical` because the two things it must push off-canvas are fixed in DEVICE
/// pixels: `shaders/selection.wgsl`'s `smoothstep(-1.0, 1.0, d)` feathers ~1 px each
/// side of the rounded-rect edge in framebuffer space, and the ordinary fill
/// pipeline's corner radius is uploaded once at construction and never multiplied by
/// `scale`. A prose claim is not a law, so this reads the radius out of
/// `src/selection.rs` and requires the bleed to still cover it — the classification
/// stops being right the moment a wider corner outgrows the bleed, and this is the
/// arm that says so instead of a Retina display saying it.
#[test]
fn the_physical_bleed_covers_the_device_fixed_corner_and_its_feather() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/selection.rs"),
    )
    .expect("src/selection.rs readable");
    let radius: f32 = src
        .lines()
        .find_map(|l| l.trim().strip_prefix("const CORNER_RADIUS: f32 = "))
        .and_then(|rest| rest.trim_end_matches(';').parse().ok())
        .expect(
            "src/selection.rs must still declare `const CORNER_RADIUS: f32 = …` — this \
             law is the reason EDGE_BLEED_PX is Physical, and it cannot grade a \
             constant it can no longer find",
        );
    // The shader's own AA band, in device px, on each side of the edge.
    const FEATHER_PX: f32 = 1.0;
    assert!(
        EDGE_BLEED_PX.0 >= radius + FEATHER_PX,
        "EDGE_BLEED_PX is {} device px but the fill pipeline's corner radius is \
         {radius} plus a {FEATHER_PX}px shader feather — the bleed no longer covers \
         what it exists to hide, and a flush edge will show a sliver of whatever is \
         underneath at row 0",
        EDGE_BLEED_PX.0
    );
    // NON-VACUITY, the other side: this is a DEVICE quantity, so it must not have
    // been quietly promoted to a scaled one. `bleed_to_canvas_edges` takes no
    // scale at all — the signature is the guarantee — and the bleed it applies is
    // the authored number itself, at any canvas size.
    let bled = bleed_to_canvas_edges([0.0, 0.0, 800.0, 30.0], 800.0);
    assert_eq!(bled[1], -EDGE_BLEED_PX.0);
    assert_eq!(
        bleed_to_canvas_edges([0.0, 0.0, 2400.0, 90.0], 2400.0)[1],
        -EDGE_BLEED_PX.0
    );
}

//! UNIT TESTS for the `TextPipeline` GPU aggregation root, split by feature
//! area (the 2026-07 code-organization pass) out of one formerly-9.7k-line
//! `render::tests` module into this `render/tests/` directory -- every test's
//! NAME is unchanged, only its module path grew one segment
//! (`render::tests::foo` -> `render::tests::<area>::foo`). `use super::*;`
//! here still resolves to the `render` root exactly as before the split; each
//! child module re-derives render access directly via its own
//! `use super::super::*;` (a single glob, so it can never collide with a
//! sibling test module of the same name as a real render/theme module -- see
//! `theme/`/`geometry.rs`) plus a targeted `use super::{..};` for whichever
//! of this module's own shared test helpers it actually calls.

use super::*;

/// What the suite may hold on the shared wgpu device, in wgpu-hal's own live
/// object counts — the oracle that travels where RSS does not. Native-only for
/// the same reason `test_gpu::shared_device_queue` is `None` on wasm: the wasm
/// test runner is Node and has no adapter to allocate on.
#[cfg(not(target_arch = "wasm32"))]
mod alloc_bound_law;
mod backgrounds_item117;
mod backgrounds_item132;
mod backgrounds_item158;
mod backgrounds_item69;
mod backgrounds_item86;
mod backgrounds_item89;
mod bowerbird_drift_item163;
mod bowerbird_finds_item176;
mod bowerbird_spacing_item191;
mod build_integrity;
mod card_texture_shape;
mod caret;
mod caret_block;
mod caret_ink_box;
mod caret_transition_item105;
mod caret_visual_body;
mod chrome_overlay;
mod chrome_panels;
mod cjk;
mod comparison_composite_item116d;
/// ITEM 116b — the RELOCATED DOCUMENT VIEWPORT: the one owner all four
/// document-geometry owners read, the private page-column bypass, the total
/// relocation, and the margin-orientation surfaces that yield to it.
mod comparison_viewport_item116b;
mod date_picker_ink;
mod diagonal_composition;
mod distinguishability;
mod dither;
mod facepitch;
mod firetail_showcase;
mod float_surface_law;
mod fold_chevron_center_item127;
mod folds;
mod frost;
mod geometry;
mod geometry_reshape;
mod glide_anchor_law;
/// The GPU program cache: amortised, single-owner, and world-neutral.
mod gpu_cache_law;
mod grapheme_click;
mod ground_space_item186;
mod hit_test;
mod hover_slop_law;
mod hud;
mod hybrid_band_snap;
mod images;
mod layout_oracle;
mod list_surfaces;
mod markdown;
mod markdown_headings;
mod nits;
mod one_bit;
mod oracle;
mod outline;
mod overlay_align_law;
mod overlay_header_band_law;
mod overlay_height_clamp_law;
mod overlay_hover_stability_law;
mod overlay_personality;
mod overlay_plan_law;
pub(super) mod overlay_probe;
mod overlay_rail_thirds_law;
mod overlay_rhythm_item112;
mod overlay_right_hug_law;
mod page_frame;
mod palette_location_item220;
mod palette_scroll_anchor_item222;
mod palette_shortcuts_item223;
mod paperbark_retina_item201;
mod pixeldiff;
mod plan_pass_law;
mod popover;
pub(in crate::render) mod potoroo_pane;
mod range_rail;
mod reanchor_crossing_law;
mod rotated_label_item235;
mod row_offset_item131;
mod scroll_pos;
mod selection_clip_law;
mod settings_row_reach_law;
mod split_pane;
mod stars;
mod surfaces_item219;
mod surfaces_item225;
mod syntax_ligatures;
mod syntax_roles;
mod tables;
mod theme;
mod theme_caps_law;
/// The TIMELINE half of the comparison workspace: the two regions never
/// overlap, every row is clickable where it is drawn, and the footer fits the
/// narrow column it rides.
mod timeline_workspace_item116d;
mod visual_selection_law;
mod warp_tunnel_item194;
mod washes;
mod waves_drift_item87;
#[cfg(not(target_arch = "wasm32"))]
mod webgl_shader_validation;
/// ITEM 114 — the summoned workspace's presentation: two regions, wide/narrow
/// staging, drawn-equals-clickable, and a focus cue asserted in real pixels.
mod workspace_item114;
mod workspace_plate_item234;
/// ITEM 116a — the shape: `workspace_shape() -> Option<WorkspaceShape>`'s
/// roster and the `rows_are_primary()` bypass-is-module-private law.
mod workspace_shape_item116a;
mod wrap_affinity;
mod wysiwyg;
mod zoom_anchor;

// 800px tall, TEXT_TOP 16, LINE_HEIGHT 32 -> floor((800-16)/32) = 24 rows.
pub(super) const H: f32 = 800.0;

/// Build a headless pipeline, or `None` if no wgpu adapter is available. The
/// device/queue underneath come from the process-wide shared pair
/// (`crate::test_gpu`) — only the `TextPipeline` (and its `Cache`) are fresh.
///
/// `with_shared_programs`, not `shared_device_queue`: it is the same shared
/// device either way, but inside it the ~33 render pipelines and 8 shader
/// modules `TextPipeline::new` stands up are built ONCE FOR THE PROCESS rather
/// than once per call. Only the device-level PROGRAMS are shared — never a
/// uniform, a bind group or an instance buffer — so each caller still gets its
/// own pipeline state and the world it asked for.
pub(super) fn headless_pipeline() -> Option<TextPipeline> {
    crate::test_gpu::with_shared_programs(|device, queue| {
        let cache = Cache::new(device);
        let mut p = TextPipeline::new(device, queue, &cache, wgpu::TextureFormat::Rgba8UnormSrgb);
        p.set_size(1200.0, 800.0);
        p
    })
}

/// A `(Device, Queue, TextPipeline)` triple sized `w`×`h`, or `None` on a
/// GPU-less machine — for tests that must READ what a real `prepare()` left in
/// the pipeline (instance counts, shaped-buffer geometry) and so need a device
/// and queue to drive it. The device/queue are cloned handles onto the shared
/// process-wide pair (`crate::test_gpu`), not a fresh device.
pub(super) fn headless_dqp(w: f32, h: f32) -> Option<(wgpu::Device, wgpu::Queue, TextPipeline)> {
    // `with_shared_programs` for the same reason `headless_pipeline` uses it.
    crate::test_gpu::with_shared_programs(|device, queue| {
        let cache = Cache::new(device);
        let mut p = TextPipeline::new(device, queue, &cache, wgpu::TextureFormat::Rgba8UnormSrgb);
        p.set_size(w, h);
        (device.clone(), queue.clone(), p)
    })
}

/// ITEM 164 — an EMPTY visual selection, for the shaping/width probes that pass
/// no selected ink and so cannot flip any row.
pub(super) fn no_vis() -> crate::render::chrome::VisualSelection {
    crate::render::chrome::VisualSelection::default()
}

pub(super) fn view(text: &str, line: usize, col: usize) -> ViewState {
    ViewState {
        text: text.to_string(),
        cursor_line: line,
        cursor_col: col,
        ..ViewState::base()
    }
}

/// ITEM 116b — a [`view`] whose summoned workspace carries its own ROWS in the
/// PRIMARY column, so the CONTENT pane becomes the RELOCATED DOCUMENT VIEWPORT
/// (`TextPipeline::comparison_viewport`) and the document layer draws there.
///
/// The fields set here are exactly the ones `sync_view` sets for a real History
/// workspace (`overlay_workspace` / `overlay_rows_primary` are its flat
/// projections of `workspace_shape` / `rows_are_primary`, and
/// `overlay_comparison` its projection of "a `ComparisonRequest` resolved"), so
/// a law that drives them is driving the production seam rather than a test-only
/// door.
pub(super) fn comparison_view(text: &str, line: usize, col: usize) -> ViewState {
    let mut v = view(text, line, col);
    v.overlay_active = true;
    v.overlay_workspace = true;
    v.overlay_rows_primary = true;
    // The shape has a comparison region AND there is prose in it.
    // Both halves are `sync_view`'s own projections, and the second is what makes
    // the document relocate: the timeline can be up with nothing to compare.
    v.overlay_comparison = true;
    v.overlay_title = "Version history";
    v.overlay_lens = vec![
        ("All".into(), true),
        ("Today".into(), false),
        ("Kept".into(), false),
    ];
    v.overlay_items = (0..8).map(|i| format!("version {i}")).collect();
    v.overlay_hint = "type to filter".into();
    v
}

/// A markdown [`view`] — same as [`view`] but with `is_markdown` set, so the
/// styling + outline passes run (used by the margin-outline tests).
pub(super) fn view_md(text: &str, line: usize, col: usize) -> ViewState {
    let mut v = view(text, line, col);
    v.is_markdown = true;
    v
}

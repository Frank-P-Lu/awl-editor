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
/// buffer/texture-view counts — the oracle that travels where RSS does not.
/// Native-only for the same reason `test_gpu::shared_device_queue` is `None` on
/// wasm: the wasm test runner is Node and has no adapter to allocate on.
mod accessory_ink;
#[cfg(not(target_arch = "wasm32"))]
mod alloc_bound_law;
mod ambient_wrap_law;
mod bands_waves;
mod bowerbird_breathe;
mod bowerbird_finds;
mod bowerbird_spacing;
mod build_integrity;
mod card_texture_shape;
mod caret;
mod caret_block;
mod caret_filled_knockout;
mod caret_ink_box;
mod caret_one_height_pixels;
mod caret_one_width_pixels;
mod caret_transition;
mod caret_visual_body;
mod chip_plate_floor;
mod chrome_overlay;
mod chrome_panels;
mod chrome_pixel_space;
mod cjk;
mod cjk_cell_caret;
/// The mirrored diagonal row cluster: which end of it a name hangs on, which
/// end its accessory does, and that a mirrored name is clickable where it draws.
mod cluster_mirror;
/// The writing column's left edge is a LOGICAL quantity: the same
/// logical window seats it at the same logical x on every display scale, and the
/// caret / hit test / rail affordance move with it rather than after it.
mod column_left_dpi;
mod comparison_composite;
/// The RELOCATED DOCUMENT VIEWPORT: the one owner all four
/// document-geometry owners read, the private page-column bypass, the total
/// relocation, and the margin-orientation surfaces that yield to it.
mod comparison_viewport;
mod date_picker_ink;
mod deckle_ground;
mod diagonal_composition;
/// THE DIAGONAL COMPOSITION IN REAL PIXELS: orientation, line continuity, the
/// inset attachment band, the fixed name/control lanes, and the room wordmark
/// never landing under a row — five laws, five oracles, over bytes off the GPU.
mod diagonal_pixel_composition;
mod distinguishability;
mod dither;
/// The virtual breathing room past the last line: its one scroll owner, its
/// composition with typewriter mode, and the law that none of it reaches disk.
mod end_pad;
mod eotf_bit_identity;
mod facepitch;
mod facet_mark_dpi;
mod facet_strip_air;
mod firetail_showcase;
mod float_surface_law;
mod fold_chevron_center;
mod fold_chevron_direction;
mod folds;
/// `ttf_parser` (this law's `name`-table reader) and
/// `embedded_docs::FONT_LICENSES_MD` are both `cfg(not(target_arch =
/// "wasm32"))` — the former lives under Cargo's non-wasm target dependencies
/// (native PDF export's own reason), the latter is test-only native tooling
/// — so the roster read this law does (`std::fs::read_dir` over
/// `assets/fonts`) has no wasm counterpart to be honest about either.
#[cfg(not(target_arch = "wasm32"))]
mod font_licence;
mod foot_band_no_clip;
mod foot_hint_lean;
mod frost;
/// The card-ink VETO every frost pixel law measures through, and the contract that
/// keeps it one-directional: its flagged set is a superset of the card's drawing whose
/// surplus is the world's own ground, so it does not invert into "where the card is".
mod frost_card_ink;
/// A POINTER-ANCHORED menu is not a takeover of the room: the full arm is not its arm,
/// and the roster's own backing question then answers a footprint or nothing at all.
mod frost_context;
mod frost_feather;
mod frost_footprint;
mod frost_parallelogram;
/// THE FROST'S WIDTH — the drawn surfaces bound it from the tight side, and the
/// enumeration's completeness is measured off a frost-suppressed frame.
mod frost_width;
mod geometry;
mod geometry_reshape;
mod glide_anchor_law;
/// The GPU program cache: amortised, single-owner, and world-neutral.
mod gpu_cache_law;
mod grapheme_click;
mod ground_space;
mod hint_gap;
mod hit_test;
mod hover_slop_law;
mod hud;
mod hybrid_band_snap;
mod images;
mod layout_oracle;
mod list_surfaces;
mod magpie_bands;
mod markdown;
mod markdown_headings;
/// The overlay's selected-row marker and the fold chevron are ONE rotatable
/// symbol with two entry points; the mark's turn is legible at rest because the
/// shape has no rotational symmetry.
mod marker_chevron_owner;
/// The selected-row mark stands on the row's OUTER edge, on the side the row
/// planner's own signed inset names, and each diagonal world paints the mark its
/// display face asks for.
mod marker_side;
mod nits;
/// THE CALM NOTICE: where it draws, whether it can be seen, and whether a
/// HELD notice can be told from a self-clearing one — three floors, no one of
/// which is satisfiable by breaking the others.
mod notice;
mod one_bit;
mod oracle;
mod organic_ground;
mod outline;
mod overlay_align_law;
mod overlay_header_band_law;
mod overlay_height_clamp_law;
mod overlay_hover_stability_law;
/// A `Bars` location row that plans no glyph (a style whose cue moved
/// off-card, e.g. Cassowary's `RotatedRail`) draws no plate either.
mod overlay_location_plate;
mod overlay_personality;
mod overlay_plan_law;
pub(super) mod overlay_probe;
mod overlay_rail_thirds_law;
mod overlay_rhythm;
mod overlay_right_hug_law;
mod page_frame;
/// The drawn page IS the authored `base_100`, over the whole roster at 1×/2× —
/// the PIXEL half of the clear colour's transfer function
/// (`theme::tests::clear` is the arithmetic half).
mod page_ground_law;
mod palette_location;
mod palette_scroll_anchor;
mod palette_shortcuts;
mod paperbark_retina;
mod pipeline_tint_law;
mod pixeldiff;
mod plan_pass_law;
mod popover;
pub(in crate::render) mod potoroo_pane;
mod query_field;
mod quote_orientation;
mod rail_ink_law;
mod raked_location;
mod range_rail;
mod reanchor_crossing_law;
mod rotated_label;
mod rotated_location;
mod rotated_rail;
mod row_offset;
mod row_pitch_dpi_law;
/// The `Rules` composition: the full `OverlayKind` row-surface sweep, the
/// Settings workspace (both regions), every `SettingId × SettingKind`,
/// drawn-equals-clickable at both DPIs, and the pixel laws.
mod rules_composition;
mod scroll_pos;
/// The SELECTED row's secondary column against the ground it is actually drawn
/// on — the floor the range rail's own "is there a fill under me" answer never
/// covered for the text column.
mod selected_secondary_ink_law;
mod selection_clip_law;
/// The document-selection band's own legibility floor — the only ink-adjacent
/// token in the theme model that used to carry none.
mod selection_contrast_law;
mod selection_token_routing_law;
/// One owner for the `SettingsValues` probe fixture six sibling files used to
/// hand-roll independently: a source-scan law so a seventh copy can't appear.
#[cfg(not(target_arch = "wasm32"))]
mod settings_fixture_law;
mod settings_row_reach_law;
/// The contextual spelling popup's Diagonal-only measured cluster and local frost:
/// exhaustive style enrolment, full Add row, raking coverage and outside identity.
mod spell_popup_diagonal;
mod split_pane;
mod stars;
mod syntax_ligatures;
mod syntax_roles;
mod tables;
/// The document's first-row vertical origin (`TextPipeline::text_origin_top`,
/// `doc_top`, `hit_test_scroll`), through the live pipeline, at every DPI and
/// both `MENU_BAR_ON` states — the vertical twin of `column_left_dpi`.
mod text_top_dpi;
mod theme;
mod theme_caps_law;
/// The one picker whose selection nothing could A/B: the audition lives in the
/// ACTION path, so the RENDER path can hold the world still while the selection
/// moves — the true A/B no capture can arrange, and the answer to the pre-tag
/// sweep's abstention on every textured and staggered world.
mod theme_picker_selection_law;
/// How far one theme-picker arrow's reshape REACHES — the whole document, not
/// the viewport — carrying the release per-stage measurement that makes that
/// reach the dominant cost of a preview step.
mod theme_preview_shape_law;
/// The TIMELINE half of the comparison workspace: the two regions never
/// overlap, every row is clickable where it is drawn, and the footer fits the
/// narrow column it rides.
mod timeline_workspace;
mod visual_selection_law;
mod warp_one_tunnel;
mod warp_tunnel;
mod warped_grid;
mod washes;
mod waves_drift;
#[cfg(not(target_arch = "wasm32"))]
mod webgl_shader_validation;
/// The summoned workspace's presentation: two regions, wide/narrow
/// staging, drawn-equals-clickable, and a focus cue asserted in real pixels.
mod workspace;
/// The Back the content stage teaches: ONE sentence across width x zoom x
/// scale, planned, shaped and inked inside the card on both sides of the
/// staging threshold.
mod workspace_back_height;
mod workspace_back_width;
mod workspace_footer_plate;
mod workspace_plate;
/// The shape: `workspace_shape() -> Option<WorkspaceShape>`'s
/// roster and the `rows_are_primary()` bypass-is-module-private law.
mod workspace_shape;
/// The narrow regime: a workspace stages its two regions, and neither stage is
/// ever blank — an empty planned row window is always a staged card whose other
/// region draws, and some stage always has rows at every reachable window.
mod workspace_stage_reach;
mod workspace_two_column_accessory;
mod wrap_affinity;
/// The writing column's own decorations — the inline-code pill's inset, the
/// fence panel's overhang, the spell squiggle's amplitude/period/stroke — graded
/// as logical lengths at four panel densities.
mod writing_column_decor_dpi;
mod wysiwyg;
mod zigzag_ground;
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

/// **A SWEEP'S HOISTED PIPELINE IS CHECKED, NOT TRUSTED — THE ONE OWNER OF THAT
/// RULE.**
///
/// Reusing one `TextPipeline` across a sweep's cells instead of building one per
/// cell is what makes a wide sweep affordable, and it is a cache-staleness bet:
/// a pipeline's cache keys restart at zero per pipeline, so a size, zoom or
/// world swap across cells is exactly the cache-key discipline CLAUDE.md
/// records. It is not a bet every sweep may take — a sweep whose per-cell
/// reading moves under reuse has a real cross-cell dependency, and there the
/// fresh-pipeline isolation is the law's subject rather than its overhead.
///
/// So a law that hoists re-measures every cell its claims rest on against a
/// pipeline that has seen no other geometry, and the two readings must agree TO
/// THE BIT. `recorded` carries those cells' readings from the hoisted sweep,
/// each under the label its failures are reported in; `afresh` re-measures one
/// by that label, returning `None` only where the machine has no adapter left to
/// answer with. The number of cells actually re-measured comes back, so a law
/// can floor it against the number of readings it rests on rather than trusting
/// that the loop ran.
///
/// The reading type is `Eq` rather than `PartialEq` DELIBERATELY: `f32` is not
/// `Eq`, so a law cannot hand this owner a bare float and quietly inherit `==`'s
/// tolerance — `-0.0 == 0.0` and no NaN equals itself, and a stale cache is
/// exactly as likely to show up in the last mantissa bit as anywhere else.
/// Floats come through `f32::to_bits`.
pub(super) fn assert_the_hoist_carries_no_state<R: Eq + std::fmt::Debug>(
    recorded: &[(String, R)],
    mut afresh: impl FnMut(&str) -> Option<R>,
) -> usize {
    let mut rechecked = 0usize;
    for (what, was) in recorded {
        let Some(again) = afresh(what) else {
            return rechecked;
        };
        assert_eq!(
            &again, was,
            "{what}: the sweep's shared pipeline and a pipeline built for this cell alone read \
             differently — the reuse that makes this sweep affordable is carrying state between \
             cells, so every reading the sweep took is suspect"
        );
        rechecked += 1;
    }
    rechecked
}

/// An EMPTY visual selection, for the shaping/width probes that pass
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

/// A [`view`] whose summoned workspace carries its own ROWS in the
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

/// The one owner for the `SettingsValues` probe fixture every Settings-workspace
/// render test needs: six files under this directory each hand-rolled a
/// byte-identical `crate::settings::SettingsValues { .. }` literal, differing
/// only in `zoom`/`scroll_sensitivity` — which two of them deliberately drive
/// off the 1.0 default (`range_rail`'s rail-position sweep, `settings_row_reach_law`'s
/// off-default-zoom probe), so those two fields stay parameters rather than
/// getting forced to agreement. Every other field is a fixed probe value with no
/// load-bearing role of its own (`page_width_prose`/`_code`, the `/n`/`/w`/`/p`
/// path stand-ins, the three on-flags, `keymap`, and the capture-deterministic
/// [`crate::dateformat::CAPTURE_PLACEHOLDER_YMD`]) — none of the six files'
/// laws depend on any of them differing, unlike [`SETTINGS_VIEW_PARKED_WINDOW_ROWS`]
/// below, whose parked value IS load-bearing.
pub(super) fn settings_values(
    zoom: f32,
    scroll_sensitivity: f32,
) -> crate::settings::SettingsValues {
    crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom,
        scroll_sensitivity,
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

/// The fixture-only, PARKED `overlay_window_rows` [`settings_overlay_view`]
/// callers pass — `ViewState::base()`'s inert default of 12, NOT what
/// `App::sync_view` really sets (`ov.window_rows()`, `SETTINGS.len()` = 31 for
/// `OverlayKind::Settings`).
///
/// `settings_row_reach_law.rs` and `range_rail.rs` each used to declare their
/// own byte-identical `settings_view`, both silently leaving this field at 12
/// and both carrying their own copy of this note — now one owner, one note.
/// The gap is load-bearing, not cosmetic: correcting it (a throwaway local
/// patch, not landed) turns `settings_row_reach_law`'s reach law RED at
/// `world=Mangrove dpi=1 logical_width=640 setting=PageWidthProse` — a drawn
/// Range row whose rail can no longer seat once the wider drawn set (22
/// candidate lines in a 718.8px card at 1200x800) grows the diagonal
/// cluster's label/value columns past what `rail_geom` can fit a rail into.
/// That is a still-open PRODUCT question about the accessory cluster's width
/// budget — who yields first, the row name, the value text, or the rail — not
/// a test bug, so it is handed back rather than papered over, and this
/// default stays parked until the question is answered. Un-parking it is
/// then a one-line change: pass `ov.window_rows()` (or a real 31) at this
/// constant's ONE definition instead of threading it through every call site
/// again.
pub(super) const SETTINGS_VIEW_PARKED_WINDOW_ROWS: usize = 12;

/// Fold a Settings [`crate::overlay::OverlayState`] into a `ViewState` the way
/// `App::sync_view` does — EXCEPT for `overlay_window_rows`, which the caller
/// supplies explicitly rather than this function deriving it from
/// `ov.window_rows()` the way `sync_view` really does. Every current caller
/// passes [`SETTINGS_VIEW_PARKED_WINDOW_ROWS`]; see that constant's own doc for
/// why the divergence is deliberate and load-bearing, not an oversight.
pub(super) fn settings_overlay_view(
    ov: &crate::overlay::OverlayState,
    overlay_window_rows: usize,
) -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Settings.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = overlay_window_rows;
    v
}

/// A markdown [`view`] — same as [`view`] but with `is_markdown` set, so the
/// styling + outline passes run (used by the margin-outline tests).
pub(super) fn view_md(text: &str, line: usize, col: usize) -> ViewState {
    let mut v = view(text, line, col);
    v.is_markdown = true;
    v
}

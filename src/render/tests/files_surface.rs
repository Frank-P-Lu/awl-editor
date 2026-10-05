//! Files' approved composition: one opaque, no-frost card with header actions,
//! a labelled query, choice rows, and a separate destination footer.

use super::super::{FilesSurfaceAction, ViewState};
use super::frost_feather::{DENSE, render_frame};
use super::{headless_dqp, view_md};

fn max_edge_delta(pixels: &[[u8; 4]], width: usize, height: usize, rect: [f32; 4]) -> f64 {
    let mut strongest = 0.0_f64;
    let left = rect[0].round() as isize;
    let right = (rect[0] + rect[2]).round() as isize;
    let top = rect[1].round() as isize;
    let bottom = (rect[1] + rect[3]).round() as isize;
    let cx = (rect[0] + rect[2] * 0.5).round() as isize;
    let cy = (rect[1] + rect[3] * 0.5).round() as isize;
    for edge in [left, right] {
        for y in (cy - 2)..=(cy + 2) {
            for boundary_x in (edge - 1)..=(edge + 1) {
                for neighbour_x in (edge - 4)..=(edge + 4) {
                    if y >= 0
                        && y < height as isize
                        && boundary_x >= 0
                        && neighbour_x >= 0
                        && boundary_x < width as isize
                        && neighbour_x < width as isize
                    {
                        let boundary = pixels[y as usize * width + boundary_x as usize];
                        let neighbour = pixels[y as usize * width + neighbour_x as usize];
                        strongest = strongest.max(super::pixeldiff::delta_e(boundary, neighbour));
                    }
                }
            }
        }
    }
    for edge in [top, bottom] {
        for x in (cx - 2)..=(cx + 2) {
            for boundary_y in (edge - 1)..=(edge + 1) {
                for neighbour_y in (edge - 4)..=(edge + 4) {
                    if x >= 0
                        && x < width as isize
                        && boundary_y >= 0
                        && neighbour_y >= 0
                        && boundary_y < height as isize
                        && neighbour_y < height as isize
                    {
                        let boundary = pixels[boundary_y as usize * width + x as usize];
                        let neighbour = pixels[neighbour_y as usize * width + x as usize];
                        strongest = strongest.max(super::pixeldiff::delta_e(boundary, neighbour));
                    }
                }
            }
        }
    }
    strongest
}

fn files_view_at(document: &str, empty: bool, location: &str) -> ViewState {
    let mut view = view_md(document, 0, 0);
    view.overlay_active = true;
    view.overlay_files_surface = true;
    view.overlay_files_location = location.to_string();
    view.overlay_title = format!("{location}  Up  Change folder  Search");
    view.overlay_query = "draft".to_string();
    view.overlay_query_caret = 5;
    view.overlay_items = if empty {
        Vec::new()
    } else {
        vec!["draft.md".into(), "deep  ›".into()]
    };
    view.overlay_sections = vec![String::new(); view.overlay_items.len()];
    view.overlay_empty = empty.then(|| "no matches".into());
    view.overlay_lens = vec![("Files".into(), true), ("Recent".into(), false)];
    view.overlay_hint = format!("New document — {location}");
    view
}

fn files_view(document: &str, empty: bool) -> ViewState {
    files_view_at(document, empty, "Writing/notes")
}

fn assert_files_action_regions(pipeline: &super::super::TextPipeline) -> [f32; 4] {
    let [up, change, create] = pipeline.files_surface_action_regions_probe();
    let (up_action, up) = up.expect("Up hit span");
    let (change_action, change) = change.expect("Change folder hit span");
    let (create_action, create) = create.expect("New document hit span");
    assert_eq!(up_action, FilesSurfaceAction::Up);
    assert_eq!(change_action, FilesSurfaceAction::ChangeFolder);
    assert_eq!(create_action, FilesSurfaceAction::NewDocument);
    assert!(
        up[1] == change[1] && up[1] < create[1],
        "{up:?} {change:?} {create:?}"
    );
    assert!(up[0] + up[2] < change[0], "header actions overlap");
    for (action, rect) in [
        (FilesSurfaceAction::Up, up),
        (FilesSurfaceAction::ChangeFolder, change),
        (FilesSurfaceAction::NewDocument, create),
    ] {
        let point = (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        assert_eq!(
            pipeline.files_surface_action_at(point.0, point.1),
            Some(action)
        );
        assert!(pipeline.overlay_row_at(point.0, point.1).is_none());
        assert!(pipeline.overlay_query_char_at(point.0, point.1).is_none());
    }
    create
}

fn assert_files_text_fits(
    pipeline: &super::super::TextPipeline,
    view: &ViewState,
    width: u32,
    dpi: f32,
    location: &str,
) -> ([f32; 4], [f32; 4]) {
    let card = pipeline.overlay_card_rect().unwrap();
    let (fitted_title, fitted_footer) = pipeline
        .files_surface_text_probe()
        .expect("Files fitted text");
    let [text, header, footer, caret] = pipeline
        .files_surface_ink_bounds_probe()
        .expect("Files ink bounds");
    for (label, ink) in [("header", header), ("footer", footer), ("caret", caret)] {
        assert!(
            ink[0] >= text[0] && ink[0] + ink[2] <= text[0] + text[2] + 0.5,
            "{width}px @{dpi}x {label} clips horizontally: text={text:?} \
             ink={ink:?} title={fitted_title:?} footer={fitted_footer:?}"
        );
    }
    for essential in ["Up", "Change…", "Search"] {
        assert!(
            fitted_title.contains(essential),
            "{width}px @{dpi}x dropped {essential:?}: {fitted_title:?}"
        );
    }
    assert!(
        fitted_footer.starts_with("New document — "),
        "{width}px @{dpi}x dropped the footer action: {fitted_footer:?}"
    );
    assert_eq!(view.overlay_files_location, location);
    if location.chars().count() > 30 && width == 720 {
        assert!(
            fitted_title.contains('…') && fitted_footer.contains('…'),
            "the long-path subjects must exercise measured elision: {fitted_title:?} / \
             {fitted_footer:?}"
        );
    }
    (card, caret)
}

fn assert_files_neighbourhood_edges(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &mut super::super::TextPipeline,
    width: u32,
    dpi: f32,
    empty: bool,
    location: &str,
) {
    pipeline.set_dpi(dpi);
    pipeline.set_size(width as f32, 800.0);
    let view = files_view_at(DENSE, empty, location);
    pipeline.set_view(&view);
    let _pixels = render_frame(device, queue, pipeline, width, 800);
    assert_eq!(
        pipeline.frost_mode(),
        None,
        "{width}px @{dpi}x empty={empty}"
    );
    let create = assert_files_action_regions(pipeline);
    let (card, caret) = assert_files_text_fits(pipeline, &view, width, dpi, location);
    let query_y = caret[1] + caret[3] * 0.5;
    let query_x = (card[0].floor() as i32..=(card[0] + card[2]).ceil() as i32)
        .map(|x| x as f32 + 0.5)
        .find(|&x| pipeline.overlay_query_char_at(x, query_y).is_some())
        .expect("labelled search retains a query input span");
    assert!(pipeline.files_surface_action_at(query_x, query_y).is_none());
    if !empty {
        let row_y = (card[1].floor() as i32..=(card[1] + card[3]).ceil() as i32)
            .find_map(|y| {
                pipeline
                    .overlay_row_at(card[0] + card[2] * 0.5, y as f32 + 0.5)
                    .map(|_| y as f32 + 0.5)
            })
            .expect("choice row span");
        assert!(
            pipeline
                .files_surface_action_at(card[0] + card[2] * 0.5, row_y)
                .is_none()
        );
        let last_choice = pipeline
            .files_surface_line_bounds_probe("deep  ›")
            .expect("last Files choice bounds");
        assert!(
            create[1] - (last_choice[1] + last_choice[3]) >= create[3] * 0.45,
            "destination footer has no internal interval from choices: \
             choice={last_choice:?} footer={create:?}"
        );
    }
}

fn assert_huge_files_path_is_bounded(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &mut super::super::TextPipeline,
) {
    let huge_location = (0..400)
        .map(|index| format!("component-{index:03}"))
        .collect::<Vec<_>>()
        .join("/");
    let huge_len = huge_location.chars().count();
    let huge_prepared = crate::overlay::PreparedDirectoryPath::new(&huge_location);
    let leaf_budget = huge_prepared.leaf_identity_budget().unwrap();
    let materialized_bound: usize =
        super::super::chrome::files_location_fit_budgets(huge_len, Some(leaf_budget))
            .into_iter()
            .sum();
    assert!(
        materialized_bound <= huge_len.saturating_mul(2) + leaf_budget + 4,
        "the fixed fit ladder would copy {materialized_bound} characters from a \
         {huge_len}-character path"
    );
    pipeline.set_dpi(2.0);
    pipeline.set_size(720.0, 800.0);
    pipeline.set_view(&files_view_at(DENSE, false, &huge_location));
    let _pixels = render_frame(device, queue, pipeline, 720, 800);
    let (split_attempts, title_attempts, hint_attempts) =
        pipeline.files_surface_fit_attempts_probe();
    assert!(
        split_attempts == 2
            && (1..=9).contains(&title_attempts)
            && (1..=9).contains(&hint_attempts),
        "long-path fitting must stay bounded: split={split_attempts}, \
         title={title_attempts}, hint={hint_attempts}"
    );
    let [text, header, footer, caret] = pipeline
        .files_surface_ink_bounds_probe()
        .expect("long-path Files ink bounds");
    for (label, ink) in [("header", header), ("footer", footer), ("caret", caret)] {
        assert!(
            ink[0] >= text[0] && ink[0] + ink[2] <= text[0] + text[2] + 0.5,
            "huge path {label} clips: text={text:?} ink={ink:?}"
        );
    }
}

fn assert_potoroo_first_narrow_files_frame() -> bool {
    let Some((device, queue, mut pipeline)) = headless_dqp(720.0, 800.0) else {
        eprintln!("skipping fresh Files first-frame law: no wgpu adapter");
        return false;
    };
    pipeline.set_dpi(2.0);
    pipeline.set_size(720.0, 800.0);
    let mut narrow = files_view_at(DENSE, true, "root");
    narrow.overlay_title = "root  Change folder  Search".into();
    narrow.overlay_query = "zzz".into();
    narrow.overlay_query_caret = 3;
    pipeline.set_view(&narrow);
    let _pixels = render_frame(&device, &queue, &mut pipeline, 720, 800);
    assert_eq!(
        pipeline.overlay_pane_fills_probe().len(),
        1,
        "Files must be one opaque surface on its very first narrow frame"
    );
    let (header, footer) = pipeline.files_surface_text_probe().unwrap();
    assert!(
        header.contains("Search files: zzz")
            && header.contains("root")
            && header.contains("Change…"),
        "first narrow frame dropped compact rows: {header:?}"
    );
    assert_eq!(footer, "New document — root");
    let [text, header_ink, footer_ink, caret] = pipeline.files_surface_ink_bounds_probe().unwrap();
    for (label, ink) in [
        ("header", header_ink),
        ("footer", footer_ink),
        ("caret", caret),
    ] {
        assert!(
            ink[0] >= text[0] && ink[0] + ink[2] <= text[0] + text[2] + 0.5,
            "first narrow {label} clips: text={text:?} ink={ink:?}"
        );
    }
    let typed_geom = pipeline.overlay_geometry(720);
    let typed_plan = pipeline.overlay_row_plan(&typed_geom);
    assert!(
        pipeline
            .overlay_panel_bands(&typed_geom, &typed_plan)
            .is_none(),
        "Files' unified text column must not inherit Potoroo's generic split-band seats"
    );
    let mut split_mutation = narrow;
    split_mutation.overlay_files_surface = false;
    pipeline.set_view(&split_mutation);
    let _pixels = render_frame(&device, &queue, &mut pipeline, 720, 800);
    assert_eq!(
        pipeline.overlay_pane_fills_probe().len(),
        2,
        "dropping the typed Files gate must restore Potoroo's ordinary split composition"
    );
    let split_geom = pipeline.overlay_geometry(720);
    let split_plan = pipeline.overlay_row_plan(&split_geom);
    assert_eq!(
        pipeline.overlay_foot_left(&split_geom, &split_plan),
        split_geom.footer_text_left(),
        "dropping the typed Files gate must restore the ordinary footer seat owner"
    );
    true
}

fn assert_files_line_has_left_ink(
    pixels: &[[u8; 4]],
    width: usize,
    label: &str,
    text: [f32; 4],
    line: [f32; 4],
) {
    assert!(
        line[0] >= text[0] && line[0] + line[2] <= text[0] + text[2] + 0.5,
        "{label} leaves the Files text column: text={text:?} line={line:?}"
    );
    let x0 = line[0].ceil() as usize;
    let x1 = (line[0] + line[2].min(80.0)).floor() as usize;
    let y0 = line[1].ceil() as usize;
    let y1 = (line[1] + line[3]).floor() as usize;
    let mut colors = std::collections::BTreeMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            *colors.entry(pixels[y * width + x]).or_insert(0usize) += 1;
        }
    }
    let total = (x1 - x0) * (y1 - y0);
    let background = colors.values().copied().max().unwrap_or_default();
    assert!(
        total.saturating_sub(background) > 8,
        "{label} has no visible left-edge ink: {line:?} colors={colors:?}"
    );
}

fn assert_nested_potoroo_files_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &mut super::super::TextPipeline,
    nested: &ViewState,
) -> Vec<[u8; 4]> {
    pipeline.set_view(&files_view_at(DENSE, false, "root"));
    let _root_pixels = render_frame(device, queue, pipeline, 1200, 800);
    let root_scroll = pipeline.panel_buffer.scroll();
    assert_eq!(
        (
            root_scroll.line,
            root_scroll.vertical,
            root_scroll.horizontal
        ),
        (0, 0.0, 0.0),
        "Files root left stale panel-buffer scroll before browse"
    );
    pipeline.set_view(nested);
    let pixels = render_frame(device, queue, pipeline, 1200, 800);
    assert_eq!(pipeline.overlay_pane_fills_probe().len(), 1);
    let nested_scroll = pipeline.panel_buffer.scroll();
    assert_eq!(
        (
            nested_scroll.line,
            nested_scroll.vertical,
            nested_scroll.horizontal
        ),
        (0, 0.0, 0.0),
        "root→nested reshape retained stale panel-buffer scroll"
    );
    let (header, footer) = pipeline.files_surface_text_probe().unwrap();
    assert!(
        header.contains("root/notes"),
        "nested location prefix was lost: {header:?}"
    );
    assert_eq!(footer, "New document — root/notes");
    let nested_geom = pipeline.overlay_geometry(1200);
    let nested_plan = pipeline.overlay_row_plan(&nested_geom);
    assert!(
        pipeline
            .overlay_panel_bands(&nested_geom, &nested_plan)
            .is_none(),
        "nested Files must suppress Potoroo's generic split-band text seats"
    );
    let text = [
        nested_geom.text_left,
        nested_geom.text_top,
        nested_geom.text_w,
        nested_geom.card_h,
    ];
    for (label, line) in [
        (
            "nested location",
            pipeline
                .files_surface_containing_line_bounds_probe("root/notes")
                .unwrap(),
        ),
        (
            "nested footer",
            pipeline.files_surface_line_bounds_probe(&footer).unwrap(),
        ),
    ] {
        assert_files_line_has_left_ink(&pixels, 1200, label, text, line);
    }
    pixels
}

#[test]
fn files_is_no_frost_in_every_world_and_the_flag_is_the_mutation_subject() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((_device, _queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Files frost law: no wgpu adapter");
        return;
    };
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        let view = files_view(DENSE, false);
        pipeline.set_view(&view);
        assert_eq!(pipeline.frost_mode(), None, "{}: Files blurred", world.name);

        let mut mutation = view;
        mutation.overlay_files_surface = false;
        mutation.overlay_crisp = false;
        mutation.overlay_retains_room = false;
        pipeline.set_view(&mutation);
        if world.render_caps.backdrop != crate::theme::Backdrop::Flat {
            assert!(
                pipeline.frost_mode().is_some(),
                "{}: dropping the typed Files flag must restore blur",
                world.name
            );
        }
    }
}

#[test]
fn header_query_choices_and_footer_keep_distinct_hit_regions_at_neighbourhood_edges() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Files spatial law: no wgpu adapter");
        return;
    };
    for (width, dpi, empty, location) in [
        (
            720,
            1.0,
            false,
            "A Very Long Working Folder Name That Cannot Fit At The Root",
        ),
        (
            720,
            2.0,
            true,
            "Writing/research/chapters/field-notes/interviews/september",
        ),
        (1200, 1.0, true, "Writing/notes"),
        (
            1200,
            2.0,
            false,
            "Writing/research/chapters/field-notes/interviews/september",
        ),
    ] {
        assert_files_neighbourhood_edges(
            &device,
            &queue,
            &mut pipeline,
            width,
            dpi,
            empty,
            location,
        );
    }
    assert_huge_files_path_is_bounded(&device, &queue, &mut pipeline);
}

#[test]
fn files_envelope_is_continuous_and_opaque_in_every_world_and_dpi() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Files opacity law: no wgpu adapter");
        return;
    };
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        for (width, dpi) in [(1200u32, 1.0), (720, 2.0)] {
            pipeline.set_dpi(dpi);
            pipeline.set_size(width as f32, 800.0);
            let baseline_dense = {
                pipeline.set_view(&view_md(DENSE, 0, 0));
                render_frame(&device, &queue, &mut pipeline, width, 800)
            };
            let baseline_blank = {
                pipeline.set_view(&view_md("", 0, 0));
                render_frame(&device, &queue, &mut pipeline, width, 800)
            };
            let dense = {
                pipeline.set_view(&files_view(DENSE, false));
                render_frame(&device, &queue, &mut pipeline, width, 800)
            };
            let card = pipeline.overlay_card_rect().expect("Files card");
            let inset = pipeline.overlay_card_opaque_inset_probe(card);
            assert_eq!(
                pipeline.overlay_pane_fills_probe().len(),
                1,
                "{} {width}px @{dpi}x split the Files envelope",
                world.name
            );
            let blank = {
                pipeline.set_view(&files_view("", false));
                render_frame(&device, &queue, &mut pipeline, width, 800)
            };
            let x0 = (card[0] + inset).max(0.0).ceil() as usize;
            let x1 = (card[0] + card[2] - inset).min(width as f32).floor() as usize;
            let y0 = (card[1] + inset).max(0.0).ceil() as usize;
            let y1 = (card[1] + card[3] - inset).min(800.0).floor() as usize;
            assert!(
                (x1 - x0) * (y1 - y0) > 10_000,
                "{} {width}px @{dpi}x has no substantial safe interior: \
                 card={card:?} inset={inset}",
                world.name
            );
            let mut witnesses = 0usize;
            let mut leaks = 0usize;
            let mut leak_points = Vec::new();
            for y in y0..y1 {
                for x in x0..x1 {
                    let index = y * width as usize + x;
                    if baseline_dense[index] != baseline_blank[index] {
                        witnesses += 1;
                        if dense[index] != blank[index] {
                            leaks += 1;
                            if leak_points.len() < 24 {
                                leak_points.push((
                                    x,
                                    y,
                                    baseline_dense[index],
                                    dense[index],
                                    blank[index],
                                ));
                            }
                        }
                    }
                }
            }
            assert!(
                witnesses > 100,
                "{} {width}px @{dpi}x needs a substantial document witness, got {witnesses}",
                world.name
            );
            assert_eq!(
                leaks, 0,
                "{} {width}px @{dpi}x leaked {leaks}/{witnesses} underlying document \
                 pixels through Files: {leak_points:?}",
                world.name,
            );
        }
    }
}

#[test]
fn files_query_caret_is_drawn_only_while_search_owns_focus() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Files focus law: no wgpu adapter");
        return;
    };
    let mut view = files_view(DENSE, false);
    view.overlay_query_field = true;
    pipeline.set_view(&view);
    let _pixels = render_frame(&device, &queue, &mut pipeline, 1200, 800);
    assert!(
        pipeline.panel_caret.is_drawn(),
        "query focus needs its caret"
    );

    view.overlay_query_field = false;
    pipeline.set_view(&view);
    let _pixels = render_frame(&device, &queue, &mut pipeline, 1200, 800);
    assert!(
        !pipeline.panel_caret.is_drawn(),
        "a selected Files choice must not compete with a query caret"
    );
}

#[test]
fn files_scope_and_controls_have_visible_hierarchy_on_every_world_and_geometry() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Files affordance law: no wgpu adapter");
        return;
    };
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        for (width, dpi, location) in [
            (1200u32, 1.0, "Writing/notes"),
            (
                720,
                2.0,
                "Writing/research/chapters/field-notes/interviews/september",
            ),
        ] {
            pipeline.set_dpi(dpi);
            pipeline.set_size(width as f32, 800.0);
            let mut view = files_view_at(DENSE, false, location);
            view.overlay_query = String::new();
            view.overlay_query_caret = 0;
            pipeline.set_view(&view);
            let pixels = render_frame(&device, &queue, &mut pipeline, width, 800);
            let [query, change, footer] = pipeline
                .files_surface_control_rects_probe()
                .expect("Files control layout");
            let [fills, rims] = pipeline.files_surface_control_quad_counts_probe();
            assert_eq!(
                fills, 2,
                "{} {width}px @{dpi}x: Files control fills were removed or parked",
                world.name
            );
            assert_eq!(
                rims, 2,
                "{} {width}px @{dpi}x: Files control rims were removed or parked",
                world.name
            );
            let (header, footer_text) = pipeline.files_surface_text_probe().unwrap();
            assert!(
                header.contains("Search files: ")
                    && header.contains("Change…")
                    && header.contains(location.rsplit('/').next().unwrap()),
                "{} {width}px @{dpi}x lost Files hierarchy: {header:?}",
                world.name
            );
            assert!(
                footer_text.starts_with("New document — "),
                "{} {width}px @{dpi}x footer regressed to metadata: {footer_text:?}",
                world.name
            );
            assert!(
                change[1] < query[1] && query[1] < footer[1],
                "{} {width}px @{dpi}x controls collapsed into one line: \
                 query={query:?} change={change:?} footer={footer:?}",
                world.name
            );
            assert!(
                query[2] > 80.0 * dpi && footer[2] > 180.0 * dpi,
                "{} {width}px @{dpi}x field/footer have no recognizable plate: \
                 query={query:?} footer={footer:?}",
                world.name
            );
            for (label, rect) in [("search", query), ("new", footer)] {
                let edge_delta = max_edge_delta(&pixels, width as usize, 800, rect);
                assert!(
                    edge_delta >= 2.3,
                    "{} {width}px @{dpi}x {label} boundary is below one JND: \
                     ΔE={edge_delta:.2} rect={rect:?}",
                    world.name
                );
                let point = (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
                assert!(
                    pipeline.overlay_row_at(point.0, point.1).is_none(),
                    "{label} became a mixed candidate/action row"
                );
            }

            let selected_before = pipeline.overlay_window_report();
            view.overlay_query_field = false;
            view.overlay_rows_focused = false;
            view.overlay_title = view
                .overlay_title
                .replace("Change folder", "› Change folder");
            pipeline.set_view(&view);
            let _focused = render_frame(&device, &queue, &mut pipeline, width, 800);
            assert_eq!(
                pipeline.overlay_window_report(),
                selected_before,
                "{} {width}px @{dpi}x action focus displaced the selected file row",
                world.name
            );
        }
    }
}

#[test]
fn every_world_fits_the_real_720_at_2x_files_header() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(720.0, 800.0) else {
        eprintln!("skipping Files all-world narrow law: no wgpu adapter");
        return;
    };
    pipeline.set_dpi(2.0);
    pipeline.set_size(720.0, 800.0);
    let location = "Writing/research/chapters/field-notes/interviews/september";
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        pipeline.set_view(&files_view_at(DENSE, false, location));
        let pixels = render_frame(&device, &queue, &mut pipeline, 720, 800);
        let [text, header, footer, caret] = pipeline
            .files_surface_ink_bounds_probe()
            .expect("Files ink bounds");
        for (label, ink) in [("header", header), ("footer", footer), ("caret", caret)] {
            assert!(
                ink[0] >= text[0] && ink[0] + ink[2] <= text[0] + text[2] + 0.5,
                "{} 720px @2x {label} clips: text={text:?} ink={ink:?}",
                world.name
            );
        }
        let (header, footer) = pipeline
            .files_surface_text_probe()
            .expect("Files rendered text");
        for essential in ["Up", "Change…", "Search"] {
            assert!(
                header.contains(essential),
                "{} dropped {essential:?}: {header:?}",
                world.name
            );
        }
        assert!(
            header.contains("september"),
            "{} dropped the named location tail: {header:?}",
            world.name
        );
        assert!(
            footer.starts_with("New document — "),
            "{} dropped the destination footer: {footer:?}",
            world.name
        );
        let location_line = pipeline
            .files_surface_containing_line_bounds_probe("september")
            .expect("named location line");
        let x0 = location_line[0].ceil().max(0.0) as usize;
        let x1 = (location_line[0] + location_line[2]).floor().min(720.0) as usize;
        let y0 = location_line[1].ceil().max(0.0) as usize;
        let y1 = (location_line[1] + location_line[3]).floor().min(800.0) as usize;
        assert!(x1 > x0 && y1 > y0, "{} has no location band", world.name);
        let mut colors = std::collections::BTreeMap::new();
        for y in y0..y1 {
            for x in x0..x1 {
                *colors.entry(pixels[y * 720 + x]).or_insert(0usize) += 1;
            }
        }
        let total = (x1 - x0) * (y1 - y0);
        let background = colors.values().copied().max().unwrap_or_default();
        assert!(
            total.saturating_sub(background) > 12,
            "{} has no visible left-location ink witness: \
             band=({x0},{y0})..({x1},{y1}) colors={colors:?}",
            world.name
        );
    }
}

#[test]
fn fresh_potoroo_pipeline_bills_compact_rows_and_keeps_nested_left_edges() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::theme::set_active_by_name("Potoroo").unwrap();

    if !assert_potoroo_first_narrow_files_frame() {
        return;
    }

    let Some((device, queue, mut nested_pipeline)) = headless_dqp(1200.0, 800.0) else {
        unreachable!("the same adapter disappeared")
    };
    let mut nested = files_view_at(DENSE, false, "root/notes");
    nested.overlay_items = vec!["deep  ›".into()];
    nested.overlay_sections = vec![String::new()];
    nested.overlay_empty = Some("no supported files in this folder".into());
    nested.overlay_query_field = false;
    let pixels = assert_nested_potoroo_files_frame(&device, &queue, &mut nested_pipeline, &nested);

    let Some((fresh_device, fresh_queue, mut fresh_pipeline)) = headless_dqp(1200.0, 800.0) else {
        unreachable!("the same adapter disappeared")
    };
    fresh_pipeline.set_view(&nested);
    let fresh_pixels = render_frame(&fresh_device, &fresh_queue, &mut fresh_pipeline, 1200, 800);
    assert_eq!(
        pixels, fresh_pixels,
        "root→nested Files transition diverged from a fresh nested frame"
    );
}

#[test]
fn files_level_notices_keep_their_own_band_above_the_destination_footer() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::theme::set_active_by_name("Potoroo").unwrap();
    let Some((device, queue, mut pipeline)) = headless_dqp(720.0, 800.0) else {
        eprintln!("skipping Files notice/footer law: no wgpu adapter");
        return;
    };
    pipeline.set_dpi(2.0);
    pipeline.set_size(720.0, 800.0);
    for (notice, empty) in [
        ("this folder is empty", true),
        ("no supported files in this folder", false),
    ] {
        let mut view = files_view_at(DENSE, empty, "Writing/notes");
        view.overlay_empty = Some(notice.to_string());
        pipeline.set_view(&view);
        let _pixels = render_frame(&device, &queue, &mut pipeline, 720, 800);
        let notice_bounds = pipeline
            .files_surface_line_bounds_probe(notice)
            .unwrap_or_else(|| panic!("notice was not shaped: {notice:?}"));
        let [_, _, footer_bounds, _] = pipeline
            .files_surface_ink_bounds_probe()
            .expect("Files footer bounds");
        assert!(
            footer_bounds[1] - (notice_bounds[1] + notice_bounds[3]) >= footer_bounds[3] * 0.45,
            "notice and footer overlap: notice={notice_bounds:?} footer={footer_bounds:?}"
        );
        let (_, footer) = pipeline
            .files_surface_text_probe()
            .expect("Files rendered footer");
        assert!(
            footer.starts_with("New document — "),
            "notice displaced destination footer: {footer:?}"
        );
    }
}

#[test]
fn folder_context_and_small_right_action_share_a_nonoverlapping_shaped_row() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        return;
    };
    let location = "Writing/a long parent folder/another long parent/deep notes";
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        for (width, dpi) in [(1200, 1.0), (720, 2.0), (1920, 2.0)] {
            p.set_dpi(dpi);
            p.set_size(width as f32, 800.0);
            let v = files_view_at(DENSE, false, location);
            p.set_view(&v);
            p.prepare(&device, &queue, width, 800).unwrap();
            let left = p.panel_buffer.layout_runs().next().expect("folder context");
            let right = p
                .panel_bind_buffer
                .layout_runs()
                .next()
                .expect("header action");
            assert!(right.text.contains("Change…"));
            assert_eq!(left.line_top, right.line_top);
            let context_right = left
                .glyphs
                .iter()
                .map(|g| g.x + g.w)
                .fold(0.0_f32, f32::max);
            let action_left = right
                .glyphs
                .iter()
                .map(|g| g.x)
                .fold(f32::INFINITY, f32::min);
            assert!(
                context_right + 2.0 * dpi <= action_left,
                "{} {width} @{dpi}: folder/action overlap ({context_right}, {action_left})",
                world.name
            );
            let geom = p.overlay_geometry(width);
            for g in right.glyphs {
                assert!(
                    g.x >= -0.5 && g.x + g.w <= geom.text_w + 0.5,
                    "{}: right action clipped at {width} @{dpi}",
                    world.name
                );
            }
            assert_files_action_regions(&p);
        }
    }
}

#[test]
fn actual_files_header_runs_and_uploaded_actions_match_the_planned_bands() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        return;
    };
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        for (width, dpi) in [(1200, 1.0), (720, 2.0), (1920, 2.0)] {
            p.set_dpi(dpi);
            p.set_size(width as f32, 800.0);
            let v = files_view_at(DENSE, false, "notes");
            p.set_view(&v);
            p.prepare(&device, &queue, width, 800).unwrap();
            let geom = p.overlay_geometry(width);
            let plan = p.overlay_row_plan(&geom);
            let runs: Vec<_> = p.panel_buffer.layout_runs().collect();
            for header in plan.header_lines().iter().take(2) {
                let run = runs.iter().find(|run| run.line_i == header.line).unwrap();
                assert!(
                    (geom.text_top + run.line_top - header.top).abs() < 0.1,
                    "{} @{dpi}: header {} starts outside its band",
                    world.name,
                    header.line
                );
                assert!(
                    (run.line_height - header.height).abs() < 0.1,
                    "{} @{dpi}: header {} has wrong line height",
                    world.name,
                    header.line
                );
            }
            let query = runs
                .iter()
                .find(|run| run.text.starts_with("Search files:"))
                .unwrap();
            let field = p.overlay_query_band(&plan).unwrap();
            assert_eq!(
                field.line, query.line_i,
                "query field must enclose the actual Search files row"
            );
            let first = runs
                .iter()
                .find(|run| run.line_i == geom.shaped_first_row_line())
                .unwrap();
            assert!(
                (geom.text_top + first.line_top - plan.first_top()).abs() < 0.1,
                "{} @{dpi}: candidate glyphs disagree with selected band",
                world.name
            );
            let bounds = glyphon::TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: 800,
            };
            let areas = super::super::chrome::files_accessory_areas(
                &p.panel_bind_buffer,
                &geom,
                &plan,
                bounds,
                geom.text_left,
                world.base_content.to_glyphon(),
            );
            let action = p.panel_bind_buffer.layout_runs().next().unwrap();
            assert!(
                (areas[0].top + action.line_top - plan.header_lines()[0].top).abs() < 0.1,
                "header action upload must share the folder's origin"
            );
            if let Some((dock, _)) = p.docked_facet_geometry_probe() {
                let line = p.docked_facet_buffer.layout_runs().next().unwrap();
                assert!(
                    line.line_height <= dock.height + 0.1,
                    "docked labels cannot be clipped to the spacing beat"
                );
            }
        }
    }
}

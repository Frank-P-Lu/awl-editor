//! TIER 1: the summoned workspace's PRESENTATION. Its geometry, its
//! two regions, its wide/narrow staging, and its focus cue, asserted where they
//! are actually decided.
//!
//! Everything here is capturable by construction: the workspace is rendered from
//! `ViewState`, so a real pipeline over a real device is the oracle, and every
//! appearance claim is arithmetic over the rendered pixels rather than over the
//! state that intended to draw them (CLAUDE.md's Wagtail tripwire — the sidecar
//! once reported `selected_index: 2` while the row rendered fully invisible).
//!
//! The VALUE side of Settings is not here and cannot be: `SettingToggle`,
//! `SettingValueCommit` and `SettingPathPick` are replay-Unsupported
//! (`docs/harness-reach.md`), so they live in
//! `app::tests::workspace`, driven through the live `App`.

use super::super::*;
use super::pixeldiff::{Region, render_frame};
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};

/// The canvas shapes every geometry law below sweeps: two genuinely wide, two
/// genuinely narrow, and one at each side of the staging threshold — plus the
/// app's own minimum window, so nothing here is only true on a comfortable
/// desktop.
const CANVASES: &[(u32, u32)] = &[
    (2400, 1600),
    (1400, 900),
    (1000, 760),
    (900, 1400),
    (700, 900),
    (620, 820),
    (480, 640),
];

pub(super) fn settings_values() -> crate::settings::SettingsValues {
    super::settings_values(1.0, 1.0)
}

/// A REAL Settings workspace card, built exactly as `overlay::build`'s Settings
/// arm builds it, standing on `lens` with `detail` deciding which region has
/// focus. Focus moves through the LIFECYCLE, never by assignment.
pub(super) fn workspace_card(lens: usize, detail: bool) -> OverlayState {
    let vals = settings_values();
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov.set_facet_lens(lens);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    if detail {
        journey.toggle_detail();
    }
    journey.card().expect("the card is up").clone()
}

/// Fold a workspace card into a `ViewState` the way `App::sync_view` does.
pub(super) fn workspace_view(ov: &OverlayState) -> ViewState {
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Settings.title().to_string();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    // `workspace_card` enters either Categories or Controls through Journey,
    // never the query field. Mirror `App::sync_view`'s Settings projection so
    // the focus-cue laws photograph the region their lifecycle state names,
    // rather than ViewState's inert query/row defaults.
    v.overlay_query_focused = false;
    v.overlay_rows_focused = ov.detail_focus;
    v.overlay_sections = ov.item_sections();
    v.overlay_hint = ov.foot_hint();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = ov.window_rows();
    v
}

/// One swept cell of the geometry axis: a canvas, a zoom and a DPI.
#[derive(Clone, Copy)]
struct Cell {
    w: u32,
    h: u32,
    zoom: f32,
    dpi: f32,
}

impl Cell {
    fn plain(w: u32, h: u32) -> Self {
        Cell {
            w,
            h,
            zoom: 1.0,
            dpi: 1.0,
        }
    }
}

fn prepared(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    ov: &OverlayState,
    cell: Cell,
) {
    p.set_dpi(cell.dpi);
    p.set_size(cell.w as f32, cell.h as f32);
    let mut v = workspace_view(ov);
    v.zoom = cell.zoom;
    p.set_view(&v);
    p.prepare(device, queue, cell.w, cell.h).unwrap();
}

/// THE RAIL IS THE AUTHORED CATEGORY LIST, and it is derived from the settings
/// corpus rather than from a second table — so a category added to `SETTINGS`
/// appears on the rail without anyone remembering to add it, and a rail entry
/// that names no category cannot exist.
///
/// The item names six categories; `All` is the home the faceting convention puts
/// at index 0 (`facets.rs`'s settled rule), so the rail is seven entries. Both
/// halves are asserted against the CORPUS, no wildcard and no literal roster, so
/// a new setting in a new category cannot dodge the sweep.
#[test]
fn the_navigation_rail_is_exactly_the_authored_settings_categories() {
    let _g = crate::testlock::serial();
    let ov = workspace_card(0, false);
    let rail: Vec<String> = ov.lens_strip().into_iter().map(|(l, _)| l).collect();
    assert_eq!(rail[0], "All", "the flat home sits first, by convention");

    let mut authored: Vec<&'static str> = crate::settings::SETTINGS
        .iter()
        .map(|r| r.category)
        .collect();
    authored.dedup();
    let mut sorted_authored = authored.clone();
    sorted_authored.sort_unstable();
    let mut sorted_rail: Vec<&str> = rail[1..].iter().map(|s| s.as_str()).collect();
    sorted_rail.sort_unstable();
    assert_eq!(
        sorted_rail, sorted_authored,
        "the rail must list exactly the categories the corpus authors — no more, \
         no fewer, and none invented here"
    );
    assert_eq!(
        rail[1..].to_vec(),
        authored.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "and in the corpus's own order, so the rail reads as the table does"
    );

    // EVERY setting is reachable: standing on its own category's rail entry
    // shows it. A category that filtered its own rows out would fail here.
    //
    // One deliberate exception: "Keymap" filters itself out entirely on
    // `Convention::Mac` (`settings::row_available_on` — the flavor is
    // structurally inert there), which is the intended behavior this same
    // predicate is meant to catch as a BUG everywhere else. `Convention` is
    // process-frozen, so this reads the ambient value rather than forcing
    // one; `native-gate.sh`'s two-convention run exercises the row's
    // reachability (Linux pass) and its absence (Mac pass) across its two
    // passes.
    let keymap_hidden =
        crate::convention::Convention::current() != crate::convention::Convention::Linux;
    for row in crate::settings::SETTINGS {
        if keymap_hidden && row.id == crate::settings::SettingId::Keymap {
            continue;
        }
        let idx = rail
            .iter()
            .position(|l| l == row.category)
            .unwrap_or_else(|| panic!("{:?}'s category has no rail entry", row.name));
        let card = workspace_card(idx, true);
        assert!(
            card.item_strings().iter().any(|s| s == row.name),
            "{:?} must be reachable from its own rail entry {:?}",
            row.name,
            row.category
        );
    }
}

/// THE RAIL IS CLICKABLE EXACTLY WHERE IT IS DRAWN, at every canvas, zoom and
/// DPI — DESIGN.md §8's rule, driven through the real pipeline.
///
/// Three claims, each of which has been a real defect class in this tree: the
/// drawn rect's own centre resolves to its own entry; the ROW hit-test refuses
/// every pixel of the rail column (so a click meant for a category can never
/// activate a settings row); and the rail's rect never overlaps the content
/// pane's band.
#[test]
fn the_rail_is_clickable_exactly_where_it_is_drawn() {
    let _g = crate::testlock::serial();
    let (device, queue, mut p) =
        headless_dqp(1400.0, 900.0).expect("workspace laws require a wgpu adapter");
    let mut checked = 0usize;
    for &(w, h) in CANVASES {
        for &zoom in &[0.8f32, 1.0, 1.6] {
            for &dpi in &[1.0f32, 2.0] {
                for &detail in &[false, true] {
                    let ov = workspace_card(0, detail);
                    prepared(&device, &queue, &mut p, &ov, Cell { w, h, zoom, dpi });
                    let n = ov.lens_strip().len();
                    let drawn = p.workspace_rail_probe(w);
                    let Some(rail_box) = drawn.rail else {
                        // The narrow DETAIL stage draws no rail — and then no
                        // pixel of the card may resolve to a rail entry.
                        assert!(
                            (0..h).step_by(7).all(|y| (0..w)
                                .step_by(7)
                                .all(|x| p.workspace_rail_at(x as f32, y as f32).is_none())),
                            "{w}x{h} zoom {zoom} dpi {dpi}: no rail is drawn, so nothing \
                             may hit-test as one"
                        );
                        continue;
                    };
                    for i in 0..n {
                        let Some([rx, ry, rw, rh]) = drawn.rows.get(i).copied().flatten() else {
                            continue;
                        };
                        let (cx, cy) = (rx + rw * 0.5, ry + rh * 0.5);
                        assert_eq!(
                            p.workspace_rail_at(cx, cy),
                            Some(i),
                            "{w}x{h} zoom {zoom} dpi {dpi}: rail entry {i} is drawn at \
                             {rx},{ry} {rw}x{rh} but its own centre hit-tests elsewhere"
                        );
                        assert_eq!(
                            p.overlay_row_at(cx, cy),
                            None,
                            "{w}x{h} zoom {zoom} dpi {dpi}: a pointer in the rail column \
                             resolved to a settings ROW — the two bands must not overlap"
                        );
                        checked += 1;
                    }
                    // THE TWO REGIONS DO NOT OVERLAP where both are drawn: the
                    // content band starts at or after the rail column's right
                    // edge. On the NARROW primary stage the rail IS the
                    // workspace and no rows are drawn, so there is no second
                    // band to collide with — asserted as such rather than
                    // exempted.
                    let [rx, _, rw, _] = rail_box;
                    if p.workspace_is_wide(w) {
                        assert!(
                            drawn.pane_x >= rx + rw,
                            "{w}x{h} zoom {zoom} dpi {dpi}: the content pane ({}) starts \
                             inside the rail column ({}..{})",
                            drawn.pane_x,
                            rx,
                            rx + rw
                        );
                        assert!(
                            drawn.pane_w > 0.0,
                            "{w}x{h} zoom {zoom} dpi {dpi}: a drawn pane with no width"
                        );
                    } else {
                        assert_eq!(
                            drawn.visible, 0,
                            "{w}x{h} zoom {zoom} dpi {dpi}: the narrow primary stage \
                             shows the rail alone"
                        );
                    }
                }
            }
        }
    }
    assert!(
        checked >= 100,
        "the sweep only checked {checked} drawn rail entries — it is not \
         exercising the axis it claims"
    );
}

/// WIDE SHOWS BOTH REGIONS; NARROW STAGES THEM — and which stage narrow shows is
/// the SAME focus fact the lifecycle owns, never a second flag.
///
/// DESIGN.md §8 rule 4: narrow layouts stage multi-region workspaces with a back
/// path; they do not compress them. So the law asserts the shape of each regime
/// rather than a pixel count: wide draws a rail AND rows, narrow draws exactly
/// one of the two, and which one follows `detail_focus`.
#[test]
fn wide_shows_both_regions_and_narrow_stages_exactly_one() {
    let _g = crate::testlock::serial();
    let (device, queue, mut p) =
        headless_dqp(1400.0, 900.0).expect("workspace laws require a wgpu adapter");
    let mut wides = 0usize;
    let mut narrows = 0usize;
    for &(w, h) in CANVASES {
        for &detail in &[false, true] {
            let ov = workspace_card(0, detail);
            prepared(&device, &queue, &mut p, &ov, Cell::plain(w, h));
            let drawn = p.workspace_rail_probe(w);
            match p.workspace_is_wide(w) {
                true => {
                    wides += 1;
                    assert!(
                        drawn.rail.is_some(),
                        "{w}x{h} detail={detail}: a wide workspace draws its rail"
                    );
                    assert!(
                        drawn.visible > 0,
                        "{w}x{h} detail={detail}: a wide workspace draws its rows too"
                    );
                }
                false => {
                    narrows += 1;
                    assert_eq!(
                        drawn.rail.is_some(),
                        !detail,
                        "{w}x{h} detail={detail}: a narrow workspace stages ONE region, \
                         and which one is the lifecycle's focus stage"
                    );
                    assert_eq!(
                        drawn.visible > 0,
                        detail,
                        "{w}x{h} detail={detail}: the other region is not drawn at all"
                    );
                }
            }
        }
    }
    assert!(
        wides >= 4 && narrows >= 2,
        "the sweep must cross the staging threshold in both directions \
         (wide {wides}, narrow {narrows}) — otherwise one regime is untested"
    );
}

/// THE WORKSPACE TAKES THE VIEWPORT, and the document survives as a backdrop
/// rather than being erased.
///
/// Two claims a card cannot satisfy and a full-bleed takeover would fail from the
/// other side: the workspace's own surface covers most of the canvas (it
/// relocated attention), and a real margin of the canvas is NOT the workspace (it
/// left the document visible around itself). Measured on the geometry the pixels
/// come from, at every swept canvas.
#[test]
fn the_workspace_takes_the_viewport_and_leaves_the_document_framing_it() {
    let _g = crate::testlock::serial();
    let (device, queue, mut p) =
        headless_dqp(1400.0, 900.0).expect("workspace laws require a wgpu adapter");
    for &(w, h) in CANVASES {
        let ov = workspace_card(0, false);
        prepared(&device, &queue, &mut p, &ov, Cell::plain(w, h));
        let drawn = p.workspace_rail_probe(w);
        let [cx, cy, cw, ch] = drawn.card;
        let area = (cw * ch) / (w as f32 * h as f32);
        assert!(
            area > 0.60,
            "{w}x{h}: a workspace relocates attention — it covered only {:.0}% of \
             the canvas",
            area * 100.0
        );
        assert!(
            cx > 0.0 && cy > 0.0 && cx + cw < w as f32 && cy + ch < h as f32,
            "{w}x{h}: the document must still frame the workspace — card \
             {cx},{cy} {cw}x{ch} touches the canvas edge"
        );
    }
}

/// THE FOCUS CUE IS REAL INK, and it is the SAME rect at a different presence.
///
/// A workspace keeps a selection in both regions, so something has to say which
/// one is live. This asserts that in the pixels, not in the state: rendering the
/// identical card with focus on the rail and then on the content, the rail's own
/// band region must carry MORE ink when the rail has focus, and the content
/// band's region must carry more when the content does. A cue that existed only
/// in `ViewState` would pass a state check and fail here.
#[test]
fn the_focused_regions_marker_carries_more_ink_than_the_unfocused_ones() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let (device, queue, mut p) =
        headless_dqp(w as f32, h as f32).expect("workspace laws require a wgpu adapter");

    let on_rail = workspace_card(0, false);
    prepared(&device, &queue, &mut p, &on_rail, Cell::plain(w, h));
    let geom = p.workspace_rail_probe(w);
    let rail_mark = geom.mark.expect("the rail marks its active category");
    let row_band = geom.selected_band.expect("the content pane marks its row");
    let rail_focused = render_frame(&mut p, &device, &queue, w, h);

    let on_rows = workspace_card(0, true);
    prepared(&device, &queue, &mut p, &on_rows, Cell::plain(w, h));
    let rows_focused = render_frame(&mut p, &device, &queue, w, h);

    // The measurement is DIFFERENTIAL against the card's own ground, so the
    // world's palette, the dither and the backdrop all cancel: for each region,
    // how far its pixels sit from the frame's own most common colour.
    let energy = |px: &[[u8; 4]], r: [f32; 4]| -> f64 {
        let ground = px[(h as usize / 2) * w as usize + 4];
        let region = Region::new(r[0], r[1], r[2], r[3]);
        let mut sum = 0.0f64;
        for y in region.y.max(0)..(region.y + region.h).min(h as i64) {
            for x in region.x.max(0)..(region.x + region.w).min(w as i64) {
                let c = px[y as usize * w as usize + x as usize];
                for k in 0..3 {
                    sum += (c[k] as f64 - ground[k] as f64).abs();
                }
            }
        }
        sum
    };

    let rail_when_focused = energy(&rail_focused, rail_mark);
    let rail_when_not = energy(&rows_focused, rail_mark);
    assert!(
        rail_when_focused > rail_when_not * 1.15,
        "the rail's marker must read as LIVE when the rail has focus \
         ({rail_when_focused:.0}) and recede when it does not ({rail_when_not:.0})"
    );

    let rows_when_focused = energy(&rows_focused, row_band);
    let rows_when_not = energy(&rail_focused, row_band);
    assert!(
        rows_when_focused > rows_when_not * 1.15,
        "and the content band must do the same, the other way round \
         ({rows_when_focused:.0} focused vs {rows_when_not:.0} unfocused)"
    );
}

/// Files reuses the same generic row-focus signal as a workspace. This is a
/// pixel oracle: the selected band must remain physically present while query
/// or view chrome owns focus, yet carry less energy than when Choices owns it.
#[test]
fn files_selected_band_stays_present_but_recedes_when_another_control_has_focus() {
    let _g = crate::testlock::serial();
    let (w, h) = (1000u32, 720u32);
    let (device, queue, mut p) =
        headless_dqp(w as f32, h as f32).expect("Files focus law requires a wgpu adapter");
    let mut v = view("background\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "files  /".into();
    v.overlay_items = vec!["alpha.md".into(), "notes/  ›".into()];
    v.overlay_lens = vec![("Files".into(), true), ("Recent".into(), false)];
    v.overlay_hint = "type to search".into();
    v.overlay_rows_focused = true;
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();
    let focused = render_frame(&mut p, &device, &queue, w, h);

    v.overlay_rows_focused = false;
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();
    let unfocused = render_frame(&mut p, &device, &queue, w, h);
    let report = super::pixeldiff::diff_region(
        &focused,
        &unfocused,
        w as i64,
        h as i64,
        Region::canvas(w as i64, h as i64),
    );
    assert!(
        report.differing > 300,
        "the focus cue emitted no real pixel population: {report:?}"
    );
    assert!(
        report.max_channel_delta >= 20,
        "focused and selected were not visibly distinct: {report:?}"
    );

    let ground = unfocused[4];
    let presence = unfocused.iter().filter(|px| **px != ground).count();
    assert!(
        presence > 1_000,
        "the unfocused card/selection vanished: {presence} pixels"
    );
}

/// A CONTEXTUAL OVERLAY IS UNTOUCHED. The workspace family is entered only by a
/// card whose kind asks for it, so every other picker's geometry is byte-for-byte
/// what it was — asserted by rendering the command palette with the workspace
/// flag off and confirming no rail, no pane narrowing, and a card that does NOT
/// take the viewport.
#[test]
fn a_contextual_overlay_never_enters_the_workspace_family() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let (device, queue, mut p) =
        headless_dqp(w as f32, h as f32).expect("workspace laws require a wgpu adapter");
    let ov = OverlayState::new_command(
        crate::commands::visible_names(),
        crate::commands::visible_effective_bindings(&[], &[], crate::keymap::KeymapFlavor::Native),
        crate::commands::visible_hidden_mask(Default::default()),
    );
    let mut v = workspace_view(&ov);
    v.overlay_title = OverlayKind::Command.title().to_string();
    v.overlay_workspace = false;
    v.overlay_lens = ov.lens_strip();
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();

    assert!(
        !p.overlay_is_workspace(),
        "a palette is a contextual overlay, whatever its lens strip says"
    );
    assert_eq!(
        p.workspace_rail_at(50.0, 200.0),
        None,
        "and it has no rail to hit-test"
    );
    let drawn = p.workspace_rail_probe(w);
    assert!(drawn.rail.is_none(), "no rail column is resolved for it");
    let [_, _, cw, ch] = drawn.card;
    assert!(
        (cw * ch) / (w as f32 * h as f32) < 0.45,
        "a contextual card keeps the document readable behind it; it covered \
         {:.0}% of the canvas",
        (cw * ch) / (w as f32 * h as f32) * 100.0
    );
}

/// THE WORKSPACE'S FOOTER HAS EXACTLY ONE VISIBLE CARRIER AND FITS IT — every
/// workspace kind, world, focus stage, width regime, menu-bar branch and DPI.
///
/// The general footer no-clip law (`chrome_panels`'s
/// `jump_hint_is_present_and_never_clips_for_every_kind`) measures the FLAT
/// card at one canvas over three worlds. That is the tightest budget for a
/// contextual picker and says nothing about a workspace, whose card comes from
/// the CANVAS and whose two stages advertise two different lines — so a
/// narrow canvas on a world with a wide chrome face is a budget nothing was
/// watching. A vision-smoke pass over this round's own captures found exactly
/// that: a footer cell added to the rows line ran off the card on Firetail at
/// 900x520 while the whole suite stayed green. This is the law that was missing.
///
/// Both STAGES are graded, because they carry different sentences (the primary
/// list's `rail_hint_actions`, the detail region's own actions), and the
/// measurement is the shaped run in whichever buffer the frame actually
/// uploads, never the hint STRING. On a narrow Settings primary stage that is
/// the rail buffer; elsewhere it is the panel buffer. This distinction is part
/// of the oracle: reading only the panel made a correctly-shaped Tawny rail
/// footer look absent, while also missing a genuinely absent timeline-detail
/// footer because that stage has no rows.
fn representative_workspace_card(kind: OverlayKind, detail: bool) -> OverlayState {
    let card = match kind {
        OverlayKind::Settings => workspace_card(0, false),
        OverlayKind::Conflict => OverlayState::new_conflict(
            std::path::PathBuf::from("/notes/a.md"),
            Some("disk version\n".to_string()),
        ),
        OverlayKind::Credits => OverlayState::new_credits(),
        // Footer composition only needs real rows and the kind's real lifecycle;
        // the History constructor's timestamp metadata does not enter shaping.
        _ => OverlayState::new(
            kind,
            vec!["a representative row".to_string()],
            Vec::new(),
            Vec::new(),
        ),
    };
    let mut journey = crate::overlay::Journey::seeded(Some(card));
    if detail {
        journey.toggle_detail();
    }
    journey.card().expect("the workspace card is up").clone()
}

#[derive(Clone, Copy)]
struct FooterCell {
    logical_w: u32,
    logical_h: u32,
    dpi: f32,
    kind: OverlayKind,
    detail: bool,
    menu_bar: bool,
    world: &'static str,
    zoom: f32,
}

#[derive(Clone, Copy, PartialEq)]
struct FooterOutcome {
    geometry: (f32, f32, f32, f32),
    panel_carrier: bool,
    wide: bool,
}

fn grade_footer_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    cell: FooterCell,
) -> FooterOutcome {
    let card = representative_workspace_card(cell.kind, cell.detail);
    let mut v = workspace_view(&card);
    let rows_primary = card
        .workspace_shape()
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    v.overlay_comparison = rows_primary;
    assert!(v.overlay_active && v.overlay_workspace);
    assert_eq!(v.overlay_detail_focus, cell.detail);
    assert_eq!(v.overlay_selected, card.selected);
    assert_eq!(v.overlay_hint, card.foot_hint());
    assert!(
        !v.overlay_hint.is_empty(),
        "{} detail={}: the lifecycle promised no teaching line",
        cell.kind.as_str(),
        cell.detail
    );

    let (cw, ch) = (
        (cell.logical_w as f32 * cell.dpi) as u32,
        (cell.logical_h as f32 * cell.dpi) as u32,
    );
    p.set_dpi(cell.dpi);
    p.set_size(cw as f32, ch as f32);
    v.zoom = cell.zoom;
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).unwrap();
    let (panel_px, panel_w, rail_px, rail_w) = p.workspace_footer_carriers_probe(cw);
    let carriers = usize::from(panel_px > 1.0) + usize::from(rail_px > 1.0);
    let wide = p.workspace_is_wide(cw);
    let regime = if wide { "wide" } else { "staged" };
    let stage = if cell.detail { "detail" } else { "primary" };
    let ctx = format!(
        "{}/{}/{}x{}/{regime}/{stage}/menu={}/dpi={}/zoom={}",
        cell.world,
        cell.kind.as_str(),
        cell.logical_w,
        cell.logical_h,
        cell.menu_bar,
        cell.dpi,
        cell.zoom,
    );
    assert_eq!(
        carriers, 1,
        "{ctx}: exactly one visible buffer must shape the promised footer; \
         panel={panel_px:.1}px rail={rail_px:.1}px"
    );
    let (footer_px, text_w) = if panel_px > 1.0 {
        (panel_px, panel_w)
    } else {
        (rail_px, rail_w)
    };
    assert!(
        footer_px <= text_w + 0.01,
        "{ctx}: the workspace footer shapes {footer_px:.1}px but its visible carrier is \
         {text_w:.1}px wide — the line is clipped, and the footer is awl's only statement \
         of what a key does"
    );
    if rows_primary && cell.detail && !wide {
        let (_, _, footer_bottom) = p
            .overlay_hint_gap_probe(cw)
            .unwrap_or_else(|| panic!("{ctx}: the detail footer has no shaped band"));
        let [_, comparison_top, _, _] = p
            .comparison_viewport()
            .unwrap_or_else(|| panic!("{ctx}: the detail comparison has no viewport"));
        assert!(
            comparison_top >= footer_bottom - 0.01,
            "{ctx}: comparison prose starts at {comparison_top:.1}, under its footer ending at \
             {footer_bottom:.1}"
        );
    }
    FooterOutcome {
        geometry: (panel_px, panel_w, rail_px, rail_w),
        panel_carrier: panel_px > 1.0,
        wide,
    }
}

#[test]
fn the_workspace_footer_fits_its_card_on_every_world_at_every_stage() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_workspace_footer_fits_its_card: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    let mut panel_carriers = 0usize;
    let mut rail_carriers = 0usize;
    let mut wide = 0usize;
    let mut staged = 0usize;
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let workspace_kinds: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|kind| kind.workspace_shape().is_some())
        .collect();
    assert!(
        !workspace_kinds.is_empty(),
        "the workspace roster must enroll at least one kind"
    );
    for menu_bar in [false, true] {
        crate::menubar::set_menu_bar_on(menu_bar);
        for world in crate::theme::THEMES {
            crate::theme::set_active_by_name(world.name).expect("a roster world");
            p.sync_theme();
            for &(logical_w, logical_h) in
                &[(1200u32, 800u32), (900, 520), (760, 620), (1600, 1000)]
            {
                for dpi in [1.0f32, 2.0] {
                    for kind in &workspace_kinds {
                        for detail in [false, true] {
                            let mut zoom_outcome = None;
                            for zoom in [0.8f32, 1.6] {
                                let outcome = grade_footer_cell(
                                    &device,
                                    &queue,
                                    &mut p,
                                    FooterCell {
                                        logical_w,
                                        logical_h,
                                        dpi,
                                        kind: *kind,
                                        detail,
                                        menu_bar,
                                        world: world.name,
                                        zoom,
                                    },
                                );
                                if outcome.wide {
                                    wide += 1;
                                } else {
                                    staged += 1;
                                }
                                if outcome.panel_carrier {
                                    panel_carriers += 1;
                                } else {
                                    rail_carriers += 1;
                                }
                                if let Some(expected) = zoom_outcome {
                                    assert_eq!(
                                        outcome.geometry,
                                        expected,
                                        "{}/{logical_w}x{logical_h}/detail={detail}/dpi={dpi}: \
                                         document zoom changed UI-metric footer geometry",
                                        kind.as_str()
                                    );
                                } else {
                                    zoom_outcome = Some(outcome.geometry);
                                }
                                graded += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    assert_eq!(
        graded,
        crate::theme::THEMES.len() * workspace_kinds.len() * 64,
        "every menu x world x canvas x DPI x kind x stage x zoom cell must be graded"
    );
    assert!(
        panel_carriers > 0 && rail_carriers > 0,
        "both real footer carriers must enroll (panel={panel_carriers}, rail={rail_carriers})"
    );
    assert!(
        wide > 0 && staged > 0,
        "the sweep must cross the workspace threshold (wide={wide}, staged={staged})"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// **THE CONTENT PANE HUGS A MAXIMUM WIDTH.** Without a ceiling, a
/// `RailOverRows` workspace's pane fills every pixel a wide window has to
/// spare, so a row's own value drifts further from its name the wider the
/// window grows — measured live at 2400x1000 with the label and value
/// standing at opposite ends of the card, ~900px apart. That is exactly a
/// "longer journey between a setting and its value", the failure mode the
/// pane's authored ceiling exists to close; this law drives an actually-too-wide
/// window and requires the pane to stop growing at its own authored ceiling
/// instead, over the whole world roster (an authored `hpad` or rail-label
/// width is a per-world number, so a roster sweep is what proves the ceiling
/// — not a coincidence of one world's own measurements).
///
/// Two non-vacuity arms keep this from being trivially true: the sweep must
/// contain a wide cell where the ceiling actually binds and a narrow cell that
/// deliberately stages one region. That pair proves both sides of the
/// responsive decision; sampling only roomy windows would hide a cap that made
/// the staged composition unreachable.
#[test]
fn the_content_pane_hugs_a_maximum_width_across_the_roster() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_content_pane_hugs_a_maximum_width_across_the_roster: no wgpu adapter"
        );
        return;
    };
    let ov = workspace_card(0, true);
    let mut bound_hit: Vec<&str> = Vec::new();
    let mut staged_seen: Vec<&str> = Vec::new();
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        for &(w, h) in &[
            (520u32, 620u32),
            (680, 700),
            (900, 760),
            (1200, 800),
            (1800, 900),
            (2600, 1200),
            (3600, 1400),
        ] {
            prepared(&device, &queue, &mut p, &ov, Cell::plain(w, h));
            if !p.workspace_is_wide(w) {
                staged_seen.push(world.name);
                continue;
            }
            let drawn = p.workspace_rail_probe(w);
            let max_px = p.workspace_max_pane_probe();
            assert!(
                (360.0..=520.0).contains(&max_px),
                "{}: the authored Settings detail ceiling is {max_px:.1} logical px, \
                 outside the approved 360..=520 band",
                world.name
            );
            assert!(
                drawn.pane_w <= max_px + 0.5,
                "{}/{w}x{h}: the content pane is {:.1}px wide against an authored \
                 ceiling of {max_px:.1}px — extra window width must become the \
                 card's own ground, not a longer reach to a row's value",
                world.name,
                drawn.pane_w
            );
            if (drawn.pane_w - max_px).abs() <= 1.0 {
                bound_hit.push(world.name);
            }
            graded += 1;
        }
    }
    p.set_size(1200.0, 800.0);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(
        graded > 0,
        "the sweep graded nothing — no wide cell was reached"
    );
    assert!(
        !bound_hit.is_empty(),
        "the sweep never actually reached the ceiling on any world — a wider canvas \
         is needed to prove this law is not vacuously true"
    );
    assert!(
        !staged_seen.is_empty(),
        "the sweep never saw the staged side of the responsive threshold \
         (cap-bound cells: {bound_hit:?}) — a cap law must prove the narrow \
         composition remains reachable rather than grading only roomy windows"
    );
}

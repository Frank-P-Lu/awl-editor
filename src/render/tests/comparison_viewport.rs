//! The relocated document viewport.
//!
//! awl has exactly ONE prose renderer, the document layer, so a workspace whose
//! content region holds prose has to MOVE it rather than grow a second one (the
//! "infrastructure complexity is a smell" CLAUDE.md forbids).
//! `TextPipeline::comparison_viewport` is that move's one owner, and these are
//! the laws that keep it one:
//!
//! 1. all FOUR document-geometry owners read it, by name;
//! 2. the unrelocated PAGE column is a module-private bypass with a named,
//!    enumerated consumer set — not a second geometry anyone may reach for;
//! 3. the whole document layer relocates TOGETHER (column, wrap, caret,
//!    selection, clip), and returns together;
//! 4. the region opens on the same line the workspace's own rows do;
//! 5. every MARGIN-ORIENTATION surface yields while it is up — including the
//!    two that reach that conclusion through item 34's `overlay_active` gate,
//!    which is proven here rather than trusted;
//! 6. the relocated document is contained by a surface a reader can SEE, in
//!    every world including the one-bit Wagtail (the claim inherited from item
//!    84's diff-panel dressing law, re-aimed at the composition that replaced
//!    it).
//!
//! COMPOSITION — whether any of it reaches the screen — is item 116d's, in
//! `render/tests/comparison_composite.rs`. This file's own boundary
//! law (`the_relocated_document_is_geometrically_placed_but_not_yet_composited`)
//! asked to be deleted and replaced by a containment-and-visibility law the day
//! the workspace drew comparison content; that day is 116d, and it was.
//!
//! **NOTHING REACHES THIS TODAY.** `OverlayKind::workspace_shape` is
//! `Some(RailOverRows)` for Settings alone and `None` for History until item
//! 116d, and `RailOverRows::rows_are_primary()` is `false` — so
//! `comparison_viewport()` is `None` on every frame the product can produce,
//! and the relocation is a structural change with zero pixel change (proven by
//! this item's world × surface fingerprint matrix, not asserted). The fixtures
//! below drive `ViewState::overlay_workspace` / `overlay_rows_primary`
//! directly, which are the SAME flat projections `sync_view` will set once
//! 116d flips the kind — the production seam, not a test-only door.

use super::super::TextPipeline;
use super::{comparison_view, headless_dqp, view};

/// The document-geometry owners, and the file each one's body lives in. Named
/// individually so a future owner that skips the relocation has to dodge an
/// explicit line here rather than a glob that would pass it by.
const DOCUMENT_GEOMETRY_OWNERS: &[(&str, &str)] = &[
    ("geometry.rs", "pub fn column_left(&self) -> f32 {"),
    ("geometry.rs", "pub fn column_width(&self) -> f32 {"),
    ("geometry.rs", "fn doc_top(&self) -> f32 {"),
    (
        "chrome/mod.rs",
        "fn doc_clip_band(&self) -> Option<(f32, f32)> {",
    ),
];

fn render_src(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/render")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{rel} must exist"))
}

/// The body of the function whose signature line is `sig`: from that line to the
/// next sibling item at the same indent, or EOF.
fn body_of<'a>(src: &'a str, sig: &str, rel: &str) -> &'a str {
    let start = src
        .find(sig)
        .unwrap_or_else(|| panic!("{rel}: missing `{sig}`"));
    let rest = &src[start..];
    let end = rest[sig.len()..]
        .find("\n    }")
        .map(|i| i + sig.len() + 6)
        .unwrap_or(rest.len());
    &rest[..end]
}

/// LAW 1 — THE FOUR OWNERS. Every document consumer in the tree routes through
/// one of these four (~45 call sites across `rects.rs`, `layers.rs`, `text.rs`,
/// `geometry.rs`, `scroll.rs`), so giving all four the override is what makes
/// the relocation total rather than a fifth thing to remember. By name,
/// no-wildcard.
#[test]
fn all_four_document_geometry_owners_read_the_comparison_viewport() {
    for &(rel, sig) in DOCUMENT_GEOMETRY_OWNERS {
        let src = render_src(rel);
        let body = body_of(&src, sig, rel);
        assert!(
            body.contains("comparison_viewport("),
            "{rel}: `{sig}` must read `comparison_viewport()` — it is one of the four owners \
             every document consumer composes off, and an owner that skips the relocation \
             leaves the document layer half-moved"
        );
    }
}

/// LAW 2 — THE BYPASS IS PRIVATE, AND ITS CONSUMERS ARE ENUMERATED.
///
/// Two ideas of "the writing column" now exist: the DOCUMENT's (relocates) and
/// the PAGE's on the canvas (never does). The second is real — the ground punch
/// and the star margin band genuinely describe the backdrop — but it is exactly
/// the shape that becomes a parallel geometry if anyone may reach for it. It
/// stays `pub(in crate::render)` and this law pins its call sites to exactly two
/// places: `render/geometry/page.rs`, which DEFINES it and holds its own
/// consumers (`page_geometry`, the one public seam, and the page-resize
/// hit-test, which reads the canvas edges in order to decide it must NOT arm),
/// and the two FALLBACK arms inside `column_left`/`column_width` themselves —
/// the whole point of the split, and the only lines outside the owner file that
/// may name it.
#[test]
fn the_unrelocated_page_column_has_exactly_the_named_consumers() {
    let render_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render");
    let mut hits: Vec<String> = Vec::new();
    scan(&render_root, &render_root, &mut hits);
    let strays: Vec<&String> = hits
        .iter()
        .filter(|h| !h.starts_with(BYPASS_OWNER) && !h.starts_with(BYPASS_FALLBACK))
        .collect();
    assert!(
        strays.is_empty(),
        "`page_column_left`/`page_column_width` are item 116b's module-private BYPASS — the \
         backdrop's own column, which the relocated document deliberately does not move. A \
         consumer outside `render/geometry/page.rs` naming one is building a second geometry \
         to keep in sync; read `column_left()`/`column_width()` and follow the document, or \
         yield with the margin family. offending lines:\n{}",
        strays
            .iter()
            .map(|s| format!("  {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    // NON-VACUOUS: the owner file really does carry them, so a rename that
    // emptied this scan trips here instead of going quiet.
    assert!(
        hits.iter().filter(|h| h.starts_with(BYPASS_OWNER)).count() >= 4,
        "the bypass must actually exist in `{BYPASS_OWNER}` (2 definitions + `page_geometry` \
         + the page-resize hit-test); found {hits:?}"
    );
    // …and the fallback arms are exactly the two the split exists for — a third
    // means a document owner started reading the backdrop's column.
    assert_eq!(
        hits.iter()
            .filter(|h| h.starts_with(BYPASS_FALLBACK))
            .count(),
        BYPASS_FALLBACK_ARMS,
        "`{BYPASS_FALLBACK}` may name the page column ONLY in `column_left`'s and \
         `column_width`'s own fallback arms; found {hits:?}"
    );
}

/// The file that OWNS the unrelocated page column.
const BYPASS_OWNER: &str = "geometry/page.rs:";
/// The only other file that may name it — and only in the two fallback arms of
/// the document owners it is the fallback FOR.
const BYPASS_FALLBACK: &str = "geometry.rs:";
const BYPASS_FALLBACK_ARMS: usize = 2;

fn scan(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                continue;
            }
            scan(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs")
            || path.file_name().and_then(|n| n.to_str()) == Some("tests.rs")
        {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("///") {
                continue;
            }
            if line.contains("page_column_left(") || line.contains("page_column_width(") {
                out.push(format!("{rel}:{}", i + 1));
            }
        }
    }
}

/// The RELOCATED beat of law 3, lifted out so the law itself reads as its three
/// beats (baseline / relocated / returned) rather than as one long block.
fn assert_relocated(p: &TextPipeline, vp: [f32; 4]) {
    let [vx, vy, vw, vh] = vp;
    assert_eq!(
        p.column_left(),
        vx,
        "column_left must be the region's own x"
    );
    assert_eq!(
        p.column_width(),
        vw,
        "column_width must be the region's own w"
    );
    assert_eq!(
        p.doc_top(),
        vy,
        "doc_top at scroll 0 must be the region's own y"
    );
    assert_eq!(
        p.doc_clip_band(),
        Some((vy, vy + vh)),
        "doc_clip_band must be the region's own vertical extent — item 84's clip, re-owned"
    );
    let clip = p.content_clip();
    assert_eq!(
        (clip.0, clip.2),
        (vx, vx + vw),
        "the content clip's X arm must be the region, not the page column"
    );
    assert!(
        p.text_left() >= vx && p.text_left() < vx + vw,
        "the text origin ({}) must sit inside the region {vp:?}",
        p.text_left()
    );
    assert!(
        p.text_wrap_width() <= vw + 0.01,
        "the wrap width ({}) must fit the region's own width ({vw})",
        p.text_wrap_width()
    );
    for r in p.selection_rects() {
        assert!(
            r[0] >= vx - 0.01 && r[0] + r[2] <= vx + vw + 0.01,
            "a selection quad {r:?} escaped the relocated region {vp:?}"
        );
    }
}

/// LAW 3 — THE WHOLE LAYER MOVES, AND THE WHOLE LAYER COMES BACK.
///
/// Not just the column: the wrap width, the text origin, the vertical origin,
/// the caret, a selection quad and the content clip all follow, because they
/// compose off the four owners rather than carrying their own placement. And
/// the SAME view with `rows_are_primary` false — Settings' own shape, the one
/// any kind reaches today — leaves every one of them exactly where an ordinary
/// frame puts them.
#[test]
fn the_comparison_viewport_relocates_the_entire_document_layer_and_returns_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_comparison_viewport_relocates_the_entire_document_layer: no adapter"
        );
        return;
    };
    let text: String = (0..40)
        .map(|i| format!("line {i} of the comparison\n"))
        .collect();

    // The BASELINE: an ordinary editing frame, nothing summoned.
    let plain = view(&text, 3, 2);
    p.set_view(&plain);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let base = (
        p.column_left(),
        p.column_width(),
        p.doc_top(),
        p.text_left(),
        p.doc_clip_band(),
    );
    assert!(
        base.4.is_none(),
        "an ordinary frame clips the document to nothing at all, got {:?}",
        base.4
    );

    // RELOCATED.
    let mut v = comparison_view(&text, 3, 2);
    v.selection = Some(((3, 0), (5, 4)));
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let vp = p
        .comparison_viewport()
        .expect("a rows-primary workspace with a visible content region relocates the document");
    let [vx, _vy, vw, vh] = vp;
    assert!(vw > 0.0 && vh > 0.0, "the region must be real: {vp:?}");

    assert_relocated(&p, vp);

    // The relocation is a genuine MOVE, not a coincidence that matches the page.
    assert!(
        (vx - base.0).abs() > 1.0,
        "precondition: the region must sit somewhere the page column does not \
         (region x {vx}, page column x {})",
        base.0
    );

    // AND IT COMES BACK — the same workspace with Settings' own shape.
    let mut settings_shaped = comparison_view(&text, 3, 2);
    settings_shaped.overlay_rows_primary = false;
    p.set_view(&settings_shaped);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        p.comparison_viewport().is_none(),
        "`rows_are_primary() == false` is Settings' shape: its rows live in the content \
         pane, so there is no comparison region and the document must not relocate"
    );
    assert_eq!(
        (
            p.column_left(),
            p.column_width(),
            p.doc_top(),
            p.text_left(),
            p.doc_clip_band()
        ),
        base,
        "a workspace that is not a comparison must leave the document layer byte-identical \
         to an ordinary editing frame"
    );
}

/// LAW 4 — THE REGION OPENS ON THE LINE THE ROWS DO.
///
/// `comparison_viewport` derives its own top (item 174 forbids a consumer
/// re-deriving a candidate ROW's slot, and a document region is not one — but
/// two independent derivations of "below the header beat" is exactly the drift
/// shape that law exists to prevent). So the two are pinned to each other here,
/// against the real plan the frame drew, across canvases and zooms.
#[test]
fn the_comparison_viewport_opens_on_the_same_line_the_rows_do() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_comparison_viewport_opens_on_the_same_line_the_rows_do: no adapter"
        );
        return;
    };
    let text: String = (0..30).map(|i| format!("line {i}\n")).collect();
    let mut graded = 0usize;
    for (cw, ch) in [(1200u32, 800u32), (1600, 1000), (900, 700), (1400, 1600)] {
        for zoom in [1.0f32, 1.6] {
            p.set_size(cw as f32, ch as f32);
            let mut v = comparison_view(&text, 0, 0);
            v.zoom = zoom;
            p.set_view(&v);
            p.prepare(&device, &queue, cw, ch).unwrap();
            let Some([_, vy, _, vh]) = p.comparison_viewport() else {
                continue;
            };
            let geom = p.workspace_geometry(cw);
            let plan = p.overlay_row_plan(&geom);
            let ctx = format!("{cw}x{ch} zoom={zoom}");
            assert!(
                (vy - plan.first_top()).abs() < 0.51,
                "{ctx}: the comparison region opens at {vy} but the timeline's first row is \
                 planned at {} — the two derivations of the header beat have drifted",
                plan.first_top()
            );
            let rail_bottom = p.workspace_rail_box(&geom, &plan).map(|[_, t, _, h]| t + h);
            if let Some(rb) = rail_bottom {
                assert!(
                    (vy + vh - rb).abs() < 0.51,
                    "{ctx}: the comparison region ends at {} but the rail's column ends at \
                     {rb} — both run to the workspace's own bottom pad",
                    vy + vh
                );
            }
            graded += 1;
        }
    }
    p.set_dpi(1.0);
    assert!(
        graded >= 6,
        "the sweep must actually grade cells, got {graded}"
    );
}

/// The CONTROL beat of law 5: every margin-orientation surface must genuinely be
/// ON before "it yields" can mean anything. Returns how many frost seeds this
/// world's control frame carried, so the caller can prove the lava arm is live.
fn assert_control_frame_draws_the_margin_family(p: &mut TextPipeline, world: &str) -> usize {
    assert!(
        p.outline_visible(800),
        "{}: precondition — the margin outline must be drawn on the control frame",
        world
    );
    assert!(
        p.gutter_visible(),
        "{}: precondition — the gutter must be drawn on the control frame",
        world
    );
    let page_edge = p.page_column_left();
    assert!(
        p.page_resize_edge_at(page_edge).is_some(),
        "{}: precondition — the page's own left edge must be draggable on the control \
         frame",
        world
    );
    assert!(
        !p.wordcount_readout_text().is_empty() && !p.notice_readout_text().is_empty(),
        "{}: precondition — both corner readouts must speak on the control frame",
        world
    );
    p.outline_frost_seeds(800).len() + p.gutter_frost_seeds(800).len()
}

/// LAW 5 — EVERY MARGIN-ORIENTATION SURFACE YIELDS.
///
/// The persistent chrome DESIGN.md §5 bounds to answering "where am I?" and
/// "how much?" composes off the four relocated owners, and none of it has
/// anything true to say about a read-only comparison of two versions the user
/// is not editing. The roster is NAMED, not globbed — and it deliberately
/// includes the outline and the gutter, which reach the same conclusion through
/// item 34's `overlay_active` gate: a subsumption is only worth relying on once
/// it is watched.
///
/// Swept over the whole world roster, because the lava worlds carry a margin
/// surface (frost seeds + pills) the other worlds do not.
#[test]
fn every_margin_orientation_surface_yields_to_a_relocated_document() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping every_margin_orientation_surface_yields: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);

    // A headed markdown doc with a long tail: an outline, a gutter, a word
    // count and a page frame all have something to draw on an ordinary frame.
    let mut text = String::new();
    for i in 0..8 {
        text.push_str(&format!(
            "# Heading {i}\n\nSome prose under heading {i}.\n\n"
        ));
    }

    let mut frost_worlds = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();

        // CONTROL — an ordinary editing frame: the surfaces must genuinely be
        // ON, or the yield below proves nothing.
        let mut ordinary = view(&text, 0, 0);
        ordinary.is_markdown = true;
        ordinary.gutter_name = "notes.md".into();
        ordinary.gutter_project = "awl".into();
        ordinary.notice = "changed on disk outside awl".into();
        p.set_view(&ordinary);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let control_frost = assert_control_frame_draws_the_margin_family(&mut p, world.name);
        if control_frost > 0 {
            frost_worlds += 1;
        }
        let page_edge = p.page_column_left();

        // RELOCATED — every one of them yields.
        let mut relocated = comparison_view(&text, 0, 0);
        relocated.is_markdown = true;
        relocated.gutter_name = "notes.md".into();
        relocated.gutter_project = "awl".into();
        relocated.notice = "changed on disk outside awl".into();
        p.set_view(&relocated);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        assert!(
            p.comparison_viewport().is_some(),
            "{}: precondition — the document must actually be relocated",
            world.name
        );
        let w = world.name;
        assert!(
            p.margin_orientation_yields(),
            "{w}: the one gate must be on"
        );
        assert!(
            !p.outline_visible(800),
            "{w}: the margin OUTLINE must yield"
        );
        assert!(
            !p.gutter_visible(),
            "{w}: the bottom-left GUTTER must yield"
        );
        assert_eq!(
            p.page_frame_pipeline.instance_count(),
            0,
            "{w}: the PAGE FRAME must yield — it draws the writing page's own edge, and \
             the page on screen is not the user's"
        );
        for x in [
            page_edge,
            page_edge + p.page_column_width(),
            p.column_left(),
            p.column_left() + p.column_width(),
        ] {
            assert!(
                p.page_resize_edge_at(x).is_none(),
                "{w}: the draggable PAGE EDGE must yield at x={x} — a measure drag against \
                 a read-only comparison's own edge is a page resize with no page"
            );
        }
        assert!(
            p.wordcount_readout_text().is_empty() && p.notice_readout_text().is_empty(),
            "{w}: the corner READOUTS (word count / calm notice) must yield, got \
             wordcount {:?} notice {:?}",
            p.wordcount_readout_text(),
            p.notice_readout_text()
        );
        assert!(
            p.outline_frost_seeds(800).is_empty() && p.gutter_frost_seeds(800).is_empty(),
            "{w}: the LAVA FROST seeds must yield with the margin ink they hug"
        );
        assert!(
            p.lava_frost_pill_rects(800).is_empty(),
            "{w}: the LAVA FROST pills must yield with the outline they carry"
        );
    }

    assert!(
        frost_worlds >= 2,
        "the frost arm must actually see the lava worlds' seeds on the control frame, got \
         {frost_worlds}"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

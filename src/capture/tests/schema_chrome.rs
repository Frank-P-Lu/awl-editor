//! Retina/narrow-margin geometry captures, the sidecar SCHEMA well-formedness
//! law, the buffers/syntax/page blocks, fenced-code-syntax highlighting, and
//! the markdown highlight/table tags -- split out of the former monolithic
//! `capture::tests` (2026-07 code-organization pass).

use super::super::*;
use super::{adapter_available, sidecar};
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

struct TypewriterRestore(bool);

impl Drop for TypewriterRestore {
    fn drop(&mut self) {
        crate::typewriter::set_typewriter_on(self.0);
    }
}

/// A long document with the caret advanced twenty lines is the non-vacuous
/// typewriter case: minimal reveal and centered pinning produce different scroll
/// offsets. Plain, timeline, and held captures must choose the same initial
/// viewport; the animated modes may then keep that viewport fixed.
#[test]
fn typewriter_scroll_initial_viewport_is_shared_by_plain_timeline_and_held() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping shared typewriter viewport test: no wgpu adapter");
        return;
    }
    let _restore = TypewriterRestore(crate::typewriter::typewriter_on());
    crate::typewriter::set_typewriter_on(true);

    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_typewriter_animated_{}", std::process::id())),
    );
    let text: String = (0..40).map(|line| format!("line {line:02}\n")).collect();
    let mut buf = Buffer::from_str(&text);
    buf.set_cursor(buf.line_col_to_char(20, 0));
    let opts = CaptureOpts::default();

    let plain = dir.join("plain.png");
    capture_with(&plain, &buf, &opts).expect("plain typewriter capture");
    let timeline = dir.join("timeline.png");
    capture_timeline(&timeline, &buf, (19, 0), &[0], &opts).expect("typewriter timeline");
    let held = dir.join("held.png");
    capture_held(&held, &buf, (20, 0), HeldDir::Down, &[0], &opts)
        .expect("typewriter held capture");

    let scroll_top = |path: &std::path::Path| {
        let text = std::fs::read_to_string(path).expect("read sidecar");
        let value: serde_json::Value = serde_json::from_str(&text).expect("parse sidecar");
        value["scroll_top_px"]
            .as_f64()
            .expect("numeric scroll_top_px")
    };
    let plain_top = scroll_top(&plain.with_extension("json"));
    let timeline_top = scroll_top(&dir.join("timeline.t0.json"));
    let held_top = scroll_top(&dir.join("held.t0.json"));
    assert!(
        plain_top > 0.0,
        "precondition: line 20 in a 40-line document must require a real typewriter scroll"
    );
    assert_eq!(
        timeline_top, plain_top,
        "timeline initial viewport must use the shared typewriter policy"
    );
    assert_eq!(
        held_top, plain_top,
        "held initial viewport must use the shared typewriter policy"
    );
}

/// The harness now reproduces the margin-class geometry: a capture at a REAL
/// retina size (2400x1600 @ dpi 2.0) yields a page column CENTERED with a margin
/// on BOTH sides (left == right within rounding, both > 0) — the assertion the old
/// hardcoded 1200/dpi-1 capture could never make. And the DEFAULT (no size/dpi)
/// column geometry is byte-for-byte unchanged (left=120, width=960 at 1200).
#[test]
fn retina_capture_centers_page_column_symmetrically() {
    // Pin before the adapter/pipeline path: both runs below deliberately shape
    // at different page inputs, and every return (including no adapter / GPU
    // error unwinds) must hand the caller exactly its incoming inputs back.
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    if !adapter_available() {
        eprintln!("skipping retina_capture_centers_page_column_symmetrically: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_capture_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str(
        "the quick brown fox jumps over the lazy dog\nsecond line of prose here\nand a third line to fill the page",
    );

    // --- RETINA run: 2400x1600 @ dpi 2.0, narrow column so margins are real. ---
    crate::page::set_page_on(true);
    crate::page::set_measure(40);
    let retina_png = dir.join("retina.png");
    let opts = CaptureOpts {
        canvas: Some((2400, 1600)),
        dpi: Some(2.0),
        ..CaptureOpts::default()
    };
    capture_with(&retina_png, &buf, &opts).expect("retina capture");
    let json = std::fs::read_to_string(retina_png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    let cw = value["canvas"]["width"]
        .as_f64()
        .expect("canvas.width number");
    let dpi = value["canvas"]["dpi"].as_f64().expect("canvas.dpi number");
    let left = value["page"]["column"]["left"]
        .as_f64()
        .expect("page.column.left number");
    let width = value["page"]["column"]["width"]
        .as_f64()
        .expect("page.column.width number");
    assert_eq!(
        cw, 2400.0,
        "sidecar canvas.width self-describes the physical size"
    );
    assert_eq!(
        dpi, 2.0,
        "sidecar canvas.dpi self-describes the scale factor"
    );
    let right = 2400.0 - (left + width);
    assert!(
        left > 0.0,
        "retina page column needs a LEFT margin, got {left}"
    );
    assert!(
        right > 0.0,
        "retina page column needs a RIGHT margin, got {right}"
    );
    assert!(
        (left - right).abs() <= 1.0,
        "retina page column must be CENTERED: left {left} vs right {right}"
    );

    // --- DEFAULT run: no size/dpi flags -> unchanged 1200/dpi-1 geometry. ---
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let def_png = dir.join("default.png");
    capture_with(&def_png, &buf, &CaptureOpts::default()).expect("default capture");
    let djson = std::fs::read_to_string(def_png.with_extension("json")).unwrap();
    let default = sidecar(&djson);
    let dleft = default["page"]["column"]["left"]
        .as_f64()
        .expect("page.column.left number");
    let dwidth = default["page"]["column"]["width"]
        .as_f64()
        .expect("page.column.width number");
    // The standard prose measure is a visibly centered 70-character column.
    assert!(
        (dleft - 96.0).abs() <= 0.5 && (dwidth - 1008.0).abs() <= 0.5,
        "default column geometry: left ~96, width ~1008 (prose measure binds), got left {dleft} width {dwidth}"
    );
    // The no-flag sidecar must NOT carry a dpi key (byte-stable canvas block).
    assert!(
        default["canvas"].get("dpi").is_none(),
        "no-flag sidecar canvas block must omit dpi for byte-identity: {}",
        default["canvas"]
    );
}

/// THE GUTTER-ELISION BUG, end to end through the real capture path: a narrow
/// (but real, not degenerate) page-mode margin used to lay the raw filename into
/// a fixed-width WRAPPING box, so a long name wrapped mid-word and the
/// fixed-height box clipped the project line right off underneath it. THE FIX
/// (corrected by a taste pass over the first landing): both lines pre-fit to
/// ONE line each and elide INDEPENDENTLY — neither yields to the other from
/// width pressure. Driven at a real `--capture-size` + `--measure`-equivalent
/// (`CaptureOpts::canvas` + `page::set_measure`, the flags this exact scenario
/// is reproduced with), this asserts the SIDECAR (not just the pipeline unit
/// test) shows: a one-line, extension-preserving elided filename, with the
/// (short-enough) project line still showing right alongside it.
#[test]
fn narrow_margin_capture_gutter_never_wraps_and_both_lines_stay_visible() {
    if !adapter_available() {
        eprintln!(
            "skipping narrow_margin_capture_gutter_never_wraps_and_both_lines_stay_visible: no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_gutter_narrow_test_{}", std::process::id())),
    );

    // The same tight-but-real margin fixture as the pipeline unit test
    // (`render::tests::chrome_overlay::narrow_gutter_never_wraps_and_both_lines_elide_independently`):
    // a window/measure combo landing comfortably between the collapse floor and
    // the generous ceiling.
    crate::page::set_page_on(true);
    crate::page::set_measure(96);

    let long_name = "a-fairly-long-descriptive-note-title.md";
    let project = "awl-next";
    let mut buf = Buffer::from_str("hello world\n");
    buf.set_path(dir.join(long_name));
    let opts = CaptureOpts {
        canvas: Some((1700, 800)),
        project: Some(ProjectInfo {
            root: dir.to_path_buf(),
            name: project.to_string(),
            branch: None,
            dirty: false,
            default_folder: None,
            workspace: None,
            keymap_flavor: "native",
        }),
        ..CaptureOpts::default()
    };
    let png = dir.join("narrow_gutter.png");
    capture_with(&png, &buf, &opts).expect("narrow-margin capture");
    let text = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("gutter sidecar is not valid JSON: {e}\n{text}"));
    let gutter = &v["gutter"];
    assert_eq!(
        gutter["visible"],
        serde_json::json!(true),
        "a tight-but-real margin still shows the gutter"
    );
    let name = gutter["name"].as_str().expect("gutter.name is a string");
    // (1) THE FIX: one line only — never mid-word wrapped.
    assert!(
        !name.contains('\n'),
        "the filename must render on ONE line, got {name:?}"
    );
    assert_ne!(
        name, long_name,
        "a name this long in this margin must actually elide"
    );
    assert!(
        name.ends_with(".md"),
        "elision preserves the extension: {name:?}"
    );
    // (2) THE CORRECTION: the project does NOT yield just because the filename
    // is eliding — it keeps showing (fit independently against the same
    // budget), here whole since it's short enough for this margin.
    assert_eq!(
        gutter["project"],
        serde_json::json!(project),
        "the project must keep showing alongside an eliding filename"
    );
}

fn sidecar_value(path: &std::path::Path, mode: &str) -> serde_json::Value {
    let text = std::fs::read_to_string(path).expect("read sidecar");
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{mode} sidecar is not valid JSON: {e}\n{text}"))
}

fn assert_plain_schema(obj: &serde_json::Map<String, serde_json::Value>) {
    assert_eq!(
        obj["schema"],
        serde_json::json!(crate::capture::schema_plain()),
        "plain schema"
    );
    for key in [
        "canvas",
        "font",
        "theme",
        "caret_mode",
        "page",
        "wysiwyg",
        "outline",
        "md_spans",
        "syn_lang",
        "syn_spans",
        "readout",
        "gutter",
        "dim_overlay",
        "debug",
        "hud",
        "peek",
        "cursor",
        "selection",
        "layout",
        "search",
        "project",
        "overlay",
        "buffers",
    ] {
        assert!(obj.contains_key(key), "plain sidecar missing {key:?}");
    }
    assert!(obj["outline"].is_object(), "outline is an object");
    assert!(obj["outline"]["on"].is_boolean(), "outline.on is a bool");
    let headings = obj["outline"]["headings"]
        .as_array()
        .expect("outline.headings is an array");
    assert_eq!(
        headings.len(),
        1,
        "one heading in the fixture: {headings:?}"
    );
    assert_eq!(headings[0]["text"], serde_json::json!("Title"));
    assert_eq!(headings[0]["level"], serde_json::json!(1));
    assert_eq!(headings[0]["line"], serde_json::json!(0));
    assert_eq!(obj["outline"]["current"], serde_json::json!(0));
}

fn assert_plain_details(obj: &serde_json::Map<String, serde_json::Value>) {
    assert_eq!(obj["buffers"]["open"], serde_json::json!(1));
    assert_eq!(obj["buffers"]["active"], serde_json::json!("doc.md"));
    assert_eq!(obj["wysiwyg"]["on"], serde_json::json!(true));
    assert!(obj["wysiwyg"]["concealed"].is_array());
    assert!(obj["gutter"].is_object());
    assert!(obj["dim_overlay"].is_boolean());
    assert!(obj["font"].get("cjk").is_some());
    assert!(obj["font"]["cjk"].is_object());
    assert_eq!(obj["font"]["zoom"].as_f64(), Some(1.0));
    assert_eq!(obj["font"]["size"].as_f64(), Some(24.0));
    assert_eq!(obj["font"]["line_height"].as_f64(), Some(32.0));
    assert!(obj["hud"].is_object());
    assert_plain_hud_details(obj);
    assert!(obj["page"].is_object());
    assert!(obj["cursor"].is_object());
    assert!(obj["project"].is_object() || obj["project"].is_null());
    assert!(obj["overlay"].is_object() || obj["overlay"].is_null());
    assert!(
        !obj.contains_key("caret"),
        "plain frame must omit the caret block"
    );
}

fn assert_plain_hud_details(obj: &serde_json::Map<String, serde_json::Value>) {
    assert!(obj["hud"]["held"].is_boolean());
    assert!(obj["hud"]["percent"].is_number());
    assert!(
        obj["hud"].get("selection").is_some(),
        "schema /205 always publishes selection as an object or null"
    );
    assert!(obj["hud"].get("file_created").is_none());
    assert!(obj["hud"].get("session").is_none());
    // The fixture is plain Latin prose, so the readout's own unit reads
    // "words" — present alongside the count on both the `hud` block and
    // the top-level `readout` block, never a second copy.
    assert_eq!(obj["hud"]["unit"], serde_json::json!("words"));
    assert_eq!(obj["readout"]["unit"], serde_json::json!("words"));
    assert!(
        !obj["md_spans"]
            .as_array()
            .expect("md_spans is an array")
            .is_empty()
    );
}

fn assert_timeline_schema(value: &serde_json::Value) {
    assert_eq!(
        value["schema"],
        serde_json::json!(crate::capture::schema_timeline())
    );
    assert!(
        value.get("caret").is_some(),
        "timeline carries a caret block"
    );
    assert!(
        value["caret"].get("trail").is_none(),
        "timeline caret has no trail block"
    );
    assert!(value["caret"].get("cosmetic_trail").is_some());
}

fn assert_held_schema(value: &serde_json::Value) {
    assert_eq!(
        value["schema"],
        serde_json::json!(crate::capture::schema_held())
    );
    assert!(
        value["caret"].get("trail").is_some(),
        "held caret carries a trail block"
    );
}

/// CONTRACT LOCK: the hand-rolled sidecar must be WELL-FORMED JSON (a real
/// parser, not the substring scanners the other tests use, would catch a stray
/// comma / unescaped value / duplicate key) AND carry the right SCHEMA + the
/// blocks the whole verification path depends on. Covers all three shapes:
/// plain (`crate::capture::schema_plain()`, no caret block), timeline (`crate::capture::schema_timeline()`, caret
/// without `trail`), held (`crate::capture::schema_held()`, caret WITH `trail`).
#[test]
fn sidecar_is_wellformed_json_with_expected_schema() {
    if !adapter_available() {
        eprintln!("skipping sidecar schema test: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl_json_test_{}", std::process::id())));
    let mut buf = Buffer::from_str("# Title\n\nsome **bold** prose to fill a line\nsecond line\n");
    buf.set_path(dir.join("doc.md")); // .md so md_spans populate

    // --- PLAIN single frame -----------------------------------------------
    let png = dir.join("plain.png");
    capture_with(&png, &buf, &CaptureOpts::default()).expect("plain capture");
    let plain = sidecar_value(&png.with_extension("json"), "plain");
    let obj = plain.as_object().expect("sidecar root is a JSON object");
    assert_plain_schema(obj);
    assert_plain_details(obj);

    // --- TIMELINE frame (caret block, no trail) ---------------------------
    let tl = dir.join("tl.png");
    capture_timeline(&tl, &buf, (0, 0), &[0, 30], &CaptureOpts::default()).expect("timeline");
    assert_timeline_schema(&sidecar_value(&dir.join("tl.t0.json"), "timeline"));

    // --- HELD frame (caret block WITH trail) ------------------------------
    let hd = dir.join("hd.png");
    capture_held(
        &hd,
        &buf,
        (0, 0),
        HeldDir::Down,
        &[0, 30],
        &CaptureOpts::default(),
    )
    .expect("held");
    assert_held_schema(&sidecar_value(&dir.join("hd.t30.json"), "held"));
}

/// MULTI-BUFFER: an explicit `opts.buffers` (what the real `--screenshot`
/// capture path in `main/run.rs` wires from a `--keys` replay's registry state)
/// is reported VERBATIM in the sidecar, distinct from the single-buffer default.
#[test]
fn buffers_block_reports_the_explicit_registry_snapshot() {
    if !adapter_available() {
        eprintln!("skipping buffers_block_reports_the_explicit_registry_snapshot: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_buffers_json_test_{}", std::process::id())),
    );
    let buf = Buffer::from_str("hello\n");
    let out = dir.join("out.png");
    let opts = CaptureOpts {
        buffers: Some(crate::capture::BuffersInfo {
            open: 2,
            active: Some("/proj/a.txt".to_string()),
        }),
        ..CaptureOpts::default()
    };
    capture_with(&out, &buf, &opts).expect("capture");
    let text = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(v["buffers"]["open"], serde_json::json!(2));
    assert_eq!(v["buffers"]["active"], serde_json::json!("/proj/a.txt"));
}

/// SYNTAX HIGHLIGHTING regression: the capture sidecar's `syn_spans` block is
/// populated for a recognized CODE buffer but EMPTY for a markdown / plain-text
/// buffer — so a `.md` / `.txt` capture stays byte-identical (the gate in
/// `Buffer::syntax_lang`). Also confirms the schema bumped to `/30`.
#[test]
fn syntax_sidecar_gated_to_code() {
    if !adapter_available() {
        eprintln!("skipping syntax_sidecar_gated_to_code: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl_syn_test_{}", std::process::id())));

    // A Rust buffer: syn_spans must carry a "comment" role span for the PROSE
    // comment AND a "comment_code" span for the commented-out statement (the
    // two-tier split, classified centrally in `syntax::spans`).
    let mut code = Buffer::from_str("// hi\n// let x = foo(bar);\nfn main() {}\n");
    code.set_path(dir.join("main.rs"));
    let code_png = dir.join("code.png");
    capture_with(&code_png, &code, &CaptureOpts::default()).expect("code capture");
    let cjson = std::fs::read_to_string(code_png.with_extension("json")).unwrap();
    let code_value = sidecar(&cjson);
    assert_eq!(
        code_value["schema"],
        crate::capture::schema_plain(),
        "schema bumped"
    );
    let syn = code_value["syn_spans"].as_array().expect("syn_spans array");
    let syn_has = |tag: &str| syn.iter().any(|span| span[2] == tag);
    assert!(
        syn_has("comment"),
        "code syn_spans must carry a comment: {syn:?}"
    );
    assert!(
        syn_has("comment_code"),
        "commented-out code must report the comment_code tier: {syn:?}"
    );
    assert!(
        syn_has("definition"),
        "code syn_spans must carry the fn name: {syn:?}"
    );
    // The companion `syn_lang` field reports the DETECTED language, agreeing
    // with the emitted spans (it is `null` when there are none, below).
    assert_eq!(code_value["syn_lang"], "rust", "code syn_lang");

    // A markdown buffer: syn_spans must be the empty array (no code highlight).
    let mut md = Buffer::from_str("# title\nsome prose\n");
    md.set_path(dir.join("notes.md"));
    let md_png = dir.join("notes.png");
    capture_with(&md_png, &md, &CaptureOpts::default()).expect("md capture");
    let mjson = std::fs::read_to_string(md_png.with_extension("json")).unwrap();
    let markdown = sidecar(&mjson);
    assert_eq!(
        markdown["syn_spans"],
        serde_json::json!([]),
        "markdown syn_spans"
    );
    assert_eq!(
        markdown["syn_lang"],
        serde_json::Value::Null,
        "markdown syn_lang"
    );

    // A plain-text buffer: syn_spans empty too.
    let mut txt = Buffer::from_str("just words\n");
    txt.set_path(dir.join("scratch.txt"));
    let txt_png = dir.join("scratch.png");
    capture_with(&txt_png, &txt, &CaptureOpts::default()).expect("txt capture");
    let tjson = std::fs::read_to_string(txt_png.with_extension("json")).unwrap();
    let plain = sidecar(&tjson);
    assert_eq!(plain["syn_spans"], serde_json::json!([]), ".txt syn_spans");
    assert_eq!(plain["syn_lang"], serde_json::Value::Null, ".txt syn_lang");
}

#[test]
fn page_sidecar_reports_class_and_measure_for_code_vs_prose() {
    // The PROSE/CODE PAGE-WIDTH SPLIT (schema `/98`): a recognized CODE file's
    // sidecar reports `page.class == "code"`; a markdown/prose file reports
    // `page.class == "prose"` — `TextPipeline::page_class`, delegating to the
    // SAME classifier `Buffer::page_class` uses, so the two can never disagree.
    // `page.measure` reports whichever measure the process-global holds at
    // capture time (set here to each class's own default, mirroring what
    // `main::args`'s `apply_sticky_globals` + `PageClass::of_path` resolve for
    // the SAME file at real launch).
    if !adapter_available() {
        eprintln!(
            "skipping page_sidecar_reports_class_and_measure_for_code_vs_prose: no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_pageclass_test_{}", std::process::id())),
    );

    crate::page::set_measure(crate::page::DEFAULT_MEASURE_CODE);
    let mut code = Buffer::from_str("fn main() {}\n");
    code.set_path(dir.join("main.rs"));
    let code_png = dir.join("main.png");
    capture_with(&code_png, &code, &CaptureOpts::default()).expect("code capture");
    let cj: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(code_png.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        cj["page"]["class"],
        serde_json::json!("code"),
        "a .rs fixture reports class=code"
    );
    assert_eq!(
        cj["page"]["measure"],
        serde_json::json!(crate::page::DEFAULT_MEASURE_CODE),
        "and the CODE default measure"
    );

    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let mut md = Buffer::from_str("# hello\n");
    md.set_path(dir.join("notes.md"));
    let md_png = dir.join("notes.png");
    capture_with(&md_png, &md, &CaptureOpts::default()).expect("md capture");
    let mj: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(md_png.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        mj["page"]["class"],
        serde_json::json!("prose"),
        "a .md fixture reports class=prose"
    );
    assert_eq!(
        mj["page"]["measure"],
        serde_json::json!(crate::page::DEFAULT_MEASURE),
        "and the PROSE default measure"
    );
}

/// FENCED-CODE SYNTAX: a markdown buffer with a ```` ```rust ```` fence AND a
/// ```` ```sh ```` fence highlights each body by its info-string language. The
/// capture sidecar's `md_spans` block carries the per-role, per-language fence
/// spans (`code_rust_comment`, `code_rust_string`, `code_bash_comment`) alongside
/// the dim `markup` for the fence markers + info string — while `syn_spans` /
/// `syn_lang` stay EMPTY (fence syntax rides the markdown seam, not the code-buffer
/// one). The role colors ride the `base_content`→`muted` ramp, so the ONLY amber in
/// the frame is the caret (DESIGN §3) — asserted by construction (no role derives
/// from `primary`) + the theme's `primary` never appearing as a span role.
#[test]
fn fenced_code_syntax_highlights_by_info_language() {
    if !adapter_available() {
        eprintln!("skipping fenced_code_syntax_highlights_by_info_language: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_fence_test_{}", std::process::id())),
    );

    let doc = "# Demo\n\n```rust\n// hi\nlet s = \"x\";\n```\n\n```sh\n# note\necho hi\n```\n";
    let mut md = Buffer::from_str(doc);
    md.set_path(dir.join("demo.md"));
    let png = dir.join("demo.png");
    capture_with(&png, &md, &CaptureOpts::default()).expect("fence capture");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    let md_spans = value["md_spans"].as_array().expect("md_spans array");
    let md_has = |tag: &str| md_spans.iter().any(|span| span[2] == tag);
    // The md_spans block carries the fenced-body ROLE spans, tagged with their
    // language, so the highlight is headless-assertable.
    assert!(
        md_has("code_rust_comment"),
        "rust fence comment role span present: {md_spans:?}"
    );
    assert!(
        md_has("code_rust_string"),
        "rust fence string role span present: {md_spans:?}"
    );
    assert!(
        md_has("code_bash_comment"),
        "sh fence maps to bash + carries a comment role span: {md_spans:?}"
    );
    // The fence markers + info strings stay dim markup (the whole block is dimmed).
    assert!(md_has("markup"), "fence markers stay markup: {md_spans:?}");

    // Fence syntax lives on the MARKDOWN seam: the code-buffer `syn_spans`/`syn_lang`
    // stay empty/null (this is a markdown buffer, not a code buffer).
    assert!(
        value["syn_spans"]
            .as_array()
            .expect("syn_spans array")
            .is_empty(),
        "markdown syn_spans stays empty"
    );
    assert!(value["syn_lang"].is_null(), "markdown syn_lang stays null");
}

/// MARKDOWN `==highlight==`: a `.md` buffer's `==marked text==` yields a
/// `"highlight"` tag in the sidecar `md_spans` block, with the `==` delimiters
/// dimmed to `"markup"` — the headless-assertable half of the queue item's
/// fixture scenario (the wash PIXELS behind it are covered by the render-level
/// `markdown_highlight_inherits_wash_and_code_buffers_never_match` unit test,
/// which reads the actual wash quads rather than pixel-diffing a PNG).
#[test]
fn markdown_highlight_tag_present_in_sidecar() {
    if !adapter_available() {
        eprintln!("skipping markdown_highlight_tag_present_in_sidecar: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_highlight_test_{}", std::process::id())),
    );

    let doc = "before ==marked text== after\n";
    let mut md = Buffer::from_str(doc);
    md.set_path(dir.join("highlight.md"));
    let png = dir.join("highlight.png");
    capture_with(&png, &md, &CaptureOpts::default()).expect("highlight capture");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    let md_spans = value["md_spans"].as_array().expect("md_spans array");
    assert!(
        md_spans.iter().any(|span| span[2] == "highlight"),
        "marked text carries the highlight tag: {md_spans:?}"
    );
    assert!(
        md_spans.iter().any(|span| span[2] == "markup"),
        "the == delimiters stay dim markup: {md_spans:?}"
    );
}

/// THE WRITER'S DIFF — the read-only prose-diff view renders end-to-end: the
/// capture harness (`AWL_DIFF_*`, here driven directly) turns the marked-up
/// transcript into a real GPU frame + sidecar. Asserts (1) the sidecar `diff` block
/// reports the view's STATE (active, a struck deletion, a washed insertion), and
/// (2) — the APPEARANCE, over the PNG's actual PIXELS, per the sidecar-vs-appearance
/// tripwire — that the washed insertion draws a FILLED wash band: a row with a
/// large run of non-background pixels, which sparse text alone can never produce
/// (the wash tints the whole line, behind glyphs and gaps alike). The struck
/// deletion's presence is state-checked (`struck >= 1`) + confirmed in the drawn
/// text; the wash BAND is the pixel-level proof.
#[test]
fn prose_diff_view_renders_wash_band_pixels_and_reports_state() {
    if !adapter_available() {
        eprintln!(
            "skipping prose_diff_view_renders_wash_band_pixels_and_reports_state: no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_diffview_test_{}", std::process::id())),
    );

    // A deletion + a LONG inserted paragraph (wraps to the full prose column, so its
    // highlight wash fills a wide band of pixels) — exactly the serializer's output.
    let old = "Keep this opening paragraph exactly as it stood before.\n\nDrop this whole paragraph entirely now, it no longer earns its place.";
    let new = "Keep this opening paragraph exactly as it stood before.\n\nAn entirely new paragraph arrives in its place, long enough to wrap across the full width of the prose column so its highlight wash paints a wide continuous band the pixel scan can find.";
    let (transcript, counts) = crate::prosediff::diff_and_render(
        old,
        new,
        crate::prosediff::Params::shipping(),
        "Comparing with earlier",
    );
    assert!(
        counts.struck >= 1 && counts.washed >= 1,
        "the fixture has both a deletion and an insertion"
    );

    let mut buf = Buffer::from_str(&transcript);
    buf.set_path(dir.join("diff.md"));
    // Park the caret on the blank line 1 so no WYSIWYG line reveals raw (the view's rule).
    buf.set_cursor(buf.line_col_to_char(1, 0));
    let opts = CaptureOpts {
        diff: Some(crate::capture::DiffInfo {
            active: true,
            label: "earlier".to_string(),
            struck: counts.struck,
            washed: counts.washed,
            modified: counts.modified,
            moved: counts.moved,
            folds: counts.folds,
        }),
        ..CaptureOpts::default()
    };
    let png = dir.join("diff.png");
    capture_with(&png, &buf, &opts).expect("diff view capture");

    // (1) STATE: the sidecar `diff` block reports an active view with the right shape.
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    assert_eq!(value["diff"]["active"], true, "diff.active");
    assert_eq!(value["diff"]["struck"], 1, "diff.struck");
    assert_eq!(value["diff"]["washed"], 1, "diff.washed");
    // The washed insertion is in the render model (its `==` dims to markup).
    let md_spans = value["md_spans"].as_array().expect("md_spans array");
    assert!(
        md_spans.iter().any(|span| span[2] == "highlight"),
        "washed insertion is a highlight span: {md_spans:?}"
    );

    // (2) APPEARANCE (real pixels): the washed insertion draws a FILLED wash band —
    // some row carries a large run of non-background pixels. The page background is
    // the top-left corner pixel; the wash tints the whole insertion line (behind
    // glyphs AND gaps), so its row's non-bg count dwarfs any sparse-text row.
    let img = image::open(&png).expect("decode diff PNG").to_rgba8();
    let (w, h) = img.dimensions();
    let bg = *img.get_pixel(3, 3);
    let differs = |p: &image::Rgba<u8>| -> bool {
        let d = |a: u8, b: u8| (a as i32 - b as i32).abs();
        d(p[0], bg[0]) + d(p[1], bg[1]) + d(p[2], bg[2]) > 30
    };
    let mut max_row_nonbg = 0u32;
    for y in 0..h {
        let mut n = 0u32;
        for x in 0..w {
            if differs(img.get_pixel(x, y)) {
                n += 1;
            }
        }
        max_row_nonbg = max_row_nonbg.max(n);
    }
    assert!(
        max_row_nonbg >= 300,
        "a filled highlight-wash band should paint a wide row of non-bg pixels; max row non-bg = {max_row_nonbg} (canvas {w}x{h})"
    );
}

/// GFM TABLE: a `.md` buffer with a table yields the three structural tags in the
/// sidecar `md_spans` block — `table_pipe` (the cell `|`), `table_sep` (the
/// `|---|` header-separator row), and `table_header` (a header cell's content) —
/// so the styled-SOURCE rendering (dim the markup, no drawn grid) is headlessly
/// assertable. The double-space nit exemption on table rows is proven separately by
/// the pure `nits::tests` + render-level unit tests.
#[test]
fn markdown_table_tags_present_in_sidecar() {
    if !adapter_available() {
        eprintln!("skipping markdown_table_tags_present_in_sidecar: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_table_test_{}", std::process::id())),
    );

    let doc = "| Name  | Value |\n|-------|:-----:|\n| foo   | 1     |\n";
    let mut md = Buffer::from_str(doc);
    md.set_path(dir.join("table.md"));
    let png = dir.join("table.png");
    capture_with(&png, &md, &CaptureOpts::default()).expect("table capture");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    let md_spans = value["md_spans"].as_array().expect("md_spans array");
    for tag in ["table_pipe", "table_sep", "table_header"] {
        assert!(
            md_spans.iter().any(|span| span[2] == tag),
            "table span {tag} present: {md_spans:?}"
        );
    }
}

#[test]
fn zero_document_capture_has_two_start_actions_and_no_page_surface() {
    if !adapter_available() {
        eprintln!("skipping zero-document capture law: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-zero-document-{}", std::process::id())),
    );
    let buffer = Buffer::scratch();
    let opts = CaptureOpts {
        document_absent: true,
        buffers: Some(crate::capture::BuffersInfo {
            open: 0,
            active: None,
        }),
        ..CaptureOpts::default()
    };
    crate::page::set_page_on(true);
    let on = dir.join("page-on.png");
    capture_with(&on, &buffer, &opts).expect("zero-document capture");
    crate::page::set_page_on(false);
    let off = dir.join("page-off.png");
    capture_with(&off, &buffer, &opts).expect("zero-document capture without page mode");

    assert_eq!(
        std::fs::read(&on).unwrap(),
        std::fs::read(&off).unwrap(),
        "page mode cannot change pixels when there is no page to draw"
    );
    let json = sidecar(&std::fs::read_to_string(on.with_extension("json")).unwrap());
    assert_eq!(json["document"]["active"], false);
    assert_eq!(
        json["document"]["start_actions"],
        serde_json::json!(["New document", "Go to"])
    );
    assert!(json["page"].is_null());
    assert!(json["text_origin"].is_null());
    assert_eq!(json["line_count"], 0);
    assert!(json["cursor"].is_null());
    assert!(json["text"].is_null());
    assert_eq!(json["first_lines"], serde_json::json!([]));
    assert!(json["layout"].is_null());
    assert_eq!(json["buffers"]["open"], 0);
    assert!(json["buffers"]["active"].is_null());
    let image = image::open(on).unwrap().to_rgba8();
    let (width, height) = image.dimensions();
    let ink_rows: Vec<u32> = (height * 2 / 5 + 1..height * 3 / 5)
        .filter(|&y| {
            (width * 2 / 5..width * 3 / 5)
                .filter(|&x| {
                    image
                        .get_pixel(x, y)
                        .0
                        .iter()
                        .zip(image.get_pixel(x, y - 1).0.iter())
                        .map(|(a, b)| a.abs_diff(*b) as u32)
                        .sum::<u32>()
                        > 24
                })
                .count()
                >= 8
        })
        .collect();
    let bands = ink_rows
        .iter()
        .fold(Vec::<(u32, u32)>::new(), |mut bands, &y| {
            if let Some(last) = bands.last_mut()
                && y <= last.1 + 4
            {
                last.1 = y;
            } else {
                bands.push((y, y));
            }
            bands
        });
    assert_eq!(
        bands.len(),
        2,
        "the exact two sidecar labels must each leave their own rendered glyph band; \
         rows={ink_rows:?}, bands={bands:?}"
    );
}

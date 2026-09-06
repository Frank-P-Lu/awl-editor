//! The live-`App` capture mode's own laws. Carved out of `live_app.rs` as a
//! sibling `tests.rs` so the production file stays under the size ceiling —
//! `code-health.py`'s `production()` exempts that exact filename precisely so
//! carving an inline `mod tests` out is a real remedy rather than moving lines
//! from one measured file to another.

use super::*;
use crate::capture::CaptureOpts;
use crate::config::Config;
use crate::settings::SettingId;
use crate::testscratch::ScratchDir;
use std::sync::Arc;

const CFG: &str = "/cfg/config.toml";

/// The chord that summons the Settings workspace in the convention this
/// pass is running under — resolved from the RUNNING convention rather than
/// hardcoded, because `native-gate.sh` runs the suite once per convention
/// and each pass must drive its own real binding.
fn open_settings_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-,",
        crate::convention::Convention::Linux => "C-,",
    }
}

fn new_document_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-n",
        crate::convention::Convention::Linux => "C-n",
    }
}

fn finish_file_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-w",
        crate::convention::Convention::Linux => "C-w",
    }
}

fn open_theme_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-t",
        crate::convention::Convention::Linux => "C-t",
    }
}

/// The real chords a user walks in with: summon the workspace, `Tab` from
/// the navigation rail into the content pane, one `Down` per row of the
/// corpus, then `Enter` on the row. The row INDEX is derived from
/// `settings::visible_rows()`, so a corpus reorder re-aims this walk instead
/// of silently landing on a neighbour.
fn walk_to(id: SettingId) -> Vec<crate::keyspec::Chord> {
    let idx = crate::settings::visible_rows()
        .iter()
        .position(|r| r.id == id)
        .expect("the row is in the visible corpus");
    let downs = std::iter::repeat_n("Down", idx)
        .collect::<Vec<_>>()
        .join(" ");
    let spec = format!("{} Tab {downs} Enter", open_settings_chord());
    crate::keyspec::parse_chords(&spec).expect("the walk parses")
}

fn proj() -> std::path::PathBuf {
    std::path::PathBuf::from("/ws/proj")
}

/// A config with a real (sandbox) path, so a settings persist has somewhere
/// to land — the write is part of what the live door actually performs.
/// The live-`App` payload these tests drive, over the scratch config.
fn spec(keys: Vec<crate::keyspec::Chord>, root: Option<PathBuf>) -> LiveAppSpec {
    LiveAppSpec {
        file: None,
        keys,
        root,
        workspace: None,
        config: cfg(),
        canvas: None,
        dpi: None,
    }
}

fn cfg() -> Config {
    Config {
        path: std::path::PathBuf::from(CFG),
        ..Config::empty()
    }
}

/// Run `body` over a seeded `InMemoryFs`, so nothing in these laws — the App's
/// config persist included — can reach the real disk. The PNG + JSON still go
/// out through `std::fs`, the documented capture-deliverable bypass.
/// [`in_sandbox`] with one document already seeded — for the laws that need
/// the App to OPEN a real (sandbox) file and write it back.
fn in_sandbox_with<T>(doc: &std::path::Path, body: impl FnOnce() -> T) -> T {
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(doc, "# Probe\n\nSome prose.\n"),
    );
    crate::fs::with_fs(mem, body)
}

fn in_sandbox<T>(body: impl FnOnce() -> T) -> T {
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj"),
    );
    crate::fs::with_fs(mem, body)
}

fn sidecar(png: &std::path::Path) -> serde_json::Value {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("a sidecar");
    serde_json::from_str(&text).expect("valid sidecar JSON")
}

fn image_pixels(png: &std::path::Path) -> (u32, u32, Vec<[u8; 4]>) {
    let image = image::open(png).expect("capture PNG opens").into_rgba8();
    let (w, h) = image.dimensions();
    let pixels = image.pixels().map(|p| [p[0], p[1], p[2], p[3]]).collect();
    (w, h, pixels)
}

fn color_distance(a: [u8; 4], b: [u8; 4]) -> f64 {
    let sq = |x: u8, y: u8| f64::from(x.abs_diff(y)).powi(2);
    (sq(a[0], b[0]) + sq(a[1], b[1]) + sq(a[2], b[2])).sqrt()
}

fn dominant_ink(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    rect: (i64, i64, i64, i64),
    ground: [u8; 4],
) -> Option<([u8; 4], usize)> {
    use std::collections::HashMap;
    let (x, y, w, h) = rect;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for py in y.max(0)..(y + h).min(height as i64) {
        for px in x.max(0)..(x + w).min(width as i64) {
            let p = pixels[(py * width as i64 + px) as usize];
            if color_distance(p, ground) >= 18.0 {
                *counts.entry(p).or_default() += 1;
            }
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n)
}

fn count_near_color(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    rect: (i64, i64, i64, i64),
    target: [u8; 4],
    tolerance: f64,
) -> usize {
    let (x, y, w, h) = rect;
    let mut count = 0usize;
    for py in y.max(0)..(y + h).min(height as i64) {
        for px in x.max(0)..(x + w).min(width as i64) {
            let p = pixels[(py * width as i64 + px) as usize];
            count += usize::from(color_distance(p, target) <= tolerance);
        }
    }
    count
}

fn inline_code_ink() -> [u8; 4] {
    let th = crate::theme::active();
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * 0.28).round() as u8;
    [
        mix(th.base_content.r, th.muted.r),
        mix(th.base_content.g, th.muted.g),
        mix(th.base_content.b, th.muted.b),
        255,
    ]
}

fn assert_mangrove_table_ink(after_png: &std::path::Path) {
    let side = sidecar(after_png);
    assert_eq!(side["driver"].as_str(), Some("live-app"));
    assert_eq!(side["theme"]["name"].as_str(), Some("Mangrove"));
    assert_eq!(side["tables"].as_array().map(Vec::len), Some(1));
    let (width, height, pixels) = image_pixels(after_png);
    let left = side["text_origin"]["left"].as_f64().unwrap() as i64;
    let top = side["text_origin"]["top"].as_f64().unwrap() as i64;
    let line_h = side["font"]["line_height"].as_f64().unwrap() as i64;
    let page_x = side["page"]["column"]["left"].as_f64().unwrap() as i64;
    let page_w = side["page"]["column"]["width"].as_f64().unwrap() as i64;
    let ground = pixels[((height as i64 - 24) * width as i64 + page_x + page_w / 2) as usize];
    let prose = dominant_ink(&pixels, width, height, (left, top, 320, line_h), ground)
        .expect("presence floor: prose reference paints ink");
    let table = dominant_ink(
        &pixels,
        width,
        height,
        (left, top + 3 * line_h, 260, line_h),
        ground,
    )
    .expect("presence floor: table header paints ink");
    let expected_code = inline_code_ink();
    let prose_code_presence = count_near_color(
        &pixels,
        width,
        height,
        (left, top + line_h, 260, line_h),
        expected_code,
        8.0,
    );
    let table_code_presence = count_near_color(
        &pixels,
        width,
        height,
        (left, top + 5 * line_h, 260, line_h),
        expected_code,
        8.0,
    );
    assert!(
        prose.1 >= 8 && table.1 >= 8 && prose_code_presence >= 8,
        "subject-presence floors: prose={prose:?} table={table:?} \
         expected_code={expected_code:?} prose_code={prose_code_presence} \
         table_code={table_code_presence}"
    );
    let contrast = color_distance(table.0, ground);
    assert!(
        contrast >= 80.0,
        "table contrast floor: ink={:?} ground={ground:?} distance={contrast:.2}",
        table.0
    );
    let agreement = color_distance(table.0, prose.0);
    assert!(
        agreement <= 8.0,
        "Mangrove accept frame retained construction-time Magpie table ink: \
         table={:?} prose={:?} ground={ground:?} agreement={agreement:.2} \
         presence(table={}, prose={}) contrast={contrast:.2}",
        table.0,
        prose.0,
        table.1,
        prose.1,
    );
    assert!(
        table_code_presence >= 80,
        "Mangrove accept frame retained construction-time inline-code table ink: \
         expected={expected_code:?} ground={ground:?} \
         presence(table={table_code_presence}, prose={prose_code_presence})"
    );
    eprintln!(
        "live-table-retint arithmetic: table={:?} prose={:?} ground={ground:?} \
         agreement={agreement:.2} contrast={contrast:.2} presence(table={}, prose={}) \
         code={expected_code:?} code_presence(table={table_code_presence}, \
         prose={prose_code_presence}) artifact={}",
        table.0,
        prose.0,
        table.1,
        prose.1,
        after_png.display(),
    );
}

/// Tier-2 transition law: one persistent offscreen pipeline is first rendered
/// under Magpie, then a real headless `App` accepts Mangrove through the theme
/// picker, and the SAME pipeline renders the next frame after its production
/// theme-sync seam. The table ink is graded relative to prose and page pixels
/// from that one destination frame. Both regions carry subject-presence floors,
/// and table-vs-page carries a contrast floor, so deleting/fading either subject
/// cannot satisfy the equality claim.
#[test]
fn live_app_magpie_to_mangrove_retints_table_glyph_ink_in_the_accept_frame() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let doc = "/ws/proj/table.md";
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(
                doc,
                concat!(
                    "Prose reference ink\n`Code prose reference`\n\n",
                    "| Table reference | Role |\n| --- | --- |\n",
                    "| `Code table ink` | Reader |\n",
                ),
            ),
    );
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-table-retint-{}", std::process::id())),
    );
    crate::fs::with_fs(mem, || {
        let config = Config {
            theme: Some("Magpie".to_string()),
            path: PathBuf::from(CFG),
            ..Config::empty()
        };
        let mut app = App::new_headless_capture(
            Some(PathBuf::from(doc)),
            PathBuf::from("/ws/proj"),
            None,
            config,
        );
        crate::theme::set_active_by_name("Magpie").unwrap();
        assert_eq!(crate::theme::active().name, "Magpie", "source world");

        let mut film = crate::capture::FilmRenderer::new(&dir)
            .expect("the live-App transition law needs a GPU adapter");
        let before_png = dir.join("magpie.png");
        let before_opts = app.capture_opts();
        let before_buffer = crate::run::CaptureSubject::buffer(&app).expect("active table doc");
        film.render_step(before_buffer, &before_opts, 1, Some(&before_png))
            .expect("Magpie seed frame");

        let keys = crate::keyspec::parse_chords(&format!("{} Up Up Enter", open_theme_chord()))
            .expect("theme picker walk parses");
        app.press_chords_headless(&keys);
        assert_eq!(
            crate::theme::active().name,
            "Mangrove",
            "the real theme picker must accept Mangrove from Magpie"
        );
        film.sync_theme();
        let after_png = dir.join("mangrove.png");
        let after_opts = app.capture_opts();
        let after_buffer = crate::run::CaptureSubject::buffer(&app).expect("active table doc");
        film.render_step(after_buffer, &after_opts, 1, Some(&after_png))
            .expect("Mangrove accept frame");
        assert_mangrove_table_ink(&after_png);
    });
}

/// The selected row's VALUE cell, straight out of the sidecar's parallel
/// `overlay.bindings` array (`opts.rs`: a Settings row carries its readout
/// there as text).
fn selected_value(v: &serde_json::Value) -> String {
    let o = &v["overlay"];
    let i = o["selected_index"].as_u64().expect("a selected row") as usize;
    o["bindings"][i].as_str().expect("a value cell").to_string()
}

fn selected_name(v: &serde_json::Value) -> String {
    let o = &v["overlay"];
    let i = o["selected_index"].as_u64().expect("a selected row") as usize;
    o["items"][i].as_str().expect("a row label").to_string()
}

#[test]
fn live_app_first_new_document_asks_for_a_folder_before_creating_a_file() {
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-live-app-{}", std::process::id())));
    let png = dir.join("folder-choice.png");
    let json = in_sandbox(|| {
        capture_live_app(
            png.clone(),
            spec(
                crate::keyspec::parse_chords(new_document_chord()).unwrap(),
                Some(crate::fs::data_root()),
            ),
        )
        .unwrap();
        sidecar(&png)
    });
    assert_eq!(json["driver"].as_str(), Some("live-app"));
    assert_eq!(json["overlay"]["mode"].as_str(), Some("goto"));
}

#[test]
fn live_app_close_last_sidecar_and_semantics_are_honestly_document_free() {
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-live-zero-{}", std::process::id())));
    let png = dir.join("zero.png");
    let doc = PathBuf::from("/ws/proj/probe.md");
    let json = in_sandbox_with(&doc, || {
        capture_live_app(
            png.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: crate::keyspec::parse_chords(finish_file_chord()).unwrap(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("live zero-document capture needs a GPU adapter");
        sidecar(&png)
    });
    assert_eq!(json["driver"], "live-app");
    assert_eq!(json["document"]["active"], false);
    assert_eq!(json["buffers"]["open"], 0);
    assert!(json["buffers"]["active"].is_null());
    assert!(json["page"].is_null());
    let nodes = json["semantic"]["nodes"].as_array().unwrap();
    assert!(nodes.iter().all(|node| node["id"] != "document"));
    let actions: Vec<_> = nodes
        .iter()
        .filter(|node| node["role"] == "button")
        .map(|node| node["name"].as_str().unwrap())
        .collect();
    assert_eq!(actions, ["New document", "Go to"]);

    let goto_png = dir.join("zero-goto.png");
    let goto = in_sandbox_with(&doc, || {
        capture_live_app(
            goto_png.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: crate::keyspec::parse_chords(&format!(
                    "{} {}",
                    finish_file_chord(),
                    match crate::convention::Convention::current() {
                        crate::convention::Convention::Mac => "s-o",
                        crate::convention::Convention::Linux => "C-o",
                    }
                ))
                .unwrap(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("live zero-document Go-to capture needs a GPU adapter");
        sidecar(&goto_png)
    });
    assert_eq!(goto["document"]["active"], false);
    assert_eq!(goto["overlay"]["active"], true);
    assert_eq!(goto["overlay"]["mode"], "goto");
    assert!(
        goto["semantic"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("overlay.goto."))),
        "the sidecar and accessibility projection see the same zero-document Go-to card"
    );
}

/// THE GUARD CHAIN HOLDS AT THE LIVE TIER: `range_apply_live`'s
/// document-dependent branch (`App::document::buffer_opt`, hardened against
/// the panicking `buffer()`) is reachable only from `SettingRangeStep`, which
/// itself needs an open Settings overlay with a range row selected —
/// unreachable through any `--keys` chord once the last document is closed,
/// because `reject_without_document` refuses every action but the two
/// zero-document start actions (see the sibling law above). This drives the
/// exact real-world attempt — summon Settings, then send the keys a range
/// row's live rail would interpret as a step — from a real zero-document
/// live `App`, and asserts the whole sequence is a no-op: no panic (a crash
/// here would fail this test by aborting the process, not by an assertion),
/// no overlay summoned, still zero-document. This is NOT a capture of
/// `range_apply_live` itself — no `--keys` vocabulary reaches that call at
/// all from this state, which is the point being proved.
#[test]
fn live_app_zero_document_settings_and_range_step_attempt_is_a_silent_no_op() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-zero-range-{}", std::process::id())),
    );
    let png = dir.join("zero-range.png");
    let doc = PathBuf::from("/ws/proj/probe.md");
    let json = in_sandbox_with(&doc, || {
        capture_live_app(
            png.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: crate::keyspec::parse_chords(&format!(
                    "{} {} Right Right Left",
                    finish_file_chord(),
                    open_settings_chord()
                ))
                .unwrap(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect(
            "live zero-document range-step attempt needs a GPU adapter — a panic here \
             would fail this call instead of reaching a sidecar assertion",
        );
        sidecar(&png)
    });
    assert_eq!(json["document"]["active"], false);
    assert_eq!(
        json["overlay"]["active"], false,
        "Settings never opens with no document to gate it, so no range row is ever there to step"
    );
    let nodes = json["semantic"]["nodes"].as_array().unwrap();
    let actions: Vec<_> = nodes
        .iter()
        .filter(|node| node["role"] == "button")
        .map(|node| node["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        ["New document", "Go to"],
        "the zero-document start surface is unchanged by the whole attempt"
    );
}

/// THE PRIMARY LAW — the transition converted from
/// Rust-only to sidecar-provable.
///
/// Picking "Emacs" in the KEYMAP picker (Enter on the Settings "Keymap" row
/// descends into `OverlayKind::Keymap`; `KeymapFlavor::ALL` is `[Native,
/// Emacs]`, so one `Down` then `Enter` accepts Emacs) is the hardest case in
/// the live-only census, not a convenient one: `docs/harness-reach.md` lists
/// `overlay_accept:Keymap` as **Unsupported** under every filesystem
/// capability, because applying the picked flavor needs a LIVE keymap
/// rebuild no filesystem capability can supply. The transition is provable
/// only in Rust otherwise (`app::tests::files::
/// keymap_picker_accept_applies_persists_notifies_and_live_reapplies`,
/// asserting `App` + config state directly).
///
/// BOTH HALVES ARE ASSERTED IN ONE TEST, and the second is what makes the
/// first mean anything: the SAME chord spec is driven through the ordinary
/// `--keys` capture door, which must still report the OLD flavor and record
/// the skip. A live-App sidecar that agreed with the replay sidecar would
/// prove nothing had been reached at all.
///
/// ONLY MEANINGFUL ON `Convention::Linux` — the flavor is structurally inert
/// on `Convention::Mac` (`crate::keymap::KeymapFlavor`'s doc), so the "Keymap"
/// row is hidden from `visible_rows()` there entirely
/// (`settings::row_available_on`). `Convention::current()` is process-frozen
/// (`AWL_CONVENTION_FORCE` read once, memoized), so this test branches on the
/// ambient value rather than forcing one: `native-gate.sh`'s two-convention
/// run is what exercises BOTH this law (Linux pass) and the row's absence
/// (Mac pass, asserted in the `else` branch below) across the two runs.
#[test]
fn a_live_app_capture_photographs_a_keymap_pick_an_ordinary_capture_cannot_see() {
    let _g = crate::testlock::serial();
    if crate::convention::Convention::current() != crate::convention::Convention::Linux {
        // THE MACOS-INERT-HIDES-THE-ROW PROOF: on this convention there is
        // nothing to walk to at all — the settings row itself is gone.
        assert!(
            !crate::settings::visible_rows()
                .iter()
                .any(|r| r.id == SettingId::Keymap),
            "Keymap must be hidden from the Settings menu on Convention::Mac"
        );
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-replay-{}", std::process::id())),
    );
    let mut keys = walk_to(SettingId::Keymap);
    keys.extend(crate::keyspec::parse_chords("Down Enter").expect("the pick-Emacs tail parses"));

    // ── THE LIVE-`App` CAPTURE ────────────────────────────────────────
    let live = dir.join("live.png");
    let live_json = in_sandbox(|| {
        capture_live_app(live.clone(), spec(keys.clone(), Some(proj())))
            .expect("the live-App capture needs a GPU adapter");
        sidecar(&live)
    });
    assert_eq!(
        live_json["overlay"]["active"].as_bool(),
        Some(true),
        "the sidecar was folded AFTER the chords were driven — the walk \
         resumed the Settings card the picker was opened from"
    );
    assert_eq!(
        selected_name(&live_json),
        "Keymap",
        "the picker's accept resumed the Settings row it was opened from"
    );
    assert_eq!(
        live_json["driver"].as_str(),
        Some("live-app"),
        "the sidecar names the tier that produced it"
    );
    assert_eq!(
        live_json["project"]["keymap_flavor"].as_str(),
        Some("emacs"),
        "THE CONVERTED CLAIM: the live keymap pick is readable from the \
         sidecar's project block, not only from a Rust assertion"
    );
    assert_eq!(
        selected_value(&live_json),
        "Emacs",
        "and the row's own value cell agrees, in plain language — two \
         independent witnesses in one artifact"
    );
    assert!(
        live_json["replay_skips"]
            .as_array()
            .is_some_and(|a| a.is_empty()),
        "a live-App capture skips nothing: it performs the effect"
    );

    // ── THE SAME SPEC THROUGH THE ORDINARY `--keys` DOOR ──────────────
    // Anti-vacuity. This is the capture that could NOT witness the pick,
    // and it must still be unable to.
    let replay = dir.join("replay.png");
    let replay_json = in_sandbox(|| {
        super::super::capture_screenshot(
            replay.clone(),
            None,
            CaptureOpts::default(),
            keys,
            crate::keymap::KeymapState::new(),
            Some(proj()),
            None,
            std::path::PathBuf::from("/ws/notes"),
            cfg(),
            false,
        )
        .expect("the ordinary capture succeeds");
        sidecar(&replay)
    });
    assert_eq!(
        replay_json["driver"].as_str(),
        Some("replay"),
        "the ordinary door stamps the shared-core tier"
    );
    assert_eq!(
        selected_name(&replay_json),
        "Keymap",
        "the ordinary replay walked to the same row — the specs are \
         identical, and the journey resume is core-level (it happens \
         whether or not the accept effect itself gets applied)"
    );
    assert_eq!(
        replay_json["project"]["keymap_flavor"].as_str(),
        Some("native"),
        "the ordinary capture still cannot see the pick — if this ever reads \
         `emacs`, the live-App law above has stopped proving anything new"
    );
    assert_eq!(
        replay_json["replay_skips"],
        serde_json::json!([{ "effect": "overlay_accept", "action": "Newline" }]),
        "and it says so out loud rather than reporting stale state silently"
    );
}

/// **A CAPTURE CARRIES A TOAST**, and the sidecar's two answers
/// about it agree.
///
/// The defect this closes was not "the notice looked wrong": no capture door
/// could SEE the notice channel at all. `CaptureOpts` had no slot for it, so a
/// real headless `App` that had genuinely raised `saved` produced a PNG
/// **byte-identical** to one that had raised nothing — while the SAME
/// sidecar's `semantic` block, which reads the `App` directly, announced that
/// notice to a screen reader. One artifact, two answers. Every "the notice is
/// set" claim in this tree was a sidecar claim, never a photographed one.
///
/// So this law asserts both halves, and the SECOND is what makes the first
/// mean something:
///
///   1. the two PNGs differ — the notice is on the frame;
///   2. the top-level `notice` block and the `semantic` tree's own status node
///      report the SAME sentence, so the artifact cannot say one thing to a
///      reader and another to an assistive client.
///
/// A byte-identity assertion is the exact shape of the bug, which is why it is
/// spelled that way rather than as "the notice block is non-null".
#[test]
fn a_live_app_capture_photographs_the_toast_the_semantic_tree_announces() {
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-notice-{}", std::process::id())));
    // The document lives in the HERMETIC sandbox, not on the real disk: the
    // App reads and (on save) WRITES through `crate::fs::active()`, and the
    // whole point of a save here is that it really happens.
    let doc = std::path::PathBuf::from("/ws/proj/probe.md");

    // A real Cmd-S (or its Emacs twin) through the real keymap: `manual_save`
    // raises the `saved` toast as its only user-visible result.
    let save = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-s",
        crate::convention::Convention::Linux => "C-x C-s",
    };
    let quiet = dir.join("quiet.png");
    let noticed = dir.join("noticed.png");
    let (quiet_json, noticed_json) = in_sandbox_with(&doc, || {
        capture_live_app(
            quiet.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: Vec::new(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("the live-App capture needs a GPU adapter");
        capture_live_app(
            noticed.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: crate::keyspec::parse_chords(save).expect("the save chord parses"),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("the live-App capture needs a GPU adapter");
        (sidecar(&quiet), sidecar(&noticed))
    });

    assert_eq!(
        noticed_json["notice"]["text"].as_str(),
        Some("saved"),
        "the sidecar reports the notice the App raised"
    );
    assert_eq!(
        noticed_json["notice"]["kind"].as_str(),
        Some("toast"),
        "and its KIND — a lifetime, which is what distinguishes it from a \
         held sticky notice"
    );
    assert!(
        quiet_json["notice"].is_null(),
        "a capture that raised no notice reports none"
    );

    // THE DEFECT, spelled as itself.
    let a = std::fs::read(&quiet).expect("read the quiet PNG");
    let b = std::fs::read(&noticed).expect("read the noticed PNG");
    assert_ne!(
        a, b,
        "a capture carrying a toast must NOT be byte-identical to one \
         without — that identity is item 296, and it held for every notice \
         this channel ever raised"
    );

    // THE TWO ANSWERS AGREE.
    let announced = noticed_json["semantic"]["nodes"]
        .as_array()
        .expect("a live-App sidecar carries a semantic tree")
        .iter()
        .find(|n| n["role"].as_str() == Some("status"))
        .and_then(|n| n["name"].as_str().map(str::to_string));
    assert_eq!(
        announced.as_deref(),
        noticed_json["notice"]["text"].as_str(),
        "the `notice` block and the `semantic` tree's status node must carry \
         the SAME sentence — they were free to disagree, and did"
    );
}

/// The live-`App` capture must write through the ORDINARY schema — one
/// const, one writer, one shape. A parallel serializer would show up here as
/// a schema string that is not the plain one, or as a sidecar missing a block
/// every other capture carries.
#[test]
fn the_live_app_sidecar_is_the_ordinary_schema_and_shape() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-schema-{}", std::process::id())),
    );
    let live = dir.join("plain.png");
    let live_json = in_sandbox(|| {
        capture_live_app(live.clone(), spec(Vec::new(), Some(proj())))
            .expect("the live-App capture needs a GPU adapter");
        sidecar(&live)
    });
    assert_eq!(
        live_json["schema"].as_str(),
        Some(crate::capture::schema_plain().as_str()),
        "the plain single-frame schema, from the one const"
    );
    // A SPOT-CHECK ACROSS THE WHOLE SHAPE, not just the blocks this mode
    // touches: the fold hands `capture_with` an ordinary `CaptureOpts`, so
    // every block a `--screenshot` carries must be present here too.
    for block in [
        "canvas",
        "font",
        "theme",
        "caret_mode",
        "page",
        "wysiwyg",
        "layout",
        "hud",
        "search",
        "project",
        "overlay",
        "buffers",
        "replay_skips",
        "cursor",
        "text",
        "md_spans",
    ] {
        assert!(
            live_json.get(block).is_some(),
            "the live-App sidecar is missing `{block}` — it is not the \
             ordinary schema, which means something grew a second serializer"
        );
    }
    assert_eq!(
        live_json["overlay"]["active"].as_bool(),
        Some(false),
        "no chords were pressed, so the shared no-overlay literal is emitted \
         — the same one every card-less capture carries"
    );
}

/// A `--capture-size`/`--capture-dpi` combination reaching this door must
/// change the RENDERED GEOMETRY, not merely echo the requested numbers back
/// into the `canvas` block — a law asserting only the latter is satisfiable
/// by a door that reports the field and then ignores it. So this drives one
/// long single-logical-line document through three renders and reads the
/// visual-row WRAP COUNT for that line straight out of the `layout` oracle:
///
/// 1. a narrower PHYSICAL canvas at dpi 1 must wrap the same line into MORE
///    visual rows than a wider one — real reflow, not an echoed number;
/// 2. a device canvas at dpi 2 that is exactly DOUBLE a dpi-1 canvas must
///    wrap identically to it — the documented `--capture-dpi` meaning
///    (`WxH` device @ dpi N == `(W/N)x(H/N)` logical) holding on this door
///    exactly as it does on the ordinary `--screenshot` one.
#[test]
fn a_live_app_capture_honors_capture_size_and_the_dpi_meaning_holds() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-canvas-{}", std::process::id())),
    );
    let doc = std::path::PathBuf::from("/ws/proj/long.md");
    // One long logical line with plentiful word-wrap points, long enough to
    // wrap several times even at the wide canvas.
    let long_line = "supercalifragilistic ".repeat(60);
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(&doc, &long_line),
    );

    let wide_out = dir.join("wide.png");
    let narrow_out = dir.join("narrow.png");
    let dpi2_out = dir.join("dpi2.png");
    let spec_at = |canvas: Option<(u32, u32)>, dpi: Option<f32>| LiveAppSpec {
        file: Some(doc.clone()),
        keys: Vec::new(),
        root: Some(proj()),
        workspace: None,
        config: cfg(),
        canvas,
        dpi,
    };
    let (wide_json, narrow_json, dpi2_json) = crate::fs::with_fs(mem, || {
        capture_live_app(wide_out.clone(), spec_at(Some((1200, 800)), None))
            .expect("the live-App capture needs a GPU adapter");
        capture_live_app(narrow_out.clone(), spec_at(Some((640, 800)), None))
            .expect("the live-App capture needs a GPU adapter");
        // Double the wide canvas AND the dpi: same LOGICAL window (1200x800).
        capture_live_app(dpi2_out.clone(), spec_at(Some((2400, 1600)), Some(2.0)))
            .expect("the live-App capture needs a GPU adapter");
        (sidecar(&wide_out), sidecar(&narrow_out), sidecar(&dpi2_out))
    });

    // The field is genuinely threaded (not left at the 1200x800 default) —
    // necessary, but per the doc comment above, not sufficient on its own.
    assert_eq!(wide_json["canvas"]["width"].as_u64(), Some(1200));
    assert_eq!(narrow_json["canvas"]["width"].as_u64(), Some(640));
    assert_eq!(dpi2_json["canvas"]["width"].as_u64(), Some(2400));
    assert_eq!(dpi2_json["canvas"]["dpi"].as_f64(), Some(2.0));

    let wraps_for_line0 = |v: &serde_json::Value| -> usize {
        v["layout"]["rows"]
            .as_array()
            .expect("a live-App sidecar carries a layout block")
            .iter()
            .filter(|row| row["line"].as_u64() == Some(0))
            .count()
    };
    let wide_wraps = wraps_for_line0(&wide_json);
    let narrow_wraps = wraps_for_line0(&narrow_json);
    let dpi2_wraps = wraps_for_line0(&dpi2_json);

    assert!(
        wide_wraps >= 2,
        "the fixture line must wrap at least twice at 1200px wide for this \
         law to be meaningful (got {wide_wraps})"
    );
    assert!(
        narrow_wraps > wide_wraps,
        "a narrower --capture-size must wrap the SAME line into MORE visual \
         rows ({narrow_wraps} at 640px vs {wide_wraps} at 1200px) — this is \
         the geometry the flag is supposed to change, not just the number \
         echoed into the canvas block"
    );
    assert_eq!(
        dpi2_wraps, wide_wraps,
        "2400x1600 @ dpi 2.0 is the SAME logical 1200x800 window as \
         1200x800 @ dpi 1.0, so the two must wrap identically — the \
         documented --capture-dpi meaning (WxH device @ dpi N == \
         (W/N)x(H/N) logical) holding on the live-App door exactly as it \
         does on the ordinary --screenshot one"
    );
}

/// **THE BOTTOM IDENTITY's FOLDER LINE ALWAYS NAMES THE ACTIVE FILE's OWN
/// FOLDER, NEVER THE NOMINALLY "active project."** `switch_project`
/// (`s-S-p`/`C-S-p`, `Action::OpenProject`) moves `project_location.root`
/// with no document opened or activated, so the gutter's project line must
/// keep naming the root the OPEN file remembers — while every destination
/// default (New document, Go to, Move, export — represented here by the
/// sidecar's own `project.root`, the one field they all read) keeps
/// following the switch. Reverting `CaptureOpts::fold_gutter`'s override
/// makes this go red on `gutter` alone, never on `project`: the two claims
/// are independent by construction, and BOTH are asserted in one frame so
/// neither drifts unnoticed.
///
/// Swept across the WHOLE world roster at two DPIs (not one hand-picked
/// world/scale) because the gutter's project line is a rendered, elided
/// margin string (`rowlayout::fit_primary`, gated on `avail_chars` — a
/// function of the label's own font metrics) — CLAUDE.md's tripwire that a
/// check validated on one scale alone has shipped real DPI-dependent chrome
/// bugs applies here as much as anywhere else. The companion PRESENCE floor
/// (`same_root`) rules out a formatter that always shows nothing: it must
/// name `notes` too, not merely differ from `archive`.
#[test]
fn switch_project_alone_names_the_open_files_folder_while_the_dispatch_root_follows() {
    let _g = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n"),
    );
    let open_project = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-S-p",
        crate::convention::Convention::Linux => "C-S-p",
    };
    // The picker's folders facet opens on the workspace row itself; one
    // `Down` reaches its first alphabetical child (`archive`, ahead of
    // `notes`), `Enter` accepts it — Switch project to `archive`, nothing
    // else.
    let switch_keys = crate::keyspec::parse_chords(&format!("{open_project} Down Enter"))
        .expect("the switch-project walk parses");

    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl-switch-project-identity-{}",
        std::process::id()
    )));

    // `Config` carries no `Clone`, so a fresh one is built per capture
    // rather than shared across the two calls a cell needs. The canvas
    // doubles WITH the dpi (2800x1800 @ 2.0) so the LOGICAL page stays
    // 1400x900 at both scales — a real Retina display, not an artificially
    // narrowed logical page. 1400 rather than the bare 1200x800 default:
    // at 1200 the gutter's own identity-line budget sits AT its presence
    // floor once the close-lane reservation is subtracted (6 - 3 = 3 chars,
    // too few to keep "index.md"'s own extension legible), which starves a
    // law that wants to read a real name/project pair, not probe that floor
    // (rowlayout.rs's own tests already sweep it directly).
    let spec_for = |world: &str, dpi: f32, keys: Vec<crate::keyspec::Chord>| LiveAppSpec {
        file: Some(PathBuf::from("/ws/notes/index.md")),
        keys,
        root: Some(PathBuf::from("/ws/notes")),
        workspace: Some(PathBuf::from("/ws")),
        config: Config {
            theme: Some(world.to_string()),
            ..cfg()
        },
        canvas: Some(if dpi > 1.0 { (2800, 1800) } else { (1400, 900) }),
        dpi: Some(dpi),
    };

    let mut checked = 0usize;
    crate::fs::with_fs(mem, || {
        for world in crate::theme::world_names() {
            for dpi in [1.0f32, 2.0f32] {
                // PRESENCE FLOOR: the ordinary same-root case, no switch at
                // all — the label must still name `notes`, not merely
                // differ from `archive`.
                let same_out = dir.join(format!("same-{world}-{dpi}.png"));
                let same = capture_live_app(same_out.clone(), spec_for(world, dpi, Vec::new()))
                    .map(|()| sidecar(&same_out))
                    .expect("the live-App capture needs a GPU adapter");
                assert_eq!(
                    same["gutter"]["project"].as_str(),
                    Some("notes"),
                    "world={world} dpi={dpi}: the ordinary same-root case \
                     must NAME its folder"
                );

                // THE DECISION: Switch project to `archive`, nothing else.
                let out = dir.join(format!("switch-{world}-{dpi}.png"));
                let v = capture_live_app(out.clone(), spec_for(world, dpi, switch_keys.clone()))
                    .map(|()| sidecar(&out))
                    .expect("the live-App capture needs a GPU adapter");
                assert_eq!(
                    v["gutter"]["project"].as_str(),
                    Some("notes"),
                    "world={world} dpi={dpi}: the identity must keep naming \
                     the OPEN file's own folder, never the switched-to \
                     project — non-vacuous by construction (this read \
                     `archive` before the fix)"
                );
                // The name line is elided independently at some world/DPI
                // combinations (`rowlayout::fit_primary`, unrelated to this
                // item's fix) — checked for PRESENCE and its preserved
                // extension, not byte-exact text.
                let drawn_name = v["gutter"]["name"].as_str().unwrap_or_default();
                assert!(
                    drawn_name.ends_with(".md") && !drawn_name.is_empty(),
                    "world={world} dpi={dpi}: the open file itself never \
                     changed, got {drawn_name:?}"
                );
                assert_eq!(
                    v["project"]["root"].as_str(),
                    Some("/ws/archive"),
                    "world={world} dpi={dpi}: the DISPATCH root (New \
                     document / Go to / Move / export's destination) must \
                     still follow Switch project — the two halves are \
                     deliberately not re-synced, only the label stops lying"
                );
                checked += 1;
            }
        }
    });
    assert_eq!(
        checked,
        crate::theme::world_names().len() * 2,
        "the roster x DPI sweep lost cells"
    );
}

/// **THE PIXEL SEAM'S OWN NON-VACUITY PROOF.** A broken frame can report a
/// correct `overlay.workspace: true`, `detail_focus: true` and
/// `items: ["credits"]` sidecar while the PIXELS draw a one-row `credits ›`
/// palette card over a frosted-blur page. So this law does not read the
/// sidecar's workspace flag as its proof; it reads the PNG.
///
/// A summoned WORKSPACE takes the viewport
/// (`render/chrome/workspace.rs::WORKSPACE_MARGIN_FRAC` leaves only ~5.5% of
/// the smaller dimension as margin on each side), so the bounding box of
/// pixels a real `--screenshot-app` capture changes when Credits opens must
/// span nearly the whole canvas. The frosted-fallback bug drew a small
/// one-row card instead — nowhere close to full-viewport — so a bounding-box
/// span is what tells the two states apart without hand-computing the
/// content pane's exact rect. [`sharp_diff_box`] carries the pixel
/// arithmetic; this function only drives the capture and grades its report.
#[test]
fn credits_opens_a_full_viewport_workspace_not_a_one_row_card() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-credits-px-{}", std::process::id())),
    );
    let doc = PathBuf::from("/ws/proj/notes.md");
    let quiet_png = dir.join("quiet.png");
    let open_png = dir.join("credits.png");

    in_sandbox_with(&doc, || {
        capture_live_app(
            quiet_png.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: Vec::new(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("quiet live-app capture succeeds");

        // `capture_live_app` drives a real App, which resolves chords through
        // its own AMBIENT convention — so the palette-open chord must match
        // whichever convention `native-gate.sh` is forcing for this pass
        // (`command_palette` is Cmd-P on Mac, C-p on Linux; the Cmd/Super slot
        // is unbound under `Convention::Linux`).
        let palette_chord = match crate::convention::Convention::current() {
            crate::convention::Convention::Mac => "s-p",
            crate::convention::Convention::Linux => "C-p",
        };
        let keys = crate::keyspec::parse_chords(&format!("{palette_chord} c r e d i t s Enter"))
            .expect("the credits palette chord parses");
        capture_live_app(
            open_png.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys,
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("credits live-app capture succeeds");
    });

    // STATE, for context only — a workspace shape can be reported correctly
    // in the sidecar while the pixels draw something else, so it is not this
    // law's proof.
    let sc = sidecar(&open_png);
    assert_eq!(
        sc["overlay"]["mode"], "credits",
        "the sidecar names Credits"
    );
    assert_eq!(
        sc["overlay"]["workspace"], true,
        "the sidecar already claimed a workspace on the broken frame"
    );

    let quiet_img = image::open(&quiet_png)
        .expect("decode quiet PNG")
        .to_rgba8();
    let open_img = image::open(&open_png)
        .expect("decode credits PNG")
        .to_rgba8();
    let (w, h) = quiet_img.dimensions();
    assert_eq!((w, h), open_img.dimensions());

    let (differing, sharp, [min_x, min_y, max_x, max_y]) = sharp_diff_box(&quiet_img, &open_img);
    assert!(
        differing > 500,
        "opening Credits changed only {differing} of {w}x{h} pixels — its content is not \
         reaching the screen at all"
    );
    assert!(
        sharp > 500,
        "opening Credits produced only {sharp} sharp-edged (text-like) pixels of {differing} \
         changed — no legible ink reached the screen, only a soft wash"
    );
    let box_w_frac = (max_x - min_x + 1) as f32 / w as f32;
    let box_h_frac = (max_y - min_y + 1) as f32 / h as f32;
    assert!(
        box_w_frac > 0.7 && box_h_frac > 0.7,
        "the SHARP (text-like) pixel bounding box is only {box_w_frac:.2}x{box_h_frac:.2} of \
         the {w}x{h} canvas (box [{min_x},{min_y}]..[{max_x},{max_y}], {sharp} sharp of \
         {differing} changed) — a summoned workspace takes the viewport \
         (WORKSPACE_MARGIN_FRAC leaves ~89% of it to the content), so legible ink confined to \
         a box this small means Credits opened as a one-row card over a blurred page rather \
         than the full-viewport workspace its own sidecar reports: exactly the \
         frosted-fallback bug this law names"
    );
}

/// A BLUR BACKDROP changes color everywhere it covers but stays SMOOTH —
/// neighbouring pixels stay close in value. Real glyph ink is the opposite:
/// sharp edges against its own plate. `SHARP_GRADIENT` is the local
/// two-neighbour colour-distance floor that tells the two apart.
const SHARP_GRADIENT: i32 = 90;

fn is_sharp_at(img: &image::RgbaImage, x: u32, y: u32) -> bool {
    let (iw, ih) = img.dimensions();
    let c = img.get_pixel(x, y).0;
    let mut g = 0i32;
    if x + 1 < iw {
        let r = img.get_pixel(x + 1, y).0;
        g += (c[0] as i32 - r[0] as i32).abs()
            + (c[1] as i32 - r[1] as i32).abs()
            + (c[2] as i32 - r[2] as i32).abs();
    }
    if y + 1 < ih {
        let d = img.get_pixel(x, y + 1).0;
        g += (c[0] as i32 - d[0] as i32).abs()
            + (c[1] as i32 - d[1] as i32).abs()
            + (c[2] as i32 - d[2] as i32).abs();
    }
    g >= SHARP_GRADIENT
}

/// Sweep two same-size frames and report: how many pixels differ at all, how
/// many of those are SHARP in `after` (real ink, not a smooth blur wash —
/// [`is_sharp_at`]), and the sharp pixels' own bounding box as
/// `[min_x, min_y, max_x, max_y]` (clamped to the frame's own corners when
/// none are sharp, which the caller's presence floor catches).
fn sharp_diff_box(before: &image::RgbaImage, after: &image::RgbaImage) -> (usize, usize, [u32; 4]) {
    let (w, h) = before.dimensions();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (w, h, 0u32, 0u32);
    let (mut differing, mut sharp) = (0usize, 0usize);
    for y in 0..h {
        for x in 0..w {
            if before.get_pixel(x, y) == after.get_pixel(x, y) {
                continue;
            }
            differing += 1;
            if is_sharp_at(after, x, y) {
                sharp += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    (differing, sharp, [min_x, min_y, max_x, max_y])
}

/// The Go-to picker's own chord, per running convention — `native-gate.sh`
/// runs the suite once per convention and each pass drives its real binding.
fn goto_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-o",
        crate::convention::Convention::Linux => "C-o",
    }
}

/// The two-file sandbox the rail-reservation laws below drive: one markdown
/// document WITH a heading and one without, in the same project.
fn rail_sandbox() -> std::sync::Arc<crate::fs::InMemoryFs> {
    Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(
                std::path::Path::new("/ws/proj/headed.md"),
                "# Title\n\nprose under it\n",
            )
            .with_file(
                std::path::Path::new("/ws/proj/plain.md"),
                "just prose, no heading anywhere\n\nanother paragraph\n",
            ),
    )
}

/// Drive one live-`App` capture at `canvas`, opening `file` and pressing
/// `spec`, and report the sidecar's own writing-column left edge.
fn column_left_after(out: &std::path::Path, file: &str, spec: &str, canvas: (u32, u32)) -> f64 {
    capture_live_app(
        out.to_path_buf(),
        LiveAppSpec {
            file: Some(std::path::PathBuf::from(file)),
            keys: crate::keyspec::parse_chords(spec).expect("the chord spec parses"),
            root: Some(proj()),
            workspace: None,
            config: cfg(),
            canvas: Some(canvas),
            dpi: None,
        },
    )
    .expect("the live-App capture needs a GPU adapter");
    sidecar(out)["page"]["column"]["left"]
        .as_f64()
        .expect("a page block with a column")
}

/// **SWITCHING FILES MUST NOT MOVE THE PAGE — THROUGH A REAL BUFFER SWITCH, IN
/// BOTH ORDERS, ACROSS THE WINDOW-WIDTH AXIS.**
///
/// The adaptive column shifts right to seat the margin outline's rail. Gated on
/// the CURRENT buffer's headings, a headed file and a heading-free one sat in
/// different regimes, so a Go-to between them slid the whole page — column,
/// gutter, margins — sideways by the rail's whole appetite. The user's report
/// was "switching between files actually causes the side bar to resize… it
/// shouldn't jump all over the place".
///
/// This is the transition an ordinary capture cannot reach: tier 1 skips the
/// live effects a real switch performs (`docs/harness-reach.md`), so the law
/// drives `--screenshot-app` — a real `App`, real chords through the real
/// keymap, the picker actually opening the second file — and reads the column
/// off the same sidecar every other capture law reads.
///
/// **Both orders**, because the two are different code paths through the
/// registry (park-then-open vs. open-then-park) and a fix that held for one
/// only would read as green from either side alone.
///
/// **Swept across the width axis, with the enrolment derived rather than
/// picked:** the jump exists only under width PRESSURE. Above it the placement
/// policy is a byte-identical passthrough and every cell agrees for reasons
/// that have nothing to do with this fix, so a law run at one wide window would
/// sweep nothing. Each width is classified by its own two single-buffer
/// captures, and both classes must be non-empty — the failure message names the
/// widths that enrolled in each.
#[test]
fn switching_buffers_never_moves_the_writing_column_in_either_order() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-rail-{}", std::process::id())),
    );
    let to_plain = format!("{} p l a i n . m d RET", goto_chord());
    let to_headed = format!("{} h e a d e d . m d RET", goto_chord());

    let mut pressure: Vec<u32> = Vec::new();
    let mut passthrough: Vec<u32> = Vec::new();
    let widths: [u32; 6] = [800, 1000, 1200, 1400, 1700, 2000];
    crate::fs::with_fs(rail_sandbox(), || {
        for w in widths {
            let canvas = (w, 800);
            let at = |name: &str| dir.join(format!("{name}-{w}.png"));
            // The two SINGLE-buffer sessions: what the reader saw on either
            // side of the switch under the old, document-keyed reservation.
            let headed_alone =
                column_left_after(&at("headed-alone"), "/ws/proj/headed.md", "", canvas);
            let plain_alone =
                column_left_after(&at("plain-alone"), "/ws/proj/plain.md", "", canvas);
            // The same two documents, both open, reached by a real Go-to.
            let headed_then_plain = column_left_after(
                &at("headed-then-plain"),
                "/ws/proj/headed.md",
                &to_plain,
                canvas,
            );
            let plain_then_headed = column_left_after(
                &at("plain-then-headed"),
                "/ws/proj/plain.md",
                &to_headed,
                canvas,
            );

            assert_eq!(
                headed_then_plain, headed_alone,
                "at {w}px: opening the heading-free file from the headed one \
                 moved the writing column ({headed_alone} -> \
                 {headed_then_plain}). The headed file is still open, so the \
                 room still owes the outline its rail; the heading-free buffer \
                 simply draws nothing in it."
            );
            assert_eq!(
                plain_then_headed, headed_alone,
                "at {w}px: the reverse order landed on a different column \
                 ({plain_then_headed}) than the same two open files reached \
                 the other way ({headed_alone}) — the reservation must not \
                 depend on which file the reader arrived at"
            );

            if headed_alone == plain_alone {
                passthrough.push(w);
            } else {
                pressure.push(w);
            }
        }
    });

    assert!(
        !pressure.is_empty(),
        "no swept width put the column under rail pressure, so every equality \
         above was trivially true and this law witnessed nothing. Swept \
         {widths:?}; all classified passthrough: {passthrough:?}"
    );
    assert!(
        !passthrough.is_empty(),
        "no swept width landed in the passthrough regime, so the wide window — \
         where the reservation must remain a no-op — went untested. Pressure \
         widths: {pressure:?}"
    );
}

/// **CLOSING THE LAST HEADED BUFFER RELEASES THE RESERVATION.** The column is
/// the room's, and closing a file changes the room — one of the moments it is
/// ALLOWED to move, and the one that proves the reservation is not simply
/// latched on forever once anything claims it.
///
/// Driven at tier 2 because `finish_buffer` is Unsupported in an ordinary
/// replay (`docs/harness-reach.md`'s effect table): only a live `App` actually
/// closes the file and activates its successor.
///
/// The single-buffer reference is captured in the SAME sandbox at the same
/// canvas, so the assertion is "back to where a heading-free session sits",
/// not "some number went down".
#[test]
fn closing_the_last_headed_buffer_releases_the_rail_reservation() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-rail-close-{}", std::process::id())),
    );
    let canvas = (1200, 800);
    let open_headed = format!("{} h e a d e d . m d RET", goto_chord());
    let then_close = format!("{open_headed} {}", finish_file_chord());

    let (plain_alone, both_open, after_close, closed_json) =
        crate::fs::with_fs(rail_sandbox(), || {
            let plain_alone = column_left_after(
                &dir.join("plain-alone.png"),
                "/ws/proj/plain.md",
                "",
                canvas,
            );
            let both_open = column_left_after(
                &dir.join("both-open.png"),
                "/ws/proj/plain.md",
                &open_headed,
                canvas,
            );
            let closed = dir.join("after-close.png");
            let after_close = column_left_after(&closed, "/ws/proj/plain.md", &then_close, canvas);
            (plain_alone, both_open, after_close, sidecar(&closed))
        });

    assert_ne!(
        both_open, plain_alone,
        "the fixture must actually be under rail pressure at this canvas, or \
         the release below is unobservable: opening the headed file left the \
         column at {both_open}, the same place a heading-free session sits"
    );
    assert_eq!(
        closed_json["buffers"]["open"].as_u64(),
        Some(1),
        "the close really happened — one buffer left open"
    );
    assert_eq!(
        after_close, plain_alone,
        "closing the last headed buffer must give the margin back: the column \
         returned to {after_close}, not the heading-free session's \
         {plain_alone}"
    );
}

/// **THE RESERVATION FOLLOWS HEADINGS APPEARING AND DISAPPEARING THROUGH
/// EDITING, WITH NO BUFFER SWITCH AT ALL.** The set-level half is a fact about
/// the buffers BEHIND the reader; the active buffer's own half must stay live,
/// re-derived from the shaped headings every sync rather than latched at open
/// time. Typing `# ` claims the rail; deleting it back out releases it.
///
/// Asserted through the live door — the same one the switch laws use — so the
/// claim covers a real editor performing real edits, and both directions are
/// present: a one-way law is satisfied by a reservation that latches on.
#[test]
fn editing_a_heading_in_and_out_claims_and_releases_the_rail_without_a_switch() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-rail-edit-{}", std::process::id())),
    );
    let canvas = (1200, 800);

    let (plain_alone, headed_alone, typed_in, edited_out) =
        crate::fs::with_fs(rail_sandbox(), || {
            let plain_alone = column_left_after(
                &dir.join("plain-alone.png"),
                "/ws/proj/plain.md",
                "",
                canvas,
            );
            let headed_alone = column_left_after(
                &dir.join("headed-alone.png"),
                "/ws/proj/headed.md",
                "",
                canvas,
            );
            // Type a heading at the top of the heading-free document.
            let typed_in = column_left_after(
                &dir.join("typed-in.png"),
                "/ws/proj/plain.md",
                "# Space T Enter Enter",
                canvas,
            );
            // Delete the `# ` marker off the headed document's ONLY heading:
            // the title text survives as plain prose, so this is the heading
            // leaving, not the line leaving.
            let edited_out = column_left_after(
                &dir.join("edited-out.png"),
                "/ws/proj/headed.md",
                "Right Right Backspace Backspace",
                canvas,
            );
            (plain_alone, headed_alone, typed_in, edited_out)
        });

    assert_ne!(
        headed_alone, plain_alone,
        "the fixture must be under rail pressure at this canvas for either \
         direction below to be observable"
    );
    assert_eq!(
        typed_in, headed_alone,
        "typing a heading into the only open buffer claims the rail: the \
         column should have moved to {headed_alone}, not stayed at {typed_in}"
    );
    assert_eq!(
        edited_out, plain_alone,
        "deleting the only heading's `# ` marker releases the rail: the column \
         should have returned to {plain_alone}, not stayed at {edited_out}"
    );
}

// --- FOLLOWING A LINK, through both drivers ------------------------------

const NOTE: &str = "/ws/proj/notes/today.md";
const SIBLING: &str = "/ws/proj/notes/sibling.md";

/// A tiny vault: one note whose FIRST BYTE is inside a relative link, and the
/// sibling it points at. The link leads the document so the caret's own
/// opening position is already inside it — no motion chords to keep in sync
/// with a fixture's layout.
fn in_vault<T>(body: impl FnOnce() -> T) -> T {
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_dir("/ws/proj/notes")
            .with_file(
                std::path::Path::new(NOTE),
                "[the sibling](sibling.md) is next door.\n",
            )
            .with_file(std::path::Path::new(SIBLING), "# Sibling\n\nArrived.\n"),
    );
    crate::fs::with_fs(mem, body)
}

/// The follow chord in the ONE spelling that resolves under BOTH conventions —
/// `Ctrl+Super` is the convention-agnostic emacs fallback layer
/// (`both_convention_modifiers_keep_the_data_backed_emacs_fallback`), so this
/// law drives the same real binding on each of `native-gate.sh`'s two passes
/// instead of picking a chord one of them displaces.
fn follow_chords() -> Vec<crate::keyspec::Chord> {
    crate::keyspec::parse_chords("C-s-c C-s-o").expect("the follow chord parses")
}

/// One ordinary `--keys` capture of `file` under the vault root — the shared
/// door all three arms of the law below drive, so the arms differ only in the
/// document and the chords, never in how the capture was set up.
fn follow_capture(out: &std::path::Path, file: &str, keys: Vec<crate::keyspec::Chord>) {
    super::super::capture_screenshot(
        out.to_path_buf(),
        Some(PathBuf::from(file)),
        CaptureOpts::default(),
        keys,
        crate::keymap::KeymapState::new(),
        Some(proj()),
        None,
        PathBuf::from("/ws/proj"),
        cfg(),
        false,
    )
    .expect("the ordinary capture succeeds");
}

/// LAW: following a RELATIVE link is a typed effect carrying its resolved
/// destination, and the destination is READABLE FROM THE SIDECAR through both
/// capture doors — the shared-core `--keys` replay and the real-`App` driver.
///
/// The relative arm is deliberately the one under the oracle: it is the arm
/// whose effect (`OpenPathAtLine`) is Applied at BOTH tiers, so a sidecar can
/// witness it honestly. The external arm is Intercepted by contract — the OS
/// opener handoff is the live-only tail — and is asserted at the effect seam
/// instead (`actions::follow::tests`), never stubbed around here. A capture
/// that followed an external URL would spawn a browser, which is exactly what
/// the Intercepted classification exists to prevent.
#[test]
fn following_a_relative_link_lands_the_destination_in_both_drivers_sidecars() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-follow-link-{}", std::process::id())),
    );

    // ── THE ORDINARY `--keys` DOOR (shared core) ──────────────────────
    let replay = dir.join("replay.png");
    let replay_json = in_vault(|| {
        follow_capture(&replay, NOTE, follow_chords());
        sidecar(&replay)
    });
    assert_eq!(replay_json["driver"].as_str(), Some("replay"));
    assert_eq!(
        replay_json["buffers"]["active"].as_str(),
        Some(SIBLING),
        "the follow chord opened the link's own destination IN awl; the sidecar \
         names the arrived-at document, not the one the chord was pressed in"
    );
    assert!(
        replay_json["replay_skips"]
            .as_array()
            .is_some_and(|a| a.is_empty()),
        "a relative follow is Applied end to end — nothing is skipped: {:?}",
        replay_json["replay_skips"]
    );

    // ── THE REAL-`App` DRIVER ─────────────────────────────────────────
    let live = dir.join("live.png");
    let live_json = in_vault(|| {
        capture_live_app(
            live.clone(),
            LiveAppSpec {
                file: Some(PathBuf::from(NOTE)),
                keys: follow_chords(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("the live-App capture needs a GPU adapter");
        sidecar(&live)
    });
    assert_eq!(live_json["driver"].as_str(), Some("live-app"));
    assert_eq!(
        live_json["buffers"]["active"].as_str(),
        Some(SIBLING),
        "the real App follows to the same destination the shared core resolved"
    );

    // ── ANTI-VACUITY, both halves. Without these, "we are in sibling.md"
    // would also pass if the capture had simply never left the file it was
    // handed, or had opened it for some unrelated reason.
    //
    // (a) NO chords at all: the capture stays in the note it was given, so the
    //     switch above is the chord's doing.
    let still = dir.join("still.png");
    let still_json = in_vault(|| {
        follow_capture(&still, NOTE, Vec::new());
        sidecar(&still)
    });
    assert_eq!(
        still_json["buffers"]["active"].as_str(),
        Some(NOTE),
        "with no chords the capture never leaves the note it was handed"
    );
    // (b) The SAME chord in a document with nothing to follow switches nothing.
    let inert = dir.join("inert.png");
    let inert_json = in_vault(|| {
        follow_capture(&inert, SIBLING, follow_chords());
        sidecar(&inert)
    });
    assert_eq!(
        inert_json["buffers"]["active"].as_str(),
        Some(SIBLING),
        "the same chord on plain prose follows nothing and switches nothing"
    );
}

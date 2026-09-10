//! The no-document start surface's folder-naming dim line.
//!
//! Two claims, at two seams:
//!   1. THE CHORD LABEL is read through `keytoken::key_token_label`, not a
//!      literal glyph — proven directly against both conventions (a pure
//!      function of its `Convention` parameter, so both are checkable in one
//!      process with no env forcing) rather than only whichever convention
//!      this host happens to build under. `keymap::tests::
//!      go_to_ctrl_o_fires_under_either_linux_keymap_flavor` is this claim's
//!      DISPATCH-truth companion — a label is worthless if the chord it
//!      names does not actually fire.
//!   2. THE LINE IS REAL INK, not just a byte in `ViewState` — a presence
//!      floor over the exact rect `prepare_start_surface` drew it at
//!      (`TextPipeline::folder_line_rect`, the SAME geometry, so this cannot
//!      drift from what the surface actually draws), enrolled from the
//!      THEMES roster and proven non-vacuous over both a light and a dark
//!      ground.

use super::super::*;
use super::{headless_dqp, pixeldiff};

const W: u32 = 480;
const H: u32 = 360;

fn zero_document_view(folder: Option<&str>) -> ViewState {
    let mut v = ViewState::base();
    v.document_active = false;
    v.start_folder = folder.map(str::to_string);
    v
}

/// LAW, found by LIVE visual verification rather than by a unit test (a real
/// gap this test now closes): a box sized for the two SHORT action rows let
/// the folder line's own longer text — worst case, a near-cap folder name
/// plus `Convention::Linux`'s word-labelled "Ctrl+O" (longer than the mac
/// glyph "⌘O") — wrap onto a second visual row, silently turning "one dim
/// line" into two and pushing the row below it further down (a
/// `--screenshot-app --keys C-w` capture under `AWL_CONVENTION_FORCE=linux`
/// showed the second action row pushed clean out of the shaped buffer).
/// `layout_runs()` yields one run per WRAPPED visual row, so a folder line
/// that wrapped raises the total above the 3 logical lines. This runs under
/// whichever convention is ambient — genuinely Linux on CI's linux job, not
/// only a forced label — so it sweeps the real axis the live check found,
/// not a re-assertion of the fix's own arithmetic.
#[test]
fn folder_line_never_wraps_even_at_the_truncation_cap() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping folder_line_never_wraps_even_at_the_truncation_cap: no wgpu adapter");
        return;
    };
    let long_name = "n".repeat(60);
    p.set_view(&zero_document_view(Some(&long_name)));
    p.prepare(&device, &queue, W, H).expect("prepare");
    let runs = p.gutter_buffer.layout_runs().count();
    assert_eq!(
        runs,
        3,
        "the folder line wrapped onto a second visual row under convention {:?} — \
         widen its box or shorten the truncation cap",
        crate::convention::Convention::current()
    );
}

#[test]
fn goto_chord_label_resolves_through_keytoken_for_both_conventions() {
    use crate::commands::Platform;
    use crate::convention::Convention;
    assert_eq!(
        crate::keytoken::key_token_label("go_to", Convention::Mac, Platform::Native),
        Some("\u{2318}O".to_string()),
        "mac: the folder line's chord must read the same glyph the catalog's \
         own Cmd-O default resolves to"
    );
    assert_eq!(
        crate::keytoken::key_token_label("go_to", Convention::Linux, Platform::Native),
        Some("Ctrl+O".to_string()),
        "linux: the naive Cmd\u{2192}Ctrl translation, word-labelled — never a \
         mac glyph surviving onto this convention"
    );
    // `goto_chord_label()` (the render-side owner `prepare_start_surface` and
    // the sidecar both call) must be BYTE-IDENTICAL to the direct keytoken
    // call for the process's own ambient convention — proving it is a thin
    // wrapper, never a second, possibly-diverging derivation.
    let ambient = crate::render::chrome::goto_chord_label();
    let expected =
        crate::keytoken::key_token_label("go_to", Convention::current(), Platform::current())
            .unwrap_or_default();
    assert_eq!(ambient, expected);
}

/// Real-pixel PRESENCE floor: the folder line's own rect, sampled by diffing a
/// frame that names a folder against an otherwise-identical frame that names
/// none, over the roster THEMES actually ships — never a named world — proven
/// non-vacuous over both a light and a dark ground so the floor cannot be
/// satisfied by a roster that happens to enrol only one.
#[test]
fn folder_line_draws_a_visible_pixel_in_every_world_light_and_dark() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping folder_line_draws_a_visible_pixel_in_every_world_light_and_dark: \
             no wgpu adapter"
        );
        return;
    };

    fn frame(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<[u8; 4]> {
        p.caret_pipeline.prepare_empty();
        p.caret_trail_pipeline.prepare_empty();
        p.caret_glyph_pipeline.clear();
        pixeldiff::render_frame(p, device, queue, W, H)
    }

    // Presence is asked by diffing an EMPTY-NAME frame against a NAMED one —
    // both `Some(..)`, so `has_folder` (and therefore every row's geometry)
    // is IDENTICAL between them; only the folder-name glyphs themselves
    // differ. Two traps this dodges: (1) diffing against a `start_folder:
    // None` frame instead would re-centre the whole two-row block (three
    // rows now share the vertical budget instead of two), so a `None`-
    // frame's OWN row-0 text can fall inside the named frame's folder-line
    // rect and register as "presence" that is really just the action rows
    // having moved. (2) diffing against a flat, hand-picked "ground" color
    // sampled once is falsified by any world whose background is a texture
    // (dots/stipple/gradient) rather than a flat fill — every pixel in the
    // rect would then "differ" from that one sample regardless of whether
    // any ink was drawn at all. A same-geometry, same-background two-frame
    // diff is immune to both.
    let prev = theme::active().name;
    let mut checked = 0usize;
    let mut light_checked = 0usize;
    let mut dark_checked = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();

        p.set_view(&zero_document_view(Some("")));
        p.prepare(&device, &queue, W, H)
            .expect("prepare (empty name)");
        let rect = p
            .folder_line_rect(W, H)
            .unwrap_or_else(|| panic!("{}: folder_line_rect must be Some once named", t.name));
        let blank = frame(&mut p, &device, &queue);

        p.set_view(&zero_document_view(Some("seeded-notes")));
        p.prepare(&device, &queue, W, H)
            .expect("prepare (folder named)");
        assert_eq!(
            p.folder_line_rect(W, H),
            Some(rect),
            "{}: naming the folder must not move the line — only its text should change",
            t.name
        );
        let named = frame(&mut p, &device, &queue);

        let (x0, y0, x1, y1) = (
            rect[0].floor().max(0.0) as i32,
            rect[1].floor().max(0.0) as i32,
            (rect[0] + rect[2]).ceil().min(W as f32) as i32,
            (rect[1] + rect[3]).ceil().min(H as f32) as i32,
        );
        let mut differing = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = y as usize * W as usize + x as usize;
                if named[idx] != blank[idx] {
                    differing += 1;
                }
            }
        }
        assert!(
            differing > 0,
            "{}: no pixel inside the folder line's own rect {rect:?} changed between an \
             empty name and a real one — the folder name draws nothing",
            t.name
        );

        checked += 1;
        if t.dark {
            dark_checked += 1;
        } else {
            light_checked += 1;
        }
    }
    theme::set_active_by_name(prev).unwrap();
    assert_eq!(
        checked,
        theme::THEMES.len(),
        "every world must be swept, not a hand-picked sample"
    );
    assert!(
        light_checked > 0,
        "no light-ground world enrolled — floor is vacuous on light"
    );
    assert!(
        dark_checked > 0,
        "no dark-ground world enrolled — floor is vacuous on dark"
    );
}

/// LAW: nothing changes when `start_folder` is `None` — the exact byte
/// contract `ViewState::base()`'s inert default promises every fixture that
/// never opts in. Checks THREE independent things a "the block just grew a
/// blank slot" regression could satisfy individually: the field, the rect,
/// the shaped line count, AND (the part a line-count check alone cannot see)
/// the two actions' own GEOMETRY — pinned against the ORIGINAL (folder-line-
/// free) two-row centring formula, so a `has_folder` computed independently of
/// `start_folder` (reserving room without adding text, or vice versa) fails
/// here even though it would leave the line count untouched.
#[test]
fn no_folder_named_renders_byte_identically_to_the_original_two_rows() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping no_folder_named_renders_byte_identically_to_the_original_two_rows: \
             no wgpu adapter"
        );
        return;
    };
    p.set_view(&zero_document_view(None));
    p.prepare(&device, &queue, W, H).expect("prepare");
    assert_eq!(
        p.start_folder(),
        None,
        "an opted-out fixture must leave start_folder at its inert default"
    );
    assert!(
        p.folder_line_rect(W, H).is_none(),
        "no rect should be reported when nothing is named"
    );
    assert_eq!(
        p.gutter_buffer.lines.len(),
        2,
        "the original two-row start block must be untouched"
    );

    // The pinned ORIGINAL two-row centring formula (`start_rows(.., false)`
    // in production) — a conscious edit to that formula updates this pinned
    // pair too, exactly like `render::tests::mod::diagonal_worlds`'s own
    // pinned roster above.
    let row_h = p.metrics.line_height * crate::markdown::type_scale::LABEL;
    let block_h = row_h * 2.0;
    let top = ((H as f32 - block_h) * 0.5).max(0.0);
    let row_w = (W as f32 * 0.4).clamp(180.0, 360.0);
    let left = (W as f32 - row_w) * 0.5;
    let cx = left + row_w * 0.5;
    assert_eq!(
        p.start_action_at(cx, top + row_h * 0.5),
        Some(crate::keymap::Action::NewDocument),
        "row 0 must sit at the ORIGINAL (no-folder-line) centring, not shifted \
         down to make room for a line that draws nothing"
    );
    assert_eq!(
        p.start_action_at(cx, top + row_h * 1.5),
        Some(crate::keymap::Action::OpenGoto),
        "row 1 must sit at the ORIGINAL (no-folder-line) centring"
    );
}

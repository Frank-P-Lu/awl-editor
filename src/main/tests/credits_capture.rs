//! THE CREDITS VIEWER, DRIVEN BY REAL `--keys` CHORDS THROUGH THE COMMAND
//! PALETTE — the end-to-end proof that `Action::OpenCredits` reaches
//! `OverlayKind::Credits` through the real keymap + palette fuzzy filter, not
//! only through a hand-built `ActionCtx` (`actions::tests::credits` owns
//! that purer-seam half of the same law). `replay_keys` is the exact seam
//! `history.rs`'s `replay_history_esc_leaves_buffer_text_exact` already uses
//! for the sibling workspace.

use super::super::*;
use super::{keyspec, replay_keys};
use crate::testscratch::ScratchDir;

/// THE ONE FIXTURE WRITE in this file. Both capture-tier laws need the same
/// quiet document on disk, and the durable-write census counts bare
/// `std::fs::write` call sites per file — so they share this rather than each
/// spelling their own.
fn write_fixture(dir: &ScratchDir) -> PathBuf {
    let fixture = dir.join("note.md");
    std::fs::write(
        &fixture,
        "# My Notes\n\nSome real prose the user is editing.\n",
    )
    .unwrap();
    fixture
}

fn credits_buffer() -> Buffer {
    let mut b = Buffer::from_str("# My Notes\n\nSome real prose the user is editing.\n");
    b.set_path(PathBuf::from("/notes/draft.md"));
    b
}

/// Cmd-P, type "credits", Enter: the palette's fuzzy filter lands on the
/// Credits row and accepting it summons `OverlayKind::Credits` standing on
/// its CONTENT stage already — there is no row to choose, so PageDown must
/// scroll immediately rather than stepping an inert one-row rail.
#[test]
fn replay_credits_opens_via_the_palette_onto_the_content_stage_and_scrolls() {
    let mut buffer = credits_buffer();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter").unwrap();
    let root = PathBuf::from("/notes");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let card = res
        .journey
        .card()
        .expect("Cmd-P -> credits -> Enter summons the viewer");
    assert_eq!(card.kind, crate::overlay::OverlayKind::Credits);
    assert!(
        card.detail_focus,
        "Credits opens with the content stage already focused"
    );
    assert_eq!(card.diff_scroll, 0);
    assert!(
        res.replay_skips.is_empty(),
        "opening Credits is fully replay-supported, not a live-App-only degrade: {:?}",
        res.replay_skips
    );

    // A second replay, one PageDown further, moves the scroll — the SAME
    // `diff_scroll` field History/Conflict already drive, proving the
    // universal workspace scroll keys reach Credits through the real keymap.
    let keys2 = keyspec::parse_keys("s-p c r e d i t s Enter PageDown").unwrap();
    let mut buffer2 = credits_buffer();
    let res2 = replay_keys(
        &mut buffer2,
        &keys2,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    );
    assert!(
        res2.journey.card().unwrap().diff_scroll > 0,
        "PageDown must move diff_scroll, or the byte-identity law below would be \
         vacuously true of a viewer that never actually opened"
    );
}

/// **THE REGRESSION LAW.** Credits used to swap the editor to a real editable
/// buffer (`App::open_credits`, routed through `load_path`). Driven
/// through the real `--keys` seam end to end — palette open, scroll, dismiss —
/// the active buffer's path and text must be byte-for-byte what they were
/// before any of it happened.
///
/// TWO `Esc`, not one: a palette-launched action PARKS the palette as the
/// summon's parent (`Effect::RunAction`'s `pending_return_to`, the same
/// breadcrumb every other palette-launched picker uses — `palette.rs`'s own
/// `"...RET Esc Esc s-t..."` is the precedent), so the first `Esc` returns to
/// the parked Command palette and the second leaves it. Neither Esc, nor the
/// parked palette, ever touches the buffer.
#[test]
fn replay_credits_open_scroll_and_esc_leave_the_buffer_exact() {
    let mut buffer = credits_buffer();
    let before_path = buffer.path().map(|p| p.to_path_buf());
    let before_text = buffer.text();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter PageDown PageDown Esc Esc").unwrap();
    let root = PathBuf::from("/notes");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "two Esc closes the parked palette and the Credits viewer beneath it: {:?}",
        res.journey.card().map(|c| c.kind)
    );
    assert!(
        res.accept.is_none(),
        "the read-only viewer never accepts anything"
    );
    assert_eq!(
        buffer.path().map(|p| p.to_path_buf()),
        before_path,
        "the active buffer's path must not change across open/scroll/dismiss"
    );
    assert_eq!(
        buffer.text(),
        before_text,
        "the active buffer's text must not change across open/scroll/dismiss"
    );
}

/// WCAG relative luminance of an sRGB byte pixel — the same formula
/// `capture::tests::panels::px_rel_lum` uses, reproduced here (that helper
/// lives in a `cfg(test)` module this file can't reach).
fn rel_lum(px: [u8; 4]) -> f64 {
    fn lin(u: u8) -> f64 {
        let s = u as f64 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2])
}

fn contrast(a: [u8; 4], b: [u8; 4]) -> f64 {
    let (x, y) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

/// **PRESENCE + LEGIBILITY, over a world sample, through the real production
/// `--keys` seam.**
///
/// The underlying relocated-document mechanism (`TextPipeline::
/// comparison_viewport`) is already proven kind-agnostic across the FULL
/// 20-world roster by `render::tests::comparison_composite`'s own law —
/// Credits draws through the exact same code History/Conflict already do, no
/// Credits-specific render path exists to re-prove. This law's job is
/// narrower and different: prove that CREDITS' OWN TEXT specifically reaches
/// real pixels when opened through the real production `--keys` seam
/// (palette + keymap + `Action::OpenCredits`), not a capture-literal
/// `OverlayInfo`. Three worlds — a warm default, a dark world, and the
/// ONE-BIT Wagtail whose ink ladder collapses — mirror the exact sample
/// `capture::tests::panels::history_comparison_is_relocated_by_the_capture_path_in_every_world`
/// already uses for this shape of check.
///
/// Two floors, deliberately not one (CLAUDE.md: "a floor over a treatment
/// needs a companion presence floor... a wash four bytes from the page
/// reports a BETTER ratio while drawing nothing"):
///   * PRESENCE — real pixels change at all when Credits opens over the
///     quiet document (`differing.len() > 500`, panels.rs's own floor).
///   * LEGIBILITY — the WCAG relative-luminance contrast between the
///     changed region's ink and its own plate clears the WCAG AA text floor
///     (4.5:1) — the same `INK_ON_PLATE_MIN` the notice channel's own
///     ink/plate law uses (`render/tests/notice.rs`).
///
/// A viewer faded toward the page fails PRESENCE outright rather than
/// passing LEGIBILITY happier, because a `differing.len()` this small — a
/// document whose ink cannot be told apart from its own quiet baseline —
/// never reaches the contrast comparison at all.
#[test]
fn credits_text_is_present_and_legible_across_a_world_sample() {
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-credits-legibility-{}", std::process::id())),
    );
    let fixture = write_fixture(&dir);
    if capture::build_oracle(&Buffer::from_file(&fixture), &CaptureOpts::default()).is_none() {
        eprintln!("skipping credits legibility capture: no wgpu adapter");
        return;
    }

    let entry_world = crate::theme::active_index();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter").unwrap();

    for world in ["Tawny", "Mopoke", "Wagtail"] {
        crate::theme::set_active_by_name(world).expect("a roster world");

        let quiet = dir.join(format!("{world}-quiet.png"));
        capture_screenshot(
            quiet.clone(),
            Some(fixture.clone()),
            CaptureOpts::default(),
            Vec::new(),
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
            Some(dir.to_path_buf()),
            None,
            dir.join("notes"),
            Config::empty(),
            false,
        )
        .expect("quiet capture succeeds");

        let open = dir.join(format!("{world}-credits.png"));
        capture_screenshot(
            open.clone(),
            Some(fixture.clone()),
            CaptureOpts::default(),
            keys.clone(),
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
            Some(dir.to_path_buf()),
            None,
            dir.join("notes"),
            Config::empty(),
            false,
        )
        .expect("credits capture succeeds");

        // STATE: the sidecar's `overlay` block names what really opened. Its
        // top-level `text` is a DISPLAY oracle, not a buffer oracle — while a
        // comparison viewer is open, `text` reports whatever got pushed into
        // the relocated document viewport for THIS frame (the same
        // diff-as-preview substitution History/Conflict already use;
        // `app/viewstate.rs`'s own doc: "the BUFFER is NEVER touched"), so the
        // correct assertion here is that the substitution is CREDITS' text
        // specifically — not the document's. The BUFFER-identity law belongs
        // to `buffer.text()` itself, already proven at the replay seam by
        // this file's other two tests (`replay_keys` hands back the real
        // `Buffer`, which this capture-tier door does not).
        let sidecar: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(open.with_extension("json")).unwrap())
                .unwrap();
        assert_eq!(
            sidecar["overlay"]["mode"], "credits",
            "{world}: the sidecar names the open overlay"
        );
        assert_eq!(
            sidecar["overlay"]["detail_focus"], true,
            "{world}: opens on the content stage"
        );
        assert_eq!(
            sidecar["text"].as_str().unwrap(),
            crate::credits::CREDITS_MD,
            "{world}: the relocated document viewport shows CREDITS.md verbatim"
        );

        let quiet_img = image::open(&quiet).expect("decode quiet PNG").to_rgba8();
        let open_img = image::open(&open).expect("decode credits PNG").to_rgba8();
        let (w, h) = quiet_img.dimensions();
        assert_eq!((w, h), open_img.dimensions());
        let mut differing: Vec<(u32, u32)> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if quiet_img.get_pixel(x, y) != open_img.get_pixel(x, y) {
                    differing.push((x, y));
                }
            }
        }
        assert!(
            differing.len() > 500,
            "{world}: opening Credits changed only {} pixels of {}x{} — its text is not \
             reaching the screen",
            differing.len(),
            w,
            h
        );

        // The PLATE: the most common colour among the changed pixels (glyph
        // ink is a small minority of its own region). The INK: the changed
        // pixel whose luminance is FURTHEST from the plate's — deliberately
        // not "the darkest pixel", which picks the plate itself on a dark
        // world (`render/tests/notice.rs::ink_against`'s own doc names this
        // exact trap).
        let mut counts: std::collections::HashMap<[u8; 4], usize> =
            std::collections::HashMap::new();
        for &(x, y) in &differing {
            *counts.entry(open_img.get_pixel(x, y).0).or_insert(0) += 1;
        }
        let plate = counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(c, _)| c)
            .expect("a non-empty changed region");
        let plate_lum = rel_lum(plate);
        let ink = differing
            .iter()
            .map(|&(x, y)| open_img.get_pixel(x, y).0)
            .max_by(|a, b| {
                (rel_lum(*a) - plate_lum)
                    .abs()
                    .partial_cmp(&(rel_lum(*b) - plate_lum).abs())
                    .expect("finite luminance")
            })
            .expect("a non-empty changed region");
        let legibility = contrast(ink, plate);
        assert!(
            legibility >= 4.5,
            "{world}: credits text ink {ink:?} on its plate {plate:?} is {legibility:.2}:1, \
             under the WCAG AA 4.5:1 text floor"
        );
    }

    // Leave the process-global active world as found, not whichever the loop
    // ended on — `panels.rs`'s own sweep documents why: leaking a non-default
    // world breaks whatever `serial()`-ordered test runs next.
    crate::theme::set_active(entry_world);
}

/// **THE READING SURFACE DRAWS NO CARET, THROUGH THE PRODUCTION `--keys` SEAM.**
///
/// The caret's world-by-world absence is the render tier's law
/// (`render::tests::read_only_caret`, which isolates the caret's own pixels by
/// re-rendering with its pipelines emptied, across the whole roster). What only
/// THIS tier can see is that the production path actually gets there: the real
/// palette + keymap journey into `Action::OpenCredits`, `App::sync_view`'s own
/// projection of the family fact, the capture door's re-derivation of it.
///
/// Every quantity is a rendered pixel compared to another rendered pixel. The
/// caret's box comes from the sidecar's `layout.caret` and its row band — a
/// GEOMETRY question, which is what the sidecar is an oracle for — and the
/// comparison value is the GROUND of that same rendered row, sampled well clear
/// of the caret. Both frames park the caret on a BLANK line, so a box that
/// matches its own row's ground holds no caret and a box that does not, does.
///
/// The presence companion is the same probe on the same document with no viewer
/// open (`Down` parks the caret on the fixture's blank line 1): it must differ
/// from its ground. Without it, "the box matches the ground" is equally true of
/// a renderer that stopped drawing carets, a caret that fell off the canvas, and
/// a capture that photographed the wrong frame.
#[test]
fn the_credits_viewer_photographs_no_caret_through_the_real_palette_journey() {
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-credits-caret-{}", std::process::id())),
    );
    let fixture = write_fixture(&dir);
    if capture::build_oracle(&Buffer::from_file(&fixture), &CaptureOpts::default()).is_none() {
        eprintln!("skipping credits caret capture: no wgpu adapter");
        return;
    }

    /// How many pixels of the caret's own box differ from the GROUND of the
    /// rendered row it sits on — zero means nothing was drawn there.
    fn caret_box_ink(png: &std::path::Path) -> usize {
        let sidecar: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
                .unwrap();
        let caret = &sidecar["layout"]["caret"];
        let row_index = caret["row"]
            .as_u64()
            .expect("the sidecar names the caret row");
        let row = sidecar["layout"]["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .find(|r| r["index"].as_u64() == Some(row_index))
            .expect("the caret's own row is laid out");
        assert_eq!(
            row["content"].as_str(),
            Some(""),
            "both arms park the caret on a BLANK row, so the row's ground is uniform \
             and a difference from it can only be the caret"
        );
        let x0 = caret["x"].as_f64().expect("caret x") as u32;
        let top = row["top"].as_f64().expect("row top") as u32;
        let bottom = top + row["height"].as_f64().expect("row height") as u32;
        let img = image::open(png).expect("decode PNG").to_rgba8();
        // The row's own ground, read off THIS render — never an authored colour.
        let ground = *img.get_pixel(x0 + 120, (top + bottom) / 2);
        (top..bottom)
            .flat_map(|y| (x0..x0 + 14).map(move |x| (x, y)))
            .filter(|&(x, y)| *img.get_pixel(x, y) != ground)
            .count()
    }

    let shot = |name: &str, keys: &str| {
        let png = dir.join(name);
        capture_screenshot(
            png.clone(),
            Some(fixture.clone()),
            CaptureOpts::default(),
            keyspec::parse_keys(keys).unwrap(),
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
            Some(dir.to_path_buf()),
            None,
            dir.join("notes"),
            Config::empty(),
            false,
        )
        .expect("capture succeeds");
        png
    };

    // PRESENCE: the ordinary document, caret parked on its blank line.
    let editing = shot("editing.png", "Down");
    let present = caret_box_ink(&editing);
    assert!(
        present > 0,
        "the ordinary document must draw a caret in its own caret box, or the \
         absence below proves nothing"
    );

    // THE SURFACE: the same document, the real palette journey into Credits.
    let viewing = shot("viewing.png", "s-p c r e d i t s Enter");
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(viewing.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        sidecar["overlay"]["mode"], "credits",
        "the journey really opened the viewer"
    );
    let ink = caret_box_ink(&viewing);
    assert_eq!(
        ink, 0,
        "the Credits viewer photographed {ink} caret pixels in its own caret box \
         (the ordinary document drew {present} there) — a reading surface refuses \
         every insertion door, so it may not draw the accent that promises one"
    );
}

/// **TYPING ON THE CREDITS RAIL DOES NOTHING AT ALL** — driven by real chords
/// through the real keymap.
///
/// Its primary "list" is one fixed row NAMING the document beside it, so a query
/// could only ever hide that row and leave the reader on `no matches` with the
/// prose it named still on screen. `OverlayKind::offers_query` is the fact and
/// `OverlayState::push` — the one door the query grows through — reads it.
///
/// `Left` off the content stage returns to that rail, which is where a typed
/// character used to land.
#[test]
fn replay_typing_on_the_credits_rail_filters_nothing() {
    let mut buffer = credits_buffer();
    let before_text = buffer.text();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter Left z z z").unwrap();
    let root = PathBuf::from("/notes");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let card = res.journey.card().expect("the viewer is still up");
    assert_eq!(card.kind, crate::overlay::OverlayKind::Credits);
    assert!(
        !card.detail_focus,
        "`Left` off the content stage returns to the rail — the stage the typed \
         characters were aimed at"
    );
    assert_eq!(
        card.query.text(),
        "",
        "a card with nothing to search grows no query"
    );
    assert_eq!(
        card.item_strings().len(),
        1,
        "its one row must survive: filtering it away would leave a reader on \
         `no matches` beside the document that row names"
    );
    assert_eq!(
        card.empty_notice(),
        None,
        "and so there is no empty state to show"
    );
    assert_eq!(
        buffer.text(),
        before_text,
        "and nothing reached the buffer behind it"
    );
}

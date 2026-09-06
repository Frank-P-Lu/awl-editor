//! **A READING SURFACE DRAWS NO CARET — IN RENDERED PIXELS, ON EVERY MEMBER OF
//! THE READ-ONLY PROSE FAMILY, IN EVERY WORLD.**
//!
//! The caret is awl's ONE accent and it means "you can write here" (DESIGN §one
//! accent). While the document layer is relocated into a read-only comparison —
//! Version History's timeline, the external-change conflict, the Credits viewer
//! — nothing on screen is writable and every insertion door is walled
//! (`app/input/text_door.rs`). A caret parked in that prose is the editor
//! promising an edit it will refuse.
//!
//! # Two carets, two different rules, and why they are not one rule
//!
//! * The DOCUMENT caret parks for the whole family. It is a fact about the
//!   relocated prose, so `TextPipeline::document_is_a_transcript` — the render
//!   side's projection of `OverlayState::shows_read_only_prose` — is the gate.
//! * The card's own QUERY caret parks only where there is nothing to search.
//!   History's timeline and the conflict's three views are real rosters a typed
//!   query really filters, and a working field with no caret would be worse than
//!   the bug. Credits' "list" is one fixed row NAMING the document, so its query
//!   can only hide that row: `OverlayKind::offers_query` is the fact and the
//!   field's own growth door reads the same one.
//!
//! Because the two rules differ, the sweep below asserts BOTH DIRECTIONS in one
//! pass — the query caret must be absent exactly where `offers_query` is false
//! and present everywhere else. That makes each half the other's presence
//! companion: neither "no caret" reading can be produced by a renderer that
//! quietly stopped drawing carets at all.
//!
//! # Every quantity here is a rendered pixel compared to another rendered pixel
//!
//! The caret's own drawn pixels are isolated by re-rendering the IDENTICAL
//! prepared state with the caret pipelines emptied and diffing the two frames —
//! `caret_one_width_pixels.rs`'s technique, which needs no authored colour and
//! so cannot go red on a backend whose rounding differs from this host's.
//!
//! # Enrolment is derived, not listed
//!
//! Members come from `OverlayKind::ALL` filtered through
//! `OverlayState::shows_read_only_prose` — `comparison_request`'s own
//! wildcard-free roster, asked as a predicate. A fourth read-only surface
//! inherits this law the day it compiles, and what enrolled is named in every
//! failure message.

use super::super::*;
use super::{comparison_view, headless_dqp, pixeldiff, view};
use crate::overlay::{OverlayKind, OverlayState};

const W: u32 = 1200;
const H: u32 = 800;

/// A transcript tall enough that the relocated viewport really holds prose —
/// a short one would leave the region empty and make every reading below
/// vacuous.
fn transcript() -> String {
    let mut s = String::from("# CREDITS\n\n");
    for i in 0..24 {
        s.push_str(&format!(
            "Line {i} of the read-only document a reader is looking at right now.\n\n"
        ));
    }
    s
}

/// The most populated card this file knows how to build for `kind` — the
/// enrolment's own subject, wildcard-free for the reason
/// `app::tests::read_only_surface::representative` gives: a bare card of a
/// read-only kind asks for no comparison, so a fourth member handed one would
/// enrol NOTHING and this law would go on passing over an empty set.
fn representative(kind: OverlayKind) -> OverlayState {
    match kind {
        OverlayKind::History => OverlayState::new_history(
            vec![crate::history::TimelineRow {
                when: "2 hr ago".into(),
                which: "edited \"Title\"".into(),
                counts: "+1 −1".into(),
                id: "1700000000000".into(),
                timestamp: 1_700_000_000_000,
                pinned: false,
                name: None,
            }],
            None,
            None,
        ),
        OverlayKind::Conflict => OverlayState::new_conflict(
            std::path::PathBuf::from("/notes/draft.md"),
            Some("disk text".into()),
        ),
        OverlayKind::Credits => OverlayState::new_credits(),
        OverlayKind::Goto
        | OverlayKind::Project
        | OverlayKind::ProjectBrowse
        | OverlayKind::Browse
        | OverlayKind::Theme
        | OverlayKind::Caret
        | OverlayKind::Dictionary
        | OverlayKind::CjkLang
        | OverlayKind::Date
        | OverlayKind::Keymap
        | OverlayKind::MoveDest
        | OverlayKind::ExportDest
        | OverlayKind::Command
        | OverlayKind::SearchFolder
        | OverlayKind::Spell
        | OverlayKind::Keybindings
        | OverlayKind::Assets
        | OverlayKind::Rename
        | OverlayKind::InsertLink
        | OverlayKind::KeepName
        | OverlayKind::Context
        | OverlayKind::TableDims
        | OverlayKind::Settings => OverlayState::new(kind, vec!["a row".into()], vec![], vec![]),
    }
}

/// THE FAMILY, derived from the roster rather than spelled here.
fn family() -> Vec<OverlayKind> {
    OverlayKind::ALL
        .into_iter()
        .filter(|k| representative(*k).shows_read_only_prose())
        .collect()
}

/// The `ViewState` `App::sync_view` would push for a summoned `kind` showing
/// read-only prose — every overlay field taken from the SAME per-kind owners
/// `sync_view` reads, never re-decided here.
fn read_only_view(kind: OverlayKind, body: &str) -> ViewState {
    let card = representative(kind);
    let mut v = comparison_view(body, 1, 0);
    v.overlay_title = card.title();
    v.overlay_items = card.item_strings();
    v.overlay_query = card.query.text().to_string();
    v.overlay_query_caret = card.query.caret();
    v.overlay_query_field = kind.offers_query();
    v
}

fn differing(a: &[[u8; 4]], b: &[[u8; 4]]) -> usize {
    a.iter().zip(b.iter()).filter(|(x, y)| x != y).count()
}

/// The DOCUMENT caret's own drawn pixels, isolated as a with/without-caret frame
/// diff over the identical prepared state.
///
/// FOUR pipelines, not three: the ONE-BIT worlds draw their caret as a true
/// inverse-video block on `caret_invert`, a pipeline the ordinary worlds never
/// populate. Emptying only the other three read ZERO caret pixels on Wagtail —
/// which is an absence assertion satisfied by not being able to see its own
/// subject, and it was the presence companion below that caught it.
fn document_caret_ink(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) -> usize {
    let with = pixeldiff::render_frame(p, device, queue, W, H);
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    p.caret_invert.prepare(device, queue, W, H, &[]);
    let without = pixeldiff::render_frame(p, device, queue, W, H);
    differing(&with, &without)
}

/// The CARD'S QUERY caret's own drawn pixels, the same way, on its own pipeline.
fn query_caret_ink(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) -> usize {
    let with = pixeldiff::render_frame(p, device, queue, W, H);
    p.panel_caret.prepare_empty();
    let without = pixeldiff::render_frame(p, device, queue, W, H);
    differing(&with, &without)
}

/// **THE LAW.** Per world × per family member: no document-caret pixel anywhere,
/// and a query caret exactly where the card has something to search.
///
/// The presence companion runs in the SAME world, on the same pipeline, before
/// each cell: an ordinary document view must draw real caret pixels. An absence
/// floor is otherwise satisfiable by deleting its own subject — a renderer that
/// stopped drawing carets, a canvas the caret fell off, a world whose caret
/// happens to be invisible — and all three would read as this law passing.
#[test]
fn a_read_only_prose_surface_draws_no_document_caret_in_any_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping a_read_only_prose_surface_draws_no_document_caret: no adapter");
        return;
    };
    let enrolled = family();
    assert!(
        !enrolled.is_empty(),
        "the read-only prose family enrolled NOTHING — this law would sweep an empty set"
    );

    let body = transcript();
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        let w = world.name;

        // PRESENCE, this world, this pipeline: an ordinary document really does
        // draw a caret.
        p.set_view(&view("ordinary prose the writer is editing\n", 0, 0));
        p.settle_caret();
        p.prepare(&device, &queue, W, H).unwrap();
        let present = document_caret_ink(&mut p, &device, &queue);
        assert!(
            present > 0,
            "{w}: an ordinary document must draw caret pixels, or the absence \
             assertions below prove nothing (family enrolled: {enrolled:?})"
        );

        for kind in &enrolled {
            p.set_view(&read_only_view(*kind, &body));
            p.settle_caret();
            p.prepare(&device, &queue, W, H).unwrap();
            assert!(
                p.document_is_a_transcript(),
                "{w} {kind:?}: the fixture must actually relocate the document, or this \
                 cell grades an ordinary frame"
            );
            let ink = document_caret_ink(&mut p, &device, &queue);
            assert_eq!(
                ink, 0,
                "{w} {kind:?}: the reading surface drew {ink} caret pixels — the one \
                 accent means \"you can write here\", and every insertion door here is \
                 walled (ordinary document drew {present} for comparison)"
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        crate::theme::THEMES.len() * enrolled.len(),
        "every world × every enrolled family member must be graded"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// **THE CARD'S OWN QUERY CARET: PRESENT WHERE THERE IS SOMETHING TO SEARCH,
/// ABSENT WHERE THERE IS NOT** — both directions, so neither is vacuous.
///
/// Credits' head line still DRAWS: its title is what that line is for on a card
/// with one fixed row. What parks is the caret that made it read as a field the
/// query door then refused to fill.
#[test]
fn the_query_caret_is_drawn_exactly_where_the_card_can_be_searched() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping the_query_caret_is_drawn_exactly_where_searchable: no adapter");
        return;
    };
    let enrolled = family();
    let searchable: Vec<OverlayKind> = enrolled
        .iter()
        .copied()
        .filter(|k| k.offers_query())
        .collect();
    let unsearchable: Vec<OverlayKind> = enrolled
        .iter()
        .copied()
        .filter(|k| !k.offers_query())
        .collect();
    assert!(
        !searchable.is_empty() && !unsearchable.is_empty(),
        "this law needs one of each: searchable {searchable:?}, unsearchable {unsearchable:?}"
    );

    let body = transcript();
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        let w = world.name;
        for kind in &enrolled {
            p.set_view(&read_only_view(*kind, &body));
            p.settle_caret();
            p.prepare(&device, &queue, W, H).unwrap();
            let ink = query_caret_ink(&mut p, &device, &queue);
            if kind.offers_query() {
                assert!(
                    ink > 0,
                    "{w} {kind:?}: a card whose rows a query really filters must show \
                     where the typing goes — a working field with no caret is worse than \
                     the caret this item removed elsewhere"
                );
            } else {
                assert_eq!(
                    ink, 0,
                    "{w} {kind:?}: nothing to search, so nothing may advertise a field \
                     ({ink} caret pixels drawn; searchable siblings {searchable:?} carry \
                     the presence half of this law)"
                );
            }
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        crate::theme::THEMES.len() * enrolled.len(),
        "every world × every enrolled family member must be graded"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

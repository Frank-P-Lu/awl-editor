//! Export's markdown gate, and the DESTINATION NAVIGATOR the same dispatch
//! summons on a platform that picks one. On a non-Markdown buffer the row stays
//! enabled (see `apply_export_action`'s own doc) but must never silently no-op —
//! the File-menu row's own half of the same defect has its law in
//! `menu::ellipsis_law`.

use super::super::*;
use crate::overlay::OverlayKind;

/// A level supplier that behaves like the real one: two folder rows at every
/// level, so a descend/ascend genuinely rebuilds the card.
fn folder_level(kind: OverlayKind, rel: Option<String>) -> Option<OverlayState> {
    Some(OverlayState::new_marked(
        kind,
        vec!["out".to_string(), "archive".to_string()],
        vec![false, false],
        vec![true, true],
        Vec::new(),
        Vec::new(),
        rel,
    ))
}

/// Drive `action` and hand back BOTH halves of what a transition can produce —
/// the effect, and the journey it left behind. The card is the observable for
/// every export on a destination-picking platform, so a helper returning only
/// the effect could not see this law's subject at all.
fn drive_with_levels(
    buffer: &mut Buffer,
    action: Action,
    levels: bool,
) -> (Transition, crate::overlay::Journey) {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |k: OverlayKind, r: Option<String>| -> Option<OverlayState> {
        levels.then(|| folder_level(k, r)).flatten()
    };
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let transition = apply_transition(&mut ctx, &action, false);
    (transition, journey)
}

/// Every export action paired with the format it must end up writing, enrolled
/// so a fourth format cannot ride in unswept. PDF is native-only, exactly as in
/// `app::files::export::tests`.
fn export_cases() -> Vec<(Action, crate::export::Format)> {
    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut cases = vec![
        (Action::ExportWord, crate::export::Format::Docx),
        (Action::ExportHtml, crate::export::Format::Html),
    ];
    #[cfg(not(target_arch = "wasm32"))]
    cases.push((Action::ExportPdf, crate::export::Format::Pdf));
    cases
}

/// MUTATION TARGET: revert `apply_export_action`'s non-markdown arms to
/// `Effect::None` and this goes red on every one of the covered formats,
/// naming which action stayed silent.
#[test]
fn export_on_a_non_markdown_buffer_raises_an_explicit_notice_never_a_silent_no_op() {
    // `Format::Pdf` (and `Action::ExportPdf`'s only real arm) is native-only —
    // on wasm the action is an unconditional, unrelated `Effect::None` this
    // law does not own, so the PDF case is native-only too.
    for (action, _format) in export_cases() {
        let mut plain = Buffer::from_str("plain text, no markdown syntax");
        plain.set_path("/notes/todo.txt".into());
        let (transition, journey) = drive_with_levels(&mut plain, action.clone(), true);
        assert_eq!(
            transition.primary(),
            Effect::Notice(NoticeEffect::Sticky(
                "can't export a non-Markdown file".to_string()
            )),
            "{action:?} on a .txt buffer must raise an explicit notice, never Effect::None"
        );
        assert!(
            journey.card().is_none(),
            "{action:?}: a refused export must not leave a destination card up"
        );
    }
}

/// THE DESTINATION NAVIGATOR OPENS FIRST, on every platform that picks a
/// destination — the property `menu::FILE_ITEMS`' restored ellipsis promises,
/// asserted at the dispatch seam rather than read off the label.
///
/// MUTATION TARGET: make `begin_export` emit `Effect::Export` directly again and
/// this fails by name on the missing card.
#[test]
fn a_markdown_export_summons_the_destination_navigator_before_it_writes_anything() {
    if !export_picks_destination(crate::commands::Platform::current()) {
        return; // the web build downloads; `export_picks_destination`'s own law covers it
    }
    for (action, format) in export_cases() {
        let mut markdown = Buffer::from_str("# heading");
        markdown.set_path("/notes/todo.md".into());
        let (transition, journey) = drive_with_levels(&mut markdown, action.clone(), true);
        assert_eq!(
            transition.primary(),
            Effect::None,
            "{action:?}: nothing is written until a destination is chosen"
        );
        let card = journey
            .card()
            .unwrap_or_else(|| panic!("{action:?}: no destination card was summoned"));
        assert_eq!(
            card.kind,
            OverlayKind::ExportDest,
            "{action:?}: the destination navigator is the surface the label promises"
        );
        assert_eq!(
            card.export_format,
            Some(format),
            "{action:?}: the card must carry the format that summoned it"
        );
    }
}

/// THE FORMAT SURVIVES THE NAVIGATION, and this is the axis the card-local
/// payload could silently lose: a level rebuild replaces the whole card
/// (`Journey::relevel`), so a format read off the accept is only correct if the
/// rebuild carries it. Descends into a folder and ascends back out before
/// accepting, and requires the emitted effect to name both the original format
/// and the folder the navigator stopped on.
///
/// MUTATION TARGET: delete the `carry_level_payload_from` call in
/// `Journey::relevel` and this fails by name with `Effect::None` — a
/// format-less card cannot emit an export. Without the descend it would pass
/// against that same defect.
#[test]
fn the_export_format_survives_a_descend_and_ascend_before_the_accept() {
    if !export_picks_destination(crate::commands::Platform::current()) {
        return;
    }
    for (action, format) in export_cases() {
        let mut markdown = Buffer::from_str("# heading");
        markdown.set_path("/notes/todo.md".into());
        let mut shift = false;
        let mut zoom = 1.0;
        let mut search = None;
        let mut journey = crate::overlay::Journey::default();
        let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
        let mut browse_to =
            |k: OverlayKind, r: Option<String>| -> Option<OverlayState> { folder_level(k, r) };
        let mut ctx = ActionCtx {
            buffer: &mut markdown,
            shift_selecting: &mut shift,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: None,
        };
        let _ = apply_transition(&mut ctx, &action, false);
        // → descends into the highlighted folder, ← comes back up: two level
        // rebuilds, either of which would drop a payload nobody carried.
        let _ = apply_transition(&mut ctx, &Action::ForwardChar, false);
        assert_eq!(
            ctx.journey.card().and_then(|c| c.browse_dir.clone()),
            Some("out".to_string()),
            "{action:?}: the descend must actually change level, or this law sweeps nothing"
        );
        let _ = apply_transition(&mut ctx, &Action::BackwardChar, false);
        let effect = apply_transition(&mut ctx, &Action::Newline, false).primary();
        assert_eq!(
            effect,
            Effect::Export(format, Some("out".to_string())),
            "{action:?}: the accept must carry the summoning format and the chosen folder"
        );
        assert!(
            ctx.journey.card().is_none(),
            "{action:?}: the navigator closes on accept"
        );
    }
}

/// THE PLATFORM SPLIT, declared rather than discovered. `export_picks_destination`
/// is the one predicate the dispatch and the ellipsis decision both read, and
/// asking it with an EXPLICIT platform is what makes the arm this host does not
/// compile assertable at all.
///
/// MUTATION TARGET: make it answer `true` on `Web` and this fails by name — the
/// web build has no folder to offer, because the browser owns where a download
/// lands.
#[test]
fn only_a_native_build_asks_where_an_export_goes() {
    assert!(
        export_picks_destination(crate::commands::Platform::Native),
        "a native export writes a real file, so the writer chooses the folder",
    );
    assert!(
        !export_picks_destination(crate::commands::Platform::Web),
        "the browser owns where a download lands; awl has nothing to offer there",
    );
}

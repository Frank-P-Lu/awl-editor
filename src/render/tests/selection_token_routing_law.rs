//! THE TWO SELECTION TOKENS STAY ON THEIR OWN SURFACES — both halves of a
//! theme switch, across the whole roster.
//!
//! `selection_document` is the authored wash that covers TEXT (document
//! selection, search matches, and the menu-title band that runs the same
//! `highlight_treatment`). `selection_ui` is the band under a SELECTED ROW in a
//! summoned surface, DERIVED by default as a value step off the surface ramp.
//! One token used to serve both roles, so nothing could tell a mis-route from a
//! design decision; these assertions are what makes the difference checkable.
//!
//! ⚠️ **WHY THIS IS A LAW AND NOT A CAPTURE.** A headless capture builds its
//! pipelines ONCE and never calls `sync_theme_colors` — the O(1) colour half of
//! a LIVE theme switch. So a token mis-routed in the sync half alone repaints
//! nothing any capture can see, and a full byte-identity sweep across the
//! whole roster stays green through it; the repaint only reaches a user who
//! switches worlds while the app is running. That was measured, not assumed:
//! swapping `sync_theme_colors`'s `selection_pipeline` seed from
//! `selection_document` to `selection_ui` moved ZERO of 120 captured files.
//! This law reads the pipeline's own colour after a sync instead, so the sync
//! half has an oracle at all.

use super::super::*;
use super::headless_pipeline;

/// Both halves of a theme switch must land the same token on the same pipeline,
/// for EVERY world in the roster (no wildcard): `selection_pipeline` carries
/// `selection_document`, `overlay_rows` carries `selection_ui`. Sweeping the
/// whole roster matters because the two tokens COINCIDE on some worlds and not
/// others — a law pinned to one hand-picked world could pass on a coincidence.
#[test]
fn sync_theme_keeps_document_and_ui_selection_on_their_own_tokens() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping sync_theme_keeps_document_and_ui_selection_on_their_own_tokens: \
             no wgpu adapter"
        );
        return;
    };

    // Non-vacuity: unless the two tokens actually DIFFER on some world, every
    // assertion below would hold even with the routing crossed over.
    let mut ever_differed = false;

    for (i, t) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        p.sync_theme();

        let doc = crate::selection::srgba_u8_to_linear(theme::selection_document().rgba_bytes());
        let ui = crate::selection::srgba_u8_to_linear(theme::selection_ui().rgba_bytes());
        ever_differed |= doc != ui;

        assert_eq!(
            p.selection_pipeline.test_color(),
            doc,
            "{}: the DOCUMENT selection band must carry selection_document \
             after a live theme switch, not the UI row token",
            t.name
        );
        assert_eq!(
            p.overlay_rows.test_color(),
            ui,
            "{}: the picker's SELECTED-ROW band must carry selection_ui after a \
             live theme switch, not the document wash",
            t.name
        );
    }

    assert!(
        ever_differed,
        "no world distinguishes the two tokens — this law cannot fail, so it is \
         asserting nothing"
    );

    theme::set_active(theme::DEFAULT_THEME);
}

//! THE PASSIVE-SURFACE ROSTER LAW: drawn in the PNG ⇔ present in `semantic`.
//!
//! Item 207 gave the summoned cards one content owner but left the semantic
//! fold unable to DERIVE a card — the render pipeline held three of its figures
//! — so a `--screenshot-app` capture, which owns no pipeline, wrote a sidecar
//! whose `semantic` had no node for a card its own PNG plainly drew. The gap was
//! silent and partial: which-key and the menu bar have no such dependency and
//! did appear.
//!
//! This sweep is the axis that gap lived on. For every passive surface, in one
//! wildcard-free roster, it drives a real `--screenshot-app`-shaped capture —
//! the App's own `capture_opts` (PNG + `semantic` from one call, exactly the
//! production door) — and asserts BOTH directions with pixel arithmetic on one
//! side and node presence on the other. A sixth card added to
//! [`crate::card::content::CardKind`] and forgotten here fails by name.

use super::*;
use crate::capture::{CaptureOpts, capture_with};
use crate::card::content::CardKind;
use crate::testscratch::ScratchDir;

enum_with_all! {
    /// Every surface `fold_passive` can announce. The cards are the roster
    /// [`CardKind`] owns; which-key and the awl-rendered menu bar are the two
    /// that were never pipeline-bound and are swept beside them so the law
    /// covers the whole family rather than the half that was broken.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PassiveSurface {
        About,
        Lifetime,
        Streaks,
        Hud,
        Peek,
        WhichKey,
        MenuBar,
    }
}

impl PassiveSurface {
    /// The card this surface is, or `None` for the two that are not cards. No
    /// wildcard: a new `PassiveSurface` must answer, and the roster-parity law
    /// below makes a new `CardKind` need a `PassiveSurface`.
    fn card_kind(self) -> Option<CardKind> {
        match self {
            PassiveSurface::About => Some(CardKind::About),
            PassiveSurface::Lifetime => Some(CardKind::Lifetime),
            PassiveSurface::Streaks => Some(CardKind::Streaks),
            PassiveSurface::Hud => Some(CardKind::Hud),
            PassiveSurface::Peek => Some(CardKind::Peek),
            PassiveSurface::WhichKey | PassiveSurface::MenuBar => None,
        }
    }

    /// The semantic node id this surface must announce when it is up, and must
    /// not when it is down.
    fn node_id(self) -> &'static str {
        match self.card_kind() {
            Some(kind) => kind.id(),
            None => match self {
                PassiveSurface::WhichKey => WHICHKEY_ID,
                PassiveSurface::MenuBar => MENUBAR_ID,
                _ => unreachable!("every card answered above"),
            },
        }
    }

    /// Summon or dismiss this surface through the same process-global the live
    /// product flips. Every arm is spelled out; there is no wildcard.
    fn set_summoned(self, on: bool) {
        match self {
            PassiveSurface::About => crate::about::set_open(on),
            PassiveSurface::Lifetime => crate::lifetime::set_open(on),
            PassiveSurface::Streaks => crate::streaks::set_open(on),
            PassiveSurface::Hud => crate::hud::set_held(on),
            PassiveSurface::Peek => crate::peek::set_open(on),
            PassiveSurface::WhichKey => crate::whichkey::set_force_shown(on),
            PassiveSurface::MenuBar => crate::menubar::set_menu_bar_on(on),
        }
    }
}

fn calm_every_passive_surface() {
    for surface in PassiveSurface::ALL {
        surface.set_summoned(false);
    }
}

/// awl ships no `C-x` defaults, so the which-key panel only ever has rows a
/// user RECLAIMED through `[keys]`. The sweep reclaims three, in both keymap
/// conventions, so the panel it summons genuinely lists something — an empty
/// panel would make the which-key arm of the ⇔ a claim about a bare frame.
fn config_with_reclaimed_cx() -> Config {
    Config {
        keys: vec![
            ("save".to_string(), vec!["C-x C-s".to_string()]),
            ("switch_theme".to_string(), vec!["C-x t".to_string()]),
            ("new_document".to_string(), vec!["C-x n".to_string()]),
        ],
        ..Config::empty()
    }
}

/// One `--screenshot-app`-shaped capture: the App builds its own `CaptureOpts`
/// (which is where both the `semantic` tree and the which-key rows come from),
/// the ordinary single-frame path renders and writes the PNG, and the caller
/// gets the pixels beside the tree that was written next to them.
fn capture_live_app(app: &App, png: &std::path::Path) -> (Vec<[u8; 4]>, CaptureOpts) {
    let opts = app.capture_opts();
    capture_with(png, app.document.buffer(), &opts).expect("live-app capture");
    let img = image::open(png).expect("read back the PNG").to_rgba8();
    let pixels = img.pixels().map(|p| p.0).collect();
    (pixels, opts)
}

fn differing_pixels(a: &[[u8; 4]], b: &[[u8; 4]]) -> usize {
    assert_eq!(a.len(), b.len(), "two captures of the same canvas");
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

fn announced(opts: &CaptureOpts, id: &str) -> bool {
    opts.semantic
        .as_ref()
        .expect("a live-app capture always carries a semantic tree")
        .nodes
        .iter()
        .any(|node| node.id == id)
}

/// A surface's card must draw ENOUGH ink to be unmistakable; a handful of
/// pixels would let an antialiasing wobble pass for a drawn panel.
const DRAWN_FLOOR: usize = 500;

/// **The item-215 law.** For every passive surface: summoned ⇒ the PNG changes
/// and the tree carries its node; dismissed ⇒ the PNG is byte-identical to the
/// calm room and the node is gone. Both directions, one roster, no wildcard.
#[test]
fn every_passive_surface_drawn_in_the_png_is_present_in_the_semantic_tree() {
    if crate::test_gpu::shared_device_queue().is_none() {
        eprintln!(
            "skipping every_passive_surface_drawn_in_the_png_is_present_in_the_semantic_tree: \
             no wgpu adapter"
        );
        return;
    }
    let _guard = crate::testlock::serial();
    calm_every_passive_surface();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_passive_roster_{}", std::process::id())),
    );
    // A hermetic App reads an in-memory filesystem, so the document is typed in
    // rather than written to disk — the buffer is what both sides read anyway.
    let mut app = App::new_hermetic(None, dir.to_path_buf(), config_with_reclaimed_cx());
    app.set_semantic_text_for_test("# Title\n\nsome prose with several words in it here\n");
    let app = app;
    let (calm_pixels, calm_opts) = capture_live_app(&app, &dir.join("calm.png"));
    for surface in PassiveSurface::ALL {
        assert!(
            !announced(&calm_opts, surface.node_id()),
            "{surface:?}: the calm room announced {} with nothing drawn",
            surface.node_id(),
        );
    }

    for surface in PassiveSurface::ALL {
        surface.set_summoned(true);
        let png = dir.join(format!("{surface:?}.png"));
        let (pixels, opts) = capture_live_app(&app, &png);
        let drawn = differing_pixels(&calm_pixels, &pixels);
        surface.set_summoned(false);

        assert!(
            drawn >= DRAWN_FLOOR,
            "{surface:?}: summoning it changed only {drawn} pixels — the sweep \
             cannot claim the PNG draws it, so the ⇔ below would be vacuous",
        );
        assert!(
            announced(&opts, surface.node_id()),
            "{surface:?}: its PNG changed {drawn} pixels but the sidecar's \
             `semantic` carries no `{}` node — a surface that is DRAWN and not \
             ANNOUNCED is exactly the gap this law exists for",
            surface.node_id(),
        );

        // …and back down: the surface leaves both the frame and the tree.
        let (after, after_opts) = capture_live_app(&app, &dir.join(format!("{surface:?}-off.png")));
        assert_eq!(
            differing_pixels(&calm_pixels, &after),
            0,
            "{surface:?}: dismissing it did not restore the calm frame",
        );
        assert!(
            !announced(&after_opts, surface.node_id()),
            "{surface:?}: dismissed, yet still announced — a tree that names an \
             undrawn surface lies in the other direction",
        );
    }
    calm_every_passive_surface();
}

/// The roster above must COVER the card roster. A sixth `CardKind` that nobody
/// adds here would otherwise be swept by nothing at all, which is the silent
/// failure the whole item is about.
#[test]
fn the_passive_roster_covers_every_card_kind_by_name() {
    let covered: Vec<CardKind> = PassiveSurface::ALL
        .iter()
        .filter_map(|surface| surface.card_kind())
        .collect();
    for kind in CardKind::ALL {
        assert!(
            covered.contains(&kind),
            "{kind:?} has no `PassiveSurface` arm, so the drawn-⇔-announced sweep \
             would never reach it",
        );
    }
    assert_eq!(
        covered.len(),
        CardKind::VARIANT_COUNT,
        "the roster maps a card twice, or maps one that is not a card",
    );
    // The two non-card surfaces are named rather than counted, so retiring one
    // is a deliberate edit here instead of a quietly smaller sweep.
    assert_eq!(
        PassiveSurface::VARIANT_COUNT,
        CardKind::VARIANT_COUNT + 2,
        "the passive family is the cards plus which-key plus the menu bar",
    );
}

/// The which-key panel is announced from the App's scheduling state but DRAWN
/// by the harness's offscreen pipeline from `CaptureOpts`. One gate feeds both,
/// so a live-`App` capture cannot announce a panel it does not draw.
#[test]
fn the_whichkey_capture_rows_and_the_announced_panel_share_one_gate() {
    let _guard = crate::testlock::serial();
    calm_every_passive_surface();
    let app = App::new_hermetic(None, PathBuf::from("/"), config_with_reclaimed_cx());
    assert!(app.whichkey_panel_rows().is_none(), "calm: no panel");
    assert!(
        app.capture_opts().whichkey.is_none(),
        "calm: the capture is told to draw no panel either",
    );

    crate::whichkey::set_force_shown(true);
    let rows = app.whichkey_panel_rows().expect("summoned: rows");
    assert!(
        !rows.is_empty(),
        "a summoned panel with no rows draws nothing"
    );
    assert_eq!(
        app.capture_opts().whichkey.as_deref(),
        Some(rows.as_slice()),
        "the capture draws exactly the rows the gate names",
    );
    let announced_rows: Vec<String> = app
        .semantic_snapshot()
        .nodes
        .iter()
        .filter(|node| node.id.starts_with(&format!("{WHICHKEY_ID}.row.")))
        .map(|node| node.name.clone())
        .collect();
    assert_eq!(
        announced_rows,
        rows.iter()
            .map(|(key, name)| format!("{key} {name}"))
            .collect::<Vec<_>>(),
        "the announced rows are the rows the capture is told to draw",
    );
    crate::whichkey::set_force_shown(false);
}

/// The renderer and the semantic fold must agree, figure for figure, about what
/// the card SAYS — not merely that there is one. The oracle is derived here
/// from the document by hand (frontmatter excluded, whitespace tokens, the
/// caret's character offset over the document's character length), never by
/// calling the owner under test.
#[test]
fn the_announced_card_carries_the_documents_own_figures() {
    let _guard = crate::testlock::serial();
    calm_every_passive_surface();
    let body = "alpha beta gamma delta\nepsilon zeta\n";
    let text = format!("---\nlang: zh-Hans\n---\n{body}");

    let mut app = App::new_hermetic(None, PathBuf::from("/"), Config::empty());
    // Parks the caret at the very end, which is 100% through the document.
    app.set_semantic_text_for_test(&text);
    crate::hud::set_held(true);
    let snapshot = app.semantic_snapshot();
    crate::hud::set_held(false);

    let card = snapshot
        .nodes
        .iter()
        .find(|node| node.id == CardKind::Hud.id())
        .expect("the held HUD announces a card");
    let lines: Vec<&str> = card.value.as_deref().unwrap_or("").split(", ").collect();
    let figure_after = |caption: &str| -> String {
        let at = lines
            .iter()
            .position(|line| *line == caption)
            .unwrap_or_else(|| panic!("no {caption} row in {lines:?}"));
        lines[at + 1].to_string()
    };

    // Independent oracle: six whitespace tokens in the manuscript, none in the
    // frontmatter block.
    let expected_words = body.split_whitespace().count();
    assert_eq!(expected_words, 6);
    assert_eq!(
        figure_after("WORD COUNT"),
        format!("{expected_words} words · 1 min")
    );
    // The frontmatter tag, read straight off the fixture's own text.
    assert_eq!(figure_after("LANGUAGE"), "zh-Hans");
    // The caret is at the last character, so the readout is 100%.
    assert_eq!(figure_after("THROUGH DOC"), "100%");
    assert_eq!(figure_after("LINE ENDINGS"), "LF");
    // And the live-only figures read as their placeholders, because no live App
    // has pushed any — which is exactly what the drawn card shows.
    assert_eq!(figure_after("SAVED"), "—");
}

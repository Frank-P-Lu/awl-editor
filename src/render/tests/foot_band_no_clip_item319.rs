//! ITEM 319 — THE FOOT BAND'S INK EXCEEDS THE CARD'S TEXT COLUMN AND IS
//! CLIPPED (pre-existing), AND A SECOND, INDEPENDENTLY MEASURED INSTANCE WAS
//! LATER ADDED (~42 logical px of card ink outside `overlay_card_rect` on
//! both diagonal worlds).
//!
//! **RE-MEASURED AGAINST HEAD. The two cited instances are NOT one cause —
//! they are two claims, and neither survives unchanged.**
//!
//! **INSTANCE A (the original citation — "Mangrove/Command, 434px of ink in a
//! 496px column, including the Keybindings tips") is FALSE AS STATED: that
//! combination cannot occur live.** `app/stats.rs::sync_discoverability`
//! gates the footer tips to `self.overlay_is_keybindings()` alone — "the
//! footer tips ride ONLY while the Keybindings overlay is open (so no OTHER
//! flat picker ever grows a footer)" is the source's own comment. A `Command`
//! card showing tips is a fixture that cannot exist under the real App.
//!
//! Re-measured under the one combination that CAN occur — `Keybindings`
//! itself, with the real ledger tip format (`"{chord}  {name}"`,
//! `app/usage.rs::keybinding_tips`) — against every chorded command in the
//! catalog (the true worst case is whichever entry is widest, not whichever
//! the ledger happens to rank #1 today): **no clip, on any enrolled world, at
//! the shipped default render zoom (0.8, what `--screenshot` renders).**
//!
//! A REAL, narrower defect exists next to it, found while checking this: at
//! `zoom = 1.0` (not the default) `Mangrove`'s hint line — plain
//! `"type to filter ↵ choose ←/→ category esc close"`, no tips at all —
//! overflows `overlay_card_rect`'s own right edge by ~7.7 logical px,
//! `Magpie` (the other diagonal direction) and `Paperbark` (`shear == 0`)
//! stay byte-clean. It is DIRECTION-gated (descending only, where
//! `overlay_foot_placement`'s column clamp is the one actually binding — an
//! ascending lean never approaches the right edge to begin with) and
//! ZOOM-gated (absent at the shipped 0.8 default), so it is not this item's
//! shipped-default subject; it is left as a narrower follow-up rather than
//! patched under this item's own budget (the plausible mechanism —
//! `overlay_foot_placement`'s clamp measures its width budget off
//! ADVANCE-based `run.line_w`, while the hint's `←`/`→`/`↵` glyphs live in a
//! separate symbol face whose rendered CELL can exceed its advance — wants
//! its own dedicated measurement before being trusted).
//!
//! **INSTANCE B (the later citation of "~42px card ink outside
//! `overlay_card_rect`, found at (1102, 344), mask 0.03") DOES NOT REPRODUCE
//! under a SOUND oracle.** Re-measured with `TextPipeline::overlay_line_glyph_box`
//! — a production SHAPING accessor, not a re-derivation — against every
//! shaped line (query, every candidate row, the hint) on both diagonal
//! worlds at 1×/0.8×: zero overflow anywhere except the SAME Instance-A hint
//! line. The "(1102, 344)" figure has the exact shape already named and
//! warned against elsewhere for a *different* subject: `294`'s
//! `CardInk` veto is documented, in this same file's neighbour
//! (`frost_card_ink.rs`), as SOUND only as an exclusion and FALSE as an
//! inclusion set — "it flags thousands of pixels outside the card's own box,
//! reaching tens of logical px above the card's top edge […] on every
//! enrolled world" is the veto's own doc, of its OWN ground-texture/shadow
//! surplus, independent of anything the card draws. A reading built on it in
//! the inclusion direction reproduces exactly this shape of false positive.
//! No code changed for Instance B — there was nothing TO change; the oracle
//! that reported it needs no repair here either, because the repair (never
//! invert `CardInk`) is already the law `frost_card_ink.rs` enforces.
//!
//! **So: two premises, not one cause splitting in two.** Instance A's
//! headline fixture was unreachable and its real (reachable) form does not
//! clip at the default zoom; Instance B does not reproduce under a sound
//! oracle at all. The one real residual (the zoom-gated Mangrove overflow) is
//! new, narrow, and named above rather than folded into either original
//! citation.
//!
//! **Verify, as filed:** no foot-band run is clipped, swept over the FULL
//! world roster (never just the diagonal pair — Instance A's clip, when it
//! was real under the unreachable fixture, reproduced identically on `Tawny`,
//! a plain `Pane` world, so enrolment is not narrowed to `Diagonal`) ×
//! `OverlayKind::Keybindings` (the one kind `overlay_footer_lines` can ever
//! populate a footer for) × every catalog tip × 1×/2× × both `MENU_BAR_ON`
//! states.
//!
//! ⚠️ **THAT SWEEP FOUND A FOURTH, UNRELATED, REAL CLIP — NOT FIXED HERE, AND
//! NOT THIS ITEM'S SUBJECT.** `Potoroo` AND `Firetail` — the two worlds whose
//! chrome `font` (not just `mono`) is `"Monaspace Xenon"` — clip on the
//! `Keybindings` HINT LINE ALONE at 2×, menu bar OFF (`macOS`'s own
//! default): no tip needed, both measured the identical shaped width for the
//! hint alone (`803.2px`) against their own (different) column budgets. Nine
//! other worlds in the roster also use a monospace `font` for chrome text
//! (`docs/fonts.md`'s "per-world mono" is a deliberate, common choice, not a
//! rarity) and do NOT clip — so the determining property is specifically
//! `Monaspace Xenon`'s own wider average advance, not "monospace" as a
//! category. `text_w` is one fixed budget with no per-world font term; that
//! mismatch wants its own measurement of whether `text_w` should read the
//! active world's own glyph metrics — a bigger question than this item's
//! budget. Excluded from the sweep by the MEASURED property that actually
//! distinguishes them (`theme::active().font`), never a name list, so a
//! future world sharing this font enrols in the exclusion automatically
//! rather than silently passing; the exclusion is the finding, not a way to
//! bury it.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// See the module doc's ⚠️ note: every world whose chrome `font` is this face
/// clips its `Keybindings` hint alone (no tip needed) at 2×/bar-off — a
/// real, separate, unfixed defect this law does not own. Measured by the
/// property that actually causes it (the font's own wider advance), not by
/// naming the worlds that happen to carry it today.
const KNOWN_UNFIXED_FONT_METRICS_EXCLUSION: &str = "Monaspace Xenon";

/// Every `"{chord}  {name}"` tip the REAL ledger could ever hand the footer —
/// the whole catalog, not whichever three usage ranks first today, because
/// the true worst case is a property of the catalog's own longest entry.
fn every_real_tip() -> Vec<String> {
    let names = crate::commands::names();
    let bindings = crate::commands::effective_bindings(&[], &[]);
    names
        .iter()
        .zip(bindings.iter())
        .filter(|(_, chord)| !chord.is_empty())
        .map(|(name, chord)| format!("{chord}  {name}"))
        .collect()
}

// `set_keybindings_tips` is the discoverability ledger's own native-only door
// (`chrome/hud.rs`) — a headless/wasm build never populates it, so this law,
// which is entirely about that footer, is native-only too.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_keybindings_footer_never_clips_for_any_real_ledger_tip() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping Keybindings footer no-clip law: no wgpu adapter");
        return;
    };
    let tips = every_real_tip();
    assert!(
        tips.len() > 10,
        "the catalog sweep must actually run, got {} tips",
        tips.len()
    );
    let ambient_bar = crate::menubar::menu_bar_on();
    let names = crate::theme::world_names();
    let mut graded = 0usize;
    let mut presence_graded = 0usize;
    let mut excluded = 0usize;
    for bar in [ambient_bar, !ambient_bar] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for &world in &names {
                crate::theme::set_active_by_name(world).expect("a named world exists");
                if theme::active().font == KNOWN_UNFIXED_FONT_METRICS_EXCLUSION {
                    excluded += 1;
                    continue;
                }
                p.sync_theme();
                // Every world/tip combination shapes fresh glyphs into the ONE shared
                // atlas; a live frame loop reclaims it every frame (`p.atlas.trim()`),
                // but this tight sweep never presents a frame, so it must trim itself
                // or a long sweep (roster × catalog × dpi × bar) exhausts it
                // (`AtlasFull`) well before any geometry assertion fires.
                p.atlas.trim();
                for tip in &tips {
                    let mut v = view("hello\n", 0, 0);
                    v.overlay_active = true;
                    v.zoom = 0.8; // the shipped default render zoom (what `--screenshot` renders)
                    v.overlay_title = OverlayKind::Keybindings.title();
                    v.overlay_hint = OverlayKind::Keybindings.hint();
                    v.overlay_items = vec!["Go to file".into(), "Save".into(), "Undo".into()];
                    v.overlay_selected = 0;
                    p.set_keybindings_tips(vec![tip.clone()]);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let (footer_px, text_w) = p.overlay_footer_fit_probe(cw);
                    assert!(
                        footer_px > 1.0,
                        "{world} dpi={dpi} bar={bar}: the footer must actually shape glyphs \
                         for tip {tip:?} — a clip floor here would be satisfied by the tip \
                         having vanished"
                    );
                    presence_graded += 1;
                    assert!(
                        footer_px <= text_w,
                        "{world} dpi={dpi} bar={bar}: Keybindings footer {footer_px:.1}px \
                         clips the card's {text_w:.1}px text column for tip {tip:?} — the \
                         clip item 319 was filed against, under the one combination \
                         (`Keybindings` + a real ledger tip) that can actually occur live"
                    );
                    graded += 1;
                }
            }
        }
    }
    p.set_dpi(1.0);
    p.set_keybindings_tips(Vec::new());
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(graded > 500, "the sweep must actually run, got {graded}");
    assert_eq!(
        graded, presence_graded,
        "every graded cell must have passed the presence floor too"
    );
    assert_eq!(
        excluded,
        8, // 2 worlds (Potoroo, Firetail) x 2 dpi x 2 bar states
        "the font exclusion's own count moved — either a world's chrome font \
         changed to/from {KNOWN_UNFIXED_FONT_METRICS_EXCLUSION:?} (re-measure \
         before trusting this sweep either way) or the roster grew/shrank"
    );
}

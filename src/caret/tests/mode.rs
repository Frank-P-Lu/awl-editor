//! Caret MODE tests -- font-mono detection, morph one-back anchoring, the
//! mode label round-trip, the demo choreography, and the default/override/
//! toggle mode-selection rules -- split out of the former monolithic
//! `caret::tests` (2026-07 code-organization pass).

use super::super::*;

#[test]
fn font_mono_detection() {
    // ALL the bundled mono faces (display faces AND the code companions in
    // theme/worlds.rs) are detected — Potoroo/Currawong/Mangrove regressed to Morph
    // defaults (and lost the block's mono cell floor) when this listed only
    // IBM Plex Mono, and Currawong/Cassowary lost the uniform caret grid again
    // while it listed only the three below.
    assert!(font_is_mono("IBM Plex Mono"));
    assert!(font_is_mono("JetBrains Mono"));
    assert!(font_is_mono("Monaspace Xenon"));
    assert!(font_is_mono("Iosevka"));
    // The proportional faces stay proportional — iA Writer Quattro S is a
    // quattro (near-mono spacing but NOT a fixed grid), not a mono.
    assert!(!font_is_mono("Literata"));
    assert!(!font_is_mono("Newsreader 16pt 16pt"));
    assert!(!font_is_mono("iA Writer Quattro S"));
    // The predicate is no longer a name list to keep in sync: it reads each
    // bundled face's own advance widths (`render::facepitch`), and the roster
    // sweep that makes a new face fail rather than be forgotten lives in
    // `render::tests::facepitch`. These spot-checks stay as the readable
    // statement of the rule.
}

/// Block is the UNIVERSAL default caret: with no explicit override,
/// `default_mode` answers Block for every world regardless of its display
/// face's measured pitch. A no-wildcard sweep of `theme::THEMES` — a new world,
/// mono or proportional, cannot quietly reintroduce the retired identity-
/// dependent `auto` this test used to encode (Block on mono, Morph on
/// proportional).
#[test]
fn default_mode_is_block_in_every_world_with_no_override() {
    let _t = crate::testlock::serial();
    let restore = crate::theme::active_index();
    // Non-vacuity: the roster must genuinely carry both pitches, or a
    // font-derived default could have agreed with Block everywhere by
    // accident and this sweep would prove nothing.
    assert!(
        crate::theme::THEMES.iter().any(|t| font_is_mono(t.font)),
        "the roster must carry a mono-faced world for this sweep to mean anything"
    );
    assert!(
        crate::theme::THEMES.iter().any(|t| !font_is_mono(t.font)),
        "the roster must carry a proportional-faced world for this sweep to mean anything"
    );
    for (i, t) in crate::theme::THEMES.iter().enumerate() {
        crate::theme::set_active(i);
        crate::caret::clear_override();
        assert_eq!(
            default_mode(),
            CaretMode::Block,
            "{} ({}, {}): the auto caret default must be Block, not follow the face's pitch",
            t.name,
            t.font,
            if font_is_mono(t.font) {
                "mono"
            } else {
                "proportional"
            }
        );
    }
    crate::theme::set_active(restore);
    crate::caret::clear_override();
}

#[test]
fn morph_anchor_col_is_one_back_but_never_across_its_own_row_start() {
    // The MORPH caret inhabits the char BEFORE the insertion point: typing
    // `abc|` (cursor col 3) anchors the `c` at col 2 — one back, within the row.
    assert_eq!(morph_anchor_col(3, 0), 2);
    assert_eq!(
        morph_anchor_col(1, 0),
        0,
        "cursor after the first char anchors it"
    );
    assert_eq!(morph_anchor_col(42, 0), 41);
    // FALLBACK: a ROW START has no previous glyph ON THIS ROW — the GEOMETRY
    // anchor stays at the cursor cell (whose left edge is the insertion x),
    // never underflowing and never reaching back across the row boundary. The
    // caret does NOT light that cell's glyph there — see `morph_row_start`.
    assert_eq!(morph_anchor_col(0, 0), 0);
    // THE WRAPPED ROW, which a logical-column rule gets wrong: column 58 is the
    // FIRST column of a soft-wrapped row, so its "previous" character sits on the
    // row ABOVE. Stepping back there drew the caret a whole visual row away from
    // its own insertion point — the entire reason this takes a `row_start`.
    assert_eq!(
        morph_anchor_col(58, 58),
        58,
        "a wrapped row's first column anchors itself, never the row above"
    );
    assert_eq!(
        morph_anchor_col(59, 58),
        58,
        "one column into a wrapped row anchors that row's own first glyph"
    );
}

/// The MORPH DEGRADE decision: exactly at a VISUAL ROW START — column 0, a fresh
/// line after Enter, an empty line, AND the first column of a soft-wrapped row —
/// there is no produced glyph before the insertion point ON THAT ROW, so the morph
/// melts to the thin insertion bar (no silhouette) instead of lighting a character
/// it does not sit beside (`|abc` must NOT glow the `a`; a wrapped row's first
/// column must not reach back to the row above). Any column past its row's start has
/// a previous glyph cell and keeps the silhouette machinery.
#[test]
fn morph_degrade_fires_at_every_visual_row_start_not_only_column_zero() {
    assert!(
        morph_row_start(0, 0),
        "col 0 (incl. empty lines) melts to the bar"
    );
    assert!(
        !morph_row_start(1, 0),
        "aI bc: the just-passed 'a' stays lit"
    );
    assert!(!morph_row_start(2, 0));
    assert!(!morph_row_start(42, 0));
    // THE WRAPPED ROW START — false under the retired `col == 0` rule, which is
    // how a Morph caret came to sit at the END OF THE ROW ABOVE its insertion point.
    assert!(
        morph_row_start(58, 58),
        "a soft-wrapped row's first column is a row start too"
    );
    assert!(
        !morph_row_start(59, 58),
        "one column in, the row's own first glyph is behind the caret"
    );
    // The decision agrees with the anchor math on EVERY row start, not just col 0:
    // the only columns whose anchor is not strictly one back are the ones that
    // degrade — the two seams can't drift apart.
    for row_start in [0usize, 1, 17, 58] {
        for col in row_start..row_start + 64 {
            assert_eq!(
                morph_row_start(col, row_start),
                morph_anchor_col(col, row_start) == col,
                "degrade ⇔ the anchor held at the cursor cell (col {col}, row_start {row_start})"
            );
        }
    }
}

#[test]
fn caret_mode_label_description_and_from_label_round_trip() {
    // ALL lists the three looks in picker order; each has a label + description.
    assert_eq!(
        CaretMode::ALL,
        [CaretMode::Block, CaretMode::Morph, CaretMode::Ibeam]
    );
    for m in CaretMode::ALL {
        assert!(!m.label().is_empty());
        assert!(!m.description().is_empty());
        // from_label is the inverse of label (and case-insensitive).
        assert_eq!(CaretMode::from_label(m.label()), Some(m));
        assert_eq!(CaretMode::from_label(&m.label().to_uppercase()), Some(m));
    }
    assert_eq!(CaretMode::from_label("I-beam"), Some(CaretMode::Ibeam));
    assert_eq!(CaretMode::from_label("nope"), None);
}

#[test]
fn caret_demo_choreography_types_edits_then_loops_and_settles() {
    let mut d = CaretDemo::new();
    // UN-SEEDED: stepping does nothing (no metrics yet) and reports not-animating —
    // the loop only lives once the renderer seeds it while the picker is open.
    assert!(!d.step(0.016));
    assert!(d.text().is_empty());
    // Seed metrics: the FIRST seed returns true and primes beat 0 (the first
    // character), so typing begins at once.
    assert!(d.set_metrics(9.0, 20.0));
    assert!(
        !d.set_metrics(9.0, 20.0),
        "only the first seed reports 'jump'"
    );
    assert_eq!(d.text(), "w", "beat 0 typed the first character");
    assert_eq!(d.cursor_char(), 1);
    assert_eq!(d.beat_index(), 0, "the timeline starts on beat 0");
    // Drive the timeline: it should type the WHOLE sample line out (each beat a real
    // apply_transition InsertChar), reaching the full line char-by-char.
    let mut typed_full = false;
    for _ in 0..4000 {
        d.step(0.016);
        if d.text() == SAMPLE {
            typed_full = true;
            break;
        }
    }
    assert!(typed_full, "the choreography types the full sample line");
    assert_eq!(d.cursor_char(), SAMPLE.chars().count());
    // Keep stepping through the edit phase: the line must SHRINK (backspaces + the
    // kill-line) below the full length — the delete-squash / gulp beats really edit.
    let mut shrank = false;
    for _ in 0..6000 {
        d.step(0.016);
        if d.text().chars().count() < SAMPLE.chars().count() {
            shrank = true;
            break;
        }
    }
    assert!(shrank, "the delete/kill beats really remove text");
    // And it eventually CLEARS + LOOPS back to re-typing from an empty line.
    let mut looped = false;
    for _ in 0..8000 {
        d.step(0.016);
        if d.text().is_empty() || d.text() == "w" {
            looped = true;
            break;
        }
    }
    assert!(looped, "the timeline clears and loops back to typing");
    // RESET (picker closed): un-seeds, so the next step idles (no animation, empty
    // buffer) until re-seeded — the preview stops the instant the picker closes.
    d.reset();
    assert!(!d.step(0.016));
    assert!(d.text().is_empty());
    // SETTLE pins the deterministic headless frame: the FULLY-TYPED line at rest.
    d.set_metrics(9.0, 20.0);
    d.anim.set_target(500.0, 50.0); // start a glide
    d.settle();
    assert_eq!(d.text(), SAMPLE, "settle shows the full sample line");
    assert!(
        !d.anim.is_animating(),
        "settle pins the preview caret at rest"
    );
}

#[test]
fn mode_is_block_on_both_mono_and_proportional_worlds_with_no_override() {
    // Mutates the shared theme global (`set_active_by_name`), not just caret's
    // own — hold BOTH test locks (theme, THEN caret, the suite-wide order) so
    // this can't race another test's theme read/write. `super::TEST_LOCK` alone
    // (caret's) does not exclude `theme::TEST_LOCK`-holding tests.
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    // Clear any explicit override so the universal default applies.
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
    // Tawny (IBM Plex Mono) -> Block.
    crate::theme::set_active_by_name("Tawny").unwrap();
    assert_eq!(mode(), CaretMode::Block);
    // Gumtree (Literata, proportional) -> ALSO Block: the default no longer
    // varies with the world's measured face pitch.
    crate::theme::set_active_by_name("Gumtree").unwrap();
    assert_eq!(mode(), CaretMode::Block);
    // Restore.
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
}

#[test]
fn explicit_override_beats_the_block_default_across_a_world_switch() {
    // Hold theme's lock too — this mutates the shared theme global (see the
    // note on `mode_is_block_on_both_mono_and_proportional_worlds_with_no_override`).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    // An explicit Morph override survives a switch between a mono world and a
    // proportional one — both now share the same Block default, so this proves
    // the override outranks the default rather than merely disagreeing with it
    // on one side.
    crate::theme::set_active_by_name("Tawny").unwrap();
    set_mode(CaretMode::Morph);
    assert_eq!(mode(), CaretMode::Morph);
    crate::theme::set_active_by_name("Gumtree").unwrap();
    assert_eq!(
        mode(),
        CaretMode::Morph,
        "an explicit pick survives a world switch, not just disagreement with one font"
    );
    // And an explicit Ibeam pick wins too, then toggle flips it to Block.
    set_mode(CaretMode::Ibeam);
    assert_eq!(mode(), CaretMode::Ibeam);
    assert_eq!(toggle_mode(), CaretMode::Block);
    assert_eq!(mode(), CaretMode::Block);
    // Restore.
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
}

#[test]
fn toggle_mode_flips_block_and_ibeam() {
    // Hold theme's lock too — this mutates the shared theme global (see the
    // note on `mode_is_block_on_both_mono_and_proportional_worlds_with_no_override`).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    // Start from the Block default (no override) — any world does, since the
    // default no longer tracks the world's font; Tawny is picked arbitrarily.
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
    crate::theme::set_active_by_name("Tawny").unwrap();
    assert_eq!(mode(), CaretMode::Block);
    // C-x c: Block -> Ibeam (the live I-beam is reachable without a flag).
    assert_eq!(toggle_mode(), CaretMode::Ibeam);
    assert_eq!(mode(), CaretMode::Ibeam);
    // C-x c again: Ibeam -> Block.
    assert_eq!(toggle_mode(), CaretMode::Block);
    assert_eq!(mode(), CaretMode::Block);
    // Morph is NOT on the toggle: from Morph the chord enters the pair at Block.
    set_mode(CaretMode::Morph);
    assert_eq!(toggle_mode(), CaretMode::Block);
    assert_eq!(mode(), CaretMode::Block);
    // Restore.
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
}

/// `is_auto`/`clear_override`: the pure primitive pair the Caret-style
/// picker's auto-aware Cancel is built on (see `overlay::state::new_caret`'s
/// `original_caret_was_auto` + `actions::overlay_nav`'s Cancel arm). Auto is
/// the construction default; any explicit `set_mode` clears it; `clear_override`
/// is the one door back, restoring the universal Block resolution — which no
/// longer depends on the active theme.
#[test]
fn is_auto_and_clear_override_round_trip() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
    assert!(is_auto(), "no override set: auto");

    // An explicit pick — of ANY mode, including Block, which is what auto
    // resolves to anyway — clears auto.
    crate::theme::set_active_by_name("Gumtree").unwrap(); // proportional world
    set_mode(CaretMode::Block);
    assert!(
        !is_auto(),
        "an explicit pick, even one auto would've chosen, is no longer auto"
    );
    assert_eq!(mode(), CaretMode::Block);

    // `clear_override` restores auto — and thus Block, on this proportional
    // world and on a mono one too: the resolved value no longer tracks theme
    // identity, so a world switch cannot move it.
    clear_override();
    assert!(is_auto());
    assert_eq!(
        mode(),
        CaretMode::Block,
        "auto resolves Block on a proportional world too"
    );
    crate::theme::set_active_by_name("Tawny").unwrap(); // mono world
    assert_eq!(
        mode(),
        CaretMode::Block,
        "auto stays Block across a world switch — the coupling to font is retired"
    );

    // Restore.
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
}

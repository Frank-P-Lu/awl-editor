//! THE START SCREEN'S TWO ACTIONS SHARE ONE INK; ONLY THE CHORD READS QUIET.
//!
//! DECIDED (queue item 525): `New document` and `Go to` no longer wear different
//! inks — the muted verb used to double as the universal disabled costume, so a
//! reader could not tell "second in order" from "not available". Hierarchy is
//! order alone now; both verbs read in `theme::base_content()`. Each row still
//! carries its real chord (`⌘N`/`⌘O`) beside the verb, in `theme::muted()` — the
//! quiet-chord/full-ink-verb split `shape_overlay_right` already established for
//! a row's secondary column, reused here rather than invented fresh.
//!
//! PRESENCE, not just equality: a law asserting only "both rows match" is
//! satisfiable by a regression that mutes BOTH rows (or inks both alike) —
//! this instead asserts each region carries the EXACT theme constant it is
//! meant to (chord == `muted`, verb == `base_content`), which cannot be
//! satisfied by silently deleting the split. One world, Wagtail, sets
//! `muted == base_content` BY DESIGN (`docs` — awl's first true 1-bit world;
//! its hierarchy rides a stipple texture, not a second ink), so it is the one
//! world the exact-value checks cannot also prove distinctness on; the roster
//! sweep separately asserts distinctness holds somewhere, so the law is not
//! vacuous in aggregate.
//!
//! Read at the byte-attrs seam (`BufferLine::attrs_list`), one step short of a
//! rendered pixel: `prepare_start_surface` bakes these colors into the shaped
//! buffer whether or not a frame ever samples them, so this is the purest seam
//! that still proves the DRAWN color, not a recomputation of what it should be.

use super::super::*;
use super::headless_dqp;

const W: u32 = 480;
const H: u32 = 360;

#[test]
fn start_screen_actions_share_full_ink_with_a_muted_chord() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping start_screen_actions_share_full_ink_with_a_muted_chord: no wgpu adapter"
        );
        return;
    };

    let prev = theme::active().name;
    let mut checked = 0usize;
    let mut distinct_worlds = 0usize;
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        if ink != muted {
            distinct_worlds += 1;
        }

        p.prepare_start_surface(&device, &queue, W, H)
            .expect("start-surface prepare");
        assert_eq!(
            p.gutter_buffer.lines.len(),
            2,
            "{}: exactly two start-screen rows",
            t.name
        );

        let mut verb_inks = Vec::with_capacity(2);
        for (row, line) in p.gutter_buffer.lines.iter().enumerate() {
            let text = line.text();
            let al = line.attrs_list();
            // Byte 0 is the chord glyph's own leading byte (⌘, `is_symbol`) on
            // both rows by construction — the CHORD end of the split.
            let chord_ink = al.get_span(0).color_opt;
            assert_eq!(
                chord_ink,
                Some(muted),
                "{}: row {row}'s chord glyph must read muted, not the verb's ink",
                t.name
            );
            // The line's last byte is inside the verb (every row ends in the
            // verb text, never the chord) — the VERB end of the split.
            let last = text.len() - 1;
            let verb_ink = al.get_span(last).color_opt;
            assert_eq!(
                verb_ink,
                Some(ink),
                "{}: row {row}'s verb must read base_content, not muted",
                t.name
            );
            verb_inks.push(verb_ink);
        }
        assert_eq!(
            verb_inks[0], verb_inks[1],
            "{}: both actions must share the SAME ink — hierarchy is order alone",
            t.name
        );
        checked += 1;
    }
    theme::set_active_by_name(prev).unwrap();
    assert_eq!(
        checked,
        theme::THEMES.len(),
        "every world must be swept, not a hand-picked sample"
    );
    assert!(
        distinct_worlds > 0,
        "no world in the roster actually distinguishes base_content from muted — \
         the exact-value checks above would be vacuous everywhere"
    );
    assert!(
        distinct_worlds < theme::THEMES.len(),
        "expected at least one deliberate muted==base_content monochrome world \
         (Wagtail); none found — if that world's palette changed, replace this \
         with a named exception instead of deleting it"
    );
}

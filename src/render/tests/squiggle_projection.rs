//! Retained per-line spell-squiggle geometry ([`super::super::rects::SquiggleProjection`]):
//! the retained proto set a reshape now derives from per-line splicing must
//! always agree with an INDEPENDENT full recompute
//! ([`TextPipeline::squiggle_full_scan_snapshot`], the pre-retention algorithm
//! kept verbatim as the ineligible-document fallback) across every axis the
//! queue item names — same-line edits, a word-boundary edit that changes a
//! NEIGHBOURING word's spelling, a structural line insert and delete, a
//! dictionary change with no reshape at all, and an unrelated buffer swap.
//! `TextPipeline::squiggle_output_snapshot` reads the same published cache
//! [`TextPipeline::spell_squiggles`] draws from, so these laws prove the
//! WIRING, not a second implementation of the geometry math.

use super::super::*;
use super::{headless_pipeline, view};
use crate::spell::Misspelling;

fn misspelling(line: usize, start_col: usize, end_col: usize) -> Misspelling {
    Misspelling {
        line,
        start_col,
        end_col,
    }
}

/// Both snapshots for the CURRENT pipeline state, asserted equal — the one
/// check every law in this file routes through.
fn assert_matches_oracle(p: &TextPipeline, context: &str) {
    let retained = p.squiggle_output_snapshot();
    let oracle = p.squiggle_full_scan_snapshot();
    assert_eq!(
        retained, oracle,
        "{context}: the retained squiggle cache diverges from an independent \
         full recompute"
    );
}

/// ORDINARY SAME-LINE EDITS: an edit on a DIFFERENT line than the misspelling
/// must retain the misspelled line's geometry untouched (proven non-vacuous
/// via `squiggle_lines_rebuilt`: the retained path visits at most the edited
/// line, never the whole document), and an edit ON the misspelled line itself
/// (the span grows) must still match the oracle once the new span lands.
#[test]
fn same_line_edits_retain_and_match_the_oracle() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping same_line_edits_retain_and_match_the_oracle: no wgpu adapter");
        return;
    };
    p.enable_text_sync_profile();

    // TWO pre-existing misspellings, on line 0 and line 2, with line 1 left
    // clean and about to be edited — the shape a full reseed and a true
    // incremental retention answer DIFFERENTLY (2 lines visited vs. 0), so
    // `squiggle_lines_rebuilt` below actually distinguishes them instead of
    // both landing on the same small number by coincidence.
    let text0 = "one wrold two\nsecond line here\nthrid line stays\n";
    let mut v0 = view(text0, 1, 0);
    v0.misspelled = vec![misspelling(0, 4, 9), misspelling(2, 0, 5)]; // "wrold", "thrid"
    p.set_view(&v0);
    assert_matches_oracle(&p, "seed");
    let seeded = p.squiggle_output_snapshot();
    assert_eq!(seeded.len(), 2, "sanity: two misspellings drawn");

    // Edit line 1 (unrelated to either misspelling) — a real product scenario
    // is typing elsewhere in the document while earlier/later lines still
    // carry outstanding misspellings.
    let text1 = "one wrold two\nsecond linex here\nthrid line stays\n";
    let mut v1 = view(text1, 1, 6);
    v1.misspelled = vec![misspelling(0, 4, 9), misspelling(2, 0, 5)];
    p.set_view(&v1);
    assert_matches_oracle(&p, "edit on a different line");
    assert_eq!(
        p.owner_scan().squiggle_lines_rebuilt,
        0,
        "an edit on the clean line1 must rebuild NOTHING — a full reseed \
         would report 2 (both misspelled lines), proving this actually \
         engaged the retained splice rather than falling back"
    );

    // Edit line 0 itself: "wrold" -> "wrolds" (still misspelled, span grows).
    // Only line 0 may be rebuilt now — line 2's cached geometry must survive
    // untouched.
    let text2 = "one wrolds two\nsecond linex here\nthrid line stays\n";
    let mut v2 = view(text2, 0, 9);
    v2.misspelled = vec![misspelling(0, 4, 10), misspelling(2, 0, 5)];
    p.set_view(&v2);
    assert_matches_oracle(&p, "edit on the misspelled line itself");
    assert_eq!(
        p.owner_scan().squiggle_lines_rebuilt,
        1,
        "only the edited line (line0) may be rebuilt — a full reseed would \
         report 2"
    );
    let final_snapshot = p.squiggle_output_snapshot();
    assert_eq!(final_snapshot.len(), 2);
}

/// A WORD-BOUNDARY EDIT ON A NEIGHBOURING WORD: deleting the space between
/// two words on line 0 merges them into a THIRD word with a DIFFERENT span
/// than either original — the item's own named hazard ("a misspelling's
/// extent depends on ... word boundaries that an edit can move from OUTSIDE
/// the changed line" trivially holds since a span never crosses `\n`, but a
/// span can still shift/merge/vanish anywhere ON the changed line, not only
/// at the exact column edited). The retained cache must reflect the NEW span
/// set exactly, not a stale mix of the old two.
#[test]
fn a_word_boundary_edit_changes_a_neighbouring_words_span_and_still_matches() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_word_boundary_edit_changes_a_neighbouring_words_span_and_still_matches: \
             no wgpu adapter"
        );
        return;
    };
    p.enable_text_sync_profile();

    // line0: "helo world stays" — "helo" (0..4) is misspelled, "world" is not.
    let text0 = "helo world stays\nanchor line\n";
    let mut v0 = view(text0, 1, 0);
    v0.misspelled = vec![misspelling(0, 0, 4)];
    p.set_view(&v0);
    assert_matches_oracle(&p, "seed");

    // Delete the space between "helo" and "world": the NEIGHBOURING word
    // fuses into "helo" + "world" -> "heloworld", a single new misspelling
    // covering what used to be two separate words' worth of columns.
    let text1 = "heloworld stays\nanchor line\n";
    let mut v1 = view(text1, 0, 4);
    v1.misspelled = vec![misspelling(0, 0, 9)]; // "heloworld"
    p.set_view(&v1);
    assert_matches_oracle(&p, "neighbouring word merged by a boundary edit");
    let snap = p.squiggle_output_snapshot();
    assert_eq!(
        snap.len(),
        1,
        "the merged span replaces the old one, not both coexisting stale"
    );
}

/// STRUCTURAL EDITS: a line SPLIT (Enter) and a line MERGE (Backspace at line
/// start) both change the document's line count, which the row-geometry
/// patch always rejects (`can_retain_geometry` requires an equal-length
/// band) — forcing the retained squiggle cache through its full-reseed
/// fallback exactly like `RowGeom` itself. Both directions must still match
/// the oracle.
#[test]
fn structural_line_insert_and_delete_reseed_and_match() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping structural_line_insert_and_delete_reseed_and_match: no wgpu adapter");
        return;
    };
    p.enable_text_sync_profile();

    // A misspelling BEFORE the split point (line0) and one AFTER it (line2) —
    // the split shifts every line from the insertion point on DOWN one visual
    // row, so line2's cached pixel top is a real, catchable regression if the
    // fallback to a full reseed is ever skipped (a `TextChange` band alone
    // can share `spans[2]`'s SPAN unchanged while its ROW moved).
    let text0 = "helo world\nkeep line\nwrold end\n";
    let mut v0 = view(text0, 0, 0);
    v0.misspelled = vec![misspelling(0, 0, 4), misspelling(2, 0, 5)];
    p.set_view(&v0);
    assert_matches_oracle(&p, "seed");

    // INSERT (line split): break line0 after "helo " into two lines. "keep
    // line" and "wrold end" both shift down one visual row each; the
    // misspelling that was on line2 is now on line3.
    let text1 = "helo \nworld\nkeep line\nwrold end\n";
    let mut v1 = view(text1, 1, 0);
    v1.misspelled = vec![misspelling(0, 0, 4), misspelling(3, 0, 5)];
    p.set_view(&v1);
    assert_matches_oracle(&p, "line split (structural insert)");
    assert_eq!(
        p.owner_scan().squiggle_lines_rebuilt,
        2,
        "a structural insert reseeds every line carrying a span — here both \
         still-misspelled lines, so the shifted line's pixels are never \
         served from a stale cache entry"
    );

    // DELETE (line merge): join line0 and line1 back together, shifting
    // "wrold end" back up by one visual row.
    let text2 = "helo world\nkeep line\nwrold end\n";
    let mut v2 = view(text2, 0, 5);
    v2.misspelled = vec![misspelling(0, 0, 4), misspelling(2, 0, 5)];
    p.set_view(&v2);
    assert_matches_oracle(&p, "line merge (structural delete)");
}

/// A DICTIONARY CHANGE WITH NO RESHAPE AT ALL: `self.misspelled` can change
/// out from under an unchanged document (a personal-dictionary addition, a
/// spellcheck toggle) — no `TextChange` band exists for this event at all, so
/// it exercises `reconcile_squiggle_projection` rather than the eager
/// `set_text`-driven splice. The scattered-line shape matters: two lines lose
/// their misspelling status in the SAME event while a third, untouched line
/// keeps its own.
#[test]
fn a_dictionary_change_with_no_reshape_reconciles_scattered_lines() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_dictionary_change_with_no_reshape_reconciles_scattered_lines: \
             no wgpu adapter"
        );
        return;
    };
    p.enable_text_sync_profile();

    let text = "wrold one\nkept line\nwrold two\nplain line\nwrold three\n";
    let mut v = view(text, 3, 0);
    v.misspelled = vec![
        misspelling(0, 0, 5),
        misspelling(2, 0, 5),
        misspelling(4, 0, 5),
    ];
    p.set_view(&v);
    assert_matches_oracle(&p, "seed: three scattered misspellings");
    assert_eq!(p.squiggle_output_snapshot().len(), 3);

    let reshapes_before = p.reshape_count;
    // "Add wrold to the dictionary": line0 and line4's spans vanish, line2's
    // does not (same text throughout — a same-word dictionary edit clears
    // EVERY occurrence at once, which is exactly the scattered, non-
    // contiguous shape a `TextChange` band cannot describe).
    let mut v2 = view(text, 3, 0);
    v2.misspelled = vec![misspelling(2, 0, 5)];
    p.set_view(&v2);
    assert_eq!(
        p.reshape_count, reshapes_before,
        "a dictionary-only change must not reshape — this law's whole point \
         is the axis `set_text`'s eager refresh never sees"
    );
    assert_matches_oracle(&p, "after the dictionary change, no reshape");
    let snap = p.squiggle_output_snapshot();
    assert_eq!(
        snap.len(),
        1,
        "only line2's misspelling should remain after the dictionary change"
    );
}

/// BUFFER SWAP: settle the projection on one document, then swap to an
/// entirely unrelated one sharing no line with the first — the swapped-to
/// document's squiggles must read exactly its own, not a residue of the
/// document it replaced.
#[test]
fn reseeds_clean_across_an_unrelated_buffer_swap() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping reseeds_clean_across_an_unrelated_buffer_swap: no wgpu adapter");
        return;
    };
    p.enable_text_sync_profile();

    let doc_a = "wrold opener\nmiddle\nclosing\n";
    let mut va = view(doc_a, 0, 0);
    va.misspelled = vec![misspelling(0, 0, 5)];
    p.set_view(&va);
    assert_matches_oracle(&p, "document A");
    assert_eq!(p.squiggle_output_snapshot().len(), 1);

    // An entirely different manuscript, no CJK/links, no shared line with A,
    // and no misspelling at all.
    let doc_b = "brand new manuscript begins\nfresh start\nclean\nlast line\n";
    let vb = view(doc_b, 0, 0);
    p.set_view(&vb);
    assert_matches_oracle(&p, "swapped to document B");
    assert!(
        p.squiggle_output_snapshot().is_empty(),
        "document B carries no misspelling at all — a residual squiggle here \
         would be the exact stale-cache regression this law guards against"
    );
}

/// INELIGIBLE FALLBACK: a document carrying a markdown LINK routes to the
/// untouched full-scan algorithm unconditionally (the same envelope
/// `NitProjection` uses), so the retained projection must simply stay out of
/// the way rather than publish something wrong.
#[test]
fn a_document_with_a_link_falls_back_to_full_scan_and_still_matches() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_document_with_a_link_falls_back_to_full_scan_and_still_matches: \
             no wgpu adapter"
        );
        return;
    };
    p.enable_text_sync_profile();

    let text = "wrold [a link](wrold) stays\nsecond line\n";
    let mut v = view(text, 1, 0);
    v.is_markdown = true;
    v.misspelled = vec![misspelling(0, 0, 5)];
    p.set_view(&v);
    assert_matches_oracle(&p, "document carrying a link");
}

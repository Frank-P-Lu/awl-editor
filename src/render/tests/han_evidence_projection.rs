//! Retained per-line Han-ambiguity EVIDENCE ([`super::super::rects::HanEvidenceProjection`],
//! [`crate::script::LineEvidence`]/[`crate::script::EvidenceCounts`]): the
//! document-scope aggregate a reshape now derives from per-line retained
//! flags must always equal a fresh whole-document [`crate::script::cjk_evidence`]
//! scan of the CURRENT text, through insertion, retraction, coalesced
//! reshapes, and an unrelated buffer swap. The exhaustive priority-order and
//! reference-counting behavior itself is unit-tested at the pure-function
//! seam in `script::evidence`; these laws prove the WIRING through
//! `TextPipeline::set_text_incremental` matches that oracle end to end.

use super::{headless_pipeline, view};

/// INSERT THEN RETRACT: a plain English document carries no evidence: a line
/// is edited to add a decisive traditional-only character (aggregate flips);
/// the SAME line is edited again to remove it (aggregate must fall back to
/// `None`, not remain stuck positive) — the item's own named regression
/// ("a deleted last decisive CJK character must retract its evidence").
/// Each step is checked against a fresh [`crate::script::cjk_evidence`] scan
/// of that exact text, not a hand-picked expectation.
#[test]
fn insert_then_delete_the_only_decisive_character_retracts_cleanly() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping insert_then_delete_the_only_decisive_character_retracts_cleanly: \
             no wgpu adapter"
        );
        return;
    };

    let text0 = "opening line\nanchor\nclosing line\n";
    let v0 = view(text0, 1, 0);
    p.set_view(&v0);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text0),
        "sanity: a plain document carries no evidence"
    );
    assert_eq!(p.han_evidence, None);

    // Line 0 gains a traditional-only character (說, per the module's own
    // spot-checked table).
    let text1 = "opening 說 line\nanchor\nclosing line\n";
    let v1 = view(text1, 1, 0);
    p.set_view(&v1);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text1),
        "the retained aggregate must match a fresh scan once evidence appears"
    );
    assert_eq!(p.han_evidence, Some(crate::frontmatter::Lang::ZhHant));

    // The SAME line loses its only decisive character.
    let text2 = "opening line\nanchor\nclosing line\n";
    let v2 = view(text2, 1, 0);
    p.set_view(&v2);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text2),
        "the retained aggregate must retract to match a fresh scan once the \
         decisive character is edited away"
    );
    assert_eq!(
        p.han_evidence, None,
        "the mechanism this law names: retraction, not a stuck positive latch"
    );
}

/// A decisive character living inside a FENCED CODE BLOCK or a FRONTMATTER
/// block still counts — evidence scanning is byte-scoped, not markdown-
/// structure-aware, exactly like the un-retained `cjk_evidence` it replaces.
#[test]
fn decisive_evidence_inside_frontmatter_and_a_fence_still_counts() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping decisive_evidence_inside_frontmatter_and_a_fence_still_counts: \
             no wgpu adapter"
        );
        return;
    };

    let text = "---\ntitle: 說\n---\n\n```\nplain code\n```\n\nprose\n";
    let mut v = view(text, 8, 0);
    v.is_markdown = true;
    p.set_view(&v);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text),
        "frontmatter's own decisive character must still be counted"
    );
    assert_eq!(p.han_evidence, Some(crate::frontmatter::Lang::ZhHant));

    // Move the SAME decisive character into the fenced block instead, out of
    // frontmatter entirely.
    let text2 = "---\ntitle: plain\n---\n\n```\n說\n```\n\nprose\n";
    let mut v2 = view(text2, 8, 0);
    v2.is_markdown = true;
    p.set_view(&v2);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text2),
        "a fence's own decisive character must still be counted"
    );
    assert_eq!(p.han_evidence, Some(crate::frontmatter::Lang::ZhHant));
}

/// COALESCED RESHAPES: two edits land back to back with no read in between
/// (`set_view` never lazily refreshes `han_evidence` from `prepare` — it is
/// refreshed eagerly inside `set_text_incremental`, mirroring
/// `NitProjection`'s own eager-refresh discipline), so the state after both
/// must reflect the LATEST text, not a stale intermediate one.
#[test]
fn evidence_patches_across_two_reshapes_before_one_read() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping evidence_patches_across_two_reshapes_before_one_read: no wgpu adapter");
        return;
    };

    // Step 0: a decisive simplified-only character on line0.
    let text0 = "这 opener\nanchor\nplain\n";
    let v0 = view(text0, 1, 0);

    // Step 1: line0's decisive character is gone, but line2 gains a NEW,
    // different decisive character (traditional-only) — pushed right after
    // step 0 with no `han_evidence` read in between.
    let text1 = "opener\nanchor\n說 closing\n";
    let v1 = view(text1, 1, 0);

    let reshapes_start = p.reshape_count;
    p.set_view(&v0);
    p.set_view(&v1);
    assert_eq!(
        p.reshape_count,
        reshapes_start + 2,
        "both edits must reshape — an unread intermediate edit is not a no-op"
    );
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(text1),
        "the one read after two un-observed reshapes must reflect the LATEST \
         text, not a stale intermediate aggregate"
    );
    assert_eq!(p.han_evidence, Some(crate::frontmatter::Lang::ZhHant));
}

/// BUFFER SWAP: settle the projection on one document, then swap to an
/// entirely unrelated one sharing no line with the first (`unchanged_band`'s
/// "replace everything" shape — the same mechanism that lets
/// `NitProjection` reseed for free across a buffer swap with no separate
/// buffer-identity key). The swapped-to document's evidence must read
/// exactly its own, not a residue of the document it replaced.
#[test]
fn reseeds_clean_across_an_unrelated_buffer_swap() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping reseeds_clean_across_an_unrelated_buffer_swap: no wgpu adapter");
        return;
    };

    // Document A: settles with decisive simplified-only evidence.
    let doc_a = "steady opener 这\nmiddle\nclosing\n";
    let va = view(doc_a, 1, 0);
    p.set_view(&va);
    assert_eq!(p.han_evidence, crate::script::cjk_evidence(doc_a));
    assert_eq!(p.han_evidence, Some(crate::frontmatter::Lang::ZhHans));

    // Document B: an entirely different, plain-English manuscript — no
    // shared line with A at any position, no CJK at all.
    let doc_b = "brand new manuscript begins\nfresh start\nmiddle line clean\nlast line\n";
    let vb = view(doc_b, 0, 0);
    p.set_view(&vb);
    assert_eq!(
        p.han_evidence,
        crate::script::cjk_evidence(doc_b),
        "the swapped-to document must read exactly its own evidence, not a \
         residue of the document it replaced"
    );
    assert_eq!(
        p.han_evidence, None,
        "document B carries no CJK at all — a residual `ZhHans` here would be \
         the exact stale-cache regression this law guards against"
    );
}

//! A CAPTURE-PROVABLE regression law for the Han-ambiguity evidence tier: an
//! untagged, bare-Han document carrying a Simplified-Chinese-only character
//! must render in the world's bundled Simplified-Chinese face, even though
//! the ordinary capture pipeline pins the `ja`-first `DEFAULT_CJK_PRIORITY`
//! (`docs/harness-reach.md`'s tier-3 boundary for the `cjk_priority`
//! SETTING). The evidence tier is text-derived, not config-derived, so it
//! sits on the OTHER side of that exact boundary — reachable by an ordinary
//! `--screenshot`, no `--config`/`--screenshot-app` needed.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

/// The item's own reported sentence: untagged, no kana, carries several
/// simplified-only characters (这/简/测/试/头/开/关/门/说/话/车).
const HEADLINE_SENTENCE: &str = "这是简体中文的一段测试文字骨头直角与其内外开关门说话车站\n";

#[test]
fn untagged_headline_sentence_resolves_bundled_zh_hans_face_in_every_world() {
    if !adapter_available() {
        eprintln!(
            "skipping untagged_headline_sentence_resolves_bundled_zh_hans_face_in_every_world: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_han_evidence_test_{}", std::process::id())),
    );

    // Enrolled from the roster (CLAUDE.md's enrolment law), not one named
    // world: every theme has its own zh_hans/ja candidate, and the fix must
    // hold on all of them, not just the one the bug was first noticed in.
    for world in crate::theme::THEMES.iter().map(|t| t.name) {
        crate::theme::set_active_by_name(world)
            .unwrap_or_else(|| panic!("{world} is a real world"));
        let mut buf = Buffer::from_str(HEADLINE_SENTENCE);
        buf.set_path(dir.join(format!("{world}.md")));
        let png = dir.join(format!("{world}.png"));
        capture_with(&png, &buf, &CaptureOpts::default())
            .unwrap_or_else(|_| panic!("{world} capture renders"));
        let j: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(png.with_extension("json")).unwrap())
                .unwrap();
        // NEVER-TOFU: the world's own zh_hans candidate is a real bundled
        // face, never absent/null, in every world.
        assert_eq!(
            j["font"]["scripts"]["zh_hans"]["bundled"],
            serde_json::json!(true),
            "{world}: zh_hans must be a real bundled face"
        );
        let zh_family = j["font"]["scripts"]["zh_hans"]["family"]
            .as_str()
            .unwrap_or_else(|| panic!("{world}: zh_hans family present"))
            .to_string();
        let ja_family = j["font"]["scripts"]["ja"]["family"]
            .as_str()
            .unwrap_or_else(|| panic!("{world}: ja family present"))
            .to_string();
        // PRESENCE FLOOR (CLAUDE.md's "law satisfiable by deleting its own
        // subject" tripwire): if a world's ja and zh_hans candidates were
        // ever the same family, this law would pass on that world without
        // proving anything — the whole point is that the evidence tier picks
        // the DIFFERENT face. Every bundled floor in docs/fonts.md keeps
        // these two script ladders on separate families by construction.
        assert_ne!(
            zh_family, ja_family,
            "{world}: ja and zh_hans must be distinguishable faces for this law to mean anything"
        );
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);

    // THE ACTUAL RESOLUTION, at the pure seam every world's `font.scripts`
    // map feeds (world-independent — the ladder answers a `FontId`, not a
    // family string; the loop above proves that `FontId` is a REAL,
    // DISTINCT bundled family in every world). Every Han run in the
    // headline sentence resolves `FontId::ZhHans` under the untagged,
    // `ja`-first pinned default the capture pipeline always uses.
    let evidence = crate::script::cjk_evidence(HEADLINE_SENTENCE);
    let effective =
        crate::script::effective_cjk_priority(evidence, &crate::frontmatter::DEFAULT_CJK_PRIORITY);
    let mut saw_han = false;
    for (_, script) in crate::script::script_runs(HEADLINE_SENTENCE) {
        assert_eq!(
            script,
            crate::script::Script::Han,
            "the fixture is pure Han"
        );
        saw_han = true;
        assert_eq!(
            crate::script::resolve_font_id(None, Some(script), &effective),
            crate::theme::FontId::ZhHans,
            "every Han run in the headline sentence resolves ZhHans"
        );
    }
    assert!(saw_han, "the fixture must actually contain Han runs");

    // NON-VACUITY: mutate away the evidence fold (read `cjk_priority.first()`
    // directly, the OLD un-evidenced ladder) and confirm the SAME sentence
    // would have resolved Japanese — the exact reported bug, proving this
    // law actually exercises the fix.
    let id_old = crate::script::resolve_font_id(
        None,
        Some(crate::script::Script::Han),
        &crate::frontmatter::DEFAULT_CJK_PRIORITY,
    );
    assert_eq!(
        id_old,
        crate::theme::FontId::Ja,
        "mutation proof: without the evidence fold this reads Japanese, \
         not Simplified Chinese"
    );
}

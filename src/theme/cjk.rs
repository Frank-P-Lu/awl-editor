//! src/theme/cjk.rs — the per-world CJK fallback LADDERS + the [`FontId`]
//! script-identity enum they're keyed by (Japanese mincho/gothic/variety,
//! Simplified Chinese serif/sans/Klee, Traditional Chinese, Korean
//! sans/serif) plus the bundled-face membership list [`EMBEDDED_CJK_FAMILIES`].
//! See [`crate::theme::worlds`] for which world picks which ladder.

// --- Per-theme CJK fallback families (mincho / gothic) ---------------------
//
// EVERY LADDER IN THIS FILE IS BUNDLED-FIRST, THEN SYSTEM. A bundled name is
// one that is ALWAYS present (loaded in `build_font_system`), so a run resolves
// without depending on any system font; the trailing system candidates are
// belt-and-suspenders.
//
// ⚠️ The trailing system entries are held pending the USER'S OWN comparison of
// bundled Noto against system Hiragino. Collapsing to bundled-only is a
// deliberate second step that waits for that nod — it would also let
// `resolve_cjk`'s weight-matching go, which exists purely because system faces
// do not register at the default Weight 400.

/// MINCHO (serif) Japanese fallback for the SERIF worlds: bundled Noto Serif
/// JP first, then Hiragino Mincho ProN (macOS) / Noto Serif CJK JP (Linux).
pub const CJK_MINCHO: &[&str] = &["Noto Serif JP", "Hiragino Mincho ProN", "Noto Serif CJK JP"];

/// GOTHIC (sans) Japanese fallback for the SANS / MONO worlds: bundled Noto
/// Sans JP first, then Hiragino Kaku Gothic ProN (macOS) / Noto Sans CJK JP
/// (Linux).
pub const CJK_GOTHIC: &[&str] = &[
    "Noto Sans JP",
    "Hiragino Kaku Gothic ProN",
    "Noto Sans CJK JP",
];

// --- Per-WORLD Japanese overrides ------------------------------------------
//
// The user's note: "with kana we probably want a couple more — they don't
// really change much across themes." Latin varies per world, so these ladders
// give Japanese the same variety: each names a distinct-character bundled JP
// face FIRST (`render::FONT_JA_VARIETY_FACES`), then falls back to the SAME
// bundled Noto FLOOR its neutral sibling uses, then the identical system
// candidates. The never-tofu floor is therefore the same for all of them, and
// `AWL_CJK_FORCE=floor` drops each cleanly to its plain Noto face. THEMES.md's
// assignment table says which world takes which, and why.

/// JAPANESE bookish-mincho ladder — the warm LITERARY serif for the book-serif
/// worlds: bundled Shippori Mincho over [`CJK_MINCHO`]'s own floor and system
/// candidates.
pub const CJK_JA_SHIPPORI: &[&str] = &[
    "Shippori Mincho",
    "Noto Serif JP",
    "Hiragino Mincho ProN",
    "Noto Serif CJK JP",
];

/// JAPANESE rounded-gothic ladder — the warm rounded "maru" sans for the
/// dedicated sans worlds: bundled Zen Maru Gothic over [`CJK_GOTHIC`]'s own
/// floor and system candidates.
pub const CJK_JA_ZENMARU: &[&str] = &[
    "Zen Maru Gothic",
    "Noto Sans JP",
    "Hiragino Kaku Gothic ProN",
    "Noto Sans CJK JP",
];

/// JAPANESE Klee ladder — the CHARACTERFUL kaisho/brush override for the
/// Klee-derived worlds, so their JA shares the brush character of their ZH
/// ([`CJK_ZH_HANS_KLEE`]'s LXGW WenKai is Klee One-derived). Bundled Klee One
/// over the Noto Sans JP floor and gothic system candidates.
pub const CJK_JA_KLEE: &[&str] = &[
    "Klee One",
    "Noto Sans JP",
    "Hiragino Kaku Gothic ProN",
    "Noto Sans CJK JP",
];

/// The bundled CJK family names — the "embedded" side of the [`FontId`]
/// resolver's asset-source classification, and the `apply_cjk_force` A/B
/// switch's "bundled" set. Data, not a code path:
/// [`crate::theme::Theme::candidates`] returns plain family-name ladders, and a
/// name here is one that is always loaded rather than one that may or may not
/// be installed on this machine. `TextPipeline::script_font_reports`' `bundled`
/// flag is only accurate while this list matches what is actually embedded.
pub(crate) const EMBEDDED_CJK_FAMILIES: &[&str] = &[
    "Noto Serif JP",
    "Noto Sans JP",
    "Noto Serif SC",
    "Noto Sans SC",
    "Noto Sans KR",
    "LXGW WenKai",
    // The per-world JA variety faces (`render::FONT_JA_VARIETY_FACES`).
    "Shippori Mincho",
    "Zen Maru Gothic",
    "Klee One",
    // The serif worlds' Korean serif override
    // (`render::FONT_CJK_COMPANION_FACES`).
    "Gowun Batang",
];

// --- Per-script font IDs + candidate ladders --------------------------------
//
// [`FontId`] names the per-script font IDENTITY awl resolves independently: the
// world's own Latin display face plus the four CJK-family scripts.
// [`crate::theme::Theme::candidates`] maps an ID to a prioritized family-name
// ladder, and the resolver (`render/text.rs::TextPipeline::resolve_font_id`)
// walks it and returns the first family actually registered in the font DB —
// ONE algorithm across all five IDs, never a per-script code path.
//
// USER TASTE CALLS, logged rather than hidden. ja keeps a bundled
// mincho/gothic split. zh-Hans mirrors that split with the user's own
// 思源宋体/思源黑体 pick ("Source Han" is Adobe/Google's shared name for the
// Noto CJK SC family), plus a characterful per-world override so ja and zh-Hans
// read as one hand on the Klee worlds. ko bundles one sans face with no
// serif/sans split of its own.
//
// zh-Hant is deliberately system-only: a Big5-class subset is ~13k chars
// against GB 2312's ~6.8k, a genuinely bigger lift, so it is BANKED rather than
// attempted. See THEMES.md's Han-unification note.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontId {
    /// The world's own Latin display/mono face (never a fallback — always
    /// resolves to the currently-shaping doc family, itself an embedded face).
    Latin,
    /// Japanese: kana + (contextually) Han. See [`crate::theme::Theme::cjk`].
    Ja,
    /// Simplified Chinese: Han. See [`crate::theme::Theme::zh_hans`].
    ZhHans,
    /// Traditional Chinese: Han + Bopomofo. See [`crate::theme::Theme::zh_hant`].
    ZhHant,
    /// Korean: Hangul + (contextually) Han. See [`crate::theme::Theme::ko`].
    Ko,
}

/// Every [`FontId`] variant — the never-tofu law test's sweep list, kept in
/// lockstep with the enum by hand (a `match` elsewhere enumerating `FontId`
/// with a no-wildcard arm is the actual compile-time guard; this is for
/// iteration convenience in tests).
pub const ALL_FONT_IDS: [FontId; 5] = [
    FontId::Latin,
    FontId::Ja,
    FontId::ZhHans,
    FontId::ZhHant,
    FontId::Ko,
];

/// Simplified Chinese SERIF ladder — the zh-Hans mincho companion, for the
/// SERIF worlds (`Theme::cjk == CJK_MINCHO`): bundled Noto Serif SC (Google
/// Fonts' Source Han Serif SC build, OFL, subset to GB 2312), then the system
/// PingFang SC / Noto Sans CJK SC candidates.
pub const CJK_ZH_HANS_SERIF: &[&str] = &["Noto Serif SC", "PingFang SC", "Noto Sans CJK SC"];

/// Simplified Chinese SANS ladder — the gothic companion, for the SANS/MONO
/// worlds (`Theme::cjk == CJK_GOTHIC`): bundled Noto Sans SC first, then the
/// same system trailing candidates as [`CJK_ZH_HANS_SERIF`].
pub const CJK_ZH_HANS_SANS: &[&str] = &["Noto Sans SC", "PingFang SC", "Noto Sans CJK SC"];

/// Simplified Chinese KLEE ladder — the CHARACTERFUL per-world override for the
/// Klee-derived worlds: bundled LXGW WenKai (霞鹜文楷, OFL,
/// github.com/lxgw/LxgwWenKai — a Klee One-derived face, subset to GB 2312),
/// falling back through [`CJK_ZH_HANS_SANS`]'s own floor and system candidates.
/// A logged TASTE CALL: it matches the Klee One display assignment those worlds
/// already carry, so their ja and zh-Hans read as one hand.
pub const CJK_ZH_HANS_KLEE: &[&str] = &[
    "LXGW WenKai",
    "Noto Sans SC",
    "PingFang SC",
    "Noto Sans CJK SC",
];

/// Traditional Chinese ladder: PingFang TC (macOS) then Noto Sans CJK TC
/// (Linux). NO bundled asset — see the module note above on why Big5 coverage
/// is banked rather than attempted.
pub const CJK_ZH_HANT: &[&str] = &["PingFang TC", "Noto Sans CJK TC"];

/// Korean SANS ladder — the SANS/MONO worlds' ko floor: bundled Noto Sans KR
/// (Google Fonts, OFL, subset to KS X 1001 modern hangul + jamo), then Apple SD
/// Gothic Neo / Noto Sans CJK KR. The SERIF worlds get [`CJK_KO_SERIF`].
pub const CJK_KO: &[&str] = &["Noto Sans KR", "Apple SD Gothic Neo", "Noto Sans CJK KR"];

/// Korean SERIF ladder, for the SERIF worlds (those whose `Theme::cjk` is a
/// mincho-family ja ladder). Bundled Gowun Batang (a Korean BATANG serif, OFL,
/// subset to the SAME KS X 1001 set as the Noto Sans KR floor) first, then that
/// floor, then serif-before-sans system candidates — [`CJK_JA_SHIPPORI`]'s
/// shape: characterful first, neutral bundled floor next, system last.
///
/// ⚠️ There is NO neutral bundled serif Korean floor, so the guaranteed floor
/// is the SANS Noto Sans KR, and that is what `AWL_CJK_FORCE=floor` drops to.
/// AppleMyungjo / Noto Serif CJK KR are reached only under
/// `AWL_CJK_FORCE=system`, which is what keeps a serif world's `system` capture
/// reading as a serif Korean.
pub const CJK_KO_SERIF: &[&str] = &[
    "Gowun Batang",
    "Noto Sans KR",
    "AppleMyungjo",
    "Noto Serif CJK KR",
    "Apple SD Gothic Neo",
    "Noto Sans CJK KR",
];

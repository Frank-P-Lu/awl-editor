//! Font-system assembly: default selection, bundled registration and fallback
//! pruning. The asset roster stays in render; its ordered loading lives here.

use super::*;

/// Family names of non-scalable / advance-breaking fallback faces to drop from
/// the font DB before shaping. These bitmap CJK faces (present in the macOS
/// system font set) return `inf` glyph advances under cosmic-text 0.18 + harfrust,
/// which breaks full-width CJK layout (every kanji forced onto its own line). With
/// them removed, fallback resolves CJK to a proper outline face. Match is
/// case-insensitive on the family name.
const BAD_FALLBACK_FAMILIES: &[&str] = &["GB18030 Bitmap"];

fn awl_font_override() -> &'static Option<std::path::PathBuf> {
    static ONCE: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var_os("AWL_FONT").map(std::path::PathBuf::from))
}

/// Select the requested monospace family while keeping the bundled UI close
/// face available. Registering that extra face never re-selects the mono family.
pub(super) fn register_default_font(font_system: &mut FontSystem, font_bytes: Vec<u8>) {
    let needs_bundled = font_bytes.as_slice() != FONT_DATA;
    let face_ids =
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(font_bytes),
            ));
    if let Some(family) = face_ids
        .first()
        .and_then(|id| font_system.db().face(*id))
        .and_then(|f| f.families.first().map(|(name, _)| name.clone()))
    {
        font_system.db_mut().set_monospace_family(family);
    }

    if needs_bundled {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(FONT_DATA.to_vec()),
            ));
    }
}

/// Build the shaping font system: register the MONO/default UI face (AWL_FONT
/// override or bundled), every per-theme display face, then prune the bad
/// fallback faces — the one-time font setup behind [`TextPipeline::new`].
pub(super) fn build_font_system() -> FontSystem {
    let mut font_system = FontSystem::new();
    // Choose the MONO/default UI font: AWL_FONT=/path/to/font.ttf overrides the
    // bundled default at runtime (handy for trying fonts). Whatever loads becomes
    // the monospace family, so the panel + the mono worlds (and any glyph a
    // proportional theme face lacks) resolve to it via Family::Monospace.
    let font_bytes: Vec<u8> = match awl_font_override() {
        Some(path) => crate::fs::active()
            .read(path.as_path())
            .unwrap_or_else(|e| {
                eprintln!("AWL_FONT {path:?}: {e}; falling back to bundled font");
                FONT_DATA.to_vec()
            }),
        None => FONT_DATA.to_vec(),
    };
    register_default_font(&mut font_system, font_bytes);

    // Load every per-theme display face so a live theme switch (or a headless
    // `--theme NAME` capture) can shape the document in that world's family via
    // `Family::Name` with no runtime font discovery. Each registers under the
    // exact family name recorded on its `Theme::font`; verified through fontdb
    // (see FONT_THEME_FACES). The mono default above stays the registered
    // monospace family, so it remains the fallback for any glyph a proportional
    // face is missing, and the panel/UI text keeps its mono look.
    for &(face_bytes, _pitch) in FONT_THEME_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // DEV-ONLY (FIRETAIL-MAXIMALIST-SHOWCASE round): `AWL_CHROME_FACE_FILE`
    // registers UNCOMMITTED audition font files (colon-separated paths) so the
    // chrome-face gallery can shoot candidate faces that are deliberately NOT
    // in the tree (candidate files stay out of the repo until a flip round
    // bundles the winner — the board's own rule). Pairs with
    // `AWL_CHROME_FACE_FORCE=<family>` to select one. Total no-op unset; a
    // missing/unreadable file prints a note and is skipped (never a crash).
    if let Ok(paths) = std::env::var("AWL_CHROME_FACE_FILE") {
        for path in paths.split(':').filter(|p| !p.trim().is_empty()) {
            match std::fs::read(path.trim()) {
                Ok(bytes) => {
                    font_system.db_mut().load_font_source(
                        glyphon::cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(bytes)),
                    );
                }
                Err(e) => eprintln!("AWL_CHROME_FACE_FILE {path:?}: {e}; skipped"),
            }
        }
    }

    // Register the bundled BOLD (700) display faces (see FONT_THEME_BOLD_FACES).
    // Each registers under the IDENTICAL family name its Regular uses, so a
    // `Weight::BOLD` request (the `**bold**` / `MdKind::Bold` arm) resolves to the
    // bold FILE instead of tripping cosmic-text's `weight_diff == 0` fallback trap
    // (which otherwise drops the Regular and lands in the mono fallback). No new
    // family and no other wiring — the bold arm is unchanged.
    for &face_bytes in FONT_THEME_BOLD_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled JAPANESE faces (Noto Serif/Sans JP — see
    // FONT_CJK_FACES) so `resolve_cjk` finds "Noto Serif JP"/"Noto Sans JP" in
    // the font DB on every machine, with no dependency on a system CJK face.
    // Named only via per-run CJK `AttrsList` spans (never a `Theme::font`), so
    // this changes zero Latin display shaping.
    for &face_bytes in FONT_CJK_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled per-WORLD JAPANESE VARIETY faces (Shippori Mincho,
    // Zen Maru Gothic, Klee One — see FONT_JA_VARIETY_FACES) so `resolve_font_id`
    // finds them for the worlds whose `Theme::cjk` ladder names them first, with
    // no dependency on a system CJK face. Named only via per-run CJK `AttrsList`
    // spans (never a `Theme::font`), so this changes zero Latin display shaping.
    for &face_bytes in FONT_JA_VARIETY_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the real Japanese heavy cuts after their Regular companions.
    // Script spans select one only when Markdown/heading styling requested a
    // heavy weight; ordinary Japanese remains on the exact Regular face.
    for &face_bytes in FONT_JA_BOLD_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled ZH-HANS + KOREAN faces (Noto Serif/Sans SC, Noto
    // Sans KR, LXGW WenKai — see FONT_ZH_KO_FACES) so `resolve_font_id` finds
    // them in the font DB on every machine, with no dependency on a system
    // PingFang/Apple SD Gothic Neo/Noto-CJK face. Named only via per-run CJK
    // `AttrsList` spans (never a `Theme::font`), so this changes zero Latin
    // display shaping — mirrors the JP faces' registration exactly.
    for &face_bytes in FONT_ZH_KO_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled CJK COMPANION faces (Gowun Batang — the serif worlds'
    // characterful Korean batang; see FONT_CJK_COMPANION_FACES) so `resolve_font_id`
    // finds it in the font DB on every machine, above the Noto Sans KR floor.
    // Named only via per-run CJK `AttrsList` spans (never a `Theme::font`), so
    // this changes zero Latin display shaping — mirrors the JP/ZH faces exactly.
    for &face_bytes in FONT_CJK_COMPANION_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    for &face_bytes in FONT_ORNAMENT_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled CHROME-VOICE faces (Archivo Black, Abril Fatface — see
    // FONT_CHROME_FACES) so `chrome_attrs`'s `Family::Name` request resolves them
    // on every machine when a world's `render_caps.chrome_face` names one. Named
    // ONLY through the chrome span (placard wordmark / title prefix / lens-strip
    // label — never a `Theme::font`), so this changes zero document display
    // shaping — a world with `ChromeFace::Body` (all but Firetail) is untouched.
    for &face_bytes in FONT_CHROME_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    font_system
        .db_mut()
        .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
            std::sync::Arc::new(FONT_SOURGUMMY_HEAVY_CANDIDATE.to_vec()),
        ));

    font_system
        .db_mut()
        .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
            std::sync::Arc::new(FONT_SYMBOLS.to_vec()),
        ));

    // Drop non-scalable / advance-breaking fallback faces before any shaping.
    // On macOS the system font DB includes bitmap CJK faces (e.g. "GB18030
    // Bitmap") that cosmic-text's fallback may pick FIRST for kanji; their
    // glyph advances come back as `inf`, which forces every kanji onto its own
    // wrapped line and drops the visual layout. Removing them lets fallback
    // resolve kanji to a proper outline JP face (e.g. Hiragino / BIZ UDGothic),
    // so full-width CJK shapes inline with finite advances. Latin is untouched.
    prune_bad_fallback_faces(&mut font_system);
    apply_cjk_force(&mut font_system);
    apply_sourgummy_heavy_force(&mut font_system);
    font_system
}

use theme::EMBEDDED_CJK_FAMILIES as BUNDLED_CJK_FAMILIES;

const SYSTEM_CJK_FAMILIES: &[&str] = &[
    "Hiragino Mincho ProN",
    "Hiragino Kaku Gothic ProN",
    "Noto Serif CJK JP",
    "Noto Sans CJK JP",
    "PingFang SC",
    "PingFang TC",
    "Noto Sans CJK SC",
    "Noto Sans CJK TC",
    "Apple SD Gothic Neo",
    "Noto Sans CJK KR",
];

const CHARACTERFUL_CJK_FAMILIES: &[&str] = &[
    "LXGW WenKai",
    "Shippori Mincho",
    "Zen Maru Gothic",
    "Klee One",
    "Gowun Batang",
];

fn awl_cjk_force() -> &'static Option<String> {
    static ONCE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var("AWL_CJK_FORCE").ok())
}

/// DEV-ONLY escape hatch for the Japanese-bundle-round + Chinese-round
/// TASTE-GATE captures (`gallery/jp-compare/`, `gallery/zh-worlds/`):
/// `AWL_CJK_FORCE=bundled` prunes the SYSTEM families from the font DB so
/// [`TextPipeline::resolve_font_id`] can only land on a bundled face;
/// `AWL_CJK_FORCE=system` prunes ALL bundled families instead, so resolution
/// falls through to whichever system CJK face is installed (Hiragino/PingFang/
/// Apple SD Gothic Neo on macOS); `AWL_CJK_FORCE=floor` prunes ONLY the
/// [`CHARACTERFUL_CJK_FAMILIES`] (LXGW WenKai / the JP-variety picks / Gowun
/// Batang), forcing every world that names a characterful override down to its
/// plain Noto floor (Klee worlds → Noto Sans SC zh-Hans; serif worlds → Noto
/// Sans KR ko; etc.) while leaving every other bundled floor face untouched.
/// Unset (the
/// default, every normal run) prunes nothing — every candidate stays
/// registered and each `Theme::candidates` ladder's priority order decides
/// (bundled/characterful first). This exists ONLY to produce the A/B(/C)
/// captures for the user's eyeball-call; it is not a product feature (no
/// config key, no CLI flag, undocumented in CAPTURE.md) and is a total no-op
/// unless the env var is set, so it changes nothing about normal/headless
/// determinism.
fn apply_cjk_force(font_system: &mut FontSystem) {
    let drop: &[&str] = match awl_cjk_force().as_deref() {
        Some("bundled") => SYSTEM_CJK_FAMILIES,
        Some("system") => BUNDLED_CJK_FAMILIES,
        Some("floor") => CHARACTERFUL_CJK_FAMILIES,
        _ => return,
    };
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.families
                .iter()
                .any(|(name, _)| drop.iter().any(|d| name.eq_ignore_ascii_case(d)))
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

fn awl_sourgummy_heavy_force() -> &'static Option<String> {
    static ONCE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var("AWL_SOURGUMMY_HEAVY_FORCE").ok())
}

/// DEV-ONLY escape hatch for the 700-vs-900 heavy-candidate gallery
/// (mirrors [`apply_cjk_force`]'s shape exactly): unset (every normal run,
/// every default capture) prunes nothing — `Weight::BOLD` resolves to the
/// 700 [`FONT_THEME_BOLD_FACES`] file by nearest-weight, and the bundled 900
/// [`FONT_SOURGUMMY_HEAVY_CANDIDATE`] just sits addressable-but-unselected.
/// `AWL_SOURGUMMY_HEAVY_FORCE=900` removes the 700 "Sour Gummy" face (by
/// family name + exact weight, so the Regular/400 face is untouched) from the
/// font DB, so the SAME `Weight::BOLD` request falls through to the 900 file
/// instead — a real in-app A/B, not a synthetic side-by-side. Not a product
/// feature (no config key, no CLI flag, undocumented in CAPTURE.md); a total
/// no-op unless the env var is set.
fn apply_sourgummy_heavy_force(font_system: &mut FontSystem) {
    if awl_sourgummy_heavy_force().as_deref() != Some("900") {
        return;
    }
    let bold_weight = glyphon::cosmic_text::fontdb::Weight(700);
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.weight == bold_weight
                && f.families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case("Sour Gummy"))
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

/// Remove [`BAD_FALLBACK_FAMILIES`] from the font system's database so cosmic-text
/// never selects them during fallback. Safe no-op if none are present (e.g. on
/// non-macOS, or if the system set changes). Only affects fallback for glyphs the
/// bundled mono font lacks (CJK); Latin still resolves to the bundled monospace.
fn prune_bad_fallback_faces(font_system: &mut FontSystem) {
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.families.iter().any(|(name, _)| {
                BAD_FALLBACK_FAMILIES
                    .iter()
                    .any(|bad| name.eq_ignore_ascii_case(bad))
            })
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

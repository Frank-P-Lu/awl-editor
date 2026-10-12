//! Immutable theme-family selection over the renderer's existing bundled faces.

use ttf_parser::Face;

use super::{FontAsset, FontRole, FontRoster};

impl FontRoster {
    pub fn for_theme(theme: &crate::theme::Theme) -> Result<Self, String> {
        let regular = display_face(theme.font, false)
            .ok_or_else(|| format!("PDF export has no bundled regular face for {}", theme.font))?;
        let bold = display_face(theme.font, true)
            .ok_or_else(|| format!("PDF export has no bundled bold face for {}", theme.font))?;
        let mono = display_face(theme.mono, false)
            .ok_or_else(|| format!("PDF export has no bundled regular face for {}", theme.mono))?;
        let mono_bold = display_face(theme.mono, true)
            .ok_or_else(|| format!("PDF export has no bundled bold face for {}", theme.mono))?;
        let (japanese_family, japanese) = theme
            .cjk
            .iter()
            .find_map(|family| japanese_face(family).map(|bytes| (*family, bytes)))
            .ok_or_else(|| {
                format!(
                    "PDF export has no bundled Japanese face in {}'s candidate ladder",
                    theme.name
                )
            })?;
        Ok(Self {
            assets: [
                themed_asset(FontRole::Serif, theme.font, "AWLThemeBody-Regular", regular),
                themed_asset(FontRole::SerifBold, theme.font, "AWLThemeBody-Bold", bold),
                themed_asset(FontRole::Mono, theme.mono, "AWLThemeMono-Regular", mono),
                themed_asset(
                    FontRole::MonoBold,
                    theme.mono,
                    "AWLThemeMono-Bold",
                    mono_bold,
                ),
                themed_asset(
                    FontRole::JapaneseSerif,
                    japanese_family,
                    "AWLThemeJapanese-Prose",
                    japanese,
                ),
                themed_asset(
                    FontRole::JapaneseSans,
                    japanese_family,
                    "AWLThemeJapanese-Code",
                    japanese,
                ),
            ],
        })
    }
}

fn themed_asset(
    role: FontRole,
    family: &'static str,
    pdf_name: &'static str,
    bytes: &'static [u8],
) -> FontAsset {
    let weight = Face::parse(bytes, 0)
        .expect("verified bundled PDF theme face")
        .weight()
        .to_number();
    FontAsset {
        role,
        family,
        pdf_name,
        bytes,
        weight,
    }
}

/// Closed mapping from the theme family's authored name to the renderer's
/// existing bundled bytes. The all-theme law keeps this exhaustive as the
/// world roster grows; an unsupported face is an error, never a substitution.
fn display_face(family: &str, bold: bool) -> Option<&'static [u8]> {
    let regular = |index: usize| crate::render::FONT_THEME_FACES[index].0;
    let heavy = |index: usize| crate::render::FONT_THEME_BOLD_FACES[index];
    Some(match (family, bold) {
        ("Literata", false) => regular(0),
        ("Literata", true) => heavy(0),
        ("Newsreader 16pt 16pt", false) => regular(1),
        ("Newsreader 16pt 16pt", true) => heavy(1),
        ("IBM Plex Sans", false) => regular(2),
        ("IBM Plex Sans", true) => heavy(2),
        ("Zilla Slab", false) => regular(3),
        ("Zilla Slab", true) => heavy(3),
        ("JetBrains Mono", false) => regular(4),
        ("JetBrains Mono", true) => heavy(12),
        ("Figtree", false) => regular(5),
        ("Figtree", true) => heavy(4),
        ("iA Writer Quattro S", false) => regular(6),
        ("iA Writer Quattro S", true) => heavy(5),
        ("Monaspace Xenon", false) => regular(7),
        ("Monaspace Xenon", true) => heavy(13),
        ("Fraunces 9pt", false) => regular(8),
        ("Fraunces 9pt", true) => heavy(6),
        ("EB Garamond", false) => regular(9),
        ("EB Garamond", true) => heavy(7),
        ("Fira Sans", false) => regular(10),
        ("Fira Sans", true) => heavy(8),
        ("Iosevka", false) => regular(11),
        ("Iosevka", true) => heavy(14),
        ("Bitter", false) => &crate::font_assets::BITTER_REGULAR,
        ("Bitter", true) => &crate::font_assets::BITTER_BOLD,
        ("Sour Gummy", false) => regular(13),
        ("Sour Gummy", true) => heavy(10),
        ("IBM Plex Mono", false) => &crate::font_assets::IBM_PLEX_MONO_LIGHT,
        ("IBM Plex Mono", true) => &crate::font_assets::IBM_PLEX_MONO_BOLD,
        _ => return None,
    })
}

fn japanese_face(family: &str) -> Option<&'static [u8]> {
    Some(match family {
        "Noto Serif JP" => crate::render::FONT_CJK_FACES[0],
        "Noto Sans JP" => crate::render::FONT_CJK_FACES[1],
        "Shippori Mincho" => crate::render::FONT_JA_VARIETY_FACES[0],
        "Zen Maru Gothic" => crate::render::FONT_JA_VARIETY_FACES[1],
        "Klee One" => crate::render::FONT_JA_VARIETY_FACES[2],
        _ => return None,
    })
}

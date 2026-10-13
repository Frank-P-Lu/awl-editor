//! Closed inventory of bundled PDF prose, fallback and ornament faces.
use super::{FontAsset, FontRole};

pub(in crate::export::pdf) const ORNAMENT_ASSET: FontAsset = FontAsset {
    role: FontRole::Ornament,
    family: crate::theme::ORNAMENT_MARKS,
    pdf_name: "AWLMarks",
    bytes: crate::render::FONT_SYMBOLS,
    weight: 400,
};

pub(in crate::export::pdf) static ASSETS: [FontAsset; 7] = [
    FontAsset {
        role: FontRole::Serif,
        family: "Bitter",
        pdf_name: "AWLBitter-Regular",
        bytes: &crate::font_assets::BITTER_REGULAR,
        weight: 400,
    },
    FontAsset {
        role: FontRole::SerifBold,
        family: "Bitter",
        pdf_name: "AWLBitter-Bold",
        bytes: &crate::font_assets::BITTER_BOLD,
        weight: 700,
    },
    FontAsset {
        role: FontRole::Mono,
        family: "IBM Plex Mono",
        pdf_name: "AWLIBMPlexMono-Light",
        bytes: &crate::font_assets::IBM_PLEX_MONO_LIGHT,
        weight: 300,
    },
    FontAsset {
        role: FontRole::MonoBold,
        family: "IBM Plex Mono",
        pdf_name: "AWLIBMPlexMono-Bold",
        bytes: &crate::font_assets::IBM_PLEX_MONO_BOLD,
        weight: 700,
    },
    FontAsset {
        role: FontRole::JapaneseSerif,
        family: "Noto Serif JP",
        pdf_name: "AWLNotoSerifJP-Regular",
        bytes: &crate::font_assets::NOTO_SERIF_JP_REGULAR,
        weight: 400,
    },
    FontAsset {
        role: FontRole::JapaneseSans,
        family: "Noto Sans JP",
        pdf_name: "AWLNotoSansJP-Regular",
        bytes: &crate::font_assets::NOTO_SANS_JP_REGULAR,
        weight: 400,
    },
    ORNAMENT_ASSET,
];

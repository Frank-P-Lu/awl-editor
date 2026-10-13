//! Paper geometry and the immutable paint snapshot for one PDF document.

use crate::export::pdf_options::PdfPaper;

use super::{Op, Page};

pub(in crate::export::pdf) const PAGE_W: f32 = PdfPaper::A4.size().0;
pub(in crate::export::pdf) const PAGE_H: f32 = PdfPaper::A4.size().1;
pub(in crate::export::pdf) const MARGIN_X: f32 = 66.638;
pub(in crate::export::pdf) const MARGIN_Y: f32 = 56.693;
/// The locked prose measure: seventy Bitter body characters at 11pt.
pub(in crate::export::pdf) const PROSE_CHARS: usize = 70;
pub(in crate::export::pdf) const MEASURE: f32 = PROSE_CHARS as f32 * 6.6;

#[derive(Clone, Copy)]
pub(in crate::export::pdf) struct PageGeometry {
    pub(in crate::export::pdf) width: f32,
    pub(in crate::export::pdf) height: f32,
    pub(in crate::export::pdf) margin_x: f32,
    pub(in crate::export::pdf) margin_y: f32,
    pub(in crate::export::pdf) measure: f32,
}

impl PageGeometry {
    pub(in crate::export::pdf) const fn for_paper(paper: PdfPaper) -> Self {
        match paper {
            PdfPaper::A4 => Self {
                width: PAGE_W,
                height: PAGE_H,
                margin_x: MARGIN_X,
                margin_y: MARGIN_Y,
                measure: MEASURE,
            },
            PdfPaper::Letter => Self {
                width: 612.0,
                height: 792.0,
                margin_x: 72.0,
                margin_y: 54.0,
                measure: 468.0,
            },
        }
    }

    pub(in crate::export::pdf) const fn bottom(self) -> f32 {
        self.height - self.margin_y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::export::pdf) struct PdfColor(pub(in crate::export::pdf) [u8; 3]);

impl From<crate::theme::Srgb> for PdfColor {
    fn from(color: crate::theme::Srgb) -> Self {
        Self([color.r, color.g, color.b])
    }
}

#[derive(Clone, Copy)]
pub(in crate::export::pdf) struct ThemePalette {
    pub(in crate::export::pdf) paper: PdfColor,
    pub(in crate::export::pdf) ink: PdfColor,
    pub(in crate::export::pdf) muted: PdfColor,
    pub(in crate::export::pdf) footer: PdfColor,
    pub(in crate::export::pdf) faint: PdfColor,
    pub(in crate::export::pdf) accent: PdfColor,
    pub(in crate::export::pdf) surface: PdfColor,
    pub(in crate::export::pdf) surface_strong: PdfColor,
}

#[derive(Clone, Copy)]
pub(in crate::export::pdf) struct Appearance {
    pub(in crate::export::pdf) ornaments: Option<(crate::theme::Ornaments, f32)>,
    pub(in crate::export::pdf) theme_name: Option<&'static str>,
    pub(in crate::export::pdf) palette: Option<ThemePalette>,
}

impl Appearance {
    pub(in crate::export::pdf) const fn plain() -> Self {
        Self {
            ornaments: None,
            theme_name: None,
            palette: None,
        }
    }

    pub(in crate::export::pdf) fn theme(theme: &crate::theme::Theme) -> Self {
        Self {
            ornaments: Some((theme.ornaments, theme.ornament_scale)),
            theme_name: Some(theme.name),
            palette: Some(ThemePalette {
                paper: theme.base_100.into(),
                ink: theme.base_content.into(),
                muted: theme.muted.into(),
                footer: if crate::theme::contrast_ratio(theme.muted, theme.base_100) >= 4.5 {
                    theme.muted.into()
                } else {
                    theme.base_content.into()
                },
                faint: theme.faint.into(),
                accent: theme.primary.into(),
                surface: theme.base_200.into(),
                surface_strong: theme.base_300.into(),
            }),
        }
    }
}

pub(super) fn page(geometry: PageGeometry, appearance: Appearance) -> Page {
    let mut page = Page::default();
    if let Some(palette) = appearance.palette {
        page.ops.push(Op::ColorRect {
            x: 0.0,
            y: 0.0,
            w: geometry.width,
            h: geometry.height,
            color: palette.paper,
        });
        page.ops.push(Op::ColorLine {
            x1: geometry.margin_x,
            y1: geometry.margin_y - 18.0,
            x2: geometry.margin_x + 64.0,
            y2: geometry.margin_y - 18.0,
            width: 1.5,
            color: palette.accent,
        });
        page.ops.push(Op::ColorLine {
            x1: geometry.margin_x,
            y1: geometry.height - geometry.margin_y + 18.0,
            x2: geometry.width - geometry.margin_x,
            y2: geometry.height - geometry.margin_y + 18.0,
            width: 0.45,
            color: palette.faint,
        });
    }
    page
}

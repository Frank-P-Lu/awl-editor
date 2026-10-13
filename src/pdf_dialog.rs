//! One geometry and control contract for the native PDF export dialog.
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct View {
    pub image: Arc<image::RgbaImage>,
    pub appearance: String,
    pub page_numbers: bool,
    pub preserve_line_breaks: bool,
    pub paper: String,
    pub page: usize,
    pub pages: usize,
    pub selected: usize,
    pub revision: u64,
    pub error: Option<String>,
}

impl View {
    pub fn labels(&self) -> [String; 7] {
        [
            format!("Appearance\n{}", self.appearance),
            format!("Paper\n{}", self.paper),
            format!(
                "{} Show page numbers",
                if self.page_numbers { "☑" } else { "☐" }
            ),
            format!(
                "{} Preserve line breaks",
                if self.preserve_line_breaks {
                    "☑"
                } else {
                    "☐"
                }
            ),
            format!("←   Page {} of {}   →", self.page + 1, self.pages),
            "Export".into(),
            "Cancel".into(),
        ]
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub controls: [[f32; 4]; 7],
    pub preview: [f32; 4],
    pub title: [f32; 2],
    pub hint: [f32; 2],
}

impl Geometry {
    pub fn new(width: f32, height: f32, dpi: f32, ratio: f32) -> Self {
        let pad = (24.0 * dpi).min(width * 0.05);
        let gap = 12.0 * dpi;
        let cell_h = (42.0 * dpi).min(height * 0.11);
        let column = (width - 2.0 * pad - gap).max(1.0) / 2.0;
        let top = (96.0 * dpi).min(height * 0.24);
        let footer = (height - pad - cell_h).max(top + cell_h + gap);
        let controls = [
            [pad, top, column, cell_h],
            [pad + column + gap, top, column, cell_h],
            [pad, top + cell_h + gap, column, 26.0 * dpi],
            [pad + column + gap, top + cell_h + gap, column, 26.0 * dpi],
            [pad, footer, (width - 2.0 * pad - 2.0 * gap) * 0.5, cell_h],
            [width - pad - column * 0.45, footer, column * 0.45, cell_h],
            [
                width - pad - column * 0.9 - gap,
                footer,
                column * 0.45,
                cell_h,
            ],
        ];
        let preview_top = top + cell_h + gap + 26.0 * dpi + gap;
        let available_h = (footer - gap - preview_top).max(1.0);
        let page_w = (width - 2.0 * pad).min(available_h * ratio).max(1.0);
        let page_h = page_w / ratio;
        Self {
            controls,
            preview: [(width - page_w) * 0.5, preview_top, page_w, page_h],
            title: [pad, (20.0 * dpi).min(height * 0.05)],
            hint: [pad, (48.0 * dpi).min(height * 0.135)],
        }
    }

    pub fn hit(&self, x: f32, y: f32) -> Option<usize> {
        self.controls.iter().position(|[left, top, width, height]| {
            x >= *left && x <= left + width && y >= *top && y <= top + height
        })
    }
}

/// Sidecars describe the audition's frozen page, separately from document layout.
pub(crate) fn metadata(view: Option<&View>) -> String {
    view.map(|v| {
        serde_json::json!({
            "appearance": v.appearance,
            "page_numbers": v.page_numbers,
            "preserve_line_breaks": v.preserve_line_breaks,
            "paper": v.paper,
            "page": v.page,
            "pages": v.pages,
            "selected": v.selected,
            "revision": v.revision,
            "error": v.error,
        })
        .to_string()
    })
    .unwrap_or_else(|| "null".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_controls_and_pages_fit_both_papers_and_dpi_values() {
        let _serial = crate::testlock::serial();
        for dpi in [1.0, 2.0] {
            for (w, h) in [(480.0, 360.0), (1200.0, 1000.0)] {
                for ratio in [595.276 / 841.890, 612.0 / 792.0] {
                    let g = Geometry::new(w * dpi, h * dpi, dpi, ratio);
                    for (i, [x, y, cw, ch]) in g.controls.into_iter().enumerate() {
                        assert!(x >= 0.0 && y >= 0.0 && x + cw <= w * dpi && y + ch <= h * dpi);
                        assert_eq!(g.hit(x + cw / 2.0, y + ch / 2.0), Some(i));
                    }
                    let [x, y, pw, ph] = g.preview;
                    assert!(pw > 0.0 && ph > 0.0 && x >= 0.0 && y >= 0.0);
                    assert!(x + pw <= w * dpi && y + ph <= h * dpi);
                }
            }
        }
    }
}

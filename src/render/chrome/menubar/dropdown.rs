use super::*;
use std::ops::Range;

pub(super) struct DropdownPlan {
    pub labels: String,
    pub chords: String,
    pub disabled_labels: Vec<Range<usize>>,
    pub disabled_chords: Vec<Range<usize>>,
    pub rows: Vec<crate::menubar::DropRow>,
    pub rows_total: f32,
    pub content_w: f32,
}

pub(super) struct CardGeometry {
    pub inner_left: f32,
    pub inner_top: f32,
}

impl TextPipeline {
    pub(super) fn prepare_dropdown_card(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: [u32; 2],
        menu_i: usize,
        bar_h: f32,
        plan: &DropdownPlan,
    ) -> CardGeometry {
        let anchor = self
            .menubar_boxes
            .get(menu_i)
            .copied()
            .unwrap_or(crate::menubar::TitleBox {
                band_left: 0.0,
                text_left: 0.0,
                text_right: 0.0,
                band_right: 0.0,
            });
        let rect = crate::menubar::drop_rect(&anchor, bar_h, plan.content_w, plan.rows_total);
        self.menu_drop_rect = Some(rect);
        self.menu_drop_rows.clone_from(&plan.rows);
        self.menu_drop_menu = Some(menu_i);
        super::set_float_quads(
            &mut self.menu_drop_shadow,
            &mut self.menu_drop_border,
            &mut self.menu_drop_card,
            device,
            queue,
            viewport[0],
            viewport[1],
            Some(rect),
            super::FloatElevation::Rimmed,
            0.0,
            None,
        );

        let inner_left = rect[0] + crate::menubar::DROP_PAD_X;
        let inner_top = rect[1] + crate::menubar::DROP_PAD_Y;
        let separators: Vec<[f32; 4]> = plan
            .rows
            .iter()
            .filter(|row| row.separator)
            .map(|row| {
                [
                    inner_left,
                    inner_top + row.top + row.height * 0.5 - 0.5,
                    plan.content_w,
                    1.0,
                ]
            })
            .collect();
        self.menu_drop_sep
            .prepare(device, queue, viewport[0], viewport[1], &separators);
        CardGeometry {
            inner_left,
            inner_top,
        }
    }
}

impl DropdownPlan {
    pub fn new(
        items: &[crate::menu::RosterItem],
        row_h: f32,
        label_char_w: f32,
        is_markdown: bool,
        scale: f32,
    ) -> Self {
        let mut labels = String::new();
        let mut chords = String::new();
        let mut disabled_labels = Vec::new();
        let mut disabled_chords = Vec::new();
        let mut separators = Vec::with_capacity(items.len());
        let mut widest_label = 0;
        let mut widest_chord = 0;
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                labels.push('\n');
                chords.push('\n');
            }
            let label_start = labels.len();
            let chord_start = chords.len();
            match item {
                crate::menu::RosterItem::Routed { id, label, .. } => {
                    let chord = crate::menu::item_chord_for_id(id);
                    widest_label = widest_label.max(label.chars().count());
                    widest_chord = widest_chord.max(chord.chars().count());
                    labels.push_str(label);
                    chords.push_str(&chord);
                    separators.push(false);
                }
                crate::menu::RosterItem::Predefined(kind) => {
                    let label = crate::menu::predefined_label(*kind);
                    widest_label = widest_label.max(label.chars().count());
                    labels.push_str(label);
                    separators.push(false);
                }
                crate::menu::RosterItem::Submenu { label, .. } => {
                    widest_label = widest_label.max(label.chars().count());
                    labels.push_str(label);
                    separators.push(false);
                }
                crate::menu::RosterItem::Separator => separators.push(true),
            }
            if !crate::menu::dropdown_item_enabled(item, is_markdown) {
                disabled_labels.push(label_start..labels.len());
                disabled_chords.push(chord_start..chords.len());
            }
        }
        let (rows, rows_total) = crate::menubar::drop_rows(&separators, row_h);
        let content_w = ((widest_label + rowlayout::GAP_CHARS + widest_chord) as f32
            * label_char_w
            * DROP_WIDTH_SLACK)
            .max(DROP_MIN_WIDTH.px(scale));
        Self {
            labels,
            chords,
            disabled_labels,
            disabled_chords,
            rows,
            rows_total,
            content_w,
        }
    }
}

pub(super) fn rich_spans<'a>(
    text: &'a str,
    disabled: &[Range<usize>],
    base: &Attrs<'a>,
    enabled_ink: glyphon::Color,
    disabled_ink: glyphon::Color,
) -> Vec<(&'a str, Attrs<'a>)> {
    let mut spans = Vec::new();
    let mut cursor = 0;
    for range in disabled {
        if cursor < range.start {
            spans.push((&text[cursor..range.start], base.clone().color(enabled_ink)));
        }
        spans.push((&text[range.clone()], base.clone().color(disabled_ink)));
        cursor = range.end;
    }
    if cursor < text.len() || spans.is_empty() {
        spans.push((&text[cursor..], base.clone().color(enabled_ink)));
    }
    spans
}

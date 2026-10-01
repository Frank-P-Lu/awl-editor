//! Find/Replace spacing is geometry, not a run of lucky spaces in Saltpan.
//! This sweeps the whole world roster at the ordinary, wrapped, and stacked
//! widths, plus both display densities. Each arm measures the actual shaped
//! label and footer ink against the published control boxes; every arm checks that
//! same-row click targets remain separated and inside the responsive card.

use super::super::*;
use super::{headless_dqp, view};

const MIN_AIR: f32 = 4.0;

fn control(g: &plan::PanelGeometry, name: &str) -> [f32; 4] {
    g.controls
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("missing panel control {name}"))
        .rect
}

fn ink_span(p: &TextPipeline, g: &plan::PanelGeometry, row: usize, needle: &str) -> [f32; 2] {
    let text = p
        .panel_buffer
        .layout_runs()
        .find(|run| run.line_i == row)
        .unwrap_or_else(|| panic!("missing panel row {row}"))
        .text;
    let start = text
        .find(needle)
        .unwrap_or_else(|| panic!("row {row} lacks {needle:?}: {text:?}"));
    let (x0, x1) = p
        .panel_span_x(row as f32, start, start + needle.len())
        .unwrap_or_else(|| panic!("{needle:?} shaped no ink on row {row}"));
    [g.text_left + x0, g.text_left + x1]
}

fn assert_gap(ctx: &str, left: f32, right: f32, dpi: f32) {
    let gap = right - left;
    assert!(
        gap + 0.05 >= MIN_AIR * dpi,
        "{ctx}: only {gap}px ({:.2} logical) of clear air; want at least {MIN_AIR}",
        gap / dpi
    );
}

fn assert_controls_separate(ctx: &str, g: &plan::PanelGeometry, dpi: f32) {
    let [cx, cy, cw, ch] = g.card;
    for c in &g.controls {
        let [x, y, w, h] = c.rect;
        assert!(
            x >= cx - 0.01 && y >= cy - 0.01 && x + w <= cx + cw + 0.01 && y + h <= cy + ch + 0.01,
            "{ctx}: {} lies outside the card: {:?} vs {:?}",
            c.name,
            c.rect,
            g.card
        );
    }
    for (i, a) in g.controls.iter().enumerate() {
        for b in &g.controls[i + 1..] {
            let ay1 = a.rect[1] + a.rect[3];
            let by1 = b.rect[1] + b.rect[3];
            if ay1.min(by1) - a.rect[1].max(b.rect[1]) <= 0.5 {
                continue;
            }
            let gap = if a.rect[0] <= b.rect[0] {
                b.rect[0] - (a.rect[0] + a.rect[2])
            } else {
                a.rect[0] - (b.rect[0] + b.rect[2])
            };
            assert!(
                gap + 0.05 >= MIN_AIR * dpi,
                "{ctx}: same-row controls {} and {} have only {gap}px air",
                a.name,
                b.name
            );
        }
    }
}

fn assert_all_text_inside(ctx: &str, p: &TextPipeline, g: &plan::PanelGeometry) {
    let pad = p.metrics.panel_ui().px(chrome::PANEL_PAD);
    let right = g.card[0] + g.card[2] - pad;
    let bottom = g.card[1] + g.card[3] - pad;
    for run in p.panel_buffer.layout_runs() {
        let run_right = g.text_left + run.line_w;
        let run_bottom = g.text_top + run.line_top + run.line_height;
        assert!(
            run_right <= right + 0.51 && run_bottom <= bottom + 0.51,
            "{ctx}: row {} escapes card: x {run_right}/{right}, \
             y {run_bottom}/{bottom}, text {:?}",
            run.line_i,
            run.text
        );
    }
}

fn assert_labels_clear_controls(ctx: &str, p: &TextPipeline, g: &plan::PanelGeometry, dpi: f32) {
    for (name, label, span) in [
        ("find_field", "Find", p.panel_control_spans.find_field),
        (
            "replace_field",
            "Replace",
            p.panel_control_spans.replace_field,
        ),
    ] {
        let row = span.expect("both field spans").row as usize;
        // Stacked labels occupy the preceding row; horizontal air applies only
        // while the label and field share a row, independent of their indices.
        if p.panel_buffer.lines[row].text().starts_with(label) {
            assert_gap(ctx, ink_span(p, g, row, label)[1], control(g, name)[0], dpi);
        }
    }
    let nav_row = p.panel_control_spans.nav_prev.unwrap().row as usize;
    assert_gap(
        ctx,
        ink_span(p, g, nav_row, "1 of 2")[1],
        control(g, "nav_prev")[0],
        dpi,
    );
    let reveal_row = p.panel_control_spans.reveal.unwrap().row as usize;
    let hint = format!(
        "{} field   {} close",
        crate::keyspec::PANEL_SWITCH_FIELD.label(),
        crate::keyspec::PANEL_CLOSE.label(),
    );
    if p.panel_buffer.lines[reveal_row].text().contains(&hint) {
        let reveal = control(g, "reveal_replace");
        assert_gap(
            ctx,
            reveal[0] + reveal[2],
            ink_span(p, g, reveal_row, &hint)[0],
            dpi,
        );
    }
}

#[test]
fn find_replace_controls_keep_shaped_air_across_worlds_widths_and_dpi() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(2400.0, 1600.0) else {
        eprintln!("skipping Find/Replace spacing law: no wgpu adapter");
        return;
    };
    let worlds: Vec<_> = crate::theme::THEMES.iter().map(|t| t.name).collect();
    assert!(
        worlds.len() >= 2 && worlds.contains(&"Saltpan"),
        "the spacing sweep must include Saltpan and neighboring worlds"
    );
    let mut row_counts = std::collections::BTreeSet::new();
    let mut cells = 0usize;
    for world in worlds {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0_f32, 2.0] {
            p.set_dpi(dpi);
            let zooms: &[f32] = if world == "Saltpan" {
                &[1.0, 2.0]
            } else {
                &[1.0]
            };
            for &zoom in zooms {
                for logical_w in [340_u32, 380, 464, 1200] {
                    let (w, h) = (
                        (logical_w as f32 * dpi).round() as u32,
                        (800.0 * dpi).round() as u32,
                    );
                    p.set_size(w as f32, h as f32);
                    let mut v = view("amber amber\n", 0, 0);
                    v.zoom = zoom;
                    v.search_active = true;
                    v.search_query = "amber".into();
                    v.search_matches = vec![((0, 0), (0, 5)), ((0, 6), (0, 11))];
                    v.search_current = Some(0);
                    v.search_replace_active = true;
                    v.search_replacement = "gold".into();
                    v.search_editing_replacement = true;
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let g = p.panel_geometry().expect("the search panel is up");
                    let ctx = format!("world={world} width={logical_w} dpi={dpi} zoom={zoom}");
                    row_counts.insert(g.rows.len());
                    assert_all_text_inside(&ctx, &p, &g);
                    assert_controls_separate(&ctx, &g, dpi);

                    assert_labels_clear_controls(&ctx, &p, &g, dpi);
                    cells += 1;
                }
            }
        }
    }
    assert!(
        row_counts.len() >= 2,
        "responsive and ordinary row plans must both be live: {row_counts:?}"
    );
    assert_eq!(cells, (crate::theme::THEMES.len() + 1) * 2 * 4);
}

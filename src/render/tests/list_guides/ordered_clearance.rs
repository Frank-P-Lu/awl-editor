//! Real painted separation between nested ordered prefixes and all guide rails.
use super::*;
use crate::render::tests::pixeldiff;

const DOC: &str = concat!(
    "- Root\n",
    "  - Branch\n",
    "    1. Parent one\n",
    "       - Detail\n",
    "    2. Sibling two\n",
    "    10. Parent ten\n",
    "        - Detail\n",
    "    11. Sibling eleven\n",
    "    99. Parent ninety-nine\n",
    "        - Detail\n",
    "    100. Parent hundred\n",
    "         - Detail\n",
    "    101. Sibling hundred\n",
    "  - [ ] Open branch\n",
    "    1. Mixed one\n",
    "       - [x] Done detail\n",
    "    2. Mixed two\n",
    "\n",
    "Outside\n",
);
type Frame = Vec<[u8; 4]>;

struct Layers {
    text: Frame,
    base: Frame,
    guides: Frame,
}

fn isolated_layers(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> Layers {
    p.prepare(device, queue, width, height).unwrap();
    p.list_guide_pipeline
        .prepare(device, queue, width, height, &[]);
    let text = pixeldiff::render_frame(p, device, queue, width, height);
    p.renderer
        .prepare(
            device,
            queue,
            &mut p.font_system,
            &mut p.atlas,
            &p.viewport,
            std::iter::empty(),
            &mut p.swash_cache,
        )
        .unwrap();
    let base = pixeldiff::render_frame(p, device, queue, width, height);
    p.prepare_list_guides(device, queue, width, height);
    let guides = pixeldiff::render_frame(p, device, queue, width, height);
    assert!(guides.iter().zip(&base).any(|(a, b)| a != b));
    Layers { text, base, guides }
}

fn marker_separation(
    p: &mut TextPipeline,
    layers: &Layers,
    canvas: [u32; 2],
    line: usize,
    dpi: f32,
) -> Option<String> {
    let item = crate::markdown::list_item(p.buffer.lines[line].text()).unwrap();
    let keys = crate::rotated_label::ink::span_key_at(
        &p.buffer,
        line,
        item.indent..item.content - 1,
        [p.text_left(), p.doc_top()],
    );
    let [left, top, width, height] =
        crate::rotated_label::ink::bounds(&mut p.font_system, &mut p.swash_cache, &keys)
            .expect("the ordered prefix paints real glyph ink");
    let [cw, ch] = canvas;
    let radius = (2.0 * dpi).ceil() as i32;
    let pad = radius + 2;
    if crate::markdown::list_item(p.buffer.lines[line + 1].text())
        .is_some_and(|next| next.indent > item.indent)
    {
        let child = p.visual_rows(line + 1)[0].clone();
        let end = (p.doc_top() + child.line_top + child.line_height) as i32;
        let center = (left + width * 0.5).round() as i32;
        let stem = ((top + height).ceil() as i32 + radius + 1..end).any(|y| {
            (center - dpi.ceil() as i32..=center + dpi.ceil() as i32).any(|x| {
                let index = y as usize * cw as usize + x as usize;
                layers.guides[index] != layers.base[index]
            })
        });
        assert!(stem, "ordered parent {line} retains a painted child stem");
    }
    let mut painted = 0;
    let mut crowded = 0;
    let mut overlaps = 0;
    let mut nearest = i32::MAX;
    for y in (top as i32 - pad).max(0)..((top + height).ceil() as i32 + pad).min(ch as i32) {
        for x in (left as i32 - pad).max(0)..((left + width).ceil() as i32 + pad).min(cw as i32) {
            let index = y as usize * cw as usize + x as usize;
            if layers.text[index] == layers.base[index] {
                continue;
            }
            painted += 1;
            let mut close = false;
            for ny in (y - radius).max(0)..=(y + radius).min(ch as i32 - 1) {
                for nx in (x - radius).max(0)..=(x + radius).min(cw as i32 - 1) {
                    let near = ny as usize * cw as usize + nx as usize;
                    if layers.guides[near] != layers.base[near] {
                        close = true;
                        let distance = (nx - x).abs().max((ny - y).abs());
                        nearest = nearest.min(distance);
                        overlaps += usize::from(distance == 0);
                    }
                }
            }
            crowded += usize::from(close);
        }
    }
    assert!(painted > 2, "a nonempty actual ordered-number pixel mask");
    (crowded > 0).then(|| {
        format!(
            "line {line}: {crowded}/{painted} numeral pixels within {radius}px of guides; \
             literal overlaps {overlaps}, nearest {nearest}px"
        )
    })
}

#[test]
fn nested_ordered_numbers_have_real_pixel_clearance_in_every_world() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = super::super::headless_dqp(1200.0, 1000.0) else {
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let ordered = DOC
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            crate::markdown::list_item(text)
                .filter(|item| item.ordered)
                .map(|_| line)
        })
        .collect::<Vec<_>>();
    assert_eq!(ordered.len(), 9);
    let mut failures = Vec::new();
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            let (width, height) = ((1200.0 * dpi) as u32, (1000.0 * dpi) as u32);
            p.set_dpi(dpi);
            p.set_size(width as f32, height as f32);
            p.set_view(&view_md(DOC, 18, 0));
            let layers = isolated_layers(&mut p, &device, &queue, width, height);
            for &line in &ordered {
                if let Some(failure) =
                    marker_separation(&mut p, &layers, [width, height], line, dpi)
                {
                    failures.push(format!("{} dpi {dpi} {failure}", world.name));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

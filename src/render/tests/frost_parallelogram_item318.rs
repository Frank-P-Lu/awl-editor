//! MEASUREMENT PROBE (item 318) — where the card's upright chrome sits relative to the
//! leaning term, so the coverage floor's remaining subject is a measured quantity rather
//! than a remembered one.

use super::super::*;
use super::dither::offscreen;
use super::{headless_dqp, view_md};

const DENSE: &str = concat!(
    "# The leaning footprint\n\n",
    "Prose is the product, and the prose is what a summoned picker draws over.\n",
    "This paragraph exists so dense glyph structure sits beside every face of the\n",
    "card at both device scales.\n\n",
    "- a list row with several short words in it\n",
    "- another list row, similar in shape\n",
);

fn theme_picker(text: &str) -> ViewState {
    let mut v = view_md(text, 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
    v.overlay_sections = vec![String::new(); v.overlay_items.len()];
    v.overlay_selected = 11;
    v.overlay_title = "themes";
    v.overlay_hint = "type to filter   ↵ keep   esc revert".to_string();
    v
}

fn enrolled_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| crate::render::blur::footprint_frost_applies(t.render_caps.list_style))
        .map(|t| t.name)
        .collect()
}

/// The LEANING term alone — the parallelogram, without the upright coverage floor beside
/// it. Positive is outside. This is what the floor's remaining subject is measured
/// against.
fn leaning_dist(rect: [f32; 4], shear: f32, px: f32, py: f32) -> f32 {
    let [x, y, w, h] = rect;
    let gy = (y - py).max(py - (y + h));
    let sx = px - shear * (py - (y + h * 0.5));
    (x - sx).max(sx - (x + w)).max(gy)
}

#[test]
fn measure_where_the_upright_chrome_sits_against_the_leaning_term() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    for world in enrolled_worlds() {
        for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping: no wgpu adapter");
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);
            p.set_view(&theme_picker(DENSE));
            p.prepare(&device, &queue, w, h).unwrap();
            let mut encoder = device.create_command_encoder(&Default::default());
            let (_t, tview) = offscreen(&device, w, h);
            p.render(&mut encoder, &tview).unwrap();
            queue.submit(Some(encoder.finish()));

            let rect = p.overlay_card_rect().expect("a card");
            let shear = match p.frost_mode() {
                Some(crate::render::blur::Frost::Footprint(f)) => f.shear,
                other => panic!("{world}: expected the footprint arm, got {other:?}"),
            };
            let geom = p.overlay_geometry(w);
            let plan = p.overlay_row_plan(&geom);
            let [cx, cy, cw, ch] = rect;
            eprintln!(
                "\n=== {world} @ {dpi}x ({w}x{h}) === card [{cx:.1},{cy:.1},{cw:.1},{ch:.1}] \
                 shear {shear:.5} text_left {:.1} text_w {:.1} text_top {:.1}",
                geom.text_left, geom.text_w, geom.text_top
            );

            // THE QUERY BAND: its planned box, and the shaped ink on its own line.
            match plan.query_band() {
                None => eprintln!("  QUERY: none planned"),
                Some(qb) => {
                    let ink_w = p
                        .panel_buffer
                        .layout_runs()
                        .find_map(|r| (r.line_i == qb.line).then_some(r.line_w))
                        .unwrap_or(0.0);
                    let (l, r) = (geom.text_left, geom.text_left + ink_w);
                    let (t, b) = (qb.top, qb.bottom());
                    let corners = [(l, t), (r, t), (l, b), (r, b)];
                    let worst = corners
                        .iter()
                        .map(|&(px, py)| leaning_dist(rect, shear, px, py))
                        .fold(f32::NEG_INFINITY, f32::max);
                    eprintln!(
                        "  QUERY line {} box y [{t:.1}..{b:.1}] ink [{l:.1}..{r:.1}] w {ink_w:.1}\
                         \n    above card top? {} (qb.top {t:.1} vs card_y {cy:.1})\
                         \n    worst corner vs LEANING term: {worst:.2} physical \
                         ({:.2} logical)  [>0 = OUTSIDE the parallelogram]",
                        qb.line,
                        b <= cy,
                        worst / dpi
                    );
                }
            }

            // THE FOOT BAND, as item 313 seats it.
            match p.overlay_foot_placement(&geom, &plan) {
                None => eprintln!("  FOOT: inert (no placement)"),
                Some(f) => {
                    let (l, r) = (f.left, f.left + f.ink_w.max(0.0));
                    let half = plan.lh() * 0.5;
                    let (t, b) = (f.center_y - half, f.center_y + half);
                    let corners = [(l, t), (r, t), (l, b), (r, b)];
                    let worst = corners
                        .iter()
                        .map(|&(px, py)| leaning_dist(rect, shear, px, py))
                        .fold(f32::NEG_INFINITY, f32::max);
                    eprintln!(
                        "  FOOT left {l:.1} ink_w {:.1} center_y {:.1} clamped {} steps {:.2}\
                         \n    worst corner vs LEANING term: {worst:.2} physical \
                         ({:.2} logical)  [>0 = OUTSIDE]",
                        f.ink_w,
                        f.center_y,
                        f.clamped,
                        f.steps,
                        worst / dpi
                    );
                }
            }

            // THE ROW CLUSTER, for reference: the rows are the leaning term's subject.
            if let (Some(first), Some(last)) = (plan.rows().first(), plan.rows().last()) {
                eprintln!(
                    "  ROWS {} planned, first top {:.1} last bottom {:.1}, band_bottom {:.1}, \
                     first_top {:.1}",
                    plan.rows().len(),
                    first.top,
                    last.bottom(),
                    plan.band_bottom(),
                    plan.first_top()
                );
            }
        }
    }
    crate::theme::set_active(entry);
}

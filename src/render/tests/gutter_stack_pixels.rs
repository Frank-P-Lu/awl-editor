//! REAL PIXELS FOR THE MARGIN WORKING SET — is the file you are editing
//! actually distinguishable from the files you are not, on every world?
//!
//! The sidecar cannot answer this. It reports the block's text and the block's
//! state, and it once reported a selected row that rendered fully invisible; a
//! stack whose active row and dimmed siblings ended up the same ink would
//! satisfy every state assertion in this repo. So this renders the real frame
//! and does arithmetic on it.
//!
//! **Every assertion here is a RATIO OF TWO QUANTITIES READ FROM ONE FRAME**,
//! never a distance to an authored constant. A backend that rounds differently
//! moves both terms together, which is what keeps the law true on a GPU it was
//! never calibrated on.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};
use crate::workingset::StackRow;

const W: u32 = 1600;
const H: u32 = 800;

fn stack_view(active: usize) -> ViewState {
    let mut v = view(
        "# A document\n\nSome prose to give the page a body.\n",
        0,
        0,
    );
    v.zoom = 1.0;
    v.gutter_name = "field-notes.md".to_string();
    v.gutter_project = "notes".to_string();
    v.gutter_files = [
        ("opening.md", ""),
        ("field-notes.md", "journal/"),
        ("ledger.md", ""),
    ]
    .into_iter()
    .enumerate()
    .map(|(at, (leaf, parent))| StackRow {
        leaf: leaf.to_string(),
        parent: parent.to_string(),
        active: at == active,
    })
    .collect();
    v
}

fn render_frame(device: &wgpu::Device, queue: &wgpu::Queue, p: &mut TextPipeline) -> Vec<[u8; 4]> {
    p.prepare(device, queue, W, H).unwrap();
    let (texture, tview) = offscreen(device, W, H);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl gutter-stack encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, W, H)
}

fn dist(a: [u8; 4], b: [u8; 4]) -> f32 {
    let d = |i: usize| a[i] as f32 - b[i] as f32;
    (d(0) * d(0) + d(1) * d(1) + d(2) * d(2)).sqrt()
}

/// THE DRAWN ROWS' BANDS, derived from the block's OWN frost seeds rather than
/// from a second call to the row planner.
///
/// [`TextPipeline::gutter_frost_seeds`] emits `[x0, x1, yc, r]` per run of ink,
/// hugging the real glyphs of every line the block draws — so its distinct `yc`
/// values ARE the drawn rows, in drawn order, and their spacing is the row
/// pitch. Reading the geometry back out of a shipped production door keeps this
/// law honest about what the frame contains: if the stack stopped drawing rows,
/// the bands would vanish with them rather than being recomputed from a formula
/// that is still true of a block nobody drew.
fn row_bands(seeds: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let mut centres: Vec<f32> = Vec::new();
    for s in seeds {
        if !centres.iter().any(|c| (c - s[2]).abs() < 0.5) {
            centres.push(s[2]);
        }
    }
    centres.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let right = seeds.iter().fold(0.0_f32, |m, s| m.max(s[1]));
    let pitch = centres
        .windows(2)
        .map(|w| w[1] - w[0])
        .fold(f32::MAX, f32::min);
    centres
        .into_iter()
        .map(|yc| [0.0, yc - pitch * 0.5, right, pitch])
        .collect()
}

/// HOW MUCH A ROW DIFFERS FROM BARE MARGIN, averaged over the last few
/// characters of the row.
///
/// Two decisions, both about not measuring the wrong thing. It is a MEAN rather
/// than an extreme because the question is what the row WEIGHS — a plate behind
/// quiet ink and a brighter ink are both ways a row comes forward, and a single
/// brightest pixel sees only the second, compressing a plated row and an
/// unplated one into nearly the same reading. And it is measured over a
/// fixed-width segment at the row's RIGHT EDGE, where every row has ink because
/// every row is right-aligned to the same box: averaging across the whole band
/// would make the answer a function of how long each filename happens to be.
fn ink_weight(px: &[[u8; 4]], band: [f32; 4], ground: [u8; 4]) -> f32 {
    let right = band[0] + band[2];
    let x0 = (right - band[3] * 3.0).max(0.0) as u32;
    let x1 = (right as u32).min(W);
    let y0 = band[1].max(0.0) as u32;
    let y1 = ((band[1] + band[3]) as u32).min(H);
    let mut total = 0.0;
    let mut n = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            total += dist(px[(y * W + x) as usize], ground);
            n += 1.0;
        }
    }
    if n == 0.0 { 0.0 } else { total / n }
}

/// **THE ACTIVE ROW COMES FORWARD AND THE SIBLINGS ARE STILL THERE — on every
/// world in the roster, with the enrolment derived from the roster itself.**
///
/// Two bounds on ONE ratio, and the second is the half that matters. A law that
/// only asked the active row to out-read its siblings gets *happier* the further
/// those siblings fade: a stack washed to within a byte of the page would report
/// the best score this law can produce, while photographing a margin with one
/// file in it. So the same ratio carries a FLOOR, calibrated against the quietest
/// ink the shipped block already spends — the project line, which is drawn in
/// exactly the `faint` the dimmed rows wear.
///
/// Both terms are read off one frame, so a backend that rounds `faint` a byte
/// differently moves the numerator and the denominator together.
#[test]
fn the_active_row_reads_forward_of_dimmed_siblings_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping the_active_row_reads_forward_of_dimmed_siblings_on_every_world: no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;
        // The ACTIVE row is the middle one on purpose: a plate or an ink rule
        // pinned to the first or last slot passes a single-row-position test.
        let v = stack_view(1);
        p.set_view(&v);
        let layout_rows = row_bands(&p.gutter_frost_seeds(H));
        assert_eq!(
            layout_rows.len(),
            4,
            "{world:?}: expected three file rows over the project line, got {}",
            layout_rows.len()
        );
        let px = render_frame(&device, &queue, &mut p);
        // The margin's own ground, sampled well ABOVE the block in the same
        // column of margin the block occupies.
        let band0 = layout_rows[0];
        let ground_y = (band0[1] - band0[3] * 3.0).max(0.0) as u32;
        let ground = px[(ground_y * W + (band0[2] * 0.5) as u32) as usize];

        // rows: [file0, file1(active), file2, project]
        let active = ink_weight(&px, layout_rows[1], ground);
        let sibling =
            ink_weight(&px, layout_rows[0], ground).max(ink_weight(&px, layout_rows[2], ground));
        let project = ink_weight(&px, layout_rows[3], ground);
        assert!(
            project > 0.0,
            "{world:?}: the project line drew no ink at all — the fixture never reached the margin"
        );
        let forward = active / sibling.max(1.0);
        let presence = sibling / project.max(1.0);
        assert!(
            forward >= 1.15,
            "{world:?}: the active row reads at {active:.1} against siblings at {sibling:.1} \
             (ratio {forward:.3}) — the file being edited is not distinguishable from the rest"
        );
        assert!(
            presence >= 0.5,
            "{world:?}: dimmed rows read at {sibling:.1} against the block's own quietest \
             shipped ink at {project:.1} (ratio {presence:.3}) — the stack has faded into the page"
        );
        judged.push(world);
    }
    assert_eq!(
        judged.len(),
        theme::THEMES.len(),
        "only {} of {} worlds were judged: {judged:?}",
        judged.len(),
        theme::THEMES.len()
    );
}

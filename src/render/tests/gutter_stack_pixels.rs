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

/// `pub(super)`: reused by `one_bit.rs`'s active-row-on-its-own-plate law, so
/// the two files drive the SAME three-file fixture rather than each keeping a
/// copy that can drift apart.
pub(super) fn stack_view(active: usize) -> ViewState {
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
        kind: crate::workingset::StackRowKind::File,
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
pub(super) fn row_bands(seeds: &[[f32; 4]]) -> Vec<[f32; 4]> {
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
            "skipping the_active_row_reads_forward_of_dimmed_siblings_on_every_world: \
             no wgpu adapter"
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
            "{world:?}: expected the project heading over three file rows, got {}",
            layout_rows.len()
        );
        let px = render_frame(&device, &queue, &mut p);
        // The margin's own ground, sampled well ABOVE the block in the same
        // column of margin the block occupies.
        let band0 = layout_rows[0];
        let ground_y = (band0[1] - band0[3] * 3.0).max(0.0) as u32;
        let ground = px[(ground_y * W + (band0[2] * 0.5) as u32) as usize];

        // rows: [project, file0, file1(active), file2]
        let active = ink_weight(&px, layout_rows[2], ground);
        let sibling =
            ink_weight(&px, layout_rows[1], ground).max(ink_weight(&px, layout_rows[3], ground));
        let project = ink_weight(&px, layout_rows[0], ground);
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

/// Groups raw frost seeds into rows by `yc`, the same clustering
/// [`row_bands`] does, but keeps each row's OWN ink extent instead of folding
/// every row to the block's single widest one. Position alone cannot tell
/// WHICH row's content sits at a given rank — two same-height rows that
/// swapped which text they draw are geometrically identical to `row_bands` —
/// so this reads the row's WIDTH as its content fingerprint: `"notes"` and
/// `"opening.md"` are different lengths, and this test keeps them that way at
/// every N, so a swap between them is a swap in this list too.
fn row_ink_widths(seeds: &[[f32; 4]]) -> Vec<f32> {
    let mut clusters: Vec<(f32, f32, f32)> = Vec::new();
    for s in seeds {
        match clusters.iter_mut().find(|c| (c.0 - s[2]).abs() < 0.5) {
            Some(c) => {
                c.1 = c.1.min(s[0]);
                c.2 = c.2.max(s[1]);
            }
            None => clusters.push((s[2], s[0], s[1])),
        }
    }
    clusters.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    clusters.into_iter().map(|(_, x0, x1)| x1 - x0).collect()
}

/// **N=1 → N=2 → N=3 IS PURE ROW INSERTION: the folder heading and the file
/// already on screen keep the same RANK and the same CONTENT at that rank as
/// the block widens — a new row only ever appends below, never between or
/// above, and never by swapping what an existing rank draws.**
///
/// The block is BOTTOM-anchored (`plan::plan_gutter_stack`'s `top = canvas_h -
/// block_h - bottom_inset`), so a grown block's rows all shift up in absolute
/// canvas Y as a unit — that shift is unrelated to this law and would happen
/// even if a row were inserted correctly, so position alone is graded
/// RELATIVELY (gap and shift-delta between the two known ranks), never
/// against an absolute Y. And position alone is not enough on its own: two
/// adjacent same-height rows that swapped content — the exact bug this
/// block's ordering was fixed for — look identical to a position-only check,
/// since both rows still occupy rank 0 and rank 1. So this also fingerprints
/// each rank by its own ink WIDTH ([`row_ink_widths`]), keeping the heading
/// (`"notes"`, short) and the pre-existing row (`"opening.md"`, long) fixed
/// strings at every N specifically so a swap changes which width lands at
/// which rank. `gutter_stack::tests::project_heads_only_the_multi_file_hierarchy`
/// pins the same claim in pure data; this asks it of the real drawn geometry,
/// off the same production door ([`TextPipeline::gutter_frost_seeds`]) the law
/// above this one already trusts. Swept over the whole theme roster because
/// the row planner reads theme-independent geometry, and this PROVES that
/// rather than assuming it.
#[test]
fn opening_a_second_file_inserts_a_row_without_moving_the_first_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping opening_a_second_file_inserts_a_row_without_moving_the_first_on_every_world: \
             no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();

    // `roster[0]` is the file already open at N=1; each larger N opens the
    // NEXT roster member on top, so the newest file is always the active one
    // and `roster[0]`'s own row never has a reason to reorder. The project
    // name is deliberately much shorter than every roster member, so the two
    // known ranks (heading, pre-existing row) never have coincidentally equal
    // ink widths.
    let roster = ["opening.md", "second.md", "third.md"];
    let view_for = |n: usize| -> ViewState {
        let mut v = view(
            "# A document\n\nSome prose to give the page a body.\n",
            0,
            0,
        );
        v.zoom = 1.0;
        v.gutter_project = "notes".to_string();
        if n == 1 {
            // The one-file shape: no working set at all, just the identity line.
            v.gutter_name = roster[0].to_string();
        } else {
            v.gutter_name = roster[n - 1].to_string();
            v.gutter_files = roster[..n]
                .iter()
                .enumerate()
                .map(|(at, leaf)| StackRow {
                    leaf: leaf.to_string(),
                    parent: String::new(),
                    active: at == n - 1,
                    kind: crate::workingset::StackRowKind::File,
                })
                .collect();
        }
        v
    };

    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;

        let seeds_by_n: Vec<Vec<[f32; 4]>> = [1usize, 2, 3]
            .into_iter()
            .map(|n| {
                p.set_view(&view_for(n));
                let seeds = p.gutter_frost_seeds(H);
                assert_eq!(
                    row_bands(&seeds).len(),
                    n + 1,
                    "{world:?}: N={n} must draw the folder heading over {n} identity row(s)"
                );
                seeds
            })
            .collect();
        let bands_by_n: Vec<Vec<[f32; 4]>> = seeds_by_n.iter().map(|s| row_bands(s)).collect();
        let widths_by_n: Vec<Vec<f32>> = seeds_by_n.iter().map(|s| row_ink_widths(s)).collect();
        assert_pure_row_insertion(world, &bands_by_n, &widths_by_n, roster[0]);
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

/// The RANK/CONTENT/POSITION invariants one world's N=1/2/3 fixtures must all
/// satisfy — split out from the sweep above purely to keep that loop body
/// short; see its own doc for what each assertion proves and why.
fn assert_pure_row_insertion(
    world: &str,
    bands_by_n: &[Vec<[f32; 4]>],
    widths_by_n: &[Vec<f32>],
    pre_existing: &str,
) {
    // The two known ranks read meaningfully different widths — the fixture's
    // own precondition, so a passing law below is discriminating content and
    // not just comparing two numbers that happen to agree.
    assert!(
        widths_by_n[0][1] > widths_by_n[0][0] * 1.3,
        "{world:?}: the fixture's rank-1 row ({:.1}px, {pre_existing:?}) is not meaningfully \
         wider than rank-0 ({:.1}px, \"notes\") — a swap between them would go undetected",
        widths_by_n[0][1],
        widths_by_n[0][0]
    );

    for n in [1, 2] {
        // RANK 0 stays the folder heading's own width, and RANK 1 stays the
        // pre-existing row's own width — a swap would show up here as rank 0
        // suddenly reading the wide row's width (or vice versa).
        for rank in [0usize, 1] {
            let base = widths_by_n[0][rank];
            let wider = widths_by_n[n][rank];
            assert!(
                (base - wider).abs() < 0.5,
                "{world:?}: rank {rank}'s ink width was {base:.1}px at N=1 and {wider:.1}px at \
                 N={} — a different row's content landed at this rank",
                n + 1
            );
        }
        // RELATIVE position: the gap between the two known ranks stays one
        // row pitch, and both ranks shift by the SAME delta as the block
        // widens (a rigid shift the bottom anchor causes, never a reflow of
        // one rank independent of the other).
        let gap = |bands: &[[f32; 4]]| bands[1][1] - bands[0][1];
        assert!(
            (gap(&bands_by_n[0]) - gap(&bands_by_n[n])).abs() < 0.01,
            "{world:?}: the gap between rank 0 and rank 1 was {} at N=1 and {} at N={} — a row \
             was inserted between them instead of appended below",
            gap(&bands_by_n[0]),
            gap(&bands_by_n[n]),
            n + 1
        );
        let heading_delta = bands_by_n[0][0][1] - bands_by_n[n][0][1];
        let identity_delta = bands_by_n[0][1][1] - bands_by_n[n][1][1];
        assert!(
            (heading_delta - identity_delta).abs() < 0.01,
            "{world:?}: rank 0 shifted by {heading_delta} but rank 1 shifted by \
             {identity_delta} going from N=1 to N={} — the pair no longer moves as a rigid unit",
            n + 1
        );
    }
}

/// The right edge of the row `TextPipeline::gutter_stack_hit` accepts at
/// `y` — found by asking that EXACT production door rather than re-deriving
/// `avail`/pad arithmetic by hand, so this cannot drift from what a real
/// pointer would land on. Scans downward from `upper` because the row's
/// frost-seed skirt overshoots `avail` by its own pad, which makes the ink
/// geometry an unreliable proxy for the hit-tested box.
fn find_row_right_edge(p: &TextPipeline, upper: f32, y: f32, h: u32) -> f32 {
    let mut x = upper;
    while x > 0.0 && p.gutter_stack_hit(x, y, h).is_none() {
        x -= 1.0;
    }
    x
}

/// **THE SINGLE-FILE ROW'S × MARK ACTUALLY REPAINTS ON HOVER — REAL PIXELS,
/// NOT JUST A HIT-TEST ANSWER.**
///
/// Wagtail's own tripwire (CLAUDE.md) is exactly the failure mode this closes:
/// a sidecar/geometry law can report `selected_index` (here, a hit resolving
/// to `row: 0` with `is_close() == true`) while the thing it names never
/// became visible pixels. `gutter_hit::tests` already proves the GEOMETRY
/// resolves; this proves the RENDER actually reveals — off the same
/// `render_frame`/`dist` doors the active-row law above uses — and that the
/// label's own ink stays untouched (the stack's own hover law: a reveal
/// changes ink only, never advances the shaped label).
#[test]
fn the_lone_row_close_mark_reveals_on_real_pixels_only_over_the_hovered_zone() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping the_lone_row_close_mark_reveals_on_real_pixels_only_over_the_hovered_zone: \
             no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    theme::set_active_by_name("Saltpan").expect("Saltpan is in the world roster");

    let mut v = view(
        "# A document\n\nSome prose to give the page a body.\n",
        0,
        0,
    );
    v.zoom = 1.0;
    v.gutter_project = "notes".to_string();
    v.gutter_name = "opening.md".to_string();
    p.set_view(&v);

    let bands = row_bands(&p.gutter_frost_seeds(H));
    assert_eq!(
        bands.len(),
        2,
        "N=1 must draw the folder heading over the identity line"
    );
    let identity = bands[1];
    let row_h = identity[3];
    let y = identity[1] + row_h * 0.5;
    let right_edge = find_row_right_edge(&p, W as f32 - 1.0, y, H);
    assert!(
        right_edge > row_h,
        "could not locate the identity row's own right edge via hit-test (got {right_edge})"
    );
    let close_x = right_edge - 0.5;
    let switch_x = (right_edge * 0.3).max(2.0);

    let switch_hit = p
        .gutter_stack_hit(switch_x, y, H)
        .expect("the switch probe must enrol");
    let close_hit = p
        .gutter_stack_hit(close_x, y, H)
        .expect("the close probe must enrol");
    assert!(
        !switch_hit.is_close(),
        "fixture bug: the switch probe landed inside the close zone"
    );
    assert!(
        close_hit.is_close(),
        "fixture bug: the close probe missed the close zone"
    );

    p.clear_gutter_stack_hover();
    let resting = render_frame(&device, &queue, &mut p);
    let changed = p.resolve_gutter_stack_hover(close_x, y, H);
    assert!(
        changed,
        "hovering the close zone must change the hover state"
    );
    let hovered = render_frame(&device, &queue, &mut p);

    // The mark's own lane: `row_h` wide, hugging the row's right edge — the
    // exact close-zone geometry `gutter_stack::CLOSE_ZONE_ROWS` reserves.
    let mark_x0 = (right_edge - row_h).max(0.0) as u32;
    let mark_x1 = (right_edge as u32).min(W);
    let y0 = identity[1].max(0.0) as u32;
    let y1 = ((identity[1] + row_h) as u32).min(H);
    let mut mark_diff = 0u32;
    for yy in y0..y1 {
        for xx in mark_x0..mark_x1 {
            let idx = (yy * W + xx) as usize;
            if dist(resting[idx], hovered[idx]) > 4.0 {
                mark_diff += 1;
            }
        }
    }
    assert!(
        mark_diff > 0,
        "hovering the close zone painted no pixels in the mark's own lane — the × never revealed"
    );

    // The label's own ink, well clear of the mark's lane, stays byte-identical:
    // the reveal is a color-only change over an already-shaped run, never a
    // reflow of the filename.
    let label_x1 = mark_x0.saturating_sub(2);
    let mut label_diff = 0u32;
    for yy in y0..y1 {
        for xx in 0..label_x1 {
            let idx = (yy * W + xx) as usize;
            if dist(resting[idx], hovered[idx]) > 4.0 {
                label_diff += 1;
            }
        }
    }
    assert_eq!(
        label_diff, 0,
        "hovering the close zone repainted {label_diff} pixels of the label's own ink"
    );
}

/// **524: THE SINGLE-FILE IDENTITY LINE'S RIGHT EDGE IS THE SAME X A
/// WORKING-SET ROW'S OWN RIGHT EDGE SITS AT** — real pixels, not just the
/// shared constant both budgets subtract (`gutter_stack::CLOSE_MARK_TEXT`).
/// The close lane's own reservation is a uniform right edge only if opening
/// a second file never shifts where that edge actually falls; this proves it
/// with the SAME hit-tested door `find_row_right_edge` already reads for the
/// lone row above, rather than re-deriving the column/pad arithmetic by hand.
#[test]
fn the_lone_identity_lines_right_edge_matches_a_stack_rows_own_right_edge() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping the_lone_identity_lines_right_edge_matches_a_stack_rows_own_right_edge: \
             no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    theme::set_active_by_name("Saltpan").expect("Saltpan is in the world roster");

    let mut lone = view(
        "# A document\n\nSome prose to give the page a body.\n",
        0,
        0,
    );
    lone.zoom = 1.0;
    lone.gutter_project = "notes".to_string();
    lone.gutter_name = "opening.md".to_string();
    p.set_view(&lone);
    let lone_bands = row_bands(&p.gutter_frost_seeds(H));
    assert_eq!(
        lone_bands.len(),
        2,
        "N=1 must draw the folder heading over the identity line"
    );
    let lone_identity = lone_bands[1];
    let lone_edge = find_row_right_edge(
        &p,
        W as f32 - 1.0,
        lone_identity[1] + lone_identity[3] * 0.5,
        H,
    );

    let stack = stack_view(0);
    p.set_view(&stack);
    let stack_bands = row_bands(&p.gutter_frost_seeds(H));
    assert_eq!(
        stack_bands.len(),
        4,
        "N=3 must draw the folder heading over three file rows"
    );
    let first_row = stack_bands[1];
    let stack_edge = find_row_right_edge(&p, W as f32 - 1.0, first_row[1] + first_row[3] * 0.5, H);

    assert!(
        lone_edge > lone_identity[3] && stack_edge > first_row[3],
        "could not locate a real right edge for both shapes (lone={lone_edge} stack={stack_edge})"
    );
    assert!(
        (lone_edge - stack_edge).abs() < 1.0,
        "the identity line's own right edge ({lone_edge}) does not match a stack row's \
         ({stack_edge}) — opening a second file must never shift where the lane sits"
    );
}

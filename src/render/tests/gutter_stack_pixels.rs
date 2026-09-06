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

/// Scan LEFTWARD from `right_edge` for the row's own close zone through the
/// EXACT production hit-test, never a hand-derived offset: the mark now sits
/// wherever the row's own ink leaves it (`close_zone`'s own doc), so a fixed
/// offset from `right_edge` would drift from the truth on every name length
/// but one. Returns `(lo, hi)`, the zone's own x-range, or `None` if no
/// close point was ever found scanning down to the row's own left edge.
fn find_close_zone(p: &TextPipeline, right_edge: f32, y: f32, h: u32) -> Option<(f32, f32)> {
    let mut x = right_edge;
    while x > 0.0
        && !p
            .gutter_stack_hit(x, y, h)
            .is_some_and(|hit| hit.is_close())
    {
        x -= 1.0;
    }
    if x <= 0.0 {
        return None;
    }
    let hi = x;
    while x > 0.0
        && p.gutter_stack_hit(x, y, h)
            .is_some_and(|hit| hit.is_close())
    {
        x -= 1.0;
    }
    Some((x + 1.0, hi))
}

/// **THE SINGLE-FILE ROW'S × MARK ACTUALLY REPAINTS ON HOVER, AT ITS OWN
/// LEADING EDGE — REAL PIXELS, NOT JUST A HIT-TEST ANSWER.**
///
/// Wagtail's own tripwire (CLAUDE.md) is exactly the failure mode this closes:
/// a sidecar/geometry law can report `selected_index` (here, a hit resolving
/// to `row: 0` with `is_close() == true`) while the thing it names never
/// became visible pixels. `gutter_hit::tests` already proves the GEOMETRY
/// resolves; this proves the RENDER actually reveals — off the same
/// `render_frame`/`dist` doors the active-row law above uses — and that the
/// label's own ink stays untouched (the stack's own hover law: a reveal
/// changes ink only, never advances the shaped label).
///
/// **LAW 2 (reveal changes ink only) and LAW 3 (hit-zone/ink agreement), at
/// real pixels.** Swept over TWO name lengths — short and near the margin's
/// own budget — because the leading mark's own position MOVES with the
/// name (unlike the trailing design's fixed right edge), so a law that only
/// ever probed one length could pass while the zone silently drifted from
/// the ink at every other one.
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

    for name in [
        "opening.md",
        "a-name-long-enough-to-spend-most-of-the-marginss-budget.md",
    ] {
        let mut v = view(
            "# A document\n\nSome prose to give the page a body.\n",
            0,
            0,
        );
        v.zoom = 1.0;
        v.gutter_project = "notes".to_string();
        v.gutter_name = name.to_string();
        p.set_view(&v);

        let bands = row_bands(&p.gutter_frost_seeds(H));
        assert_eq!(
            bands.len(),
            2,
            "name={name:?}: N=1 must draw the folder heading over the identity line"
        );
        let identity = bands[1];
        let row_h = identity[3];
        let y = identity[1] + row_h * 0.5;
        let right_edge = find_row_right_edge(&p, W as f32 - 1.0, y, H);
        assert!(
            right_edge > row_h,
            "name={name:?}: could not locate the identity row's own right edge via \
             hit-test (got {right_edge})"
        );
        let (zone_lo, zone_hi) = find_close_zone(&p, right_edge, y, H).unwrap_or_else(|| {
            panic!("name={name:?}: no close point found scanning the whole row")
        });
        let close_x = (zone_lo + zone_hi) * 0.5;
        // Switch territory now sits BETWEEN the zone and the row's own right
        // edge — the name's own ink — the inverse of the trailing design's
        // own switch/close split.
        let switch_x = (zone_hi + right_edge) * 0.5;

        let switch_hit = p
            .gutter_stack_hit(switch_x, y, H)
            .unwrap_or_else(|| panic!("name={name:?}: the switch probe must enrol"));
        assert!(
            !switch_hit.is_close(),
            "name={name:?}: fixture bug: the switch probe landed inside the close zone"
        );

        p.clear_gutter_stack_hover();
        let resting = render_frame(&device, &queue, &mut p);
        let changed = p.resolve_gutter_stack_hover(close_x, y, H);
        assert!(
            changed,
            "name={name:?}: hovering the close zone must change the hover state"
        );
        let hovered = render_frame(&device, &queue, &mut p);

        // The mark's own lane, padded either side of the hit-tested zone: a
        // char-count estimate (`stack_hit_from_plan`'s own doc) and the real
        // shaped glyph agree closely but not to the pixel on a proportional
        // face, so the pad clears that estimate/shaping slop and any
        // antialiasing at the mark's own edges — a few px, not the tens of
        // px a genuinely wrong lane would show.
        const MARK_PAD_PX: f32 = 6.0;
        let mark_x0 = (zone_lo - MARK_PAD_PX).max(0.0) as u32;
        let mark_x1 = ((zone_hi + MARK_PAD_PX) as u32).min(W);
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
            "name={name:?}: hovering the close zone painted no pixels in the mark's own \
             lane — the × never revealed"
        );

        // Everything ELSE on the row — the ragged margin left of the mark
        // AND the name's own ink right of it — stays byte-identical: the
        // reveal is a color-only change over the mark's own already-shaped
        // run, never a reflow of anything else on the line.
        let mut label_diff = 0u32;
        for yy in y0..y1 {
            for xx in (0..mark_x0).chain(mark_x1..W) {
                let idx = (yy * W + xx) as usize;
                if dist(resting[idx], hovered[idx]) > 4.0 {
                    label_diff += 1;
                }
            }
        }
        assert_eq!(
            label_diff, 0,
            "name={name:?}: hovering the close zone repainted {label_diff} pixels outside \
             the mark's own lane"
        );
    }
}

/// **LAW 1 (flush-right alignment), at real pixels, on a MAXIMAL-width
/// name** — the align-clamp risk `prepare_gutter`'s own box-widen/shift
/// comment names: cosmic-text clamps its own right-align offset at zero
/// rather than overflowing negative, so a name spending its entire budget is
/// the ONE case that would have shown the whole line shoved right of `avail`
/// had the box not been widened by the leading mark's own reserved width.
/// Proved as a RATIO the way this whole file's own header promises: the
/// distance from the row's own rightmost ink pixel to the hit-tested right
/// edge, against the row's own height — a name whose ink stops one whole
/// mark-width short of the edge (the bug this item exists to close) fails
/// this floor on every DPI/backend the same way, while sub-pixel font
/// metrics never approach it.
#[test]
fn a_maximal_width_names_own_ink_reaches_the_stacks_flush_right_edge() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping a_maximal_width_names_own_ink_reaches_the_stacks_flush_right_edge: \
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
    // Long enough to spend the identity line's entire budget on the label
    // alone (`rowlayout::GUTTER_MIN_NAME_CHARS`-and-up margins all elide
    // something this long) — the maximal-width case.
    v.gutter_name =
        "a-genuinely-very-long-filename-that-must-spend-the-entire-margin-budget.md".to_string();
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

    let pixels = render_frame(&device, &queue, &mut p);
    let y0 = identity[1].max(0.0) as u32;
    let y1 = ((identity[1] + row_h) as u32).min(H);
    // Background reference sampled PER COLUMN, from the canvas's own bottom
    // padding below the block (`CANVAS_INSET`'s own blank strip) — the SAME
    // x as the column under test, a DIFFERENT y outside any row. A single
    // corner sample crosses the gutter's own lava-carve boundary at `avail`
    // (`gutter_carve_rect`'s `[0, avail]` span), which reads as ink on its
    // own and made an earlier cut of this law pass under every mutation
    // tried against it; same-x/different-y stays inside the SAME carved (or
    // uncarved) ground the row itself sits on.
    let bg_y = H.saturating_sub(2);
    let scan_hi = (right_edge as u32).min(W - 1);
    let scan_lo = scan_hi.saturating_sub(row_h as u32 * 3);
    let mut ink_edge = None;
    for x in (scan_lo..=scan_hi).rev() {
        let bg = pixels[(bg_y * W + x) as usize];
        let hit = (y0..y1).any(|yy| dist(pixels[(yy * W + x) as usize], bg) > 12.0);
        if hit {
            ink_edge = Some(x as f32);
            break;
        }
    }
    let ink_edge =
        ink_edge.unwrap_or_else(|| panic!("no ink found scanning back from the right edge at all"));
    let gap = right_edge - ink_edge;
    // A tight floor, well under a mark-width's worth of pixels (the
    // trailing design's own bug: a gap of roughly `CLOSE_MARK_TEXT`'s own
    // shaped width, tens of px at LABEL scale) — wide enough only to clear
    // real font antialiasing/hinting slop at the exact edge column, a few px
    // at most.
    const FLUSH_TOLERANCE_PX: f32 = 8.0;
    assert!(
        gap < FLUSH_TOLERANCE_PX,
        "the maximal name's own rightmost ink ({ink_edge}) sits {gap}px short of the row's \
         flush-right edge ({right_edge}, row height {row_h}) — the trailing mark's own \
         reserved width is leaking back into the visible label"
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

/// The two cells "the file you are editing is marked" has to hold in: the
/// lone open file, and a file freshly opened among several. `active` names
/// which slot of `n` is the current buffer; at `n == 1` there is no working
/// set at all and the block draws the bare identity line.
fn active_file_view(n: usize, active: usize) -> ViewState {
    let roster = ["opening.md", "second.md", "third.md"];
    let mut v = view(
        "# A document\n\nSome prose to give the page a body.\n",
        0,
        0,
    );
    v.zoom = 1.0;
    v.gutter_project = "notes".to_string();
    v.gutter_name = roster[active].to_string();
    if n > 1 {
        v.gutter_files = roster[..n]
            .iter()
            .enumerate()
            .map(|(at, leaf)| StackRow {
                leaf: leaf.to_string(),
                parent: String::new(),
                active: at == active,
                kind: crate::workingset::StackRowKind::File,
            })
            .collect();
    }
    v
}

/// The interior colour a plate is mostly made of: the MEDIAN interior pixel by
/// luminance. The label's ink is a minority of a plate's area at every name
/// length the margin can hold, so the median lands on flat fill — which is
/// what makes "how far is the extreme from the fill" a question about ink
/// rather than about which pixel happened to be sampled.
fn plate_fill_and_ink(px: &[[u8; 4]], plate: [f32; 4]) -> ([u8; 4], f32, usize) {
    // Inset past the rounded corner's own antialiasing, so every sample is
    // flat fill or real ink — never an edge blend against the margin beyond
    // (the failure `one_bit.rs`'s own plate law records for a hand-estimated
    // span).
    const INSET: f32 = 5.0;
    let [x, y, w, h] = plate;
    let x0 = ((x + INSET) as i64).clamp(0, W as i64 - 1);
    let x1 = ((x + w - INSET) as i64).clamp(x0 + 1, W as i64);
    let y0 = ((y + INSET) as i64).clamp(0, H as i64 - 1);
    let y1 = ((y + h - INSET) as i64).clamp(y0 + 1, H as i64);
    let mut samples: Vec<[u8; 4]> = Vec::new();
    for sy in y0..y1 {
        for sx in x0..x1 {
            samples.push(px[(sy as u32 * W + sx as u32) as usize]);
        }
    }
    let mut by_lum = samples.clone();
    by_lum.sort_by_key(|c| c[0] as u32 + c[1] as u32 + c[2] as u32);
    let fill = by_lum[by_lum.len() / 2];
    let ink = samples.iter().fold(0.0_f32, |m, c| m.max(dist(*c, fill)));
    (fill, ink, samples.len())
}

/// **THE FILE YOU ARE EDITING IS PLATED IN BOTH CELLS, ON EVERY WORLD —
/// REAL PIXELS.**
///
/// The two cells are the ones the user's report and its follow-up name: ONE
/// file open (the identity line, which used to draw bare — "when you first
/// open a file, it doesn't seem to be selected?") and a file freshly opened
/// AMONG SEVERAL (already correct, and swept here so this law cannot be
/// satisfied by fixing one shape and breaking the other). Both are asserted
/// through the same production door, `gutter_stack_plate_rect`, so a plate
/// this law samples is the plate the frame actually filled.
///
/// THREE INDEPENDENT FLOORS, because any two of them are jointly satisfiable
/// by a broken frame:
///
/// 1. **PRESENCE, by pixel COUNT and by SEPARATION.** The plate's interior
///    must be MOSTLY one flat fill (a majority of interior pixels within a
///    tight radius of the median), and that fill must sit a real distance
///    from the bare margin beside it at the same rows. A plate washed toward
///    the page — the four-bytes-from-the-ground failure this repo records —
///    dies here, and it dies on an absolute count rather than on a ratio
///    that would only get happier as the wash deepened.
/// 2. **LEGIBILITY.** The furthest interior pixel from that fill is the
///    label's own ink, and it must clear a real distance: the ink chosen for
///    a bare line (`muted`) collapses into `surface_selected` across most of
///    the roster, so a plate that kept it would photograph a filled band
///    with nothing written on it.
/// 3. **FORWARDNESS**, a ratio of two quantities read from ONE frame: the
///    active row must weigh more against the margin than the block's own
///    quietest shipped line (the folder heading, drawn `muted` on bare
///    ground). This is the user's actual complaint, and it is the floor a
///    plate-shaped-but-invisible treatment cannot buy its way past.
///
/// Enrolment is the roster itself (`theme::THEMES`, all of it — Cassowary
/// included, which lives in its own module and is missed by a grep over
/// `worlds.rs`), and the count is asserted at the end so a world that
/// silently stopped enrolling is a failure rather than a smaller sweep.
#[test]
fn the_active_file_is_plated_alone_and_among_several_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping the_active_file_is_plated_alone_and_among_several_on_every_world: \
             no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();

    // Floors set UNDER the roster's own tightest MEASURED value rather than at
    // round numbers, so each has real room above it and still fires well before
    // the treatment it guards is gone. The tightest cells measured: uniformity
    // 0.62 (Mangrove), fill-vs-margin 33.1 (Bilby, the roster's palest plate
    // against the palest page), ink-vs-fill 134.5 (Mopoke), forwardness 1.75
    // (Firetail among three). The two failures these are calibrated against
    // measure far below them: a plate washed to `0x04` alpha reads 3.0 on the
    // second, and an identity line handed the margin's plain `muted` instead of
    // the routed plated ink reads 53.7 on the third. Every cell names its own
    // numbers on failure, so a world that lands under one is legible rather
    // than mysterious.
    const FILL_UNIFORMITY: f32 = 0.5;
    const FILL_VS_MARGIN: f32 = 24.0;
    const INK_VS_FILL: f32 = 100.0;
    const FORWARD: f32 = 1.15;

    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;
        // `(files open, which one is active)` — the lone file, and the
        // newest of three, which is the fresh-open-among-several cell.
        for (n, active) in [(1usize, 0usize), (3, 2)] {
            let cell = format!("{world:?} n={n} active={active}");
            p.set_view(&active_file_view(n, active));
            let bands = row_bands(&p.gutter_frost_seeds(H));
            assert_eq!(
                bands.len(),
                n + 1,
                "{cell}: expected the folder heading over {n} identity row(s)"
            );
            let plate = p
                .gutter_stack_plate_rect(H)
                .unwrap_or_else(|| panic!("{cell}: the active file drew no plate at all"));
            assert!(
                plate[2] > 16.0 && plate[3] > 8.0,
                "{cell}: plate {plate:?} is too small to sample inside its own corners"
            );
            let px = render_frame(&device, &queue, &mut p);

            // (1) PRESENCE. The margin reference is read at the SAME rows as
            // the plate, a plate-width to its left — bare margin the block
            // never reaches, so a fill that faded into the page has nowhere
            // to hide.
            let (fill, ink, total) = plate_fill_and_ink(&px, plate);
            let uniform = {
                const INSET: f32 = 5.0;
                let x0 = ((plate[0] + INSET) as i64).clamp(0, W as i64 - 1);
                let x1 = ((plate[0] + plate[2] - INSET) as i64).clamp(x0 + 1, W as i64);
                let y0 = ((plate[1] + INSET) as i64).clamp(0, H as i64 - 1);
                let y1 = ((plate[1] + plate[3] - INSET) as i64).clamp(y0 + 1, H as i64);
                let mut near = 0usize;
                for y in y0..y1 {
                    for x in x0..x1 {
                        if dist(px[(y as u32 * W + x as u32) as usize], fill) < 8.0 {
                            near += 1;
                        }
                    }
                }
                near as f32 / total as f32
            };
            assert!(
                uniform >= FILL_UNIFORMITY,
                "{cell}: PRESENCE — only {:.0}% of the plate's interior is one flat fill \
                 {fill:?}; a plate that is mostly glyph and edge is not a plate",
                uniform * 100.0
            );
            // The reference is the SAME rect one row UP — bare margin the
            // block's own quiet ink barely touches, twelve pixels away, so a
            // ground gradient across the margin cannot stand in for a plate.
            // A single point further to the left CANNOT do this job: the
            // margin's own ground varies enough across the gutter's width to
            // report a two-byte wash as a 28-unit "separation".
            let band0 = bands[0];
            let above = [plate[0], plate[1] - band0[3], plate[2], plate[3]];
            let (margin, _, _) = plate_fill_and_ink(&px, above);
            let sep = dist(fill, margin);
            assert!(
                sep >= FILL_VS_MARGIN,
                "{cell}: PRESENCE — the plate's fill {fill:?} sits only {sep:.1} from the \
                 bare margin one row above it {margin:?}; a plate washed toward the page \
                 passes every ratio in this law and photographs as nothing"
            );

            // (2) LEGIBILITY — the label's own ink, over the fill it sits on.
            assert!(
                ink >= INK_VS_FILL,
                "{cell}: LEGIBILITY — the furthest pixel inside the plate is only {ink:.1} \
                 from its own fill {fill:?}; the name has vanished into the band that is \
                 supposed to be marking it"
            );

            // (3) FORWARDNESS — the ratio, against the block's quietest line.
            let ground_y = (band0[1] - band0[3] * 3.0).max(0.0) as u32;
            let ground = px[(ground_y * W + (band0[2] * 0.5) as u32) as usize];
            let identity = ink_weight(&px, bands[1 + active], ground);
            let project = ink_weight(&px, band0, ground);
            assert!(
                project > 0.0,
                "{cell}: the folder heading drew no ink — the fixture never reached the margin"
            );
            let forward = identity / project.max(1.0);
            assert!(
                forward >= FORWARD,
                "{cell}: the active file reads at {identity:.1} against the block's own \
                 quietest line at {project:.1} (ratio {forward:.3}) — it does not read as \
                 the file being edited"
            );
        }
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

//! EVERY WORLD'S COMMAND PALETTE DRAWS ITS KEY CHORDS.
//!
//! Mangrove's `⌘P` palette shipped with no visible shortcuts at all. The cause
//! was not Mangrove: it was the shared presentation owner. A right-anchored card
//! HUGS its measured content, and that measurement counted the rows
//! alone — while the diagonal composition then spends the card's band
//! on its attachment inset, its spine-to-cluster connector and its selected
//! row's outward step before a row is laid at all. The card came out exactly one
//! cluster wide, `diagonal_cluster_budget` cut that same territory back out of
//! `text_w`, `rowlayout::fits` failed, and the whole secondary column yielded.
//! Mangrove is simply the one world that is BOTH right-anchored and diagonal;
//! Magpie (diagonal, left-anchored) and Cassowary (right-anchored, upright) each
//! met one half of the condition and kept their chords. The repair is in the
//! shared owner — `TextPipeline::diagonal_side_reserve_px`, folded into
//! `measure_overlay_content_w` — and no world is named anywhere in it.
//!
//! The oracle here is neither `overlay_right_shown` nor `bars_inline_shortcut`,
//! the two switches the shaper itself branches on. It is the DRAWN FRAME: blank
//! one row's chord, re-render, and require the pixels to move inside that row's
//! own band and nowhere else. A world that silently drops its shortcuts fails by
//! NAME, and a world that draws them on the wrong row fails the same way.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::render::chrome::diagonal::DiagonalComposition;

/// The REAL `⌘P` palette — `crate::commands`' own names and effective bindings,
/// the same pair `rowlayout`'s kind table and `overlay::build` read.
fn palette_view() -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = crate::commands::names();
    v.overlay_bindings = crate::commands::effective_bindings(&[], &[]);
    v.overlay_selected = 0;
    v.overlay_window_rows = 12;
    v.overlay_lens = crate::facets::scheme(crate::overlay::OverlayKind::Command)
        .map(|s| {
            s.strip
                .iter()
                .enumerate()
                .map(|(i, f)| (f.label.to_string(), i == 0))
                .collect()
        })
        .unwrap_or_default();
    v
}

/// A world's LIST STYLE, matched with NO WILDCARD so a fourth composition cannot
/// join the roster without deciding what "its chords are drawn" means for it.
fn style_name(style: theme::ListStyle) -> &'static str {
    match style {
        theme::ListStyle::Pane => "Pane",
        theme::ListStyle::Bars => "Bars",
        theme::ListStyle::Diagonal(theme::DiagonalDirection::Descending) => "Diagonal(Descending)",
        theme::ListStyle::Diagonal(theme::DiagonalDirection::Ascending) => "Diagonal(Ascending)",
        theme::ListStyle::Rules(theme::RuleSelection::Weight) => "Rules(Weight)",
        theme::ListStyle::Rules(theme::RuleSelection::Gutter) => "Rules(Gutter)",
    }
}

/// The candidate row bands of the current frame, as canvas rects.
fn row_bands(p: &TextPipeline, w: u32) -> Vec<(usize, pixeldiff::Region)> {
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let card = p.overlay_card_rect().expect("the palette card");
    plan.rows()
        .iter()
        .map(|row| {
            (
                row.display,
                pixeldiff::Region::new(
                    card[0],
                    row.top,
                    card[2],
                    (row.bottom() - row.top).max(1.0),
                ),
            )
        })
        .collect()
}

type Verdicts<'a> = (&'a mut Vec<String>, &'a mut Vec<String>);

/// Blanking row `display`'s chord must change that row's own band and no other.
#[allow(clippy::too_many_arguments)]
fn grade_row(
    world: &theme::Theme,
    display: usize,
    chord: &str,
    bands: &[(usize, pixeldiff::Region)],
    (with_chords, without): (&[[u8; 4]], &[[u8; 4]]),
    (w, h): (u32, u32),
    (silent, misplaced): Verdicts<'_>,
) {
    let style = style_name(world.render_caps.list_style);
    let own = bands
        .iter()
        .find(|(d, _)| *d == display)
        .map(|(_, r)| *r)
        .expect("the probed row's own band");
    if pixeldiff::diff_region(with_chords, without, w as i64, h as i64, own).differing == 0 {
        silent.push(format!(
            "{} ({style}): row {display} owns chord {chord:?} and drew nothing for it",
            world.name,
        ));
    }
    for (d, band) in bands {
        if *d == display {
            continue;
        }
        let other = pixeldiff::diff_region(with_chords, without, w as i64, h as i64, *band);
        if other.differing > 0 {
            misplaced.push(format!(
                "{} ({style}): blanking row {display}'s chord changed row {d} \
                 ({} px) — a chord drawn off its own row",
                world.name, other.differing,
            ));
            break;
        }
    }
}

/// EVERY WORLD DRAWS THE CHORD OF EVERY ROW THAT HAS ONE — and draws it on that
/// row. Swept over the WHOLE roster with the real command palette; the verdict
/// per world is pixel arithmetic over two real frames, and every failure is
/// collected so the panic names the worlds rather than the first one.
#[test]
fn every_world_draws_its_palette_key_chords_on_the_owning_row() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping every_world_draws_its_palette_key_chords: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    let base = palette_view();
    let mut silent: Vec<String> = Vec::new();
    let mut misplaced: Vec<String> = Vec::new();
    let mut resized: Vec<String> = Vec::new();
    let mut swept: Vec<&'static str> = Vec::new();

    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        swept.push(world.name);

        p.set_view(&base);
        p.prepare(&device, &queue, w, h).unwrap();
        let with_chords = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
        let rect = p.overlay_card_rect().expect("the palette card");
        let bands = row_bands(&p, w);

        // A row whose chord is NOT the widest in the roster: blanking it cannot
        // change the card's hug width, so any pixel that moves is the chord
        // itself rather than a relayout. (The card rect is asserted below
        // regardless, so a future roster that breaks the assumption says so.)
        let widest = base
            .overlay_bindings
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0);
        let Some((display, item)) = bands.iter().find_map(|(display, _)| {
            let geom = p.overlay_geometry(w);
            let plan = p.overlay_row_plan(&geom);
            let item = plan.item_at(*display)?;
            let bind = base.overlay_bindings.get(item)?;
            (!bind.is_empty() && bind.chars().count() < widest).then_some((*display, item))
        }) else {
            panic!(
                "{}: the visible palette window holds no chord-bearing row to probe",
                world.name
            );
        };

        let mut muted_view = palette_view();
        muted_view.overlay_bindings[item] = String::new();
        p.set_view(&muted_view);
        p.prepare(&device, &queue, w, h).unwrap();
        let without = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
        let rect_after = p.overlay_card_rect().expect("the palette card");

        if rect != rect_after {
            resized.push(format!(
                "{} ({style}): blanking one chord moved the card {rect:?} -> {rect_after:?}",
                world.name,
                style = style_name(world.render_caps.list_style)
            ));
            continue;
        }

        grade_row(
            &world,
            display,
            &base.overlay_bindings[item],
            &bands,
            (&with_chords, &without),
            (w, h),
            (&mut silent, &mut misplaced),
        );
    }
    theme::set_active(theme::DEFAULT_THEME);

    assert_eq!(
        swept.len(),
        theme::THEMES.len(),
        "the sweep must cover every world, got {swept:?}"
    );
    assert!(
        resized.is_empty(),
        "a one-chord edit must not resize the card (the probe's own precondition):\n  {}",
        resized.join("\n  ")
    );
    assert!(
        misplaced.is_empty(),
        "these worlds draw a row's chord outside that row's own band:\n  {}",
        misplaced.join("\n  ")
    );
    assert!(
        silent.is_empty(),
        "these worlds' command palettes draw NO key chord for a row that has one:\n  {}",
        silent.join("\n  ")
    );
}

/// THE HUG WIDTH RESERVES WHAT THE COMPOSITION SPENDS. The shared owner's own
/// arithmetic, swept over the whole roster at both densities: a diagonal world
/// reserves its inset + connector + selected step + the deepest row's travel,
/// and every upright world reserves exactly nothing (so their hug width, and
/// every capture of it, is untouched).
#[test]
fn only_diagonal_worlds_reserve_side_territory_and_they_reserve_all_of_it() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping only_diagonal_worlds_reserve_side_territory: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let v = palette_view();

    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        for world in theme::THEMES {
            theme::set_active_by_name(world.name).unwrap();
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let geom = p.overlay_geometry(w);
            let rows_planned = p.overlay_row_plan(&geom).rows().len();
            let reserve = p.diagonal_side_reserve_px(rows_planned);
            match world.render_caps.list_style {
                // `Rules` is upright too: its rules run ALONG the rows rather
                // than raking across them, so it reserves no side territory
                // either. Its heavy rule does reach past the text measure, but
                // only into the card band the composition already owns.
                theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {
                    assert_eq!(
                        reserve, 0.0,
                        "{}: an upright world reserves no side territory (dpi {dpi})",
                        world.name
                    )
                }
                theme::ListStyle::Diagonal(direction) => {
                    let c = DiagonalComposition::resolve(direction, dpi);
                    let rows = rows_planned.saturating_sub(1) as f32;
                    let want = c.attachment_inset
                        + c.connector
                        + c.selected_outward
                        + c.row_step.abs() * rows;
                    assert!(
                        (reserve - want).abs() < 0.001,
                        "{}: reserve {reserve} != inset+connector+step+travel {want} (dpi {dpi})",
                        world.name
                    );
                    assert!(
                        reserve > c.attachment_inset,
                        "{}: the reserve must carry the deepest row's TRAVEL as well as the \
                         inset — without it the spine has no room to rake (dpi {dpi})",
                        world.name
                    );
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
}

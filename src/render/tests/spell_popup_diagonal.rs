//! THE CONTEXTUAL SPELLING POPUP ON A DIAGONAL COMPOSITION.
//!
//! The popup is not a room takeover: the document stays live, and the composition draws
//! neither a card nor row plates behind its labels. Its backdrop is therefore the same
//! feathered, raking footprint as the rows themselves — never a full-canvas frost. Its
//! width is the widest shaped row plus the Diagonal composition's own side territory,
//! rather than the document mono face's broad character grid.
//!
//! The laws sweep the complete theme roster with no wildcard. That makes the off-arm as
//! important as the enrolled arm: Pane, Bars and Ruled retain the old geometry and no
//! spell frost. The real-pixel law then grades both Diagonal directions at 1×/2× with a
//! short and long Add row, including exact identity at every pixel where the shipping
//! footprint mask is zero.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

const LOGICAL_W: u32 = 1200;
const LOGICAL_H: u32 = 800;
const SPELL_CASES: [(&str, &[&str]); 2] = [
    ("teh", &["the", "eh", "tech", "tee", "tea"]),
    (
        "accomodation",
        &[
            "accommodation",
            "accommodating",
            "accommodate",
            "commendation",
            "commodification",
        ],
    ),
];

fn spell_view(word: &str, suggestions: &[&str]) -> ViewState {
    let popup = crate::overlay::OverlayState::new_spell(
        suggestions.iter().map(|s| (*s).to_string()).collect(),
        (0, 0, word.chars().count()),
        word.to_string(),
    );
    let mut v = view(
        &format!(
            "{word} rests over dense ordinary prose\n\
             Words continue directly behind every candidate row\n\
             Another textured line makes local defocus measurable\n\
             The surrounding document must remain exactly live\n"
        ),
        0,
        0,
    );
    v.overlay_active = true;
    v.overlay_items = popup.item_strings();
    // The live spell picker carries one empty secondary cell per row.
    v.overlay_bindings = vec![String::new(), String::new()];
    v.overlay_selected = 0;
    v.overlay_spell = Some((0, 0, word.chars().count()));
    v
}

fn shaped_rows(p: &TextPipeline) -> Vec<String> {
    p.panel_buffer
        .lines
        .iter()
        .map(|line| line.text().to_string())
        .collect()
}

struct FrostRestore;

impl Drop for FrostRestore {
    fn drop(&mut self) {
        crate::render::blur::set_frost_suppressed(false);
    }
}

struct ActiveThemeRestore(String);

impl Drop for ActiveThemeRestore {
    fn drop(&mut self) {
        crate::theme::set_active_by_name(&self.0).expect("restore entry world");
    }
}

fn grade_geometry_cell(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    world: &crate::theme::Theme,
    dpi: f32,
    word: &str,
    suggestions: &[&str],
) -> bool {
    let w = (LOGICAL_W as f32 * dpi) as u32;
    let h = (LOGICAL_H as f32 * dpi) as u32;
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    let v = spell_view(word, suggestions);
    let add = v
        .overlay_items
        .last()
        .expect("production spell popup always appends Add")
        .clone();
    p.set_view(&v);
    p.prepare(device, queue, w, h).unwrap();

    let rows = shaped_rows(p);
    let card = p.overlay_card_rect().expect("spell popup card");
    let widest_chars = v
        .overlay_items
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    let slack = 1 + crate::render::rowlayout::GAP_CHARS + 1;
    let grid = (widest_chars + slack) as f32 * p.metrics.char_width;
    let measured = p.overlay_spell_w;
    let pad = p.metrics.px(Logical(10.0));
    let margin = p.metrics.px(Logical(8.0));
    let legacy = (measured.max(grid) + 2.0 * pad)
        .clamp(
            p.metrics.px_grow_only(LogicalGrowOnly(140.0)),
            p.metrics.px_grow_only(LogicalGrowOnly(520.0)),
        )
        .min(w as f32 - 2.0 * margin);
    let label = format!("{} @ {dpi}x, {word}", world.name);

    let spine = match world.render_caps.list_style {
        crate::theme::ListStyle::Diagonal(spine) => spine,
        crate::theme::ListStyle::Pane
        | crate::theme::ListStyle::Bars
        | crate::theme::ListStyle::Ruled(_) => {
            assert_eq!(
                card[2].to_bits(),
                legacy.to_bits(),
                "{label}: non-Diagonal geometry left the historical expression"
            );
            assert_eq!(
                p.frost_mode(),
                None,
                "{label}: non-Diagonal popup enrolled in the new frost"
            );
            return false;
        }
    };
    assert!(
        rows.iter().any(|row| row == &add),
        "{label}: widest action clipped/elided; shaped {rows:?}; measured {measured}; \
         card {card:?}"
    );
    assert!(
        rows.iter().all(|row| !row.contains('…')),
        "{label}: ordinary roster elided; shaped {rows:?}"
    );
    assert!(
        card[2] + 0.5 < legacy,
        "{label}: popup did not tighten: card {} legacy {legacy}; measured {measured}; \
         grid {grid}",
        card[2]
    );
    let Some(crate::render::blur::Frost::Footprint(foot)) = p.frost_mode() else {
        panic!("{label}: Diagonal spell popup did not route to local frost")
    };
    assert!(!p.dims_doc(), "{label}: local frost dimmed the document");
    assert!(
        foot.rect[2] > 0.0 && foot.rect[3] > 0.0,
        "{label}: degenerate footprint {foot:?}"
    );
    assert_eq!(
        foot.shear.signum(),
        spine.direction.sign().signum(),
        "{label}: frost rake has the wrong direction"
    );
    true
}

/// EXHAUSTIVE ENROLMENT + GEOMETRY + OUTPUT LAW.
///
/// Every roster member declares one of the four `ListStyle` variants explicitly. The
/// Diagonal arm must (a) shape the widest Add row whole, (b) use less width than the
/// retired generic character-grid expression, and (c) route to a non-degenerate local
/// footprint whose shear has the authored direction. Every other arm must retain the
/// retired width bit-for-bit and route to no spell frost at all.
#[test]
fn only_diagonal_spell_popups_use_measured_clusters_and_local_frost() {
    let _g = crate::testlock::serial();
    let _theme_restore = ActiveThemeRestore(crate::theme::active().name.to_string());
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL_W as f32, LOGICAL_H as f32) else {
        eprintln!(concat!(
            "skipping only_diagonal_spell_popups_use_measured_clusters_",
            "and_local_frost: no wgpu adapter"
        ));
        return;
    };
    crate::menubar::set_menu_bar_on(false);

    let mut enrolled = 0usize;
    let mut inert = 0usize;
    for world in &crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0_f32, 2.0] {
            for (word, suggestions) in SPELL_CASES {
                if grade_geometry_cell(&mut p, &device, &queue, world, dpi, word, suggestions) {
                    enrolled += 1;
                } else {
                    inert += 1;
                }
            }
        }
    }
    eprintln!(
        "MEASURED spell-popup roster: {} Diagonal cells, {} byte-identical geometry/frost cells",
        enrolled, inert
    );
    assert_eq!(enrolled, 8, "two Diagonal worlds × dpi1/2 × short/long");
    assert_eq!(
        inert,
        (crate::theme::THEMES.len() - 2) * 4,
        "every non-Diagonal roster cell must take the inert arm"
    );
}

/// REAL PIXELS: the local frost changes the document beneath the raking popup, and no
/// pixel outside its feathered shipping mask changes at all. The inner presence count
/// prevents a deleted/no-op frost from satisfying the outside-identity half.
#[test]
fn diagonal_spell_frost_changes_its_rake_and_nothing_beyond_the_feather() {
    let _g = crate::testlock::serial();
    let _theme_restore = ActiveThemeRestore(crate::theme::active().name.to_string());
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL_W as f32, LOGICAL_H as f32) else {
        eprintln!(concat!(
            "skipping diagonal_spell_frost_changes_its_rake_and_",
            "nothing_beyond_the_feather: no wgpu adapter"
        ));
        return;
    };
    crate::menubar::set_menu_bar_on(false);
    let mut cells = 0usize;

    for world in crate::theme::THEMES.iter().filter(|world| {
        matches!(
            world.render_caps.list_style,
            crate::theme::ListStyle::Diagonal(_)
        )
    }) {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0_f32, 2.0] {
            let (w, h) = (
                (LOGICAL_W as f32 * dpi) as u32,
                (LOGICAL_H as f32 * dpi) as u32,
            );
            p.set_dpi(dpi);
            p.set_size(w as f32, h as f32);
            for (word, suggestions) in SPELL_CASES {
                let v = spell_view(word, suggestions);
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let frost = p.frost_mode().expect("Diagonal spell footprint");
                let frosted = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

                let _frost_restore = FrostRestore;
                crate::render::blur::set_frost_suppressed(true);
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                assert!(p.frost_mode().is_none(), "suppression door must hold");
                let crisp = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                crate::render::blur::set_frost_suppressed(false);

                let mut inside_changed = 0usize;
                let mut outside_changed = 0usize;
                for y in 0..h {
                    for x in 0..w {
                        let i = (y * w + x) as usize;
                        if frosted[i] == crisp[i] {
                            continue;
                        }
                        let mask = crate::render::blur::footprint_mask_for(
                            frost,
                            dpi,
                            x as f32 + 0.5,
                            y as f32 + 0.5,
                        );
                        if mask > 0.0 {
                            inside_changed += 1;
                        } else {
                            outside_changed += 1;
                        }
                    }
                }
                let label = format!("{} @ {dpi}x, {word}", world.name);
                eprintln!(
                    "MEASURED {label}: frost-changed {inside_changed} pixels inside mask, \
                     {outside_changed} outside"
                );
                assert!(
                    inside_changed > (100.0 * dpi * dpi) as usize,
                    "{}: local frost has no visible subject; only {} changed pixels",
                    label,
                    inside_changed
                );
                assert_eq!(
                    outside_changed, 0,
                    "{label}: frost changed pixels beyond its feathered raking footprint"
                );
                cells += 1;
            }
        }
    }
    assert_eq!(cells, 8, "both directions × dpi1/2 × short/long");
}

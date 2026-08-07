//! ITEM 220 — THE PALETTE'S TWO-LEVEL LOCATION.
//!
//! A summoned faceting picker names where you are twice: its own title (the
//! content level — "commands") and the active lens (the category inside it).
//! The category was drawn by the grouped card's SECTION-HEADER machinery, in
//! that machinery's voice — a small faint uppercase whisper, one line above the
//! rows — while the lens strip a line above it already carried the same word
//! under an active mark. The complaint was that it reads as a repeat of the
//! title rather than as the level below it, and it is right: a section header
//! divides a list into parts, and there are no parts.
//!
//! **THE PREMISE THAT MADE IT UNIVERSAL.** Every lens of every faceting picker
//! in the product groups into exactly ONE section, whose label is
//! character-for-character the lens's own — six pickers, 27 lenses. So that
//! line was never a section header on any of them; it was always the location,
//! drawn as list chrome. `every_lens_of_every_faceting_picker_names_one_section`
//! is that premise as a law, so a future lens with two real sections has to
//! notice this file before it ships.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};

/// **THE PREMISE.** Index 0 is the All HOME: it groups nothing and names no
/// narrower place. Every other lens groups into exactly one section and that
/// section IS the lens — which is what makes the header above the rows a
/// LOCATION on every faceting picker rather than on the one it was reported on.
#[test]
fn every_lens_of_every_faceting_picker_names_one_section_and_it_is_the_lens_itself() {
    let mut lenses = 0usize;
    let mut pickers = 0usize;
    for kind in OverlayKind::ALL {
        let Some(scheme) = crate::facets::scheme(kind) else {
            continue;
        };
        pickers += 1;
        assert!(
            scheme.strip[0].sections.is_empty() && scheme.location(0).is_none(),
            "{kind:?}: strip index 0 is the All home — it groups nothing and names no \
             narrower place"
        );
        for (i, facet) in scheme.strip.iter().enumerate().skip(1) {
            lenses += 1;
            assert_eq!(
                facet.sections,
                [facet.label],
                "{kind:?} lens {i} ({}) groups into something other than itself — the \
                 line above its rows is no longer purely a location, and this file's \
                 whole premise needs revisiting before it ships",
                facet.label
            );
            assert_eq!(
                scheme.location(i),
                Some(facet.label),
                "{kind:?} lens {i}: the location owner must answer with the lens's own label"
            );
        }
    }
    assert_eq!(
        (pickers, lenses),
        (6, 22),
        "the faceting roster moved — re-read this file's premise, then update the pin"
    );
}

/// The reconstruction the headless capture path uses (it holds a serialized
/// strip and no live picker) is the SAME answer the scheme gives, at every lens
/// index of every scheme. Without this, `facets::strip_location` would be a
/// second owner of the hierarchy, free to disagree with the strip's own mark.
#[test]
fn a_rebuilt_strips_location_is_the_schemes_own_at_every_lens() {
    for kind in OverlayKind::ALL {
        let Some(scheme) = crate::facets::scheme(kind) else {
            continue;
        };
        for i in 0..scheme.strip.len() {
            let strip = scheme.strip_labels(i);
            assert_eq!(
                crate::facets::strip_location(&strip),
                scheme.location(i),
                "{kind:?} lens {i}: the rebuilt strip and the scheme disagree about where \
                 this picker is"
            );
        }
    }
}

/// A COMMAND palette folded the way `App::sync_view` folds one, at lens `lens`.
fn palette_view(lens: usize) -> ViewState {
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov =
        OverlayState::new_command(names, crate::commands::effective_bindings(&[], &[]), hidden);
    ov.set_facet_lens(lens);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Command.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_selected = ov.selected;
    v
}

/// **THE HIERARCHY, ON THE FRAME THE PIXELS COME FROM.** At the All home the
/// band opens on a command — no category label of any kind. On a category the
/// band opens on a LOCATION line carrying that category's own name, and the
/// card holds no section header anywhere: the duplicated low-contrast label is
/// gone, not restyled in place.
///
/// NON-VACUITY IS THE THIRD ARM, and it is the pre-fix product itself: with the
/// location datum withheld (`overlay_location: None` — exactly what every frame
/// carried before item 220), the same geometry owner emits the uppercase
/// `Header` this law exists to forbid, in the same slot.
#[test]
fn a_category_heads_its_band_with_a_location_and_the_home_shows_none() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping palette location plan law: no wgpu adapter");
        return;
    };
    let files = crate::facets::scheme(OverlayKind::Command)
        .expect("the command palette facets")
        .strip
        .iter()
        .position(|f| f.label == "Files")
        .expect("the command palette has a Files lens");

    let mut graded = 0usize;
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();

        // THE HOME: no second level at all.
        p.set_view(&palette_view(0));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let home = p.overlay_geometry(1200);
        assert!(
            home.plan_labels_probe().is_empty(),
            "{world}: the All home drew {:?} — the home names no narrower place",
            home.plan_labels_probe()
        );

        // A CATEGORY: one location line, no section headers.
        let mut v = palette_view(files);
        assert_eq!(v.overlay_location.as_deref(), Some("Files"));
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let cat = p.overlay_geometry(1200);
        assert_eq!(
            cat.plan_labels_probe(),
            ["loc:Files"],
            "{world}: the Files category must head its band with exactly one LOCATION and \
             no section header"
        );

        // NON-VACUITY — the product as it stood before the datum existed.
        v.overlay_location = None;
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let pre = p.overlay_geometry(1200);
        assert_eq!(
            pre.plan_labels_probe(),
            ["hdr:FILES"],
            "{world}: withholding the location must reproduce the duplicated uppercase \
             header, or this law is grading nothing"
        );
        // The SLOT never moved: the defect was a heading in a list's voice, not
        // geometry, and a reader relying on that has to be told if it changes.
        assert_eq!(
            cat.plan_len_probe(),
            pre.plan_len_probe(),
            "{world}: the location must occupy the same slot the section header did"
        );
        graded += 1;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        graded,
        theme::THEMES.len(),
        "the hierarchy law must sweep the whole world roster"
    );
}

/// **THE SECOND LEVEL READS AS ONE, IN REAL PIXELS, ON EVERY WORLD.** The two
/// frames differ in exactly one thing — whether the card knows where it is — so
/// ground, texture, placard, strip and rows all cancel and what is left on that
/// one line is the treatment. The location's strongest ink against the card's
/// own ground must beat the faint whisper it replaced — on every world that
/// draws its cue IN the card. A world that composes it against the ROOM instead
/// takes the mirrored arm (its card band must be glyph-free), because this
/// oracle scans the card and would otherwise report a weak cue where it is
/// really measuring an empty line; the sweep's own doc has both rosters.
/// Perceptual luma of a captured pixel.
fn luma(c: [u8; 4]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

/// The strongest departure from `ground` anywhere in a 1200-wide capture's
/// `x` by `y` window — the peak-of-|Δluma| oracle both arms of the heading
/// comparison are measured with.
fn peak_departure(pixels: &[[u8; 4]], ground: f32, x: (usize, usize), y: (usize, usize)) -> f32 {
    let mut peak = 0.0f32;
    for row in y.0..y.1 {
        for col in x.0..x.1 {
            peak = peak.max((luma(pixels[row * 1200 + col]) - ground).abs());
        }
    }
    peak
}

/// ⚠️ WHERE A ROW'S OWN INK IS IS NOT A CONSTANT, and on a diagonal world
/// neither half of the band is the answer. A mirrored composition hangs its
/// names on the SPINE end and right-aligns them there, so the left-half scan
/// this law was written with read the card's empty surface and its own vacuity
/// guard fired — the WINDOW was the untested hypothesis, not the claim.
///
/// On those worlds the window is the row's WHOLE side of its spine: it must hold
/// both arms' ink, and the two do not share a column — the raked location cue
/// hangs flush at the card's own text edge while the section header it replaced
/// hangs at the spine — and it must exclude the SPINE, full-strength `muted` ink
/// present in BOTH arms, which saturated the peak-of-|Δluma| oracle at 128.2
/// apiece and compared the heading to nothing. The GROUND then comes from the
/// band's far side, past the spine, where no row draws at all.
///
/// Returns `((x0, x1), ground_x)` for a band `[x, w, ..]`.
fn scan_window(p: &TextPipeline, band: [f32; 4]) -> ((f32, f32), f32) {
    let mid = band[0] + band[1] * 0.5;
    match p.diagonal_cluster_probe() {
        None => ((band[0], mid), band[0] + band[1] - 6.0),
        Some(cluster) => {
            let attach = cluster.label_anchor(0);
            match cluster.label_flow() {
                crate::render::rowlayout::ColumnFlow::Rightward => {
                    ((attach, band[0] + band[1]), band[0] + 6.0)
                }
                crate::render::rowlayout::ColumnFlow::Leftward => {
                    ((band[0], attach), band[0] + band[1] - 6.0)
                }
            }
        }
    }
}

/// One offscreen frame of the palette, read back.
fn shoot_palette(device: &wgpu::Device, queue: &wgpu::Queue, p: &mut TextPipeline) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, 1200, 800);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item220 location encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, 1200, 800)
}

/// The active world's two peaks — `[located, header]` — plus the band they were
/// measured in, for the failure message. Split from the sweep below so the
/// sweep stays a readable loop over its own rosters.
fn located_and_header_peaks(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    files: usize,
) -> ([f32; 2], [f32; 4]) {
    let mut peaks = [0.0f32; 2];
    let mut band = [0.0f32; 4];
    for (arm, located) in [true, false].into_iter().enumerate() {
        let mut v = palette_view(files);
        if !located {
            v.overlay_location = None;
        }
        p.set_view(&v);
        p.prepare(device, queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        let first = plan.rows().first().expect("the card plans a first line");
        band = [
            geom.band_x_probe(),
            geom.band_w_probe(),
            first.top,
            first.height,
        ];
        let pixels = shoot_palette(device, queue, p);
        let ((x_lo, x_hi), gx) = scan_window(p, band);
        let gy = (band[2] + band[3] * 0.5).round().clamp(0.0, 799.0) as usize;
        let gx = gx.round().clamp(0.0, 1199.0) as usize;
        let ground = luma(pixels[gy * 1200 + gx]);
        let x0 = x_lo.round().max(0.0) as usize;
        let x1 = x_hi.round().min(1199.0) as usize;
        // ⚠️ THE SCANNED BAND IS THE ROW'S INTERIOR, NOT ITS WHOLE SLOT: a
        // slot's outer edges belong to whatever a world draws BETWEEN rows,
        // and a `Rules` world puts a rule there in full-strength ink — the
        // same in both arms, saturating a peak-of-|Δluma| oracle at 211.0
        // apiece so the heading was compared to nothing. Inset by the same
        // half-gap the row-pitch owner folded in, zero on every world with
        // no air between its rows.
        let air = p.overlay_row_gap() * 0.5;
        let y0 = (band[2] + air).round().max(0.0) as usize;
        let y1 = (band[2] + band[3] - air).round().min(799.0) as usize;
        peaks[arm] = peak_departure(&pixels, ground, (x0, x1), (y0, y1));
    }
    (peaks, band)
}

#[test]
fn the_location_heading_reads_stronger_than_the_faint_header_it_replaced_in_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping palette location ink law: no wgpu adapter");
        return;
    };
    let files = crate::facets::scheme(OverlayKind::Command)
        .expect("the command palette facets")
        .strip
        .iter()
        .position(|f| f.label == "Files")
        .expect("the command palette has a Files lens");

    let mut ratios: Vec<(String, f32)> = Vec::new();
    // A world whose `faint` and `muted` are the SAME ink cannot express this
    // hierarchy by value, and one of them ships: Wagtail is 1-bit, so its whole
    // palette is two tones and every secondary is the same tone. There the
    // location still reads as the level below the title — by the CHROME face and
    // its own authored case rather than by contrast — and the ink arm holds it to
    // "no weaker" instead. The roster of such worlds is PINNED, so a second
    // one-ink world has to arrive here and say so.
    let mut one_ink: Vec<String> = Vec::new();
    // ⚠️ A WORLD MAY COMPOSE ITS CUE OUTSIDE THE CARD, and then this oracle is
    // structurally blind to it rather than measuring a weak one. `RotatedRail`
    // seats its run in the ROOM's own outer margin beside the wordmark placard,
    // so the card band scanned below holds no cue ink at all — the claim there
    // is not "stronger than the whisper" but "the inline slot stays glyph-free",
    // asserted as such, with the cue's own strength graded against the
    // wordmark's ink by `rotated_rail_item297`. The roster of such worlds is
    // PINNED, so a second one has to arrive here and say so.
    let mut off_card: Vec<String> = Vec::new();
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let distinct = theme::faint().rgba_bytes() != theme::muted().rgba_bytes();
        let off = theme::active().render_caps.location_style == theme::LocationStyle::RotatedRail;
        let (peaks, band) = located_and_header_peaks(&device, &queue, &mut p, files);
        assert!(
            peaks[1] > 2.0,
            "{world}: the retired faint header is invisible to this oracle at band \
             {band:?} (peak {:.1}) — the comparison would be vacuous",
            peaks[1]
        );
        if off {
            // The inline slot really is glyph-free: whatever the card draws on
            // that line in the LOCATED arm must be weaker than the header the
            // other arm draws there, because the located arm draws nothing.
            assert!(
                peaks[0] < peaks[1],
                "{world}: composes its cue outside the card, so the card's own location \
                 band must be glyph-free — it reads {:.1} against the retired header's \
                 {:.1}",
                peaks[0],
                peaks[1]
            );
            off_card.push(world.to_string());
        } else if distinct {
            assert!(
                peaks[0] > peaks[1],
                "{world}: the location heading reads at {:.1} against the card's ground \
                 where the faint section header it replaced read at {:.1} — the second \
                 level is no stronger than the whisper it was meant to replace",
                peaks[0],
                peaks[1]
            );
            ratios.push((world.to_string(), peaks[0] / peaks[1]));
        } else {
            assert!(
                peaks[0] >= peaks[1] - 0.5,
                "{world}: a one-ink world's location must still read no weaker than the \
                 header it replaced ({:.1} against {:.1})",
                peaks[0],
                peaks[1]
            );
            one_ink.push(world.to_string());
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        one_ink,
        ["Wagtail"],
        "the roster of worlds whose faint and muted inks are the same tone moved"
    );
    assert_eq!(
        off_card,
        ["Cassowary"],
        "the roster of worlds that compose their location cue outside the card moved"
    );
    let worst = ratios.iter().fold(f32::MAX, |acc, (_, r)| acc.min(*r));
    assert!(
        ratios.len() + one_ink.len() + off_card.len() == theme::THEMES.len() && worst > 1.0,
        "ink law swept {} worlds by value, worst ratio {worst:.2}: {ratios:?}",
        ratios.len()
    );
}

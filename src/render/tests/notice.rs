//! THE CALM NOTICE — where it sits, whether it can be seen, and whether the two
//! KINDS can be told apart. Every claim here is pixel arithmetic over a PNG
//! written by `capture::capture_with`, the exact function `--screenshot` calls.
//!
//! # Why these laws exist, and why a contrast floor alone is not one of them
//!
//! The notice channel had ~ten production callers and had never been seen. The
//! measured reason was three-fold and only ONE part of it was legibility: a
//! `saved` toast drew 221 pixels — 0.023% of the canvas — fourteen pixels from
//! the BOTTOM edge, wedged into the interline gap between the last two rows of
//! prose, in `muted` at 0.8× body size. Its contrast against the page was
//! **4.84:1**, which passes any ordinary floor. So a legibility law would have
//! been green over the whole defect, and the three floors below are deliberately
//! chosen so that no one of them can be satisfied by breaking the others:
//!
//!   * **LEGIBILITY** — the darkest ink on the plate against the plate.
//!   * **PRESENCE** — the plate against the page it covers. Without this, the
//!     legibility floor gets *happier* as the plate fades toward the page, and a
//!     plate four bytes from the ground would report a better ratio than the
//!     shipped one while drawing nothing a reader can find.
//!   * **DISTINCTION** — the two kinds' plates against each other. Without this,
//!     "each kind is legible on a visible plate" is satisfied by giving them the
//!     same plate, which answers half the channel: a HELD refusal the writer must
//!     act on would look exactly like a self-clearing acknowledgement.
//!
//! Enrolment is derived from `theme::THEMES` rather than pinned to a named
//! world, and the failure message names the world that failed.

use super::super::*;
use super::pixeldiff::{Region, diff_region, render_frame};
use crate::actions::NoticeKind;
use crate::capture::{CaptureOpts, capture_with};

/// The canvas every capture in this file renders at — `capture`'s own default.
const CANVAS: (u32, u32) = (crate::capture::CANVAS_WIDTH, crate::capture::CANVAS_HEIGHT);

/// A document long enough that prose reaches BOTH the top and the bottom of the
/// canvas. That matters: the notice has to be checked against a page that is
/// occupied where it lands, not against empty ground that would flatter any
/// placement.
fn crowded_doc() -> String {
    let mut s = String::from("Opening line of the document, at the very top.\n");
    for i in 1..60 {
        s.push_str(&format!(
            "Line {i} of prose that runs the length of the writing column.\n"
        ));
    }
    s
}

#[derive(Clone, Copy, Debug)]
enum ToastSurface {
    Document,
    Picker,
    Workspace,
}

fn toast_surface_view(surface: ToastSurface, notice: Option<NoticeKind>) -> ViewState {
    let mut document = crowded_doc();
    if matches!(surface, ToastSurface::Document) {
        document.replace_range(.."Opening line".len(), "# Opening line");
    }
    let mut v = super::view(&document, 0, 0);
    if let Some(kind) = notice {
        v.notice = "saved".into();
        v.notice_kind = kind;
    }
    match surface {
        ToastSurface::Document => {}
        ToastSurface::Picker => {
            v.overlay_active = true;
            v.overlay_title = "Commands".to_string();
            v.overlay_items = vec![
                "Save".into(),
                "Open".into(),
                "Move".into(),
                "Duplicate".into(),
            ];
            v.overlay_window_rows = v.overlay_items.len();
        }
        ToastSurface::Workspace => {
            v.overlay_active = true;
            v.overlay_workspace = true;
            v.overlay_title = "Settings".to_string();
            v.overlay_lens = vec![
                ("All".into(), true),
                ("Writing".into(), false),
                ("Files".into(), false),
            ];
            v.overlay_items = vec!["Theme".into(), "Language".into(), "Page width".into()];
            v.overlay_window_rows = v.overlay_items.len();
        }
    }
    v
}

fn rectangles_clear(a: [f32; 4], b: [f32; 4], gap: f32) -> bool {
    let [ax, ay, aw, ah] = a;
    let [bx, by, bw, bh] = b;
    ax + aw + gap <= bx || bx + bw + gap <= ax || ay + ah + gap <= by || by + bh + gap <= ay
}

/// The production pipeline's full assigned roster: every world, its authored
/// anchor, all three surface families, narrow/ordinary/wide logical canvases,
/// both densities, and BOTH notice kinds — a HELD `Sticky` and a self-clearing
/// `Toast` share the one authored anchor and the one collision planner, so
/// this law sweeps `NoticeKind` rather than assuming only `Toast` reaches it.
/// The pure planner law crosses each world with every possible anchor; this
/// law proves real overlay/workspace geometry is what the collision owner
/// receives.
#[test]
fn every_worlds_toast_is_in_canvas_and_clear_across_the_full_surface_roster() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping toast surface roster: no wgpu adapter");
        return;
    }
    let entry = theme::active_index();
    let mut cells = 0usize;
    let mut fallbacks = 0usize;
    for world in theme::THEMES {
        theme::set_active_by_name(world.name);
        let Some((device, queue, mut p)) = super::headless_dqp(1200.0, 800.0) else {
            break;
        };
        for kind in [NoticeKind::Toast, NoticeKind::Sticky] {
            for surface in [
                ToastSurface::Document,
                ToastSurface::Picker,
                ToastSurface::Workspace,
            ] {
                for (logical_w, logical_h) in [(480u32, 360u32), (1200, 800), (1800, 1000)] {
                    for dpi in [1.0f32, 2.0] {
                        let (w, h) = (
                            (logical_w as f32 * dpi) as u32,
                            (logical_h as f32 * dpi) as u32,
                        );
                        p.set_dpi(dpi);
                        p.set_size(w as f32, h as f32);
                        p.set_view(&toast_surface_view(surface, Some(kind)));
                        p.prepare(&device, &queue, w, h)
                            .expect("notice frame prepares");
                        let (plate, resolved) = p
                            .notice_geometry_probe(w, h)
                            .expect("a notice must commit plate geometry");
                        let safe = p.metrics.px(crate::render::chrome::TOAST_SAFE_INSET);
                        let gap = p.metrics.px(crate::render::chrome::TOAST_COLLISION_GAP);
                        let label = format!(
                            "{} / {kind:?} / {:?}->{resolved:?} / {surface:?} / \
                             {logical_w}x{logical_h} / {dpi}x",
                            world.name, world.toast_anchor
                        );
                        assert!(
                            plate[0] >= safe - 0.01 && plate[1] >= safe - 0.01,
                            "{label}: {:?} crossed the safe inset {safe}",
                            plate
                        );
                        assert!(
                            plate[0] + plate[2] <= w as f32 - safe + 0.01
                                && plate[1] + plate[3] <= h as f32 - safe + 0.01,
                            "{label}: {:?} left {w}x{h}",
                            plate
                        );
                        let obstacles = p.notice_active_chrome_probe(w, h);
                        assert!(
                            obstacles
                                .iter()
                                .all(|&obstacle| rectangles_clear(plate, obstacle, gap)),
                            "{label}: {:?} collided with {:?}",
                            plate,
                            obstacles
                        );
                        assert!(
                            plate[2] >= 1.0 && plate[3] >= 1.0,
                            "{label}: presence is vacuous: {plate:?}"
                        );
                        fallbacks += usize::from(resolved != world.toast_anchor);
                        cells += 1;
                    }
                }
            }
        }
    }
    theme::set_active(entry);
    assert_eq!(cells, 20 * 2 * 3 * 3 * 2);
    assert!(
        fallbacks > 0,
        "NON-VACUITY: no real surface forced fallback"
    );
    eprintln!("notice surface roster: cells={cells} fallbacks={fallbacks}");
}

/// Five affordance-locating gallery cells. The oracle asks only whether the
/// resolved plate's own real pixels appeared; geometry intent alone cannot
/// satisfy it (the Wagtail tripwire).
#[test]
fn five_world_surface_gallery_draws_a_visible_toast_plate() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping toast gallery smoke: no wgpu adapter");
        return;
    }
    let entry = theme::active_index();
    let gallery = [
        ("Gumtree", ToastSurface::Document, 1.0f32),
        ("Potoroo", ToastSurface::Picker, 2.0),
        ("Bilby", ToastSurface::Workspace, 1.0),
        ("Wagtail", ToastSurface::Picker, 1.0),
        ("Cassowary", ToastSurface::Workspace, 2.0),
    ];
    let mut total_changed = 0usize;
    for (world, surface, dpi) in gallery {
        theme::set_active_by_name(world);
        let (w, h) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        let Some((device, queue, mut p)) = super::headless_dqp(w as f32, h as f32) else {
            break;
        };
        p.set_dpi(dpi);
        p.set_size(w as f32, h as f32);
        p.set_view(&toast_surface_view(surface, None));
        p.prepare(&device, &queue, w, h)
            .expect("plain frame prepares");
        let plain = render_frame(&mut p, &device, &queue, w, h);
        p.set_view(&toast_surface_view(surface, Some(NoticeKind::Toast)));
        p.prepare(&device, &queue, w, h)
            .expect("toast frame prepares");
        let (plate, _) = p.notice_geometry_probe(w, h).expect("toast geometry");
        let toast = render_frame(&mut p, &device, &queue, w, h);
        let report = diff_region(
            &plain,
            &toast,
            w as i64,
            h as i64,
            Region::new(
                plate[0] - 1.0,
                plate[1] - 1.0,
                plate[2] + 2.0,
                plate[3] + 2.0,
            ),
        );
        assert!(
            report.differing >= 80 && report.max_channel_delta >= 12,
            "{world} / {surface:?} / {dpi}x: locate the toast at {plate:?}; only {} pixels \
             changed (max channel delta {})",
            report.differing,
            report.max_channel_delta
        );
        total_changed += report.differing;
    }
    theme::set_active(entry);
    assert!(total_changed >= 5 * 80, "all five gallery cells must draw");
    eprintln!("toast five-shot vision smoke: changed_pixels={total_changed}");
}

fn render(world: &str, notice: Option<(&str, NoticeKind)>, tag: &str) -> image::RgbaImage {
    let _g = crate::testlock::serial();
    assert!(
        crate::theme::set_active_by_name(world).is_some(),
        "unknown world {world:?}"
    );
    let buf = crate::buffer::Buffer::from_str(&crowded_doc());
    let opts = CaptureOpts {
        notice: notice.map(|(t, k)| (t.to_string(), k)),
        ..CaptureOpts::default()
    };
    let dir = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-notice-law-{}", std::process::id())),
    );
    let png = dir.join(format!("{world}-{tag}.png"));
    capture_with(&png, &buf, &opts).expect("the notice capture renders");
    image::open(&png).expect("decode notice png").to_rgba8()
}

/// Every pixel that differs between two same-size images.
fn changed(a: &image::RgbaImage, b: &image::RgbaImage) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for y in 0..a.height() {
        for x in 0..a.width() {
            if a.get_pixel(x, y) != b.get_pixel(x, y) {
                out.push((x, y));
            }
        }
    }
    out
}

fn relative_luminance(p: [u8; 4]) -> f64 {
    fn chan(c: u8) -> f64 {
        let c = c as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * chan(p[0]) + 0.7152 * chan(p[1]) + 0.0722 * chan(p[2])
}

fn contrast(a: [u8; 4], b: [u8; 4]) -> f64 {
    let (x, y) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

/// The most common colour among `pixels` in `img` — the PLATE, since glyph ink
/// covers a small minority of a one-line label's own area.
fn mode_colour(img: &image::RgbaImage, pixels: &[(u32, u32)]) -> [u8; 4] {
    let mut counts: std::collections::HashMap<[u8; 4], usize> = std::collections::HashMap::new();
    for &(x, y) in pixels {
        *counts.entry(img.get_pixel(x, y).0).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .expect("a non-empty notice region")
}

/// The INK: the pixel whose luminance is FURTHEST from the plate's. Deliberately
/// not "the darkest pixel" — that reads correctly on a light world and picks the
/// PLATE ITSELF on a dark one, where the ink is the lighter of the two. A law
/// written that way reported a 1.00:1 ratio on the first dark world it met, which
/// is how this comment came to exist.
fn ink_against(img: &image::RgbaImage, pixels: &[(u32, u32)], plate: [u8; 4]) -> [u8; 4] {
    let pl = relative_luminance(plate);
    pixels
        .iter()
        .map(|&(x, y)| img.get_pixel(x, y).0)
        .max_by(|a, b| {
            (relative_luminance(*a) - pl)
                .abs()
                .partial_cmp(&(relative_luminance(*b) - pl).abs())
                .expect("finite luminance")
        })
        .expect("a non-empty notice region")
}

/// The tree's ONE perceptual distance, shared with every other appearance floor
/// (`pixeldiff::delta_e`) — including the footer-plate presence gate in
/// `overlay_plan_law`, which was converted from an absolute 8-bit luminance gap
/// to this oracle for exactly the reasons that function's doc records. The two
/// value-step floors below are the original callers.
use super::pixeldiff::delta_e;

/// The one-pixel ring of a region's bounding box, INSET by one pixel — the notice
/// rim's own solid core, just inside its antialiased outer edge.
///
/// Both offsets are load-bearing and both were learned by measurement. The ring
/// rather than the whole footprint, because the footprint's dominant colour is the
/// FILL, and the fill is exactly the part a collapsed world ramp can make vanish.
/// The one-pixel INSET, because the outermost ring is the rim feathering into the
/// page: sampled there, a `muted` rim on Currawong read as a page-ward blend and
/// the law's own extremum picked up a stray glyph instead, reporting the two kinds
/// ΔE 3.51 apart when their rims are an ink rung apart.
fn rim_ring(px: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let (x0, x1) = (
        px.iter().map(|p| p.0).min().expect("pixels") + 1,
        px.iter()
            .map(|p| p.0)
            .max()
            .expect("pixels")
            .saturating_sub(1),
    );
    let (y0, y1) = (
        px.iter().map(|p| p.1).min().expect("pixels") + 1,
        px.iter()
            .map(|p| p.1)
            .max()
            .expect("pixels")
            .saturating_sub(1),
    );
    let mut out = Vec::new();
    for x in x0..=x1 {
        out.push((x, y0));
        out.push((x, y1));
    }
    for y in y0..=y1 {
        out.push((x0, y));
        out.push((x1, y));
    }
    out
}

/// THE FLOORS. Legibility is the ordinary 4.5:1 WCAG body-text bar, which IS a
/// luminance question. The two value steps are perceptual ΔE, each set UNDER the
/// roster's own tightest real value — and the law reports all three tightest
/// values on every run, so a reader can see the headroom rather than take it on
/// trust. (For scale: ΔE ≈ 2.3 is the classic just-noticeable difference.)
const INK_ON_PLATE_MIN: f64 = 4.5;
/// Shared with `overlay_plan_law`'s footer-plate presence law — the SAME
/// floor, not a re-derived one, because both channels earn it the same way: a
/// value-stepped fill plus a one-pixel rim off the ink ladder.
pub(super) const PLATE_PRESENCE_MIN: f64 = 15.0;
const KIND_DISTINCTION_MIN: f64 = 4.0;

/// THE HELD-NOTICE PLACEMENT LAW. A HELD `Sticky` notice and a self-clearing
/// `Toast` share the SAME authored `toast_anchor` and the SAME `plan_toast`
/// collision planner (decided: two different locations for the two lifetimes
/// was "overkill and kind of bizarre") — so, given identical text and
/// geometry, the two kinds must resolve to the IDENTICAL plate, not merely to
/// two independently-legible ones (the floors law below already covers that
/// weaker claim). Swept over the whole world roster, because `toast_anchor`
/// is a per-world axis.
#[test]
fn a_held_notice_resolves_the_same_placement_as_a_toast_with_the_same_text() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping notice placement law: no wgpu adapter");
        return;
    }
    let (w, h) = CANVAS;
    // The sweep swaps the process-wide world; put it back before the standing
    // guard's own leak check runs, or a green law fails on its own housekeeping.
    let entry_world = theme::active_index();
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name);
        let Some((device, queue, mut p)) = super::headless_dqp(w as f32, h as f32) else {
            break;
        };
        let mut plans = Vec::new();
        for kind in [NoticeKind::Toast, NoticeKind::Sticky] {
            let mut v = super::view(&crowded_doc(), 0, 0);
            v.notice = "changed elsewhere".into();
            v.notice_kind = kind;
            p.set_view(&v);
            p.prepare(&device, &queue, w, h)
                .expect("notice frame prepares");
            let (plate, resolved) = p
                .notice_geometry_probe(w, h)
                .expect("a notice must commit plate geometry");
            plans.push((plate, resolved));
        }
        let ((toast_plate, toast_resolved), (sticky_plate, sticky_resolved)) = (plans[0], plans[1]);
        assert_eq!(
            toast_resolved, sticky_resolved,
            "{}: a Toast resolved {toast_resolved:?} but a Sticky carrying the \
             same text resolved {sticky_resolved:?} — placement is a per-world \
             axis, not a per-lifetime one",
            world.name
        );
        assert_eq!(
            toast_plate, sticky_plate,
            "{}: Toast plate {toast_plate:?} and Sticky plate {sticky_plate:?} \
             differ despite sharing one placement owner",
            world.name
        );
        assert!(
            toast_plate[2] >= 1.0 && toast_plate[3] >= 1.0,
            "{}: presence is vacuous: {toast_plate:?}",
            world.name
        );
    }
    theme::set_active(entry_world);
}

/// THE THREE FLOORS, over every world. See this module's own doc for why one
/// floor would not have been a law at all.
#[test]
fn every_world_seats_both_notice_kinds_on_a_visible_plate_in_legible_ink() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping notice floors law: no wgpu adapter");
        return;
    }
    // The tightest real values on the roster, reported so a reader can see how
    // much headroom the floors above actually have.
    let (mut worst_ink, mut worst_presence, mut worst_kind) = (f64::MAX, f64::MAX, f64::MAX);
    let entry_world = theme::active_index();
    for theme in theme::THEMES.iter() {
        let plain = render(theme.name, None, "plain");
        let mut plates = Vec::new();
        for (kind, tag) in [(NoticeKind::Toast, "toast"), (NoticeKind::Sticky, "sticky")] {
            // The SAME sentence for both kinds, so any difference measured below
            // is the treatment's and never the text's length.
            let shot = render(theme.name, Some(("changed elsewhere", kind)), tag);
            let px = changed(&shot, &plain);
            assert!(
                !px.is_empty(),
                "{} / {tag}: nothing drew at all",
                theme.name
            );
            let plate = mode_colour(&shot, &px);
            let ink = ink_against(&shot, &px, plate);

            let legibility = contrast(ink, plate);
            worst_ink = worst_ink.min(legibility);
            assert!(
                legibility >= INK_ON_PLATE_MIN,
                "{} / {tag}: notice ink {ink:?} on its plate {plate:?} is \
                 {legibility:.2}:1, under the {INK_ON_PLATE_MIN}:1 floor",
                theme.name
            );

            // PRESENCE: the notice's own RIM against the PAGE it covers — the page
            // read as the MODE colour of the no-notice capture over the same
            // pixels, so this is literally "what was there before" rather than a
            // nominal token value. The mode rather than a single sample: the
            // fixture's document is crowded on purpose, so one pixel can land on
            // prose ink and measure the rim against a line of text.
            let page = mode_colour(&plain, &px);
            let r = rim_ring(&px);
            // The rim's own colour, as its MODE over that ring — not the ring's
            // extremum, which a single glyph pixel touching the box can hijack.
            let rim = mode_colour(&shot, &r);
            // PRESENCE is satisfied by EITHER axis, for the same reason the kind
            // distinction is: which one carries it is world-dependent. Potoroo's
            // authored ramp puts the sticky FILL within ΔE 6 of its page while the
            // rim is unmistakable; Mangrove's `muted` RIM sits close to its page
            // while the fill steps clearly. A law on one axis alone would demand a
            // product change on whichever world that axis happened to be weak on —
            // and both of those demands were actually issued, by earlier drafts of
            // this law, before it measured both.
            let presence = delta_e(rim, page).max(delta_e(plate, page));
            worst_presence = worst_presence.min(presence);
            assert!(
                presence >= PLATE_PRESENCE_MIN,
                "{} / {tag}: neither the notice's rim {rim:?} nor its fill \
                 {plate:?} clears ΔE {PLATE_PRESENCE_MIN} from the page {page:?} \
                 it covers (best was {presence:.2}) — a plate that fades into the \
                 page makes the ink floor above REPORT BETTER while drawing \
                 nothing anyone can find",
                theme.name
            );
            plates.push((rim, plate));
        }

        // DISTINCTION: a held notice and a self-clearing one must not look the
        // same. Measured on BOTH axes the treatment uses — the rim's ink rung and
        // the fill's surface plane — and satisfied by either, because which axis
        // carries the distinction is world-dependent: a world that collapses its
        // surface ramp still separates its ink ladder, and a world whose muted and
        // content inks sit close (Gumtree: rim ΔE 5.43) still steps its planes. A
        // law on one axis alone would go quiet on exactly the worlds where that
        // axis is the weak one.
        let ((rim_a, fill_a), (rim_b, fill_b)) = (plates[0], plates[1]);
        let step = delta_e(rim_a, rim_b).max(delta_e(fill_a, fill_b));
        worst_kind = worst_kind.min(step);
        assert!(
            step >= KIND_DISTINCTION_MIN,
            "{}: a toast (rim {rim_a:?}, fill {fill_a:?}) and a sticky (rim \
             {rim_b:?}, fill {fill_b:?}) differ by only ΔE {step:.2} on their best \
             axis (floor {KIND_DISTINCTION_MIN}) — the two kinds are a LIFETIME \
             distinction and a held refusal must not read as an acknowledgement",
            theme.name
        );
    }
    theme::set_active(entry_world);
    eprintln!(
        "notice floors, tightest on the roster: ink {worst_ink:.2}:1, \
         plate presence ΔE {worst_presence:.2}, kind step ΔE {worst_kind:.2}"
    );
}

/// NEVER CLIPPED. A sentence wider than the column is elided to fit, and the
/// sidecar reports the ELIDED form — so the artifact cannot state a message its
/// own pixels do not carry. Swept over canvas widths rather than one, because the
/// column budget is a function of width and an off-by-one in the fit passes at
/// one geometry and fails at another.
#[test]
fn a_notice_wider_than_its_column_is_elided_and_reported_as_drawn() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = super::headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping notice elision law: no wgpu adapter");
        return;
    };
    // Long enough to overflow even the widest geometry below.
    let long = "a very long calm notice sentence that no writing column in this \
                editor is ever wide enough to hold without shortening it first";
    let mut ever_elided = false;
    for w in [420u32, 640, 900, 1200, 1600] {
        let h = 800u32;
        p.set_size(w as f32, h as f32);
        let mut v = super::view(&crowded_doc(), 0, 0);
        v.notice = long.to_string();
        v.notice_kind = NoticeKind::Sticky;
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).expect("prepare");

        let (drawn, _) = p
            .notice_report()
            .expect("a notice was set, so something is drawn");
        assert!(
            !drawn.is_empty(),
            "w={w}: the notice must not vanish entirely under width pressure"
        );
        if drawn != long {
            ever_elided = true;
            assert!(
                drawn.ends_with('…'),
                "w={w}: a shortened notice says so with the shared elision mark, \
                 not by silently truncating: {drawn:?}"
            );
        }
        let (pad_x, _) = crate::render::chrome::notice_plate_padding(
            p.metrics.line_height * crate::markdown::type_scale::LABEL,
        );
        let budget = p.column_width() - 2.0 * pad_x;
        let shaped = p.notice_shaped_width_probe();
        assert!(
            shaped <= budget + 1.0,
            "w={w}: the shaped notice is {shaped:.1}px inside a {budget:.1}px \
             plate budget — it would run off the plate"
        );
    }
    assert!(
        ever_elided,
        "NON-VACUITY: no geometry in the sweep was narrow enough to force an \
         elision, so this law proved nothing about the fit"
    );
}

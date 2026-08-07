//! Overlay summon/dismiss + the bottom-left gutter (page-mode visibility,
//! narrow-window elision, blur-signature invalidation) and the caret-preview
//! panel's appear/close -- split out of the former monolithic
//! `render::tests` (2026-07 code-organization pass). See `chrome_panels` for
//! the spell/replace panels + the rest of the overlay-row contract.

use super::super::*;
use super::{headless_dqp, headless_pipeline, view};

/// PURE GEOMETRY LAW (no GPU) — the secondary column and the selected-row band
/// share ONE y-origin. A right-column buffer is uniform-line-height and leads
/// with `header_rows` empty lines, so uploading it at the plan's own
/// `secondary_top()` must land label N exactly on the band the plan draws for
/// candidate row N — for EVERY header count (0 spell, 1 flat/nav, 2 faceted) and
/// EVERY header gap. This is the invariant the composition-round header gap broke
/// (the right column stayed flush at `text_top`); no element may compute its own
/// row y again.
#[test]
fn overlay_secondary_column_shares_the_band_row_origin() {
    const TEXT_TOP: f32 = 56.0;
    const LH: f32 = 27.2;
    for &header_rows in &[0usize, 1, 2] {
        for &gap in &[0.0f32, 5.0, 15.0] {
            let sec_top = crate::render::plan::test_header_plan(TEXT_TOP, header_rows, gap, LH)
                .secondary_top();
            for r in 0usize..8 {
                // Label N sits at `sec_top + (header_rows + r) leading lines`.
                let label_top = sec_top + (header_rows as f32 + r as f32) * LH;
                let band_top = crate::render::plan::test_row_top(TEXT_TOP, header_rows, gap, r, LH);
                assert!(
                    (label_top - band_top).abs() < 1e-3,
                    "secondary label row {r} (hdr={header_rows}, gap={gap}) at {label_top} \
                     must land on the band at {band_top}"
                );
            }
        }
    }
}

/// WIDTH-SWEEP LAW (item 7) — the summoned card stays fully on-canvas with a
/// margin no smaller than the floor at EVERY window width, for every anchor: the
/// edge inset collapses toward [`chrome::CARD_EDGE_INSET_FLOOR`] as the window
/// tightens, then the card re-centers and fills. Pure (no GPU) so it always
/// runs. Samples very-narrow → wide.
#[test]
fn overlay_card_box_stays_on_canvas_across_the_width_sweep() {
    let anchors = [
        theme::CardAnchor::TopLeft,
        theme::CardAnchor::TopCenter,
        theme::CardAnchor::Inset { x_frac: 0.5 },
        theme::CardAnchor::Inset { x_frac: 1.0 },
    ];
    // Widths from a tight editor window up to a big monitor, both card caps, at
    // both DPI tiers — the policy's floor is a LOGICAL length, so the whole
    // sweep has to run at the scale it is resolved against.
    for &scale in &[1.0f32, 2.0] {
        let floor = chrome::CARD_EDGE_INSET_FLOOR.px(scale);
        for &cap in &[chrome::CARD_MAX_W, chrome::CARD_MAX_W_FACETED] {
            let desired = cap.px(scale);
            for ww in (320u32..=1800).step_by(40) {
                let ww = (ww as f32) * scale;
                for &anchor in &anchors {
                    let (left, w) = chrome::overlay_card_box_policy(anchor, ww, desired, scale);
                    let right = left + w;
                    let ctx = format!("scale={scale} ww={ww} desired={desired} anchor={anchor:?}");
                    assert!(w > 24.0, "{ctx}: card width {w} must leave room for text");
                    assert!(
                        left >= floor - 0.01,
                        "{ctx}: left margin {left} >= floor {floor}"
                    );
                    assert!(
                        right <= ww - floor + 0.01,
                        "{ctx}: right edge {right} must keep a floor margin inside {ww}"
                    );
                    assert!(
                        w <= desired + 0.01,
                        "{ctx}: never wider than desired {desired}"
                    );
                    // FILL REGIME: once the desired width can't seat with floor pads,
                    // the card fills (ww - 2*floor) and re-centers (symmetric margins).
                    if desired > ww - 2.0 * floor {
                        assert!(
                            (w - (ww - 2.0 * floor)).abs() < 0.01,
                            "{ctx}: fill regime card must span the window minus floor pads"
                        );
                        let right_margin = ww - right;
                        assert!(
                            (left - right_margin).abs() < 1.0,
                            "{ctx}: fill regime re-centers (left {left} ~ right {right_margin})"
                        );
                    }
                }
            }
        }
    }
    // WIDE: the top-left card holds the FULL interior-rail inset (item 67 — the
    // card centers near the viewport's one-third mark).
    let (left, _) = chrome::overlay_card_box_policy(
        theme::CardAnchor::TopLeft,
        1200.0,
        chrome::CARD_MAX_W.px(1.0),
        1.0,
    );
    let want_inset = chrome::overlay_rail_inset(1200.0, 1.0);
    assert!(
        (left - want_inset).abs() < 0.01,
        "a wide window seats the card one full rail inset ({want_inset}) in, got {left}"
    );
}

/// ZOOM-AWARE CARD-WIDTH OUTCOME LAW (the user's 200%-palette report: at 200% in
/// a WIDE window every palette row came back brutally elided — "Comp…ion…",
/// "Clean unu…d assets…" — because the 520/600 width caps were device-px
/// constants blind to zoom while the glyphs doubled). Parameterized over zoom
/// (1.0, 1.6, 2.0) the way the popover no-clip laws were parameterized over DPI.
///
/// The card width now scales through the ONE owner [`overlay_card_desired_w`], so
/// on a window with ROOM the primary cells NEVER elide as the type grows;
/// elision fires only when the WINDOW genuinely lacks room. Asserted as an
/// OUTCOME over the shaper's real elision decision fed by the LIVE
/// `overlay_geometry().text_w` ([`TextPipeline::overlay_elided_candidates`], which
/// reruns `full_budget` + `fit_primary` off the true card width — a card-width
/// regression re-elides and trips this). PURE geometry (no GPU frame), so it
/// always runs.
#[test]
fn overlay_card_width_is_zoom_aware_no_elision_when_the_window_has_room() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping overlay_card_width_is_zoom_aware: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    // The real palette's longest labels — the ones the user watched collapse.
    let items: Vec<String> = [
        "Go to file…",
        "Switch project…",
        "Recent projects…",
        "Compare with version…",
        "Clean unused assets…",
        "Toggle typewriter scroll",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    // A chord on the first two rows (a right column exists, like the live palette).
    let binds: Vec<String> = vec![
        "\u{2318}O".into(),
        "\u{2318}\u{21e7}P".into(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];

    // The Cmd-P palette is FACETED — a populated lens strip routes `overlay_geometry`
    // to the faceted card (the exact path the user summoned). Builds the view fresh
    // per (width, zoom) so `set_view` rebuilds the metrics from `v.zoom`.
    let palette = |p: &mut TextPipeline, width: u32, zoom: f32| -> Vec<String> {
        let mut v = view("hello\n", 0, 0);
        v.zoom = zoom;
        v.overlay_active = true;
        v.overlay_title = "commands";
        v.overlay_items = items.clone();
        v.overlay_bindings = binds.clone();
        v.overlay_lens = vec![("All".into(), true), ("File".into(), false)];
        p.set_view(&v);
        p.overlay_elided_candidates(width)
    };

    // ROOM: on the 1200-px canvas the card grows with the glyphs, so nothing elides
    // at ANY of the three zooms — the whole point of the fix.
    for &zoom in &[1.0f32, 1.6, 2.0] {
        let elided = palette(&mut p, 1200, zoom);
        assert!(
            elided.is_empty(),
            "zoom {zoom}: a WIDE (1200px) window has room — no palette primary may \
             elide, but these did: {elided:?}"
        );
    }

    // LEGITIMATE-ELISION CONTROL: a genuinely NARROW window at 200% truly lacks
    // room, so the card fills the window and the shaper DOES elide (correct — the
    // fix must not paper over a real space shortage).
    let narrow = palette(&mut p, 360, 2.0);
    assert!(
        !narrow.is_empty(),
        "zoom 2.0 in a 360px window genuinely lacks room — elision MUST still fire \
         (else the fix is over-widening past the window)"
    );
}

/// Y-AGREEMENT OUTCOME LAW (Wagtail-lesson: assert what the shaped buffers +
/// upload owners actually place, not a mechanism count) — across FLAT and
/// FACETED pickers, both DPIs, and SEVEN worlds (incl. the four Bars poster
/// worlds), every candidate row's PRIMARY name, its SECONDARY chord label, and
/// (for the selected row) the highlight BAND all sit on ONE y; and the amber
/// caret rides the query line. Every element reads the shared owners
/// (`overlay_row_top` / `overlay_secondary_top` / `overlay_query_center`)
/// through [`TextPipeline::overlay_row_y_probe`], the same geometry the render
/// path uploads from — so a shortcut can never ride a half-row high of its row
/// again (the user-reported composition-round bug).
///
/// EVERY-ROW PITCH clause (Firetail ↑/↓ "every second row" report, 2026-07-17):
/// the SELECTED-row band check alone can't catch a shaper-vs-plate PITCH drift
/// — a uniform per-row error would slide the whole list, and only the selected
/// row is band-checked. So this law now asserts EVERY shaped row top equals the
/// pitch the plates step by (`band_top + (k - sel_disp) * lh`). Under `Bars` the
/// row-gap is folded into that one `lh`, so if the shaper ever read a different
/// pitch than the plate/band renderer (the prime suspect for the report),
/// row `k` would deviate and this fails — for Pane AND Bars, at 1× AND 2× DPI
/// (retina, where the report came from), on the real poster fonts (Firetail's
/// Monaspace Xenon). The report did NOT reproduce headlessly; this clause is the
/// standing guard so the class can never regress silently.
#[test]
fn overlay_row_elements_agree_in_y_flat_and_faceted_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping overlay_row_elements_agree_in_y: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    // A binding on EVERY row so both columns are present per candidate row.
    let items: Vec<String> = (0..8).map(|i| format!("Command number {i}")).collect();
    let binds: Vec<String> = (0..8).map(|i| format!("C-{i}")).collect();

    // Sweep BOTH list styles: `Pane` (default) and `Bars` (the no-pane layout,
    // where the flat picker inflates the query line by `header_gap` and cosmic-text
    // half-leads the glyphs down — the full-bleed caret bug lived here).
    let styles = [("pane", None), ("bars", Some(theme::ListStyle::Bars))];
    // Harmless for the "pane" arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    // Retina too: the report was on a HiDPI display, where the row `lh` (and the
    // unscaled Bars `gap` folded into it) shape at 2×. `set_dpi` rebuilds the
    // pipeline metrics exactly like the live app's monitor scale.
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        // ITEM 104 — THE FULL ROSTER, not a hand-picked subset: e10b9fa's original
        // seven (four Bars poster worlds + three calm ones) left eleven worlds
        // unswept, Mopoke (the live Settings "every second row" witness,
        // 2026-07-26) among them — Pane/RenderCaps::DEFAULT, never exercised by
        // the original list. `crate::theme::world_names()` is the SAME ordered
        // roster `THEMES` derives from (law-pinned:
        // `world_names_mirrors_themes_order_exactly`), so a new world is swept
        // for free. The `set_list_style_test_override` below still forces BOTH
        // Pane and Bars on every world regardless of its own `list_style`, so the
        // pitch clause covers both uniformly — but each world also exercises its
        // own real face + facet skin.
        for world in crate::theme::world_names() {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for faceted in [false, true] {
                for (sname, style) in styles {
                    crate::render::set_list_style_test_override(style);
                    let mut v = view("hello\n", 0, 0);
                    v.overlay_active = true;
                    v.overlay_title = "themes";
                    v.overlay_items = items.clone();
                    v.overlay_bindings = binds.clone();
                    v.overlay_selected = 3;
                    if faceted {
                        // Make a real FACET active (index >= 1) so the active-lens
                        // underline is recorded — the C2 y-owner assertion below reads it.
                        v.overlay_lens = vec![("All".into(), false), ("File".into(), true)];
                    }
                    p.set_view(&v);
                    p.prepare(&device, &queue, 1200, 800).unwrap();
                    let pr = p.overlay_row_y_probe();
                    let ctx = format!("world={world} dpi={dpi} faceted={faceted} list={sname}");

                    // Per row: the name and the chord label sit on the same y.
                    for (row, &prim) in &pr.primary {
                        let sec = pr.secondary.get(row).copied().unwrap_or_else(|| {
                            panic!("{ctx}: row {row} has a primary name but no secondary label")
                        });
                        assert!(
                            (prim - sec).abs() <= 1.0,
                            "{ctx}: row {row} primary y={prim} vs secondary y={sec} must agree \
                     (the shortcut must not ride high of its name)"
                        );
                    }
                    // The selected row's band sits on its primary name.
                    let sel_prim = pr.primary.get(&pr.sel_disp).copied().unwrap_or_else(|| {
                        panic!(
                            "{ctx}: selected display row {} has no primary run",
                            pr.sel_disp
                        )
                    });
                    assert!(
                        (sel_prim - pr.band_top).abs() <= 1.0,
                        "{ctx}: selected band top {} must sit on its name top {sel_prim}",
                        pr.band_top
                    );
                    // EVERY-ROW PITCH: the shaped text row `k` must land exactly where the
                    // plates step to it — `band_top + (k - sel_disp) * lh`. This is the
                    // shaper-pitch == plate/band-pitch invariant whose violation reads as
                    // the "every second row" desync. A tolerance of 1px allows sub-pixel
                    // rounding but nothing near a half-row; a per-row drift accumulates and
                    // trips well before the list ends.
                    for (&k, &prim) in &pr.primary {
                        let pitch_expected = pr.band_top + (k as f32 - pr.sel_disp as f32) * pr.lh;
                        assert!(
                            (prim - pitch_expected).abs() <= 1.0,
                            "{ctx}: row {k} text top {prim} must sit on the plate pitch \
                     {pitch_expected} (lh={}, band={}, sel_disp={}) — a drift here is \
                     the shaper reading a different pitch than the plate renderer",
                            pr.lh,
                            pr.band_top,
                            pr.sel_disp
                        );
                    }
                    // The caret rides the query line (centered on its REAL shaped height,
                    // never above/below). On the flat pickers under a beat, that line is
                    // inflated by `header_gap`, so the caret must ride the inflated height,
                    // NOT the bare `lh` — the old `lh`-based centre floated a half-beat high.
                    assert!(
                        pr.caret_center >= pr.query_line_top
                            && pr.caret_center <= pr.query_line_top + pr.query_line_height,
                        "{ctx}: caret center {} must sit on the query line [{}, {}]",
                        pr.caret_center,
                        pr.query_line_top,
                        pr.query_line_top + pr.query_line_height
                    );
                    assert!(
                        (pr.caret_center - (pr.query_line_top + pr.query_line_height * 0.5)).abs()
                            <= 1.0,
                        "{ctx}: caret center {} must be centered on the query line's real height",
                        pr.caret_center
                    );
                    // C2 STRIP-UNDERLINE Y-OWNER LAW (the element round A's law missed):
                    // a `Text`-facet card records an active-lens UNDERLINE; it MUST sit
                    // at/BELOW the strip label's shaped baseline (never mid-glyph — the
                    // Tawny/Firetail strike-through) and stay within the strip row box.
                    // SCOPED to the `Text` facet skin: the poster worlds carry a
                    // `Band`/`Chips` mark whose shape + recording is a SEPARATE (in-flux,
                    // held-back) concern, not this row-geometry law's subject — they are
                    // in the sweep for the row-agreement + pitch clauses above, which do
                    // not depend on the facet skin.
                    if faceted
                        && matches!(
                            crate::render::effective_facet_style(),
                            theme::FacetStyle::Text
                        )
                    {
                        let base = pr.strip_baseline.unwrap_or_else(|| {
                            panic!("{ctx}: a faceted card must expose a strip baseline")
                        });
                        let bottom = pr.strip_line_bottom.unwrap();
                        let uy = pr.strip_underline_y.unwrap_or_else(|| {
                            panic!("{ctx}: an active Text facet must record an underline y")
                        });
                        assert!(
                            uy >= base,
                            "{ctx}: underline y={uy} must sit at/below the strip baseline \
                     {base} (never strike through the label)"
                        );
                        assert!(
                            uy <= bottom + 0.5,
                            "{ctx}: underline y={uy} must stay within the strip row \
                     (bottom {bottom})"
                        );
                    }
                    // INDEPENDENT (non-circular) witness: cosmic-text half-leads the query
                    // glyphs so their baseline sits near the BOTTOM of the (possibly
                    // inflated) line. The caret centre must land a sane ~1/3-row ABOVE that
                    // baseline — covering the x-height — not a half-beat above the whole
                    // line. This is the assertion the full-bleed bug failed: it put the
                    // caret centre a full `header_gap * 0.5` (≈ a third of the *card*)
                    // above the baseline instead of a fraction of one row.
                    let above_baseline = pr.query_baseline - pr.caret_center;
                    assert!(
                        above_baseline > 0.0 && above_baseline <= pr.lh * 0.55,
                        "{ctx}: caret center {} must sit just above the query baseline {} \
                 (0 < {above_baseline} <= {}), not float a half-beat high",
                        pr.caret_center,
                        pr.query_baseline,
                        pr.lh * 0.55
                    );
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
}

#[test]
fn gutter_visible_only_in_page_mode_and_dim_overlay_tracks_takeover() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping gutter_visible_only_in_page_mode: no wgpu adapter");
        return;
    };
    // A named buffer + a NARROW measure so the left margin is wide enough to hold
    // the gutter (the gate also requires a min margin width).
    crate::page::set_measure(40);
    crate::page::set_page_on(true);
    let mut v = view("hello world\n", 0, 0);
    v.gutter_name = "notes.md".to_string();
    v.gutter_project = "awl".to_string();
    p.set_view(&v);
    assert_eq!(
        p.gutter_report(),
        Some(("notes.md".to_string(), "awl".to_string(), false)),
        "page mode + a name + a wide margin => the gutter is drawn"
    );

    // EDGE-TO-EDGE (page off): no margin, so the gutter hides.
    crate::page::set_page_on(false);
    p.set_view(&v);
    assert_eq!(p.gutter_report(), None, "edge-to-edge hides the gutter");

    // An UNNAMED buffer hides the gutter even in page mode.
    crate::page::set_page_on(true);
    let mut blank = view("", 0, 0);
    blank.gutter_name = String::new();
    p.set_view(&blank);
    assert_eq!(p.gutter_report(), None, "no name => no gutter");

    // DIM-OVERLAY tracks a FULL-takeover overlay (not the search split panel).
    let mut over = view("hello\n", 0, 0);
    over.overlay_active = true;
    p.set_view(&over);
    assert!(p.dims_doc(), "a full overlay dims the document behind it");
    let mut peek = view("hello\n", 0, 0);
    peek.search_active = true; // the SPLIT search panel, not a takeover
    p.set_view(&peek);
    assert!(
        !p.dims_doc(),
        "the search split panel keeps the document bright"
    );
}

/// OVERLAY IS INSTANT (no summon/dismiss motion): a summoned card appears at its
/// settled resting geometry immediately, and a close drops it the same frame the
/// view clears `overlay_active` — no rise-in offset, no retained sink-out. Guards
/// the removal of the old overlay-motion round.
#[test]
fn overlay_appears_and_closes_instantly_no_motion() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping overlay_appears_and_closes_instantly_no_motion: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let mut over = view("hello\n", 0, 0);
    over.overlay_active = true;
    over.overlay_items = vec!["Alpha".into(), "Beta".into(), "Gamma".into()];
    p.set_view(&over);

    // OPEN: the card is present at its resting geometry immediately, and advancing
    // the live clock never moves it (nothing is animating the overlay).
    let rest = p.overlay_card_rect().expect("overlay card present");
    assert!(p.dims_doc(), "the overlay is open");
    assert!(
        !p.advance(1.0 / 60.0),
        "an open overlay schedules no motion frames"
    );
    assert_eq!(
        p.overlay_card_rect().unwrap(),
        rest,
        "the card never moves — it appears at its settled position"
    );

    // CLOSE: syncing a view with the overlay logically gone drops the card the SAME
    // frame — no retained sink-out.
    let mut closed = view("hello\n", 0, 0);
    closed.overlay_active = false;
    p.set_view(&closed);
    assert!(!p.dims_doc(), "the overlay closes instantly");
    assert!(
        p.overlay_card_rect().is_none(),
        "the card is gone the same frame"
    );
}

/// THE BUG (user screenshot): at a narrow page-column width the gutter used to
/// lay the raw filename into a fixed-width wrapping box, so a long name
/// WRAPPED mid-word ("DESIGN.md" -> "DESIG" / "N.md") and the fixed-height box
/// clipped the project line right off underneath it. THE FIX (corrected by a
/// taste pass over the first landing): the gutter pre-fits BOTH the filename
/// AND the project line to ONE line EACH through the shared `rowlayout`
/// elision door, sharing the same column-width budget — but fit
/// INDEPENDENTLY. Neither line yields to the other from width pressure; only
/// the hard floor hides the whole gutter.
#[test]
fn narrow_gutter_never_wraps_and_both_lines_elide_independently() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping narrow_gutter_never_wraps_and_both_lines_elide_independently: no wgpu adapter"
        );
        return;
    };

    // A window/measure combo landing the margin comfortably BETWEEN the small
    // collapse floor and the generous ceiling — a real but TIGHT margin, not a
    // degenerate one. Derived from the same pure geometry the pipeline itself
    // uses (not hand-guessed), so a future constant tweak can't silently make
    // this fixture meaningless.
    let window_w = 1700.0;
    let measure = 96usize;
    crate::page::set_measure(measure);
    crate::page::set_page_on(true);
    p.set_size(window_w, 800.0);

    let long_name = "a-fairly-long-descriptive-note-title.md";
    let project = "awl-next";
    let mut v = view("hello world\n", 0, 0);
    v.gutter_name = long_name.to_string();
    v.gutter_project = project.to_string();
    p.set_view(&v);

    // The SAME budget math `gutter_layout` derives, computed here from the
    // pure free functions so the fixture is self-checking.
    let col_left = column_left_for(window_w, CHAR_WIDTH, true, measure, 1.0);
    let gap = CHAR_WIDTH * 1.5;
    let avail = col_left - gap;
    let label_char_w = CHAR_WIDTH * crate::markdown::type_scale::LABEL;
    let avail_chars = (avail / label_char_w).floor().max(0.0) as usize;
    assert!(
        avail_chars > rowlayout::GUTTER_MIN_NAME_CHARS && avail_chars < long_name.chars().count(),
        "fixture must land the gutter in the ELIDING band (hard floor < avail < name), \
         got avail_chars={avail_chars} name_chars={}",
        long_name.chars().count()
    );
    assert!(
        project.chars().count() <= avail_chars,
        "fixture project must be short enough to stay whole at this avail, \
         got avail_chars={avail_chars} project_chars={}",
        project.chars().count()
    );

    let (name, reported_project, _) = p
        .gutter_report()
        .expect("a tight-but-real margin still shows the gutter");
    // (1) THE FIX: the filename is ALWAYS one line — never mid-word wrapped —
    // and the sidecar reports EXACTLY what was drawn.
    assert!(
        !name.contains('\n'),
        "the filename must render on ONE line, got {name:?}"
    );
    assert!(
        name.chars().count() <= avail_chars,
        "the reported name must fit the same budget the pixels draw at, got {name:?} (budget {avail_chars})"
    );
    assert_ne!(
        name, long_name,
        "a name this long in this margin must actually elide"
    );
    assert!(
        name.ends_with(".md"),
        "elision preserves the extension: {name:?}"
    );
    // (2) THE CORRECTION: the project line does NOT yield just because the
    // filename is eliding — it stays visible, fit independently against the
    // SAME budget. Here it's short enough to still show whole.
    assert_eq!(
        reported_project, project,
        "the project must keep showing (fit independently) alongside an eliding filename"
    );

    // A SHORT name at this SAME narrow margin is never elided (elision is the
    // last resort) — the fixture isn't just "narrow enough to hide everything".
    let mut short = view("hello world\n", 0, 0);
    short.gutter_name = "short.md".to_string();
    short.gutter_project = project.to_string();
    p.set_view(&short);
    let (short_name, short_project, _) = p
        .gutter_report()
        .expect("a short name always fits this margin");
    assert_eq!(short_name, "short.md", "a short name is never elided");
    assert_eq!(
        short_project, project,
        "a short name leaves plenty of room for the project too"
    );

    // The SYMMETRIC case: a genuinely long PROJECT elides independently too,
    // while a short filename stays whole right alongside it — proving the
    // correction isn't just "name always wins."
    let long_project = "a-fairly-long-project-directory-name";
    assert!(
        avail_chars < long_project.chars().count(),
        "fixture must also land the project in its own eliding band, \
         got avail_chars={avail_chars} project_chars={}",
        long_project.chars().count()
    );
    let mut swapped = view("hello world\n", 0, 0);
    swapped.gutter_name = "short.md".to_string();
    swapped.gutter_project = long_project.to_string();
    p.set_view(&swapped);
    let (swapped_name, elided_project, _) = p
        .gutter_report()
        .expect("a tight-but-real margin still shows the gutter");
    assert_eq!(
        swapped_name, "short.md",
        "the short name is unaffected by the project eliding"
    );
    assert_ne!(
        elided_project, long_project,
        "a project this long in this margin must actually elide"
    );
    assert!(elided_project.chars().count() <= avail_chars);
    assert!(
        !elided_project.contains('\n'),
        "the project must render on ONE line too"
    );
}

/// ITEM 307 — THE GUTTER'S VISIBILITY GATE READS A LOGICAL QUANTITY, NOT A
/// DEVICE-PIXEL ONE. Found by item 242's residual lane: `--capture-dpi 2`
/// reported `gutter.visible: false` at the SAME `--measure` where `--capture-dpi
/// 1` showed it drawn, on the SAME 1200x800 *device* canvas. That turned out to
/// be a correct decline, not a bug — a fixed device canvas shows LESS logical
/// content at a higher dpi (`WxH` at dpi N is a `(W/N)x(H/N)` logical window,
/// per `--capture-dpi`'s own contract), so the margin the gate reads is
/// genuinely narrower. This law is the proof: `gutter_layout`'s `avail_chars`
/// is a ratio of `column_left()` and `label_char_w`, both of which scale by
/// `Metrics::scale` (`zoom * dpi`) alike, so growing the PHYSICAL canvas in
/// lockstep with dpi (`(logical_w*dpi) x (logical_h*dpi)`) holds the LOGICAL
/// page fixed and must reproduce the exact same gate decision at every dpi —
/// swept across the `--measure` range so the boundary is crossed (asserted on
/// BOTH sides: a one-sided sweep would pass on a gate that never turns on),
/// over two independent logical windows and three dpi tiers.
#[test]
fn gutter_visibility_boundary_is_dpi_invariant_at_matched_logical_geometry() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping gutter_visibility_boundary_is_dpi_invariant_at_matched_logical_geometry: \
             no wgpu adapter"
        );
        return;
    };
    crate::page::set_page_on(true);

    for &(logical_w, logical_h) in &[(1200.0f32, 800.0f32), (900.0f32, 700.0f32)] {
        let visible_at = |p: &mut TextPipeline, measure: usize, dpi: f32| -> bool {
            crate::page::set_measure(measure);
            // The physical canvas grows WITH dpi so the LOGICAL page — the thing
            // the gate is supposed to reason about — never moves.
            p.set_size(logical_w * dpi, logical_h * dpi);
            p.set_dpi(dpi);
            let mut v = view("hello world\n", 0, 0);
            v.gutter_name = "notes.md".to_string();
            v.gutter_project = "awl".to_string();
            p.set_view(&v);
            p.gutter_report().is_some()
        };

        let mut saw_visible = false;
        let mut saw_hidden = false;
        for measure in 10..=100usize {
            let v1 = visible_at(&mut p, measure, 1.0);
            let v2 = visible_at(&mut p, measure, 2.0);
            let v3 = visible_at(&mut p, measure, 3.0);
            saw_visible |= v1;
            saw_hidden |= !v1;
            assert_eq!(
                v1, v2,
                "logical {logical_w}x{logical_h} measure={measure}: dpi 1 visible={v1} \
                 but the SAME logical page at matched dpi 2 visible={v2}"
            );
            assert_eq!(
                v1, v3,
                "logical {logical_w}x{logical_h} measure={measure}: dpi 1 visible={v1} \
                 but the SAME logical page at matched dpi 3 visible={v3}"
            );
        }
        assert!(
            saw_visible && saw_hidden,
            "logical {logical_w}x{logical_h}: the measure sweep never crossed the gate \
             (saw_visible={saw_visible} saw_hidden={saw_hidden}) — the boundary must be \
             crossed for this law to prove anything"
        );
    }
}

/// FIX: `blur_signature` must invalidate on a PAGE/WRAP geometry change — a page
/// drag, `C-x {`/`}`, or a page-mode toggle re-wraps the document (`set_size` /
/// `sync_wrap_width`) WITHOUT bumping `reshape_count` (that only fires on a text
/// reshape), so before this fix the cached frosted backdrop stayed stale, showing
/// the OLD column behind a freshly-reopened overlay. `row_geom.generation()` is
/// bumped by `RowGeom::invalidate` exactly when the shaped runs actually re-wrap,
/// and `page::page_on()`/`page::measure()` cover the rare case where the page
/// flags flip without the wrap width itself changing.
#[test]
fn blur_signature_invalidates_on_page_geometry_change_not_on_a_no_op_frame() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping blur_signature_invalidates_on_page_geometry_change: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(false);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    p.set_size(1200.0, 800.0);
    let sig_edge_to_edge = p.blur_signature(1200, 800);

    // A NO-OP frame (same size, same page state, no text edit): the signature
    // must NOT change — this is the "settled overlay-open frame re-blurs
    // nothing" guarantee (a caret spring alone must never invalidate it).
    p.set_size(1200.0, 800.0);
    let sig_no_op = p.blur_signature(1200, 800);
    assert_eq!(
        sig_edge_to_edge, sig_no_op,
        "an unchanged page/wrap state must not perturb the blur signature"
    );

    // PAGE-MODE TOGGLE + a narrower measure re-wraps the document at a new
    // column width: the signature must invalidate.
    crate::page::set_page_on(true);
    crate::page::set_measure(40);
    p.set_size(1200.0, 800.0);
    let sig_page_on_narrow = p.blur_signature(1200, 800);
    assert_ne!(
        sig_edge_to_edge, sig_page_on_narrow,
        "toggling page mode (a real wrap-width change) must invalidate the blur signature"
    );

    // A MEASURE-ONLY change (still in page mode) re-wraps again: must invalidate
    // once more.
    crate::page::set_measure(60);
    p.set_size(1200.0, 800.0);
    let sig_measure_wider = p.blur_signature(1200, 800);
    assert_ne!(
        sig_page_on_narrow, sig_measure_wider,
        "a measure-only change must also invalidate the blur signature"
    );
}
#[test]
fn blur_signature_invalidates_when_the_live_world_phase_changes() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping blur_signature phase law: no wgpu adapter");
        return;
    };
    let before = p.blur_signature(1200, 800);
    p.advance_lava(crate::lava::LAVA_TICK_SECONDS);
    let after = p.blur_signature(1200, 800);
    assert_ne!(
        before, after,
        "a new lava phase must invalidate the frost source"
    );
}

/// The CARET-STYLE preview PANEL: it appears BELOW the picker (a floating card with
/// the settled sample line + an animated caret) while the caret-style picker is
/// open, and PARKS (nothing drawn, demo reset) the instant it closes — the panel
/// primitive's elevation quads and the demo caret all go empty (DESIGN §6 idle).
#[test]
fn caret_preview_panel_appears_below_picker_and_stops_on_close() {
    // The pipeline construction reads theme globals and the law mutates them below;
    // acquire BEFORE device/pipeline work, not after it.
    let _g = crate::testlock::serial();
    // Build a headless pipeline but KEEP the device/queue so we can drive `prepare`
    // (the elevation-quad instance counts are only set during prepare).
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping caret_preview_panel_appears_below_picker_and_stops_on_close: no wgpu adapter"
        );
        return;
    };

    // OPEN the caret-style picker (the familiar Block/Morph/I-beam list), Block row
    // highlighted. Headless: pin the deterministic SETTLED end-state (the loop is
    // live-only), then prepare the frame.
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = vec!["Block".into(), "Morph".into(), "I-beam".into()];
    v.overlay_selected = 0;
    v.overlay_hint = "Enter apply".to_string();
    v.caret_preview = Some(crate::caret::CaretMode::Block);
    p.set_view(&v);
    p.settle_caret_preview();
    p.prepare(&device, &queue, 1200, 800).unwrap();

    // The panel is present, holds the FULL sample line (settled), is a non-degenerate
    // ~2-line box, and hangs clearly BELOW the picker card (whose top is y≈52).
    let (rect, text, _beat, silhouette) = p
        .caret_preview_panel_report()
        .expect("the preview panel is summoned with the picker");
    assert_eq!(
        text,
        crate::caret::SAMPLE,
        "the settled panel shows the full sample line"
    );
    assert!(!silhouette, "Block never paints the Morph silhouette");
    assert!(
        rect[2] > 300.0,
        "the panel spans the picker width: {rect:?}"
    );
    assert!(
        rect[3] > p.metrics.line_height,
        "a two-line-tall box: {rect:?}"
    );
    assert!(
        rect[1] > 52.0 + 3.0 * p.metrics.line_height,
        "the panel floats below the picker card: {rect:?}"
    );
    // The panel primitive's elevation quads + the demo caret are all drawn. NO
    // drop shadow (dark-depth Option C, 2026-07-22): the shadow quad is
    // retired outright, on every world — the border's own muted surface-step
    // rim + the card's value step carry the depth (DESIGN §5).
    assert_eq!(
        p.float_card.instance_count(),
        1,
        "the float card is summoned"
    );
    assert_eq!(
        p.float_shadow.instance_count(),
        0,
        "no drop shadow — retired (dark-depth Option C)"
    );
    assert_eq!(
        p.float_border.instance_count(),
        1,
        "and a crisp raised edge"
    );
    assert!(
        p.caret_preview_pipeline.is_drawn(),
        "the demo caret rides the sample line"
    );

    // CLOSE the picker: the panel + caret park (nothing drawn), the demo resets.
    let closed = view("hello world\n", 0, 0);
    p.set_view(&closed);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        p.caret_preview_panel_report().is_none(),
        "no panel once the picker is closed"
    );
    assert_eq!(
        p.float_card.instance_count(),
        0,
        "float card parked on close"
    );
    assert_eq!(p.float_shadow.instance_count(), 0, "shadow parked on close");
    assert_eq!(p.float_border.instance_count(), 0, "border parked on close");
    assert!(
        !p.caret_preview_pipeline.is_drawn(),
        "preview caret parked on close"
    );
}

/// ITEM 119 — every authored world must keep the caret-preview float alive below
/// a picker, even when the list skin is forced to the card-less Bars treatment.
/// This sweeps the real roster and independently forces both list layouts at both
/// monitor scales; the report proves state while the float/border/demo counts prove
/// the frame actually retained the shared GPU trio.
#[test]
fn caret_preview_float_owner_sweeps_world_style_and_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping caret_preview_float_owner_sweeps_world_style_and_dpi: no wgpu adapter");
        return;
    };
    let styles = [
        ("Pane", Some(theme::ListStyle::Pane)),
        ("Bars", Some(theme::ListStyle::Bars)),
    ];
    // Harmless for the "Pane" arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        for world in crate::theme::world_names() {
            crate::theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for (style_name, style) in styles {
                crate::render::set_list_style_test_override(style);
                let mut v = view("hello world\n", 0, 0);
                v.overlay_active = true;
                v.overlay_crisp = true;
                v.overlay_items = vec!["Block".into(), "Morph".into(), "I-beam".into()];
                v.overlay_selected = 0;
                v.caret_preview = Some(crate::caret::CaretMode::Block);
                p.set_view(&v);
                p.settle_caret_preview();
                p.prepare(&device, &queue, 1200, 800).unwrap();
                let ctx = format!("world={world} style={style_name} dpi={dpi}");
                assert!(
                    p.caret_preview_panel_report().is_some(),
                    "{ctx}: preview report"
                );
                assert_eq!(p.float_card.instance_count(), 1, "{ctx}: float card");
                assert_eq!(p.float_border.instance_count(), 1, "{ctx}: float border");
                assert!(p.caret_preview_pipeline.is_drawn(), "{ctx}: demo caret");

                p.set_view(&view("hello world\n", 0, 0));
                p.prepare(&device, &queue, 1200, 800).unwrap();
                assert_eq!(p.float_card.instance_count(), 0, "{ctx}: close parks card");
                assert_eq!(
                    p.float_border.instance_count(),
                    0,
                    "{ctx}: close parks border"
                );
                assert_eq!(
                    p.float_shadow.instance_count(),
                    0,
                    "{ctx}: close parks shadow"
                );
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// The forced `list_style` is a process global: its reader and writer must both
/// remain behind the one serial guard, so an override cannot leak across a parallel
/// render test's frame window.
#[test]
fn list_style_override_reader_writer_are_serialized() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    let outer = crate::testlock::serial();
    crate::render::set_list_style_test_override(Some(theme::ListStyle::Pane));
    let barrier = Arc::new(Barrier::new(2));
    let entered = Arc::new(AtomicBool::new(false));
    let worker = {
        let barrier = barrier.clone();
        let entered = entered.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let _g = crate::testlock::serial();
            crate::render::set_list_style_test_override(Some(theme::ListStyle::Bars));
            assert!(matches!(
                crate::render::effective_list_style(),
                theme::ListStyle::Bars
            ));
            entered.store(true, Ordering::SeqCst);
            crate::render::set_list_style_test_override(None);
        })
    };
    barrier.wait();
    std::thread::sleep(std::time::Duration::from_millis(25));
    assert!(
        !entered.load(Ordering::SeqCst),
        "writer/reader must wait behind the outer guard"
    );
    // Each window clears its own override BEFORE releasing the lock. Without
    // that, the forced value outlives both windows and the next thread through
    // the mutex reads it — which is how a forced `Bars` once reached an
    // unrelated law in another file.
    crate::render::set_list_style_test_override(None);
    drop(outer);
    worker.join().unwrap();
}

/// PARK-ON-CLOSE: a CLOSED summoned overlay must leave ZERO stale overlay
/// pixels for the next frame — the exact live repro is OPEN palette → Esc →
/// HOLD Option-Cmd-I (the stats HUD), where the HUD forces the frosted-blur backdrop
/// path that draws the overlay card UNCONDITIONALLY. So after the overlay
/// closes the text renderer must carry no glyphs and every overlay quad must
/// be parked (0 instances), regardless of HUD state.
#[test]
fn closed_overlay_parks_text_and_quads_even_while_the_hud_is_held() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping closed_overlay_parks_text_and_quads_even_while_the_hud_is_held: no wgpu adapter"
        );
        return;
    };

    // OPEN a command-palette-style overlay with a few rows, one selected.
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = vec![
        "Go to file…".into(),
        "Switch project…".into(),
        "Finish file".into(),
    ];
    v.overlay_selected = 0;
    v.overlay_hint = "↵ run  ←/→ lens".to_string();
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    // The overlay is drawn: the card + a selected-row band + real glyphs. The
    // default (Pane) world SPLITS its card into two surfaces (SPLIT-PANE round),
    // so the fill quad count is the pane-fill count (2 split / 1 unified), not a
    // fixed 1 — either way non-zero while open, parked to 0 on close below.
    assert_eq!(
        p.panel_card.instance_count() as usize,
        p.overlay_pane_fills_probe().len(),
        "the overlay card fill(s) are drawn while open"
    );
    assert!(
        p.panel_card.instance_count() >= 1,
        "at least one card surface is drawn"
    );
    assert_eq!(
        p.overlay_rows.instance_count(),
        1,
        "the selected-row band is drawn"
    );
    assert!(
        p.overlay_text_glyph_count() > 0,
        "the overlay text carries the palette rows while open"
    );

    // CLOSE the overlay AND hold the stats HUD — the exact live repro that
    // forces the frosted-blur path (which draws the overlay card
    // unconditionally). The overlay must now be fully parked anyway.
    crate::hud::set_held(true);
    let closed = view("hello world\n", 0, 0);
    p.set_view(&closed);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    crate::hud::set_held(false);

    assert_eq!(
        p.overlay_text_glyph_count(),
        0,
        "the closed overlay's text renderer carries no stale palette glyphs"
    );
    assert_eq!(
        p.panel_card.instance_count(),
        0,
        "the card quad is parked on close"
    );
    assert_eq!(
        p.overlay_rows.instance_count(),
        0,
        "the row band is parked on close"
    );
    assert_eq!(
        p.overlay_lens_underline.instance_count(),
        0,
        "the theme-lens underline is parked on close"
    );
    assert!(
        !p.panel_caret.is_drawn(),
        "the amber query caret is parked on close"
    );
}

/// EMPTY STATE (pass 3): a picker with NO candidate rows draws ONE dim message
/// row (the shared `overlay_empty` text) in the candidate area — the card grows a
/// row for it, the shaped panel actually carries the message glyphs, and NO
/// selected-row highlight band is drawn (the message is not selectable). A picker
/// WITH rows reserves no such row (regression guard).
#[test]
fn overlay_empty_state_draws_a_dim_message_row() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping overlay_empty_state_draws_a_dim_message_row: no wgpu adapter");
        return;
    };

    // A go-to picker with a query but NO matching rows → the shared "no matches".
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = Vec::new();
    v.overlay_query = "zzz".into();
    v.overlay_empty = Some("no matches".to_string());
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    // The card reserves a candidate row for the message (query + 1 message row,
    // no hint set here) and the shaped panel carries the message text.
    let joined: String = p
        .panel_buffer
        .lines
        .iter()
        .map(|l| l.text().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("no matches"),
        "shaped panel shows the message: {joined:?}"
    );
    // No selected-row highlight band: the empty-state message is not selectable.
    assert_eq!(
        p.overlay_rows.instance_count(),
        0,
        "no highlight band over an empty-state message"
    );

    // Regression: a picker WITH rows draws no empty-state message.
    let mut v2 = view("hello\n", 0, 0);
    v2.overlay_active = true;
    v2.overlay_crisp = true;
    v2.overlay_items = vec!["alpha.md".into()];
    v2.overlay_empty = None;
    p.set_view(&v2);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let joined2: String = p
        .panel_buffer
        .lines
        .iter()
        .map(|l| l.text().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !joined2.contains("no matches"),
        "no message row when there are rows"
    );
}

/// The CARET-STYLE preview PANEL, MORPH highlighted: the settled demo caret
/// actually paints the glyph-SILHOUETTE (the preview's OWN `CaretGlyphPipeline`,
/// never the document's), not a permanent thin bar — the picker's one job is to
/// demonstrate what the highlighted look does to real text, and Morph's whole
/// point is the recolored letter, not a bar. Closing the picker parks it too.
#[test]
fn caret_preview_panel_morph_paints_the_glyph_silhouette() {
    // This preview test reaches the list-style reader through overlay preparation;
    // take the same guard before creating its device/pipeline.
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping caret_preview_panel_morph_paints_the_glyph_silhouette: no wgpu adapter"
        );
        return;
    };

    // OPEN the caret-style picker with MORPH highlighted; settle (headless: the
    // choreography loop is live-only) to the fully-typed sample line at rest.
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = vec!["Block".into(), "Morph".into(), "I-beam".into()];
    v.overlay_selected = 1;
    v.overlay_hint = "Enter apply".to_string();
    v.caret_preview = Some(crate::caret::CaretMode::Morph);
    p.set_view(&v);
    p.settle_caret_preview();
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let (_rect, text, _beat, silhouette) = p
        .caret_preview_panel_report()
        .expect("the preview panel is summoned with the picker");
    assert_eq!(
        text,
        crate::caret::SAMPLE,
        "settled: the full sample line, caret at rest"
    );
    // Settled at rest on a real letter (the sample ends "...morph", a real glyph
    // one back of the insertion point): the SILHOUETTE pipeline paints (reported
    // straight from the sidecar-facing seam), and the plain block/bar pipeline is
    // suppressed so the two never double-draw.
    assert!(
        silhouette,
        "Morph, settled on a real glyph, must paint the preview's own silhouette"
    );
    assert!(
        p.caret_preview_glyph_pipeline.is_drawn(),
        "the pipeline behind the report is genuinely holding an instance"
    );
    assert!(
        !p.caret_preview_pipeline.is_drawn(),
        "the block/bar pipeline is suppressed while the silhouette paints"
    );

    // CLOSE the picker: both preview caret pipelines park.
    let closed = view("hello world\n", 0, 0);
    p.set_view(&closed);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        !p.caret_preview_glyph_pipeline.is_drawn(),
        "silhouette parked once the picker closes"
    );
    assert!(
        !p.caret_preview_pipeline.is_drawn(),
        "block/bar caret parked too"
    );
}

/// **THE PERSISTENT `changed elsewhere` AFFORDANCE** — the state it reports and
/// the shape it adds. (Its INK, and the pointer targets under it, are the
/// sibling law below, asserted over real pixels.)
///
///   1. **State.** `gutter_report` carries the flag, so a capture can assert the
///      affordance is up without reading pixels — and it is `false` on an
///      ordinary document, so the assertion is reading a signal, not a constant.
///   2. **Shape.** The block grows by exactly one LABEL row and stays
///      bottom-anchored, and every geometry consumer (carve, frost seeds,
///      hit-test) grows with it, because they all read the one `lines()` owner.
#[test]
fn the_changed_elsewhere_affordance_reports_and_grows_the_block() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let (w, h) = (1200u32, 800u32);
    let Some((_device, _queue, mut p)) = super::headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the changed-elsewhere affordance law: no wgpu adapter");
        return;
    };
    crate::page::set_measure(40);
    crate::page::set_page_on(true);

    let base = |changed: bool| {
        let mut v = view("hello world\n", 0, 0);
        v.gutter_name = "notes.md".to_string();
        v.gutter_project = "awl".to_string();
        v.gutter_changed = changed;
        v
    };

    // ── (1) STATE, both ways round.
    p.set_view(&base(false));
    let (name, project, quiet) = p.gutter_report().expect("the gutter is drawn");
    assert_eq!((name.as_str(), project.as_str()), ("notes.md", "awl"));
    assert!(!quiet, "an ordinary document has no affordance");
    let calm_rect = p.gutter_carve_rect(h).expect("a drawn gutter carves");
    let calm_seeds = p.gutter_frost_seeds(h).len();

    p.set_view(&base(true));
    let (name, project, loud) = p.gutter_report().expect("the gutter is still drawn");
    assert_eq!(
        (name.as_str(), project.as_str()),
        ("notes.md", "awl"),
        "the affordance joins the block; it never displaces a line"
    );
    assert!(loud, "…and the sidecar can see it");

    // ── (2) SHAPE: one more row, still bottom-anchored, and the derived
    //        geometry followed. A consumer that kept its own `if project…` count
    //        would leave its rect a row short here.
    let loud_rect = p.gutter_carve_rect(h).expect("a drawn gutter carves");
    assert_eq!(
        loud_rect[3], calm_rect[3],
        "the block stays anchored to the same bottom edge"
    );
    assert!(
        loud_rect[1] < calm_rect[1],
        "the carve must grow UPWARD by the new row: {loud_rect:?} vs {calm_rect:?}"
    );
    let row_h = p.metrics.line_height * crate::markdown::type_scale::LABEL;
    let grew = calm_rect[1] - loud_rect[1];
    assert!(
        (grew - row_h).abs() < 1.0,
        "exactly one LABEL row taller (grew {grew}, row {row_h})"
    );
    assert!(
        p.gutter_frost_seeds(h).len() > calm_seeds,
        "the lava frost seeds follow the block's own line list"
    );
}

/// The affordance's APPEARANCE, asserted over the PNG's pixels (CLAUDE.md's
/// Wagtail tripwire) rather than inferred from the state above: it is drawn in a
/// STRONGER ink than the filename beneath it — a three-step value ladder, no new
/// accent — and it is a LABEL, so the pointer targets under it do not shift.
#[test]
fn the_changed_elsewhere_affordance_reads_stronger_than_the_name_it_sits_over() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = super::headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the changed-elsewhere ink law: no wgpu adapter");
        return;
    };
    crate::page::set_measure(40);
    crate::page::set_page_on(true);
    let mut v = view("hello world\n", 0, 0);
    v.gutter_name = "notes.md".to_string();
    v.gutter_project = "awl".to_string();
    v.gutter_changed = true;
    p.set_view(&v);

    let row_h = p.metrics.line_height * crate::markdown::type_scale::LABEL;
    let block_top = h as f32 - row_h * 3.0 - 8.0;
    p.prepare(&device, &queue, w, h).unwrap();
    let px = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    // The darkest ink drawn in a band, as a luminance — the glyph strokes are
    // antialiased, so a percentile-free extreme is the honest reading of "how
    // dark did this row get" (the `mac_about` ink lesson, one axis simpler
    // because both rows here are the same face at the same size).
    let ink_of = |row: f32| -> f32 {
        let y0 = (block_top + row * row_h).round().max(0.0) as u32;
        let y1 = ((block_top + (row + 1.0) * row_h).round() as u32).min(h);
        let mut best = f32::MAX;
        let mut ground = f32::MIN;
        for y in y0..y1 {
            for x in 0..(p.column_left().max(1.0) as u32).min(w) {
                let c = px[(y * w + x) as usize];
                let l = 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
                best = best.min(l);
                ground = ground.max(l);
            }
        }
        assert!(
            ground - best > 8.0,
            "row {row} drew no ink at all (range {best}..{ground}) — this law \
             would be vacuous"
        );
        best
    };
    let affordance = ink_of(0.0);
    let filename = ink_of(1.0);
    let project_ink = ink_of(2.0);
    assert!(
        affordance < filename,
        "the affordance must read STRONGER than the filename beneath it \
         (ink {affordance} vs {filename}) — it is the one line here that is news"
    );
    assert!(
        filename < project_ink,
        "…and the existing two-step ladder is unchanged beneath it \
         (name {filename} vs project {project_ink})"
    );

    // …and the affordance is a LABEL, not a target: the rows below it still
    // hit-test as themselves, so nothing a pointer could reach moved.
    let mid = |row: f32| block_top + (row + 0.5) * row_h;
    let at = |row: f32| p.gutter_context_target(4.0, mid(row), h);
    assert_eq!(
        at(0.0),
        None,
        "the affordance names a state; the things you can do about it are named \
         palette rows, not a click here"
    );
    assert_eq!(
        at(1.0),
        Some(crate::context_menu::ContextTarget::Filename),
        "the filename row still targets the filename"
    );
    assert_eq!(
        at(2.0),
        Some(crate::context_menu::ContextTarget::Folder),
        "and the project row still targets the folder"
    );
}

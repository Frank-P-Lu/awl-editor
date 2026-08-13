use super::super::*;

#[test]
fn worlds_eleven_dark_nine_light() {
    assert_eq!(THEMES.len(), 20);
    let dark = THEMES.iter().filter(|t| t.dark).count();
    let light = THEMES.iter().filter(|t| !t.dark).count();
    // 11 dark (Tawny/Mopoke/Currawong/Potoroo/Bombora/Bowerbird/Mulga/
    // Mangrove/Wagtail/Firetail/Cassowary) / 9 light (Gumtree/Bilby/Saltpan/
    // Quokka/Galah/Magpie/Brolga/Paperbark/Kite). Brolga is the COOL LIGHT
    // POLE, a pale sky-blue world; Cassowary is the NERV-terminal statement
    // world; Paperbark is the handmade-paper studio and the roster's only
    // `Background::Deckle` ground; Kite is the LIGHT statement world,
    // travelling through a `Background::WarpedGrid` tunnel and the deliberate
    // counterpart to dark warm Firetail. Twenty is PHILOSOPHY.md's authored
    // roster target, and this roster sits exactly at it.
    assert_eq!(dark, 11);
    assert_eq!(light, 9);
}

#[test]
fn every_toast_anchor_is_authored_by_the_world_roster() {
    let mut counts = std::collections::HashMap::new();
    for world in THEMES {
        *counts.entry(world.toast_anchor).or_insert(0usize) += 1;
    }
    assert_eq!(
        counts.len(),
        ToastAnchor::ALL.len(),
        "every toast-anchor arm must be carried by at least one world"
    );
    for anchor in ToastAnchor::ALL {
        assert!(
            counts.get(&anchor).copied().unwrap_or(0) > 0,
            "toast anchor {anchor:?} is dormant"
        );
    }
    eprintln!("toast anchor roster: {counts:?}");
}

/// `world_names()` (the one code-owned roster source, read by `--help`,
/// the unknown-`--theme` error, and `--list-worlds`) is exactly `THEMES`'
/// names in `THEMES`' own order — no reordering, no drift, no duplicate.
#[test]
fn world_names_mirrors_themes_order_exactly() {
    let names = super::world_names();
    assert_eq!(names.len(), THEMES.len());
    for (name, theme) in names.iter().zip(THEMES.iter()) {
        assert_eq!(*name, theme.name);
    }
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        names.len(),
        "world_names() contains a duplicate"
    );
}

/// `Theme::is_one_bit` — Wagtail's 2026-07 rework, from greyscale (any grey
/// permitted) to a true 1-bit world (only pure black/white) — is `true` for
/// Wagtail alone, and (the stricter sub-case relationship) every one-bit
/// world is ALSO monochrome (`is_monochrome`'s broader "no hue" signal).
#[test]
fn wagtail_alone_is_one_bit() {
    let one_bit: Vec<&str> = THEMES
        .iter()
        .filter(|t| t.is_one_bit())
        .map(|t| t.name)
        .collect();
    assert_eq!(one_bit, ["Wagtail"], "exactly Wagtail should be one-bit");
    for t in THEMES.iter().filter(|t| t.is_one_bit()) {
        assert!(
            t.is_monochrome(),
            "{}: a one-bit world must also be monochrome",
            t.name
        );
    }
}

#[test]
fn default_is_saltpan() {
    // 2026-07-11 taste round: Saltpan (a warm light world) is awl's first
    // impression now, not the original dark Tawny (see `DEFAULT_THEME`'s doc).
    assert!(!THEMES[DEFAULT_THEME].dark);
    assert_eq!(THEMES[DEFAULT_THEME].name, "Saltpan");
}

/// DEBT-AUDIT LAW (2026-07-18) — INDEX-VS-NAME world access. A world INSERTED
/// mid-roster must not change any OTHER world's behaviour or a user's PERSISTED
/// selection. The two things that could break on such an insertion are:
///   (1) a position-derived constant (only `DEFAULT_THEME`, now name-derived via
///       `world_index("Saltpan")`), and
///   (2) the sticky-theme round-trip, which stores a NAME (`config.toml`'s
///       `theme` key via `App::persist_theme` → `Config::apply_sticky_globals` →
///       `set_active_by_name`), never an array index.
/// This law pins BOTH so a future roster insert can't silently repoint the
/// default or resurface a user under a different world:
///   - names are UNIQUE (name-addressing is well-defined),
///   - EVERY world round-trips through `set_active_by_name` back to itself
///     (the persisted-selection path is position-independent for all worlds),
///   - the default is name-derived (so a FRESH launch is insertion-stable too),
///   - a NON-world name is `None` (a stale/retired name falls back leniently,
///     never a crash and never a neighbour by position).
#[test]
fn roster_position_is_name_stable() {
    let _g = crate::testlock::serial();

    // (1) Names are unique — name-addressing has exactly one target per name.
    for (i, a) in THEMES.iter().enumerate() {
        for b in THEMES.iter().skip(i + 1) {
            assert_ne!(a.name, b.name, "two worlds share the name {:?}", a.name);
        }
    }

    // (2) Persisted selection is a NAME: every world round-trips to ITSELF
    // regardless of its array position, so inserting a world before/after any
    // other cannot change which world that other's remembered name reopens.
    for t in THEMES.iter() {
        let got = set_active_by_name(t.name)
            .unwrap_or_else(|| panic!("{} unreachable by its own name", t.name));
        assert_eq!(got.name, t.name);
        // Case-insensitive too (the config value is compared ASCII-insensitively).
        assert_eq!(
            set_active_by_name(&t.name.to_ascii_lowercase())
                .unwrap()
                .name,
            t.name
        );
    }

    // (3) The FRESH-launch default is name-derived — a mid-roster insert leaves
    // it on Saltpan by construction (this is the const `world_index("Saltpan")`,
    // re-checked here so the property is a test, not only a compile-time fact).
    assert_eq!(THEMES[DEFAULT_THEME].name, "Saltpan");

    // (4) A name that is NOT a world falls back leniently to None (never a
    // panic, never a by-position neighbour) — the door retired names lean on.
    assert!(set_active_by_name("NotAWorld").is_none());

    set_active(DEFAULT_THEME);
}

/// RETIRED-WORLD LENIENT FALLBACK (the renames Outback→Mulga,
/// Kingfisher→Bowerbird, Undertow→Bombora). A `config.toml` that still names one
/// of the three RETIRED worlds — a user who upgrades with `theme = "Outback"`
/// persisted — must not crash and must not resurface a neighbour by position:
/// `set_active_by_name` returns `None` for each retired name, and the config
/// apply seam (`Config::apply_sticky_globals`) discards that `None`, so the
/// built-in default (Saltpan) is kept. This test pins the NAME half; the
/// apply-seam half lives in `config::tests`.
#[test]
fn retired_world_names_fall_back_leniently() {
    let _g = crate::testlock::serial();
    for retired in ["Outback", "Kingfisher", "Undertow"] {
        assert!(
            set_active_by_name(retired).is_none(),
            "retired world {retired:?} must resolve to None (lenient fallback), not a live world"
        );
        // Case-insensitive: a lower-cased persisted value is equally retired.
        assert!(
            set_active_by_name(&retired.to_ascii_lowercase()).is_none(),
            "retired world {retired:?} (lower-cased) must resolve to None"
        );
    }
    // The successor names DO resolve (the rename actually landed).
    assert_eq!(set_active_by_name("Mulga").unwrap().name, "Mulga");
    assert_eq!(set_active_by_name("Bowerbird").unwrap().name, "Bowerbird");
    assert_eq!(set_active_by_name("Bombora").unwrap().name, "Bombora");
    set_active(DEFAULT_THEME);
}

#[test]
fn cycle_wraps_both_ways() {
    let _g = crate::testlock::serial();
    set_active(0);
    // Forward through all and back to start.
    for i in 1..=THEMES.len() {
        let t = cycle(1);
        assert_eq!(t.name, THEMES[i % THEMES.len()].name);
    }
    assert_eq!(active_index(), 0);
    // Backward wraps to the last world.
    let t = cycle(-1);
    assert_eq!(t.name, THEMES[THEMES.len() - 1].name);
    // restore default for other tests
    set_active(DEFAULT_THEME);
}

#[test]
fn set_by_name_is_case_insensitive() {
    let _g = crate::testlock::serial();
    assert_eq!(set_active_by_name("quokka").unwrap().name, "Quokka");
    assert_eq!(set_active_by_name("MULGA").unwrap().name, "Mulga");
    assert!(set_active_by_name("nope").is_none());
    set_active(DEFAULT_THEME);
}

fn render_caps_fields() -> Vec<String> {
    let src = include_str!("../model.rs");
    let start = src
        .find("pub struct RenderCaps {")
        .expect("RenderCaps declaration");
    let body = &src[start..];
    let end = body.find("\n}").expect("RenderCaps closing brace");
    body[..end]
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub ")
                .and_then(|line| line.split_once(':'))
                .map(|(name, _)| name.to_string())
        })
        .collect()
}

fn capability_adopters(field: &str) -> Vec<&'static str> {
    let d = RenderCaps::DEFAULT;
    THEMES
        .iter()
        .filter(|t| match field {
            "selection_style" => t.render_caps.selection_style != d.selection_style,
            "caret_block_style" => t.render_caps.caret_block_style != d.caret_block_style,
            "backdrop" => t.render_caps.backdrop != d.backdrop,
            "elevation" => t.render_caps.elevation != d.elevation,
            "decorative_wash" => t.render_caps.decorative_wash != d.decorative_wash,
            "image_reveal" => t.render_caps.image_reveal != d.image_reveal,
            "highlight_texture" => t.render_caps.highlight_texture != d.highlight_texture,
            "title_style" => t.render_caps.title_style != d.title_style,
            "page_frame" => t.render_caps.page_frame != d.page_frame,
            "card_anchor" => t.render_caps.card_anchor != d.card_anchor,
            "chrome_face" => t.render_caps.chrome_face != d.chrome_face,
            "list_style" => t.render_caps.list_style != d.list_style,
            "facet_style" => t.render_caps.facet_style != d.facet_style,
            "location_style" => t.render_caps.location_style != d.location_style,
            "ambient" => t.render_caps.ambient != d.ambient,
            "spell_underline_gap" => t.render_caps.spell_underline_gap != d.spell_underline_gap,
            "fold_afford" => t.render_caps.fold_afford != d.fold_afford,
            "card_texture" => t.render_caps.card_texture != d.card_texture,
            "card_shape" => t.render_caps.card_shape != d.card_shape,
            other => {
                panic!("new RenderCaps field `{other}` has no adoption census or classification")
            }
        })
        .map(|t| t.name)
        .collect()
}

/// Every top-level capability carries an explicit data-model verdict. This is
/// intentionally exhaustive even for common fields: adding a field without a
/// verdict fails by its declaration name. Zero/single-adopter fields are
/// printed with their live adopters so a green run is also the audit report.
#[test]
fn every_zero_or_single_adopter_capability_is_named_and_classified() {
    let fields = render_caps_fields();
    assert_eq!(fields.len(), 19, "RenderCaps declaration census drifted");
    let mut sparse = Vec::new();
    for field in fields {
        let verdict = match field.as_str() {
            "selection_style" => "keep: authored document-selection expression",
            "caret_block_style" => "keep: authored block-caret expression",
            "backdrop" => "keep: authored flat/blur expression",
            "elevation" => "keep: authored surface expression",
            "decorative_wash" => "keep: authored one-bit exclusion",
            "image_reveal" => "keep: authored one-bit exclusion",
            "highlight_texture" => "keep: authored one-bit texture",
            "title_style" => "keep: authored title composition",
            "page_frame" => "keep: authored frame expression",
            "card_anchor" => "keep: authored card composition",
            "chrome_face" => "keep: authored display face",
            "list_style" => "keep: authored Pane/Diagonal/Bars/Rules composition",
            "facet_style" => "keep: authored facet composition",
            "location_style" => "keep: authored location composition",
            "ambient" => "keep: Stars is authored aliveness, not corrective geometry",
            "spell_underline_gap" => "excluded: separately resolved corrective dial",
            "fold_afford" => "keep: two measured lava-palette corrections",
            "card_texture" => "keep: Halftone is authored material",
            "card_shape" => "keep: Chamfer is authored geometry",
            other => panic!("RenderCaps field `{other}` has no classification"),
        };
        let adopters = capability_adopters(&field);
        if adopters.len() <= 1 {
            sparse.push(format!("{field}: {adopters:?} — {verdict}"));
        }
    }
    assert!(!sparse.is_empty(), "sparse-capability report is vacuous");
    eprintln!(
        "zero/single-adopter RenderCaps census:\n{}",
        sparse.join("\n")
    );
}

fn background_kind(t: &Theme) -> &'static str {
    match t.background {
        Background::Gradient { .. } => "Gradient",
        Background::Dots { .. } => "Dots",
        Background::Pinstripe { .. } => "Pinstripe",
        Background::Stripes { .. } => "Stripes",
        Background::Lava { .. } => "Lava",
        Background::Bands { .. } => "Bands",
        Background::Waves { .. } => "Waves",
        Background::Zigzag { .. } => "Zigzag",
        Background::Organic { .. } => "Organic",
        Background::Deckle { .. } => "Deckle",
        Background::WarpedGrid { .. } => "WarpedGrid",
    }
}

/// Sparse enum arms are expressions, not dead fields. This pins the explicitly
/// protected roster: Rules, both Diagonal directions, chamfer, ambient stars,
/// icon-ground presets, and every background kind all remain name-free data.
#[test]
fn sparse_authored_variants_remain_classified_data() {
    let adopters = |pred: &dyn Fn(&Theme) -> bool| {
        THEMES
            .iter()
            .filter(|t| pred(t))
            .map(|t| t.name)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        adopters(&|t| matches!(t.render_caps.list_style, ListStyle::Rules(_))),
        ["Paperbark"]
    );
    assert_eq!(
        adopters(&|t| matches!(
            t.render_caps.list_style,
            ListStyle::Diagonal(crate::theme::DiagonalSpine {
                direction: crate::theme::DiagonalDirection::Ascending,
                ..
            })
        )),
        ["Magpie"]
    );
    assert_eq!(
        adopters(&|t| matches!(
            t.render_caps.list_style,
            ListStyle::Diagonal(crate::theme::DiagonalSpine {
                direction: crate::theme::DiagonalDirection::Descending,
                ..
            })
        )),
        ["Mangrove"]
    );
    assert_eq!(
        adopters(&|t| matches!(t.render_caps.card_shape, CardShape::Chamfered { .. })),
        ["Quokka"]
    );
    assert_eq!(
        adopters(&|t| matches!(t.render_caps.ambient, AmbientStyle::Stars { .. })),
        ["Currawong"]
    );
    assert_eq!(
        adopters(&|t| t.icon_ground == IconGround::Blend40),
        ["Firetail"]
    );
    assert!(
        adopters(&|t| t.icon_ground == IconGround::Blend25).is_empty(),
        "Blend25 is a classified closed-preset exploration arm"
    );

    for kind in [
        "Gradient",
        "Dots",
        "Pinstripe",
        "Stripes",
        "Lava",
        "Bands",
        "Waves",
        "Zigzag",
        "Organic",
        "Deckle",
        "WarpedGrid",
    ] {
        let worlds: Vec<_> = THEMES
            .iter()
            .filter(|t| background_kind(t) == kind)
            .map(|t| t.name)
            .collect();
        assert!(
            !worlds.is_empty(),
            "background kind {kind} has no live world"
        );
        if worlds.len() == 1 {
            eprintln!("background.{kind}: {worlds:?} — keep: authored ground vocabulary");
        }
    }
}

/// Removed/no-variation facts have exactly one product owner, while their test
/// overrides still mutation-prove the latent renderer axes.
#[test]
fn promoted_facts_have_renderer_owners_and_no_theme_data_branch() {
    let model = include_str!("../model.rs");
    assert!(!model.contains("pub selection_ui:"));
    assert!(!model.contains("pub motion:"));
    assert!(!model.contains("pub pane_split:"));
    assert!(!model.contains("pub frost:"));

    let derive = include_str!("../derive.rs");
    assert!(!derive.contains("resolve_selection_ui"));
    assert!(derive.contains("pub fn selection_ui() -> Srgb {\n    derived_selection_ui()\n}"));

    let render = include_str!("../../render.rs");
    assert!(render.contains("None => theme::MotionJuice::CALM"));
    assert!(render.contains("None => theme::PaneSplit::Split"));
    assert!(render.contains("set_overlay_motion_test_override"));
    assert!(render.contains("set_pane_split_test_override"));

    let layers = include_str!("../../render/layers.rs");
    let outline = include_str!("../../render/chrome/outline.rs");
    let gutter = include_str!("../../render/chrome/gutter.rs");
    assert!(layers.contains("crate::lava::FROST_DIM"));
    assert!(layers.contains("crate::lava::FROST_BLUR_PX"));
    assert!(outline.contains("crate::lava::FROST_FEATHER_PX"));
    assert!(gutter.contains("crate::lava::FROST_FEATHER_PX"));
    for source in [layers, outline, gutter] {
        assert!(!source.contains("render_caps.frost"));
    }
}

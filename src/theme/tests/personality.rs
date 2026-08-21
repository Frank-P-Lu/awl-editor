use super::super::*;
use crate::render::Logical;

/// The expected page frame at a given LOGICAL weight — one spelling, so the
/// roster table below stays one line per capability.
fn frame(w: f32) -> model::PageFrame {
    model::PageFrame::Line {
        weight_px: Logical(w),
    }
}

/// THE PERSONALITY ASSIGNMENT TABLE — a byte-identity gate. Every world's
/// `render_caps` must be EXACTLY its decided value: the four placard worlds
/// (Galah/Magpie the Ghost reference look, Mangrove the stipple — the Bayer
/// dither is its own language, Firetail the loud-end statement — a big/Bold
/// smooth placard plus the Archivo Black chrome voice), the three functional-
/// elevation borders (Currawong's OLED rim, the two lava worlds' edge over
/// motion, the six LIGHT worlds' pale-ground rim), the Wagtail page frame
/// (2px, its ladder white), Wagtail's user-confirmed NO-placard silence, and
/// deliberate defaults elsewhere. Potoroo alone recesses both split Pane
/// surfaces so they separate from its striped Frame without a rim or accent.
/// The assignment is DATA, and a new world fails the exhaustive match until
/// consciously placed.
#[test]
fn personality_assignments_are_exactly_the_decided_table() {
    use model::{
        ChipVariant, Elevation, FacetStyle, ListStyle, PlacardCorner, PlacardInk, RenderCaps,
        TitleStyle,
    };
    // The SHIPPING poster list surface every statement world carries — the
    // Bars HUG-ALL HYBRID (`HugLabel`: plate hugs the LABEL, chord bare in the
    // right column) at the gate's mid radius, every row a bar.
    // `ListStyle::Bars` carries no fields of its own (nothing has ever varied
    // them): `theme::BarConfig::SHIPPED`, read by the renderer rather than by
    // any per-world `Theme`, is the one owner of that hug-all-hybrid shape.
    let poster_bars = ListStyle::Bars;
    let wagtail_swap =
        model::TwoColour::new(model::PaletteRole::Base300, model::PaletteRole::BaseContent);
    // Selection and block caret author capabilities independently even though
    // Wagtail deliberately assigns both the same palette pair.
    let expected = |name: &str| -> RenderCaps {
        // COMPOSITION-C2: the placard worlds anchor their card TOP-LEFT and let
        // the poster corner DERIVE from that anchor (`Auto` → bottom-RIGHT),
        // opening the opposite corner. Firetail alone keeps an explicit BL.
        // Cassowary + Mangrove are the fable RIGHT picks — TopRight card, Auto
        // corner deriving bottom-LEFT (the mirror composition).
        let auto = |ink: PlacardInk| TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink,
        };
        match name {
            // Galah / Magpie: the light-world placard PLUS the composition
            // round's light-world border; C2 TopLeft anchor + Auto corner.
            "Galah" => RenderCaps {
                title_style: auto(PlacardInk::Ghost),
                card_anchor: model::CardAnchor::TopLeft,
                elevation: Elevation::Bordered,
                // A poster world → the Bars hug-all hybrid; Galah wears
                // HAIRLINE chips (user's confirmed chip map).
                list_style: poster_bars,
                facet_style: FacetStyle::Chips(ChipVariant::Hairline),
                ..RenderCaps::DEFAULT
            },
            "Magpie" => RenderCaps {
                title_style: auto(PlacardInk::Ghost),
                card_anchor: model::CardAnchor::TopLeft,
                elevation: Elevation::Bordered,
                // The mirrored editorial diagonal composition.
                list_style: ListStyle::Diagonal(super::DiagonalSpine::ascending(
                    super::DiagonalMark::HAIRLINE,
                )),
                facet_style: FacetStyle::Chips(ChipVariant::Underline),
                // The location cue joins the diagonal line itself — slanted
                // to the spine's own rake, gradient between its two authored
                // tones — rather than sitting upright beside it.
                location_style: model::LocationStyle::Raked(model::LocationLabelStyle {
                    face: model::LocationFace::Chrome,
                    scale: 0.92,
                    ink: model::LocationInk::Gradient(
                        model::PaletteRole::Muted,
                        model::PaletteRole::BaseContent,
                    ),
                    tracking_em: 0.0,
                    locator: model::LocationLocator::Label,
                }),
                ..RenderCaps::DEFAULT
            },
            "Mangrove" => RenderCaps {
                title_style: auto(PlacardInk::Stipple),
                // The fable pick: the tidal margin is a RIGHT rail (Auto corner
                // then derives bottom-LEFT).
                card_anchor: model::CardAnchor::TopRight,
                elevation: Elevation::Bordered,
                // The mirrored tidal diagonal composition.
                list_style: ListStyle::Diagonal(super::DiagonalSpine::descending(
                    super::DiagonalMark::CRISP,
                )),
                facet_style: FacetStyle::Chips(ChipVariant::Bracket),
                // Both marks lifted — see `worlds::MANGROVE`'s own doc.
                fold_afford: model::FoldAfford {
                    chevron_lift: 0.60,
                    tail_lift: 0.75,
                },
                ..RenderCaps::DEFAULT
            },
            // The loud-end world's own loud
            // overlay — BL placard dialed to the combo-shot scale + Bold ink,
            // and the Archivo Black chrome voice on the placard/title/strip.
            // C2: KEEPS its user-picked explicit BL corner (overrides the Auto
            // derivation) and anchors its card TopLeft.
            "Firetail" => RenderCaps {
                title_style: TitleStyle::Placard {
                    corner: PlacardCorner::BL,
                    scale: 4.5,
                    ink: PlacardInk::Bold,
                },
                card_anchor: model::CardAnchor::TopLeft,
                chrome_face: model::ChromeFace::Named("Archivo Black"),
                elevation: Elevation::Bordered,
                // The maximalist showcase world → the Bars hug-all hybrid;
                // Firetail wears FILLED chips (the loudest — user's confirmed
                // chip map).
                list_style: poster_bars,
                facet_style: FacetStyle::Chips(ChipVariant::FilledActive),
                // The tail alone is lifted — the chevron already reads fine —
                // see `worlds::FIRETAIL`'s own doc.
                fold_afford: model::FoldAfford {
                    chevron_lift: 0.0,
                    tail_lift: 0.40,
                },
                ..RenderCaps::DEFAULT
            },
            // C2: the iconic dark-technical statement world anchors TopLeft.
            // TWINKLING STARS (the user's verdict): Currawong
            // stays, differentiated by the ambient star field — the maximally-
            // quiet, unmistakably-alive pole ("aliveness ≠ loudness"). The
            // params are the authored taste data (BUILD + GALLERY + HOLD).
            "Currawong" => RenderCaps {
                elevation: Elevation::Bordered,
                card_anchor: model::CardAnchor::TopLeft,
                ambient: model::AmbientStyle::Stars {
                    // Chroma lifted at no greater luminance. Mirrors
                    // `worlds::CURRAWONG`.
                    tint: Srgb::rgb(0x9B, 0xB0, 0xD2),
                    cell_px: 34.0,
                    // A dense candidate field (~half dark-dwelling at any
                    // moment); the visibility band is the per-star shine range
                    // (a real visible floor, a calm ceiling above the muted
                    // whisper cap).
                    density: 0.30,
                    size_px: 2.6,
                    peak: 0.5,
                    floor: 0.18,
                },
                ..RenderCaps::DEFAULT
            },
            // Wagtail: the 1-bit escape hatch (every field away from default)
            // + the page frame's first assignment + NO placard (the silent
            // pole announces nothing — user-confirmed).
            "Wagtail" => RenderCaps {
                selection_style: model::SelectionStyle::InverseVideo(wagtail_swap),
                caret_block_style: model::CaretBlockStyle::InverseVideo(wagtail_swap),
                backdrop: model::Backdrop::Flat,
                elevation: Elevation::Bordered,
                decorative_wash: model::DecorativeWash::Off,
                image_reveal: model::ImageReveal::Opaque,
                highlight_texture: model::HighlightTexture::Stipple {
                    color: Srgb::rgb(0xFF, 0xFF, 0xFF),
                    density: crate::render::dither::WAGTAIL_HIGHLIGHT_DITHER_DENSITY,
                },
                title_style: TitleStyle::InlinePrefix,
                page_frame: frame(2.0),
                card_anchor: model::CardAnchor::TopLeft,
                chrome_face: model::ChromeFace::Body,
                list_style: model::ListStyle::Pane,
                pane_split: model::PaneSplit::Split,
                facet_style: model::FacetStyle::Text,
                // The silent pole keeps the shared inline treatment (only
                // Cassowary opts to `RotatedRail`).
                location_style: model::LocationStyle::Inline,
                // No ambient life on the silent pole (and a fractional-alpha
                // breath is 1-bit-illegal besides).
                ambient: model::AmbientStyle::None,
                // The silent pole keeps the shared default gap.
                spell_underline_gap: model::SPELL_UNDERLINE_GAP_DEFAULT,
                // Dormant default (no lava ground — the silent pole's column
                // stays flat).
                fold_afford: model::FoldAfford::DEFAULT,
                // Dormant default — a fractional-alpha halftone dot
                // is 1-bit-illegal, and the chamfer is Quokka's own separate
                // personality statement.
                card_texture: model::CardTexture::DEFAULT,
                card_shape: model::CardShape::DEFAULT,
            },
            // Bilby is the LIGHT POLE and ships FRAMELESS. A 1px light-pole
            // page frame — its own night-violet ink around the writing column,
            // mirroring Wagtail's 2px white one — was put to the user and
            // REJECTED live ("the frame is so weird"), so that idea is settled
            // rather than pending. It keeps the light-world card border.
            "Bilby" => RenderCaps {
                elevation: Elevation::Bordered,
                // The tighter per-world baseline dial (see `worlds::BILBY`'s
                // own doc).
                spell_underline_gap: model::SPELL_UNDERLINE_GAP_TIGHT,
                ..RenderCaps::DEFAULT
            },
            // LIGHT-WORLD BORDER: the remaining pale-ground worlds carry the
            // summoned-card border, DATA-only. Brolga (the cool light pole)
            // is among them — a crisp rim off its pale sky-blue ground;
            // deliberately NO page frame (a 1px light-pole frame was
            // user-rejected).
            "Gumtree" | "Saltpan" | "Brolga" => RenderCaps {
                elevation: Elevation::Bordered,
                ..RenderCaps::DEFAULT
            },
            // Quokka alone assigns the non-default printed-card caps
            // (see `worlds::QUOKKA`'s own doc): a small rotated dot lattice
            // rolling off toward the left content side, and a crisp 45°
            // chamfer replacing the small rounded card corner.
            "Quokka" => RenderCaps {
                elevation: Elevation::Bordered,
                card_texture: model::CardTexture::HalftoneDots {
                    angle_deg: 18.0,
                    cell_px: 8.0,
                    density: 0.30,
                },
                card_shape: model::CardShape::Chamfered { cut_px: 11.0 },
                ..RenderCaps::DEFAULT
            },
            "Potoroo" => expected_potoroo_caps(),
            "Tawny" | "Mopoke" | "Bombora" | "Mulga" | "Bowerbird" => RenderCaps::DEFAULT,
            // CASSOWARY (the NERV-terminal statement world): the loud NERV console
            // overlay — a bold Archivo-Black wordmark placard (Auto corner derives
            // bottom-LEFT off the fable-pick RIGHT card), BORDERED elevation, the
            // poster Bars list, and BRACKET facet chips (terminal corner-ticks). The
            // writing page stays calm.
            "Cassowary" => RenderCaps {
                // The authentic CRT phosphor cursor — an ink caret (primary ==
                // base_content) needs the Filled block so a lit green cell knocks
                // the glyph out in the ground rather than erasing it green-on-green.
                caret_block_style: model::CaretBlockStyle::Filled,
                title_style: TitleStyle::Placard {
                    corner: PlacardCorner::Auto,
                    scale: 3.0,
                    ink: PlacardInk::Bold,
                },
                // The fable pick: the terminal readout is a RIGHT rail (Auto
                // corner then derives bottom-LEFT).
                card_anchor: model::CardAnchor::TopRight,
                chrome_face: model::ChromeFace::Named("Archivo Black"),
                elevation: Elevation::Bordered,
                list_style: model::ListStyle::Pane,
                pane_split: model::PaneSplit::Unified,
                facet_style: FacetStyle::DockedTab,
                // The active facet reads as a subordinate technical locator:
                // mono, muted, tracked through the shaper, and truthfully
                // numbered from the real lens strip.
                location_style: model::LocationStyle::RotatedRail(model::LocationLabelStyle {
                    face: model::LocationFace::Mono,
                    scale: 0.28,
                    ink: model::LocationInk::Flat(model::PaletteRole::Muted),
                    tracking_em: 0.06,
                    locator: model::LocationLocator::IndexOnly { digits: 2 },
                }),
                ..RenderCaps::DEFAULT
            },
            // PAPERBARK (the handmade-paper studio): a LIGHT world, so
            // it carries the composition round's light-world card border, and
            // ⚠️ THE ONE CARRIER OF `Rules` — the quiet fourth list style,
            // organised by absence rather than by enclosure. The room's whole
            // personality is its material ground, and a ruled index is that
            // ground one register up where a floating card was an object dropped
            // on it. Otherwise deliberately DEFAULT: no placard, no rail move,
            // no frame. `Weight` is the shipped selection treatment: the selected
            // row's own bounding rules thicken and run past the text measure,
            // leaving its interior plain ground. A second carrier needs a
            // findability check on a DARK ground — every pixel law for this
            // style runs on cream — and a `FacetStyle` that is not `Chips`,
            // which would put a filled pill back on the lens strip.
            "Paperbark" => RenderCaps {
                elevation: Elevation::Bordered,
                list_style: model::ListStyle::Rules(model::RuleSelection::Weight),
                ..RenderCaps::DEFAULT
            },
            // KITE (the light warped-grid statement world). ⚠️ A STATEMENT
            // WORLD MUST STATE ITSELF IN THE CHROME, not only in the frame: a
            // world that states itself only in its margins has nothing left to
            // state when the margins narrow, and at `page_width_code` they
            // narrow to a stripe. Moving ONE of these twenty-two dials would
            // leave Kite chrome-identical to five QUIET worlds while its
            // declared deliberate counterpart Firetail moves seven. Six dials,
            // each traceable to the world's own four words (cool / geometric /
            // crisp / directional) and each mirroring Firetail rather than
            // copying it.
            "Kite" => RenderCaps {
                title_style: TitleStyle::Placard {
                    corner: PlacardCorner::BR,
                    scale: 1.4,
                    ink: PlacardInk::Muted,
                },
                card_anchor: CardAnchor::TopRight,
                chrome_face: ChromeFace::Named("Figtree"),
                elevation: Elevation::Bordered,
                page_frame: frame(1.0),
                facet_style: FacetStyle::Band,
                ..RenderCaps::DEFAULT
            },
            other => panic!(
                "{other}: a NEW world must decide its personality here (placard? border? \
                 frame? or deliberately DEFAULT) — the assignment table is conscious data, \
                 never an accident"
            ),
        }
    };
    for t in THEMES.iter() {
        assert_eq!(
            t.render_caps,
            expected(t.name),
            "{}: render_caps drifted from the decided personality table",
            t.name
        );
    }
    // Corner discipline is the COMPOSITION-C2 no-clip OUTCOME law
    // (`render::tests::overlay_personality::every_shipped_placard_world_wordmark_stays_on_canvas`)
    // + the data-sanity guard (`every_shipped_placard_world_has_sane_corner_and_scale`),
    // not a BL pin: shrink-to-fit makes every corner clip-safe, so the poster
    // corner DERIVES from the card anchor (complementary) with per-world overrides.
}

/// The HUG-ALL HYBRID's own five dials, pinned by literal value.
/// `personality_assignments_are_exactly_the_decided_table` only checks that
/// every Bars world resolves to the `Bars` VARIANT, not what the shared layout
/// is, and `BarConfig::SHIPPED` is read by the renderer instead of any
/// per-`Theme` field — so this is the one place that fails if its values drift
/// from the decided shape.
#[test]
fn bar_config_shipped_is_the_hug_all_hybrid() {
    assert_eq!(
        model::BarConfig::SHIPPED,
        model::BarConfig {
            radius: 6.0,
            gap: 10.0,
            grow_px: 24.0,
            extent: model::BarExtent::HugLabel,
            coverage: model::BarCoverage::All,
        }
    );
}

fn expected_potoroo_caps() -> model::RenderCaps {
    model::RenderCaps {
        elevation: model::Elevation::Recessed,
        ..model::RenderCaps::DEFAULT
    }
}

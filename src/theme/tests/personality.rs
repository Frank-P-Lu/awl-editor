use super::super::*;

/// THE PERSONALITY ASSIGNMENT TABLE — successor to the all-InlinePrefix
/// capability gate. Potoroo alone recesses both split Pane surfaces so they
/// separate from its striped Frame without a rim or accent. The assignment
/// remains data alongside the established personality roster, and a new world
/// still fails the exhaustive match until consciously placed.
/// byte-identity gate. Every world's `render_caps` must be EXACTLY its
/// decided value: the four placard worlds (Galah/Magpie the Ghost reference
/// look, Mangrove the stipple — the Bayer dither is its own language,
/// Firetail the loud-end statement — a big/Bold smooth placard plus the
/// Archivo Black chrome voice, the CHROME-VOICES flip), the three functional-
/// elevation borders (Currawong's OLED rim, the two lava worlds' edge over
/// motion, the six LIGHT worlds' pale-ground rim — composition round item 6),
/// the Wagtail page frame (2px, its ladder white), Wagtail's
/// user-confirmed NO-placard silence, and deliberate defaults elsewhere.
#[test]
fn personality_assignments_are_exactly_the_decided_table() {
    use model::{
        ChipVariant, Elevation, FacetStyle, ListStyle, PageFrame, PlacardCorner, PlacardInk,
        RenderCaps, TitleStyle,
    };
    // FLIP ROUND (user FINAL PICKS 2026-07-17): the SHIPPING poster list surface
    // every statement world carries — the Bars HUG-ALL HYBRID (`HugLabel`: plate
    // hugs the LABEL, chord bare in the right column) at the gate's mid radius,
    // every row a bar. `ListStyle::Bars` carries no fields of its own any
    // more (nothing has ever varied them): `theme::BarConfig::SHIPPED`, read
    // by the renderer rather than by any per-world `Theme`, is the one owner
    // of that hug-all-hybrid shape now.
    let poster_bars = ListStyle::Bars;
    let expected = |name: &str| -> RenderCaps {
        // COMPOSITION-C2: the placard worlds anchor their card TOP-LEFT and let
        // the poster corner DERIVE from that anchor (`Auto` → bottom-RIGHT),
        // opening the opposite corner. Firetail alone keeps an explicit BL.
        // ITEM 45 (2026-07-23): Cassowary + Mangrove are the fable RIGHT picks —
        // TopRight card, Auto corner deriving bottom-LEFT (the mirror composition).
        let auto = |ink: PlacardInk| TitleStyle::Placard {
            corner: PlacardCorner::Auto,
            scale: 3.0,
            ink,
        };
        match name {
            // Galah / Magpie: the light-world placard PLUS the composition
            // round's light-world border (item 6); C2 TopLeft anchor + Auto corner.
            "Galah" => RenderCaps {
                title_style: auto(PlacardInk::Ghost),
                card_anchor: model::CardAnchor::TopLeft,
                elevation: Elevation::Bordered,
                // FLIP ROUND (2026-07-17): poster world → the Bars hug-all hybrid;
                // Galah wears HAIRLINE chips (user's confirmed chip map).
                list_style: poster_bars,
                facet_style: FacetStyle::Chips(ChipVariant::Hairline),
                ..RenderCaps::DEFAULT
            },
            "Magpie" => RenderCaps {
                title_style: auto(PlacardInk::Ghost),
                card_anchor: model::CardAnchor::TopLeft,
                elevation: Elevation::Bordered,
                // The mirrored editorial diagonal composition.
                list_style: ListStyle::Diagonal(super::DiagonalDirection::Ascending),
                facet_style: FacetStyle::Chips(ChipVariant::Underline),
                // The location cue joins the diagonal line itself — slanted
                // to the spine's own rake, gradient between its two authored
                // tones — rather than sitting upright beside it.
                location_style: model::LocationStyle::Raked,
                ..RenderCaps::DEFAULT
            },
            "Mangrove" => RenderCaps {
                title_style: auto(PlacardInk::Stipple),
                // ITEM 45 fable pick (2026-07-23): the tidal margin flipped to a
                // RIGHT rail (Auto corner then derives bottom-LEFT).
                card_anchor: model::CardAnchor::TopRight,
                elevation: Elevation::Bordered,
                // The mirrored tidal diagonal composition.
                list_style: ListStyle::Diagonal(super::DiagonalDirection::Descending),
                facet_style: FacetStyle::Chips(ChipVariant::Bracket),
                // ITEM 65 (Fable adjustment): both marks lifted — see
                // `worlds::MANGROVE`'s own doc.
                fold_afford: model::FoldAfford {
                    chevron_lift: 0.60,
                    tail_lift: 0.75,
                },
                ..RenderCaps::DEFAULT
            },
            // CHROME-VOICES FLIP (2026-07-16): the loud-end world's own loud
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
                // FLIP ROUND (2026-07-17): the maximalist showcase world → the Bars
                // hug-all hybrid; Firetail wears FILLED chips (the loudest — user's
                // confirmed chip map).
                list_style: poster_bars,
                facet_style: FacetStyle::Chips(ChipVariant::FilledActive),
                // ITEM 65 (Fable adjustment): the tail alone is lifted — the
                // chevron already read fine — see `worlds::FIRETAIL`'s own doc.
                fold_afford: model::FoldAfford {
                    chevron_lift: 0.0,
                    tail_lift: 0.40,
                },
                ..RenderCaps::DEFAULT
            },
            // C2: the iconic dark-technical statement world anchors TopLeft.
            // TWINKLING STARS (2026-07-18, the user's morning verdict): Currawong
            // stays, differentiated by the ambient star field — the maximally-
            // quiet, unmistakably-alive pole ("aliveness ≠ loudness"). The
            // params are the authored taste data (BUILD + GALLERY + HOLD).
            "Currawong" => RenderCaps {
                elevation: Elevation::Bordered,
                card_anchor: model::CardAnchor::TopLeft,
                ambient: model::AmbientStyle::Stars {
                    // ITEM 62 (2026-07-24): +10.8% chroma over the shipped
                    // #9DB0CF, at no greater luminance. Mirrors `worlds::CURRAWONG`.
                    tint: Srgb::rgb(0x9B, 0xB0, 0xD2),
                    cell_px: 34.0,
                    // LIFECYCLE round (2026-07-23): denser candidate field
                    // (~half dark-dwelling at any moment) and the visibility band
                    // re-scoped to the per-star shine range (a real visible floor,
                    // a calm ceiling above the muted whisper cap).
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
                selection_style: model::SelectionStyle::InverseVideo,
                caret_block_style: model::CaretBlockStyle::InverseVideo,
                backdrop: model::Backdrop::Flat,
                elevation: Elevation::Bordered,
                decorative_wash: model::DecorativeWash::Off,
                image_reveal: model::ImageReveal::Opaque,
                highlight_texture: model::HighlightTexture::Stipple {
                    color: Srgb::rgb(0xFF, 0xFF, 0xFF),
                    density: crate::render::dither::WAGTAIL_HIGHLIGHT_DITHER_DENSITY,
                },
                title_style: TitleStyle::InlinePrefix,
                page_frame: PageFrame::Line { weight_px: 2.0 },
                card_anchor: model::CardAnchor::TopLeft,
                // FIRETAIL-MAXIMALIST-SHOWCASE round: both new dials landed
                // INERT on every world — the silent pole included.
                chrome_face: model::ChromeFace::Body,
                motion: model::MotionJuice::CALM,
                // PER-ITEM LIST SURFACES round: both new dials landed INERT on
                // every world — the silent pole included.
                list_style: model::ListStyle::Pane,
                facet_style: model::FacetStyle::Text,
                // The silent pole keeps the shared inline treatment (only
                // Cassowary opts to `RotatedRail`).
                location_style: model::LocationStyle::Inline,
                // SPLIT-PANE COMPOSITION round: the silent pole takes the DEFAULT
                // split like every Pane world (only Cassowary opts to `Unified`).
                pane_split: model::PaneSplit::Split,
                // TWINKLING-STARS round: no ambient life on the silent pole
                // (and a fractional-alpha breath is 1-bit-illegal besides).
                ambient: model::AmbientStyle::None,
                // SPELL-SQUIGGLE round: the silent pole keeps the shared
                // default gap.
                spell_underline_gap: model::SPELL_UNDERLINE_GAP_DEFAULT,
                // FROST-AS-CAPABILITY round: dormant default (no lava ground).
                frost: model::Frost::DEFAULT,
                // ITEM 65: dormant default (no lava ground — the silent pole's
                // column stays flat).
                fold_afford: model::FoldAfford::DEFAULT,
                // ITEM 70: dormant default — a fractional-alpha halftone dot
                // is 1-bit-illegal, and the chamfer is Quokka's own separate
                // personality statement.
                card_texture: model::CardTexture::DEFAULT,
                card_shape: model::CardShape::DEFAULT,
            },
            // DAWN ROUND (2026-07-18): Bilby is the LIGHT POLE — the roster
            // decision ("the dark-line-on-light page frame is reserved for a
            // future light-silent pole world") lands here: 1px of its own
            // night-violet ink around the writing column, Wagtail's 2px white
            // frame mirrored at the light end of the spectrum. Keeps the
            // light-world card border.
            // Dawn round: the proposed 1px light-pole page frame was REJECTED by
            // the user live ("the frame is so weird") — Bilby ships frameless.
            "Bilby" => RenderCaps {
                elevation: Elevation::Bordered,
                // SPELL-SQUIGGLE round: the tighter per-world baseline dial
                // (see `worlds::BILBY`'s own doc).
                spell_underline_gap: model::SPELL_UNDERLINE_GAP_DEFAULT - 2.0,
                ..RenderCaps::DEFAULT
            },
            // LIGHT-WORLD BORDER (composition round item 6): the remaining
            // pale-ground worlds gain the summoned-card border, DATA-only.
            // Brolga (the SEVENTEENTH world, the cool light pole) joins them —
            // a crisp rim off its pale sky-blue ground; deliberately NO page
            // frame (the DAWN round's 1px light-pole frame was user-rejected).
            "Gumtree" | "Saltpan" | "Brolga" => RenderCaps {
                elevation: Elevation::Bordered,
                ..RenderCaps::DEFAULT
            },
            // ITEM 70 — Quokka alone assigns the non-default printed-card caps
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
            // ITEM 86 — Bowerbird's item-71 woven `JaggedWave` card texture
            // was RETIRED (summoned cards returned to plain flat); it now
            // rides the plain default alongside its neighbors here.
            "Potoroo" => expected_potoroo_caps(),
            "Tawny" | "Mopoke" | "Bombora" | "Mulga" | "Bowerbird" => RenderCaps::DEFAULT,
            // CASSOWARY (the NERV-terminal statement world): the loud NERV console
            // overlay — a bold Archivo-Black wordmark placard (Auto corner derives
            // bottom-LEFT off the ITEM-45 RIGHT card), BORDERED elevation, the poster
            // Bars list, and BRACKET facet chips (terminal corner-ticks). The writing
            // page stays calm.
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
                // ITEM 45 fable pick (2026-07-23): the terminal readout flipped to
                // a RIGHT rail (Auto corner then derives bottom-LEFT).
                card_anchor: model::CardAnchor::TopRight,
                chrome_face: model::ChromeFace::Named("Archivo Black"),
                elevation: Elevation::Bordered,
                list_style: poster_bars,
                facet_style: FacetStyle::Chips(ChipVariant::Bracket),
                // The active facet reads as a vertical secondary heading
                // flush with the card's own left border, not the inline
                // treatment every other world uses.
                location_style: model::LocationStyle::RotatedRail,
                // SPLIT-PANE COMPOSITION round: the NERV console is the ONE Pane
                // exception — a UNIFIED room (dormant under its poster Bars list).
                pane_split: model::PaneSplit::Unified,
                ..RenderCaps::DEFAULT
            },
            // PAPERBARK (item 158, the handmade-paper studio): a LIGHT world, so
            // it carries the composition round's light-world card border and
            // nothing else — no placard, no rail move, no frame. The room's whole
            // personality is its material ground; the summoned chrome stays out
            // of the way. Deliberately otherwise DEFAULT.
            "Paperbark" => RenderCaps {
                elevation: Elevation::Bordered,
                ..RenderCaps::DEFAULT
            },
            // KITE (the light warped-grid statement world). ⚠️ THE OLD ENTRY
            // HERE READ "loud in the FRAME, quiet in the chrome... a placard or
            // a moved rail would compete with it", and the user overturned it
            // ("we need to update the chrome for kite as well"). The reasoning
            // had a measurable hole: Kite moved ONE of
            // these twenty-two dials, which made it chrome-identical to five
            // QUIET worlds, while its declared deliberate counterpart Firetail
            // moved seven. A world that states itself only in its margins has
            // nothing left to state when the margins narrow — and at
            // `page_width_code` they narrow to a stripe. Six dials now, each
            // traceable to the world's own four words (cool / geometric / crisp
            // / directional) and each mirroring Firetail rather than copying it.
            "Kite" => RenderCaps {
                title_style: TitleStyle::Placard {
                    corner: PlacardCorner::BR,
                    scale: 1.4,
                    ink: PlacardInk::Muted,
                },
                card_anchor: CardAnchor::TopRight,
                chrome_face: ChromeFace::Named("Figtree"),
                elevation: Elevation::Bordered,
                page_frame: PageFrame::Line { weight_px: 1.0 },
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
    // Corner discipline is now the COMPOSITION-C2 no-clip OUTCOME law
    // (`render::tests::overlay_personality::every_shipped_placard_world_wordmark_stays_on_canvas`)
    // + the data-sanity guard (`every_shipped_placard_world_has_sane_corner_and_scale`),
    // not a BL pin: the shrink-to-fit made every corner clip-safe, so the poster
    // corner DERIVES from the card anchor (complementary) with per-world overrides.
}

/// The FLIP-ROUND HUG-ALL HYBRID's own five dials, pinned by literal value —
/// the coverage `personality_assignments_are_exactly_the_decided_table` lost
/// when `ListStyle::Bars` stopped carrying them: that test now only checks
/// EVERY Bars world resolves to the `Bars` variant, not what the shared
/// layout actually is. `BarConfig::SHIPPED` is read by the renderer instead
/// of any per-`Theme` field, so this is the one place left that fails if its
/// values ever drift from the decided shape.
#[test]
fn bar_config_shipped_is_the_flip_round_hug_all_hybrid() {
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

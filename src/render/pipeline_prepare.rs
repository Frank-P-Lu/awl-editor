//! FRAME PREPARE — per-frame buffer preparation and blur-cache state.

use super::*;

impl TextPipeline {
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        // INVARIANT: the document buffer's soft-wrap width must ALWAYS equal the
        // live page COLUMN width. `column_left()` / `column_width()` and the margin
        // background are recomputed from the live page state EVERY frame, but the
        // buffer is only re-wrapped at the scattered `set_size` / `set_dpi` /
        // `set_text` call sites. Any state flip those sites miss (a page-mode toggle
        // or measure change that doesn't re-wrap, the width-preserving theme reshape)
        // leaves the buffer wrapped at a STALE, wider width while the column re-centers
        // — so the text wraps too wide from the centered left, overflowing the right
        // edge with NO right margin. Re-deriving here makes divergence impossible at
        // any window size / DPI. cosmic-text no-ops when the width is unchanged, so a
        // settled frame stays free.
        self.sync_wrap_width();
        self.viewport.update(queue, Resolution { width, height });

        self.prepare_background_layer(queue, width, height);
        // THE LAVA-LAMP GROUND: over the flat margin ground, before the washes.
        // A no-op (draws nothing) for every non-lava world.
        self.prepare_lava_layer(queue, width, height);
        // TWINKLING STARS: the ambient star field in the margins (zero
        // instances for every AmbientStyle::None world — byte-identical).
        self.prepare_stars_layer(device, queue, width, height);
        // THE PAGE FRAME: the thin writing-column frame (zero rects for every
        // PageFrame::None world, so those stay byte-identical).
        self.prepare_page_frame(device, queue, width, height);
        self.prepare_wash_layer(device, queue, width, height);
        self.prepare_wysiwyg_wash_layer(device, queue, width, height);
        self.prepare_text_layer(device, queue, width, height)?;
        // Seal the exact shaped partition glyphon just prepared. Layout reports
        // can only borrow this frame; they never shape or assemble rows.
        self.row_geom.seal_frame(&self.buffer, &self.metrics);
        // THE X-RAY: stash the caret's table-row floated source BEFORE the caret /
        // selection layers, so their `col_x_and_advance` redirects onto it (the
        // concealed doc row is zero-width). A no-op off a table row.
        self.prepare_table_xray();
        self.prepare_caret_layer(device, queue, width, height);
        self.prepare_selection_layer(device, queue, width, height);
        self.prepare_ornaments(device, queue, width, height)?;
        // THE FOLD CHEVRON: rotated-quad arms, drawn OUTSIDE the glyphon ornament
        // pipeline above because it must turn a quarter turn on fold/unfold and
        // glyphon 0.11 has no transform (`layers::fold_chevron`'s module doc).
        self.prepare_fold_chevron_marks(device, queue, width, height);
        self.prepare_table_grid(device, queue, width, height)?;
        // INLINE IMAGES: the tall rows are reserved at reshape (the per-line height
        // override in `build_line_attrs`); this decodes each visible off-cursor image
        // (`image_cache`, downscaled), builds the textured quads (fit-to-column,
        // centered in the reserved row), and the calm missing-file placeholders. All
        // three layers park empty when off / no images, so a capture is byte-identical.
        self.prepare_images(device, queue, width, height)?;
        self.prepare_chrome_layer(device, queue, width, height)?;
        self.prepare_spell_layer(device, queue, width, height);
        self.prepare_nit_layer(device, queue, width, height);
        self.prepare_strike_layer(device, queue, width, height);
        self.prepare_link_underline_layer(device, queue, width, height);
        self.prepare_blur(device, queue, width, height);
        Ok(())
    }

    /// True when the FROSTED-BLUR backdrop applies this frame: a full-takeover
    /// overlay is up AND it is NOT a crisp-exception picker (theme / caret) NOR the
    /// contextual SPELL panel (a small floating popup at the word — it recedes
    /// nothing, DESIGN §5). The search SPLIT panel (`search_active`, not
    /// `overlay_active`) is never blurred.
    fn overlay_blur(&self) -> bool {
        self.overlay_active && !self.overlay_crisp && self.overlay_spell.is_none()
    }

    /// True when the SUMMONED-WHILE-HELD stats HUD should actually DRAW this frame.
    /// The HUD and a full summoned overlay are MUTUALLY EXCLUSIVE (the overlay wins):
    /// a still-held Option-Cmd-I must not draw its card over an open picker — nor force the
    /// frosted blur that would defeat the theme picker's crisp live-color preview.
    /// One owner for both gates (`backdrop_blur` + `prepare_hud`), keyed off the same
    /// `overlay_active` flag the overlay draw path already reads, so they can't drift;
    /// the HUD reappears once the overlay closes if the key is still held.
    pub(in crate::render) fn hud_showing(&self) -> bool {
        crate::card::hud_shown(self.overlay_active)
    }

    /// True when the HOLD-⌘ SHORTCUT PEEK should DRAW this frame. Like the held HUD, it
    /// yields to an open summoned overlay (`!overlay_active`) so it never draws its card
    /// over a picker — the bare-⌘ hold that summons it can't coexist with a modal picker
    /// in practice, but the gate keeps the two mutually exclusive by construction, same
    /// as `hud_showing`.
    pub(in crate::render) fn peek_showing(&self) -> bool {
        crate::card::peek_shown(self.overlay_active)
    }

    /// True when ANY frosted-blur backdrop applies this frame: a blur-eligible full
    /// overlay ([`Self::overlay_blur`]) OR the SUMMONED-WHILE-HELD stats HUD. The HUD now
    /// recedes the document behind the SAME hue-preserving frost the palette uses — not
    /// the old neutral grey scrim — so the two takeovers read consistently (DESIGN §5:
    /// the doc recedes by BLUR, not grey). Drives both the blur prepare + the render
    /// path's offscreen-capture branch.
    ///
    /// **TRUE 1-BIT WORLDS (`Theme::render_caps.backdrop == Backdrop::Flat`) forgo the frost entirely.** A
    /// gaussian defocus of a document that is only ever pure black or pure
    /// white mathematically SMEARS every edge into intermediate grey — there
    /// is no tuning of the blur that avoids this, it is the nature of the
    /// operation. Every consumer (overlay takeover, held HUD, the lifetime
    /// card, hold-peek) falls back to the EXISTING crisp path instead — the
    /// same "document stays bright, no blur, no scrim" exception the
    /// theme/caret pickers already use — so the solid white-bordered card
    /// still reads clearly over a SHARP, not smeared, black/white document.
    pub(in crate::render) fn backdrop_blur(&self) -> bool {
        self.frost_mode().is_some()
    }

    /// THE ONE OWNER OF *WHETHER* AND *WHERE* THIS FRAME FROSTS — the whole decision,
    /// as one value ([`blur::Frost`]), so no consumer can read half of it.
    ///
    /// Order matters and is not alphabetical: every FULL-takeover condition is asked
    /// FIRST, so every frame that frosted the whole canvas before the footprint arm
    /// existed still does (the byte-identity argument for every world and every
    /// non-crisp overlay is that this arm is unchanged and reached first).
    ///
    /// THE FOOTPRINT ARM is reached only by a CRISP picker — the theme picker and the
    /// caret-style picker, the two whose job is previewing live world colours — over a
    /// composition that backs its rows with NOTHING
    /// ([`blur::footprint_frost_applies`]). Those two used to leave the document and
    /// the list interleaving glyph-for-glyph, because frost was a property of the plate
    /// and those compositions draw none. They now frost the card's own box and not one
    /// pixel outside it, so the surrounding page keeps the live colours the picker
    /// exists to show. No carve-outs inside the box: whatever the card covers is
    /// frosted, the caret included.
    ///
    /// THE FOOTPRINT'S EXTENT IS THE FROST'S ALONE. `overlay_card_rect` has a second
    /// consumer — the pointer's click-away and wheel hit-test — and the two now want
    /// different shapes: the frost leans and feathers, and the region a click means
    /// something in is still the box the ROWS occupy. So the shear and the feather are
    /// added HERE, on the way to [`blur::Frost`], and the hit region is left exactly
    /// the rect it always was. A click a hair outside the card still dismisses the
    /// picker even where the frost's skirt reaches, which is the right answer: the
    /// skirt is a defocus, not a surface.
    pub(in crate::render) fn frost_mode(&self) -> Option<blur::Frost> {
        // TRUE 1-BIT: a gaussian of a pure-black-or-white document smears every edge
        // into grey. That is true of a footprint too, so the exclusion is asked once,
        // above both arms.
        if theme::active().render_caps.backdrop == theme::Backdrop::Flat {
            return None;
        }
        if self.overlay_blur()
            || self.hud_showing()
            || crate::lifetime::lifetime_open()
            || crate::streaks::streaks_open()
            || self.peek_showing()
        {
            return Some(blur::Frost::Full);
        }
        if self.overlay_active
            && self.overlay_crisp
            && blur::footprint_frost_applies(crate::render::effective_list_style())
        {
            return self.overlay_card_rect().map(|rect| {
                let shear = self.footprint_shear();
                blur::Frost::Footprint(blur::Footprint {
                    rect: blur::footprint_box(rect, shear, self.footprint_upright_chrome()),
                    shear,
                })
            });
        }
        None
    }

    /// THE FOOTPRINT'S LEAN, READ FROM THE COMPOSITION THE FRAME DREW — physical px of
    /// horizontal displacement per physical px down.
    ///
    /// It is the MEASURED rail's own per-row step over the row pitch it steps across,
    /// which is exactly the slope of the two points `DiagonalClusterRail::spine`
    /// hands the spine quad: successive rows are one `overlay_lh` apart and one
    /// `spine_step` over. Reading the authored `ROW_STEP` instead would lean the frost
    /// more than the spine beside it actually rakes on a card too cramped to afford the
    /// step outright, because the rail resolves that yield (`TRAVEL_MAX_BAND_FRACTION`)
    /// and a constant cannot. The same pair `location_axis_deg` reads, for the same
    /// reason.
    ///
    /// ZERO when the frame drew no rail at all, which is how the enrolment derives:
    /// shear is a property of a spine, and an enrolled world with no spine — a `Rules`
    /// composition — takes the feather and keeps its upright rectangle without anything
    /// here naming it.
    /// THE CARD'S OWN CHROME THAT THE RAKE DOES NOT CARRY, as a box the frost's shape
    /// must contain — `None` when there is none.
    ///
    /// It is the HEAD band, and only the head band, and that is a measured claim rather
    /// than a survey: the rows and their accessory column are seated per row by the
    /// diagonal rail, and the foot band hangs on the same rail's own extrapolated line,
    /// so all three rake. The head band is the one `TextArea` still seated at the card's
    /// text edge, because a query FIELD is an input and mirroring one onto the spine would
    /// make its sigil travel as the user types.
    ///
    /// ⚠️ **The enumeration is not what the guarantee rests on.** `frost_footprint`'s
    /// coverage law derives the card's ink from the PIXELS — the same picker over an empty
    /// document — and requires every ink pixel to be frosted, so a fourth upright surface
    /// fails there by existing rather than by being remembered here.
    fn footprint_upright_chrome(&self) -> Option<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        self.overlay_head_band_ink(&geom, &plan)
    }

    fn footprint_shear(&self) -> f32 {
        let Some(step) = self.diagonal_cluster.map(|c| c.spine_step()) else {
            return 0.0;
        };
        let pitch = self.overlay_lh();
        if !step.is_finite() || !pitch.is_finite() || pitch <= 0.0 {
            return 0.0;
        }
        step / pitch
    }

    /// Is this frame's frost the FULL-canvas one? The question every consumer that
    /// changes the DOCUMENT ITSELF for the blur's sake must ask, rather than "is there
    /// a frost at all": under a footprint frost the document outside the card is still
    /// on screen, live, and must be exactly what an unfrosted frame draws.
    ///
    /// Its two consumers are both about the lava lamp. `lava::dither_for_blur`
    /// suppresses the authored ordered posterization because its grid aliases with the
    /// downsampled frost — but only the whole-canvas frost consumes the entire page, so
    /// under a footprint the page keeps the treatment its world asked for.
    /// `lava_blur_active` freezes the lamp's ambient animation because a cached backdrop
    /// makes a moving lamp behind it pure re-blur; under a footprint the lamp is still
    /// on screen outside the card, so it keeps moving and the frost inside follows it
    /// through the recompute signature.
    pub(in crate::render) fn full_frost(&self) -> bool {
        self.frost_mode() == Some(blur::Frost::Full)
    }

    /// Size the blur textures + decide whether the cached frosted backdrop must be
    /// RECOMPUTED this frame. Only does work while a blur-eligible overlay is up; the
    /// actual doc-capture + blur passes run in [`Self::render`] (they need the frame
    /// encoder). The recompute gate compares a signature of the doc/size/theme behind
    /// the overlay, so an idle overlay-open frame re-blurs nothing (DESIGN §6).
    pub(in crate::render) fn prepare_blur(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let Some(frost) = self.frost_mode() else {
            return;
        };
        let base100 = srgb_u8_to_linear3(theme::base_100().rgba_bytes());
        let recreated = self.blur.ensure(
            device,
            queue,
            crate::render::blur::BlurSurface {
                width,
                height,
                dpi: self.dpi,
            },
            base100,
            frost,
        );
        let sig = self.blur_signature(width, height);
        self.blur_recompute = recreated || self.blur_sig != Some(sig);
        if self.blur_recompute {
            self.blur_sig = Some(sig);
        }
    }

    /// A cheap signature of everything that affects the BACKDROP pixels: the canvas
    /// size + DPI, the active theme, the document's render state (reshape count,
    /// scroll, cursor, zoom, markdown-ness), and the PAGE / WRAP geometry. The live
    /// caret SPRING is deliberately excluded so an in-flight caret settle behind a
    /// freshly-opened overlay does not keep re-blurring — the backdrop is frozen the
    /// moment it is captured.
    ///
    /// The page/wrap piece fixes a real staleness bug: `reshape_count` only bumps on
    /// a TEXT reshape (`set_text`), not on a pure re-wrap from a width change (page
    /// drag, `C-x {`/`}`, a page-mode toggle) — `set_size`/`sync_wrap_width` re-wrap
    /// without touching `reshape_count`. So on a width-only change the cached frosted
    /// backdrop passed stale, rendering the OLD column behind a freshly-opened
    /// overlay. `prepare` calls `sync_wrap_width` before `prepare_blur`, so by the
    /// time this runs, `row_geom`'s generation (bumped by `RowGeom::invalidate`
    /// whenever the shaped runs actually re-wrap) already reflects this frame's wrap
    /// width — the same generation the squiggle/nit proto caches key on. Hashing
    /// `page::page_on()` + `page::measure()` alongside it also catches the rare case
    /// where those flip WITHOUT changing the resulting wrap width (e.g. toggling page
    /// mode when the window is already narrower than the measure) — the page surface
    /// itself still needs a recompute even though `row_geom` wouldn't invalidate.
    pub(in crate::render) fn blur_signature(&self, width: u32, height: u32) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        width.hash(&mut h);
        height.hash(&mut h);
        self.dpi.to_bits().hash(&mut h);
        theme::active().name.hash(&mut h);
        self.reshape_count.hash(&mut h);
        self.row_geom.generation().hash(&mut h);
        crate::page::page_on().hash(&mut h);
        crate::page::measure().hash(&mut h);
        self.rendered_scroll_top_px(self.scroll)
            .to_bits()
            .hash(&mut h);
        self.cursor_line.hash(&mut h);
        self.cursor_col.hash(&mut h);
        self.metrics.zoom.to_bits().hash(&mut h);
        self.md_enabled.hash(&mut h);
        self.lava_render_phase().to_bits().hash(&mut h);
        // WHAT the offscreen capture contains, not merely how it looks: while
        // the document is relocated into a workspace's comparison region the
        // backdrop is the GROUND ALONE (`render`'s comparison arm). Crossing that
        // line changes the captured pixels without changing anything above it —
        // Settings' workspace and History's, at one scroll and one size, would
        // otherwise sign identically and the frame would keep the wrong frost.
        self.comparison_viewport().is_some().hash(&mut h);
        h.finish()
    }
}

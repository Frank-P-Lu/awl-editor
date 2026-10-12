//! Bounded PDF page preview: shared image quads and glyphon control text.
use super::*;
use std::sync::Arc;

pub(super) struct PdfDialogLayer {
    view: Option<Arc<crate::pdf_dialog::View>>,
    prepared: bool,
    buffers: [GlyphBuffer; 8],
    renderer: TextRenderer,
    controls: SelectionPipeline,
    image: crate::image_pipeline::ImageQuadPipeline,
    texture: Option<(Arc<image::RgbaImage>, wgpu::Texture, wgpu::TextureView)>,
}

pub(super) struct Prepare<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub size: [u32; 2],
    pub dpi: f32,
    pub fs: &'a mut FontSystem,
    pub atlas: &'a mut TextAtlas,
    pub viewport: &'a Viewport,
    pub swash: &'a mut SwashCache,
}

impl PdfDialogLayer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cache: &Cache,
        format: wgpu::TextureFormat,
        fs: &mut FontSystem,
        atlas: &mut TextAtlas,
    ) -> Self {
        let _ = (queue, cache);
        Self {
            view: None,
            prepared: false,
            buffers: std::array::from_fn(|_| GlyphBuffer::new(fs, GlyphMetrics::new(14.0, 18.0))),
            renderer: TextRenderer::new(atlas, device, wgpu::MultisampleState::default(), None),
            controls: SelectionPipeline::new(
                device,
                &crate::selection::selection_shader(device),
                format,
                [0, 0, 0, 0],
            ),
            image: crate::image_pipeline::ImageQuadPipeline::new(device, format),
            texture: None,
        }
    }

    pub fn set_view(&mut self, view: Option<Arc<crate::pdf_dialog::View>>) {
        if view.is_none() {
            self.texture = None;
        }
        self.view = view;
        self.prepared = false;
    }
    pub fn active(&self) -> bool {
        self.view.is_some()
    }

    pub fn prepared(&self) -> bool {
        self.prepared
    }

    pub fn prepare(&mut self, context: Prepare<'_>) -> anyhow::Result<()> {
        self.prepared = false;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(view) = self.view.clone() else {
                self.texture = None;
                self.image.prepare(
                    context.device,
                    context.queue,
                    context.size[0],
                    context.size[1],
                    &[],
                );
                return Ok(());
            };
            let ratio = view.image.width() as f32 / view.image.height() as f32;
            let g = crate::pdf_dialog::Geometry::new(
                context.size[0] as f32,
                context.size[1] as f32,
                context.dpi,
                ratio,
            );
            self.prepare_page(&context, &g, &view);
            self.prepare_controls(&context, &g, &view);
            self.prepare_text(context, &g, &view)?;
            self.prepared = true;
        }
        #[cfg(target_arch = "wasm32")]
        let Prepare {
            device,
            queue,
            size,
            dpi,
            fs,
            atlas,
            viewport,
            swash,
        } = context;
        #[cfg(target_arch = "wasm32")]
        let _ = (device, queue, size, dpi, fs, atlas, viewport, swash);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn prepare_page(
        &mut self,
        context: &Prepare<'_>,
        g: &crate::pdf_dialog::Geometry,
        view: &crate::pdf_dialog::View,
    ) {
        let (device, queue, [width, height]) = (context.device, context.queue, context.size);
        let preview_size = [g.preview[2].ceil() as u32, g.preview[3].ceil() as u32];
        if self.texture.as_ref().is_none_or(|(old, texture, _)| {
            !Arc::ptr_eq(old, &view.image)
                || texture.width() != preview_size[0]
                || texture.height() != preview_size[1]
        }) {
            // Prefilter to physical output pixels: a single GPU bilinear sample
            // can miss thin glyphs when a small window minifies the whole page.
            let pixels = image::imageops::resize(
                view.image.as_ref(),
                preview_size[0],
                preview_size[1],
                image::imageops::FilterType::Triangle,
            );
            let size = wgpu::Extent3d {
                width: pixels.width(),
                height: pixels.height(),
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("PDF page preview"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                pixels.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pixels.width() * 4),
                    rows_per_image: Some(pixels.height()),
                },
                size,
            );
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.texture = Some((view.image.clone(), texture, texture_view));
        }
        let (_, _, texture_view) = self.texture.as_ref().unwrap();
        self.image.prepare(
            device,
            queue,
            width,
            height,
            &[crate::image_pipeline::PlacedImage {
                dst: g.preview,
                alpha: 1.0,
                corner: 0.0,
                view: texture_view,
            }],
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn prepare_controls(
        &mut self,
        context: &Prepare<'_>,
        g: &crate::pdf_dialog::Geometry,
        view: &crate::pdf_dialog::View,
    ) {
        let (device, queue, [width, height]) = (context.device, context.queue, context.size);
        let rgba = |c: theme::Srgb| [c.r, c.g, c.b, c.a];
        let mut quads: Vec<_> = g
            .controls
            .iter()
            .enumerate()
            .map(|(i, rect)| {
                (
                    *rect,
                    rgba(if i == view.selected {
                        theme::base_300()
                    } else {
                        theme::base_200()
                    }),
                )
            })
            .collect();
        let edge = context.dpi.max(1.0);
        let [x, y, w, h] = g.preview;
        quads.insert(
            0,
            (
                [x - edge, y - edge, w + 2.0 * edge, h + 2.0 * edge],
                rgba(theme::base_300()),
            ),
        );
        self.controls
            .prepare_multicolor(device, queue, width, height, &quads);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn prepare_text(
        &mut self,
        context: Prepare<'_>,
        g: &crate::pdf_dialog::Geometry,
        view: &crate::pdf_dialog::View,
    ) -> anyhow::Result<()> {
        let Prepare {
            device,
            queue,
            size: [width, height],
            dpi,
            fs,
            atlas,
            viewport,
            swash,
        } = context;
        let labels = view.labels();
        let texts = [
            "Export PDF".to_owned(),
            view.error
                .as_ref()
                .map(|e| format!("PDF error: {e}"))
                .unwrap_or_else(|| "← → change · Tab focus · Esc cancel".into()),
            labels[0].clone(),
            labels[1].clone(),
            labels[2].clone(),
            labels[3].clone(),
            labels[4].clone(),
            labels[5].clone(),
        ];
        for (i, (buffer, text)) in self.buffers.iter_mut().zip(texts.iter()).enumerate() {
            let available = if i < 2 {
                width as f32 - g.title[0] * 2.0
            } else {
                g.controls[i - 2][2] - 20.0 * dpi
            };
            Self::configure_buffer(fs, buffer, i, text, available, dpi);
        }
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let areas: Vec<_> = self
            .buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| {
                let (left, top) = if i == 0 {
                    (g.title[0], g.title[1])
                } else if i == 1 {
                    (g.hint[0], g.hint[1])
                } else {
                    let r = g.controls[i - 2];
                    (r[0] + 10.0 * dpi, r[1] + 3.0 * dpi)
                };
                TextArea {
                    buffer,
                    left,
                    top,
                    scale: 1.0,
                    bounds,
                    default_color: theme::base_content().to_glyphon(),
                    custom_glyphs: &[],
                }
            })
            .collect();
        self.renderer
            .prepare(device, queue, fs, atlas, viewport, areas, swash)
            .map_err(|e| anyhow::anyhow!("PDF dialog text: {e:?}"))?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn configure_buffer(
        fs: &mut FontSystem,
        buffer: &mut GlyphBuffer,
        i: usize,
        text: &str,
        available: f32,
        dpi: f32,
    ) {
        let (size, leading) = if i == 0 {
            (20.0, 26.0)
        } else if i == 1 {
            (12.0, 16.0)
        } else {
            (14.0, 18.0)
        };
        buffer.set_metrics(fs, GlyphMetrics::new(size * dpi, leading * dpi));
        buffer.set_size(fs, Some(available.max(1.0)), (i == 1).then_some(32.0 * dpi));
        buffer.set_wrap(
            fs,
            if i == 1 {
                Wrap::WordOrGlyph
            } else {
                Wrap::None
            },
        );
        buffer.set_text(
            fs,
            text,
            &Attrs::new().family(Family::Name(theme::active().font)),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(fs, false);
    }

    pub fn draw<'a>(
        &'a self,
        atlas: &'a TextAtlas,
        viewport: &'a Viewport,
        pass: &mut wgpu::RenderPass<'a>,
    ) -> anyhow::Result<()> {
        self.controls.draw(pass);
        self.image.draw(pass);
        self.renderer
            .render(atlas, viewport, pass)
            .map_err(|e| anyhow::anyhow!("PDF dialog text: {e:?}"))
    }
}

impl TextPipeline {
    /// The PDF layer owns its frame inputs and leaves the parked renderer alone.
    pub(in crate::render) fn prepare_pdf_dialog(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> anyhow::Result<bool> {
        let active = self.pdf_dialog_layer.active();
        if active {
            self.viewport.update(
                queue,
                Resolution {
                    width: size[0],
                    height: size[1],
                },
            );
        }
        self.pdf_dialog_layer.prepare(Prepare {
            device,
            queue,
            size,
            dpi: self.dpi,
            fs: &mut self.font_system,
            atlas: &mut self.atlas,
            viewport: &self.viewport,
            swash: &mut self.swash_cache,
        })?;
        Ok(active)
    }
}

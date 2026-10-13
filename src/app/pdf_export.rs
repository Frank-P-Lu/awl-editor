//! A native PDF audition snapshots prose and owns its controls until commit.
use crate::app::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::export::ImageSource;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
struct FrozenImages {
    source: crate::export::FsImages,
    cache:
        std::cell::RefCell<std::collections::BTreeMap<String, Option<crate::export::ExportImage>>>,
}
#[cfg(not(target_arch = "wasm32"))]
impl ImageSource for FrozenImages {
    fn resolve(&self, src: &str) -> Option<crate::export::ExportImage> {
        let mut cache = self.cache.borrow_mut();
        cache
            .entry(src.into())
            .or_insert_with(|| self.source.resolve(src))
            .clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct Origin {
    path: Option<PathBuf>,
    root: PathBuf,
    stem: String,
}

#[cfg(not(target_arch = "wasm32"))]
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

#[cfg(not(target_arch = "wasm32"))]
pub(in crate::app) struct Dialog {
    markdown: String,
    images: FrozenImages,
    theme_index: usize,
    origin: Origin,
    options: PdfOptions,
    artifact: PdfDocument,
    image: Arc<image::RgbaImage>,
    page: usize,
    selected: usize,
    revision: u64,
    error: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Dialog {
    fn build(
        markdown: String,
        images: crate::export::FsImages,
        options: PdfOptions,
        origin: Origin,
    ) -> Result<Self, String> {
        let images = FrozenImages {
            source: images,
            cache: Default::default(),
        };
        let theme_index = crate::theme::active_index();
        let artifact = PdfDocument::build(&markdown, &images, options)?;
        let image = Self::preview(&artifact, 0)?;
        Ok(Self {
            markdown,
            images,
            theme_index,
            origin,
            options,
            artifact,
            image,
            page: 0,
            selected: 0,
            revision: 0,
            error: None,
        })
    }

    fn preview(artifact: &PdfDocument, page: usize) -> Result<Arc<image::RgbaImage>, String> {
        let (width, _) = artifact.page_size();
        artifact.preview_page(page, 900.0 / width).map(Arc::new)
    }

    fn refresh(&mut self, options: PdfOptions, page: usize) -> Result<(), String> {
        let artifact = PdfDocument::build(&self.markdown, &self.images, options)?;
        let page = page.min(artifact.page_count().saturating_sub(1));
        let image = Self::preview(&artifact, page)?;
        self.artifact = artifact;
        self.options = options;
        self.error = None;
        self.image = image;
        self.page = page;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    fn change(&mut self, direction: isize) -> Result<(), String> {
        let mut options = self.options;
        let mut page = self.page;
        match self.selected {
            0 => {
                options.appearance = match options.appearance {
                    PdfAppearance::Plain => PdfAppearance::CurrentTheme(self.theme_index),
                    PdfAppearance::CurrentTheme(_) => PdfAppearance::Plain,
                }
            }
            1 => {
                options.paper = match options.paper {
                    PdfPaper::A4 => PdfPaper::Letter,
                    PdfPaper::Letter => PdfPaper::A4,
                }
            }
            2 => options.page_numbers = !options.page_numbers,
            3 => options.preserve_line_breaks = !options.preserve_line_breaks,
            4 => {
                page = page
                    .saturating_add_signed(direction)
                    .min(self.artifact.page_count().saturating_sub(1))
            }
            _ => return Ok(()),
        }
        if options != self.options {
            self.refresh(options, page)?;
        } else if page != self.page {
            self.image = Self::preview(&self.artifact, page)?;
            self.page = page;
            self.revision = self.revision.wrapping_add(1);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::app) fn fixture() -> Self {
        Self::build(
            "# Sample\n\nSynthetic prose.".into(),
            crate::export::FsImages { doc_dir: None },
            PdfOptions::default(),
            Origin {
                path: None,
                root: "/synthetic".into(),
                stem: "sample".into(),
            },
        )
        .unwrap()
    }

    pub(in crate::app) fn view(&self) -> Arc<crate::pdf_dialog::View> {
        Arc::new(crate::pdf_dialog::View {
            image: self.image.clone(),
            appearance: match self.options.appearance {
                PdfAppearance::Plain => "Plain paper",
                PdfAppearance::CurrentTheme(_) => "Current theme",
            }
            .into(),
            paper: match self.options.paper {
                PdfPaper::A4 => "A4",
                PdfPaper::Letter => "Letter",
            }
            .into(),
            page_numbers: self.options.page_numbers,
            preserve_line_breaks: self.options.preserve_line_breaks,
            page: self.page,
            pages: self.artifact.page_count(),
            selected: self.selected,
            revision: self.revision,
            error: self.error.clone(),
        })
    }
}

impl App {
    pub(in crate::app) fn pdf_export_view(&self) -> Option<Arc<crate::pdf_dialog::View>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.workspace_state.pdf_export().map(Dialog::view)
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl App {
    pub(in crate::app) fn open_pdf_export(&mut self) {
        if !self.document.active_is_markdown() {
            self.set_sticky_notice("can't export a non-Markdown file");
            return;
        }
        let options = PdfOptions {
            page_numbers: self.config.pdf_page_numbers.unwrap_or(true),
            preserve_line_breaks: self.config.pdf_preserve_line_breaks.unwrap_or(true),
            paper: if self.config.pdf_paper.as_deref() == Some("letter") {
                PdfPaper::Letter
            } else {
                PdfPaper::A4
            },
            appearance: if self.config.pdf_appearance.as_deref() == Some("plain") {
                PdfAppearance::Plain
            } else {
                PdfAppearance::CurrentTheme(crate::theme::active_index())
            },
        };
        let images = crate::export::FsImages {
            doc_dir: self
                .document
                .buffer()
                .path()
                .and_then(|p| p.parent())
                .map(Path::to_path_buf),
        };
        let markdown = self.document.buffer().text();
        let origin = Origin {
            path: self.document.buffer().path().map(Path::to_path_buf),
            root: self.project_location.root.clone(),
            stem: crate::web_export::export_stem(self.document.buffer()),
        };
        match Dialog::build(markdown, images, options, origin) {
            Ok(dialog) => {
                self.input.retire_pointer_gestures();
                if let Some(gpu) = self.frame.gpu_mut() {
                    gpu.pipeline.set_image_preview(None);
                    gpu.pipeline.set_page_drag_readout(None);
                    gpu.window.set_cursor(winit::window::CursorIcon::Default);
                }
                self.workspace_state.set_pdf_export(Some(dialog));
                self.reconcile_text_focus();
            }
            Err(e) => self.report_export_failure(&e),
        }
        self.sync_view(false);
        self.request_frame();
    }

    pub(in crate::app) fn apply_pdf_export_action(&mut self, action: &Action) {
        let Some(mut dialog) = self.workspace_state.take_pdf_export() else {
            return;
        };
        match action {
            Action::Cancel | Action::Quit => {}
            Action::BackwardChar | Action::ForwardChar => {
                let direction = if matches!(action, Action::BackwardChar) {
                    -1
                } else {
                    1
                };
                if let Err(e) = dialog.change(direction) {
                    dialog.error = Some(e.clone());
                    self.report_export_failure(&e);
                }
                self.workspace_state.set_pdf_export(Some(dialog));
            }
            Action::PreviousLine | Action::NextLine | Action::InsertTab | Action::Outdent => {
                dialog.selected = match action {
                    Action::PreviousLine => dialog.selected.saturating_sub(1),
                    Action::NextLine => (dialog.selected + 1).min(6),
                    Action::Outdent => (dialog.selected + 6) % 7,
                    _ => (dialog.selected + 1) % 7,
                };
                dialog.revision = dialog.revision.wrapping_add(1);
                self.workspace_state.set_pdf_export(Some(dialog));
            }
            Action::Newline | Action::InsertChar(' ') => match dialog.selected {
                5 => self.finish_pdf_export(dialog),
                6 => {}
                _ => {
                    if let Err(e) = dialog.change(1) {
                        dialog.error = Some(e.clone());
                        self.report_export_failure(&e);
                    }
                    self.workspace_state.set_pdf_export(Some(dialog));
                }
            },
            _ => self.workspace_state.set_pdf_export(Some(dialog)),
        }
        self.sync_view(false);
        self.request_frame();
    }

    pub(in crate::app) fn apply_pdf_semantic(
        &mut self,
        request: crate::semantic::SemanticRequest,
    ) -> bool {
        use crate::semantic::SemanticRequest;
        let (id, action) = match request {
            SemanticRequest::Focus { id } => (id, None),
            SemanticRequest::Click { id } => (id, Some(Action::Newline)),
            SemanticRequest::Increment { id } => (id, Some(Action::ForwardChar)),
            SemanticRequest::Decrement { id } => (id, Some(Action::BackwardChar)),
            _ => return false,
        };
        let Some(index) = id
            .strip_prefix("pdf-export.control.")
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|i| *i < 7)
        else {
            return false;
        };
        if let Some(mut dialog) = self.workspace_state.take_pdf_export() {
            dialog.selected = index;
            self.workspace_state.set_pdf_export(Some(dialog));
            if let Some(action) = action {
                self.apply_pdf_export_action(&action);
            } else {
                self.sync_view(false);
                self.request_frame();
            }
            true
        } else {
            false
        }
    }

    pub(in crate::app) fn click_pdf_export(&mut self, x: f32, y: f32) {
        let Some(view) = self.pdf_export_view() else {
            return;
        };
        let Some(gpu) = self.frame.gpu() else {
            return;
        };
        let geometry = crate::pdf_dialog::Geometry::new(
            gpu.config.width as f32,
            gpu.config.height as f32,
            self.frame.dpi(),
            view.image.width() as f32 / view.image.height() as f32,
        );
        if let Some(index) = geometry.hit(x, y)
            && let Some(mut dialog) = self.workspace_state.take_pdf_export()
        {
            dialog.selected = index;
            self.workspace_state.set_pdf_export(Some(dialog));
            let action = if index == 4 {
                let rect = geometry.controls[4];
                if x < rect[0] + rect[2] * 0.5 {
                    Action::BackwardChar
                } else {
                    Action::ForwardChar
                }
            } else {
                Action::Newline
            };
            self.apply_pdf_export_action(&action);
        }
    }

    fn finish_pdf_export(&mut self, mut dialog: Dialog) {
        let bytes = match dialog.artifact.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                dialog.error = Some(e.clone());
                self.report_export_failure(&e);
                self.workspace_state.set_pdf_export(Some(dialog));
                return;
            }
        };
        let target = super::files::export::export_target(
            dialog.origin.path.as_deref(),
            &dialog.origin.root,
            &dialog.origin.stem,
            crate::export::Format::Pdf,
            None,
        );
        #[cfg(target_os = "macos")]
        let target = if self.frame.gpu().is_some() {
            let dir = target.path.parent().unwrap_or(&dialog.origin.root);
            let name = target
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            crate::mac_chrome::pick_save_destination(dir, &name).map(|chosen| {
                super::files::export::ExportTarget::at(chosen, dialog.origin.path.as_deref())
            })
        } else {
            Some(target)
        };
        #[cfg(not(target_os = "macos"))]
        let target = Some(target);
        self.commit_pdf_destination(dialog, bytes, target);
    }

    fn commit_pdf_destination(
        &mut self,
        mut dialog: Dialog,
        bytes: Vec<u8>,
        target: Option<super::files::export::ExportTarget>,
    ) {
        let Some(target) = target else {
            self.workspace_state.set_pdf_export(Some(dialog));
            return;
        };
        if self.write_export(&bytes, target) {
            let appearance = if matches!(dialog.options.appearance, PdfAppearance::Plain) {
                "\"plain\""
            } else {
                "\"current_theme\""
            };
            let paper = if matches!(dialog.options.paper, PdfPaper::Letter) {
                "\"letter\""
            } else {
                "\"a4\""
            };
            self.persist_pref("pdf_appearance", appearance);
            self.persist_pref("pdf_paper", paper);
            self.persist_pref(
                "pdf_preserve_line_breaks",
                if dialog.options.preserve_line_breaks {
                    "true"
                } else {
                    "false"
                },
            );
            self.persist_pref(
                "pdf_page_numbers",
                if dialog.options.page_numbers {
                    "true"
                } else {
                    "false"
                },
            );
        } else {
            dialog.error = self.frame.notice().text().map(str::to_owned);
            self.workspace_state.set_pdf_export(Some(dialog));
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;

//! Typed numeric settings and range-row live/persist doors.

use crate::app::*;

impl App {
    pub(in crate::app) fn setting_value_commit(&mut self, key: &str, raw: &str) {
        match key {
            "page_width_prose" | "page_width_code" => {
                let id = if key == "page_width_prose" {
                    crate::settings::SettingId::PageWidthProse
                } else {
                    crate::settings::SettingId::PageWidthCode
                };
                let spec = crate::settings::range_spec(id).unwrap();
                if let Some(value) = spec.parse(raw) {
                    self.range_apply_live(id, value);
                    self.range_persist(key);
                    self.sync_view(true);
                }
            }
            "zoom" => {
                if let Some(z) = crate::settings::parse_zoom(raw) {
                    self.set_zoom(z);
                    self.settle_zoom_persist();
                    self.sync_view(true);
                }
            }
            "scroll_sensitivity" => {
                if let Some(s) = crate::range::SCROLL_SENSITIVITY.parse(raw) {
                    self.scroll_sensitivity = s;
                    crate::settings::set_scroll_sensitivity(s);
                    self.persist_pref(key, &crate::range::SCROLL_SENSITIVITY.persist_value(s));
                }
            }
            _ => {}
        }
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        self.refresh_settings_overlay();
    }

    pub(in crate::app) fn setting_range_step(&mut self, key: &str) {
        if let Some(cell) = self.overlay.as_ref().and_then(|o| o.selected_range())
            && crate::settings::value_key(cell.id) == Some(key)
        {
            let spec = crate::settings::range_spec(cell.id).unwrap();
            self.range_apply_live(cell.id, spec.value_of_step(cell.step));
        }
        if key == "zoom" {
            self.zoom_reflow.queue();
        } else if key == "scroll_sensitivity" {
            self.scroll_sensitivity = crate::settings::scroll_sensitivity();
        }
        self.range_persist(key);
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
        self.refresh_settings_overlay();
    }

    pub(in crate::app) fn range_apply_live(&mut self, id: crate::settings::SettingId, value: f32) {
        if id == crate::settings::SettingId::Zoom {
            self.set_zoom(value);
        } else if id == crate::settings::SettingId::ScrollSensitivity {
            self.scroll_sensitivity = crate::range::SCROLL_SENSITIVITY.quantize(value);
            self.config.scroll_sensitivity = Some(self.scroll_sensitivity);
            crate::settings::set_scroll_sensitivity(self.scroll_sensitivity);
        } else if matches!(
            id,
            crate::settings::SettingId::PageWidthProse | crate::settings::SettingId::PageWidthCode
        ) {
            let spec = crate::settings::range_spec(id).unwrap();
            let width = spec.quantize(value) as usize;
            let class = match id {
                crate::settings::SettingId::PageWidthProse => {
                    self.config.page_width_prose = Some(width);
                    crate::page::PageClass::Prose
                }
                crate::settings::SettingId::PageWidthCode => {
                    self.config.page_width_code = Some(width);
                    crate::page::PageClass::Code
                }
                _ => unreachable!(),
            };
            if self.active.buffer.page_class() == class {
                self.sync_page_measure();
            }
        }
    }

    pub(in crate::app) fn range_persist(&mut self, key: &str) {
        if key == "zoom" {
            self.settle_zoom_persist()
        } else if key == "scroll_sensitivity" {
            self.scroll_sensitivity = crate::settings::scroll_sensitivity();
            self.persist_pref(
                key,
                &crate::range::SCROLL_SENSITIVITY.persist_value(self.scroll_sensitivity),
            );
        } else if matches!(key, "page_width_prose" | "page_width_code") {
            let (spec, width) = if key == "page_width_prose" {
                (
                    &crate::range::PAGE_WIDTH_PROSE,
                    self.config.measure_for(crate::page::PageClass::Prose),
                )
            } else {
                (
                    &crate::range::PAGE_WIDTH_CODE,
                    self.config.measure_for(crate::page::PageClass::Code),
                )
            };
            self.persist_pref(key, &spec.persist_value(width as f32));
        }
    }
}

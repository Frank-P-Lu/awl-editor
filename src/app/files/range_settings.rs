//! Typed numeric settings and range-row live/persist doors.

use crate::app::*;

impl App {
    pub(in crate::app) fn setting_value_commit(&mut self, key: &str, raw: &str) {
        match key {
            "page_width_prose" | "page_width_code" => {
                if let Ok(n) = raw.trim().parse::<usize>() {
                    let clamped = crate::settings::clamp_page_width(n);
                    self.persist_pref(key, &clamped.to_string());
                    self.sync_page_measure();
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
        }
    }
}

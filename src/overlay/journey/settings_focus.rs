//! Settings' exact recipient within the journey's coarse workspace stage.

use super::Journey;
use crate::overlay::{OverlayKind, workspace::SettingsFocus};

impl Journey {
    /// The exact Settings recipient, distinct within the lifecycle's coarse
    /// primary/detail projection. `None` when Settings is not the active card.
    pub fn settings_focus(&self) -> Option<SettingsFocus> {
        self.card()
            .is_some_and(|card| card.kind == OverlayKind::Settings)
            .then_some(self.settings_focus)
    }

    /// Walk Settings' focus stops, skipping Controls when the filter has no
    /// matches. Every writer stays beside the lifecycle projection it updates.
    pub fn step_settings_focus(&mut self, delta: isize) -> bool {
        let Some(card) = self.card() else {
            return false;
        };
        if card.kind != OverlayKind::Settings {
            return false;
        }
        let next = self.settings_focus.step(delta, !card.items.is_empty());
        self.set_settings_focus(next)
    }

    /// Put Settings on one named recipient. A request for Controls with no
    /// matching row lands on Search, the only useful input at that point.
    pub fn focus_settings(&mut self, requested: SettingsFocus) -> bool {
        let Some(card) = self.card() else {
            return false;
        };
        if card.kind != OverlayKind::Settings {
            return false;
        }
        let focus = if requested == SettingsFocus::Controls && card.items.is_empty() {
            SettingsFocus::Search
        } else {
            requested
        };
        self.set_settings_focus(focus)
    }

    pub(super) fn set_settings_focus(&mut self, focus: SettingsFocus) -> bool {
        let Some(card) = self.card_mut() else {
            return false;
        };
        if card.kind != OverlayKind::Settings {
            return false;
        }
        card.detail_focus = focus != SettingsFocus::Categories;
        self.settings_focus = focus;
        true
    }
}

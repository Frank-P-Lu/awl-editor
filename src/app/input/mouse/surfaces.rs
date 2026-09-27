//! Press handling for the search panel and drawn menu bar.

use crate::app::*;

impl App {
    /// Route every panel control through the shared search action door. Interior
    /// gaps consume the press; `false` allows an off-panel press to reach text.
    pub(in crate::app) fn panel_click(&mut self, exit: &dyn schedule::Exit) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let hit = self.frame.gpu().and_then(|g| g.pipeline.panel_hit(px, py));
        let control = match hit {
            Some(crate::render::PanelHit::Close) => Some(crate::search::PanelControl::Close),
            Some(crate::render::PanelHit::RevealReplace) => {
                Some(crate::search::PanelControl::RevealReplace)
            }
            Some(crate::render::PanelHit::CaseToggle) => {
                Some(crate::search::PanelControl::CaseToggle)
            }
            Some(crate::render::PanelHit::NavPrev) => Some(crate::search::PanelControl::NavPrev),
            Some(crate::render::PanelHit::NavNext) => Some(crate::search::PanelControl::NavNext),
            Some(crate::render::PanelHit::ReplaceButton) => {
                Some(crate::search::PanelControl::ReplaceButton)
            }
            Some(crate::render::PanelHit::ReplaceAllButton) => {
                Some(crate::search::PanelControl::ReplaceAllButton)
            }
            Some(crate::render::PanelHit::Find) => Some(crate::search::PanelControl::FocusFind),
            Some(crate::render::PanelHit::Replace) => {
                Some(crate::search::PanelControl::FocusReplace)
            }
            Some(crate::render::PanelHit::Elsewhere) => None,
            None => return false,
        };
        if let Some(control) = control {
            self.apply(
                Action::SearchPanel(control),
                false,
                exit,
                crate::stats::Door::Chord,
            );
        }
        self.sync_view(true);
        self.request_frame();
        true
    }

    /// Handle the drawn menu through catalog actions. Titles toggle their dropdown
    /// and dismiss conflicting pickers; click-away closes a dropdown. Dead menu
    /// chrome consumes the press without moving the document caret.
    pub(in crate::app) fn menubar_press(&mut self, exit: &dyn schedule::Exit) -> bool {
        if !crate::menubar::menu_bar_on() {
            return false;
        }
        let (px, py) = self.input.pointer.cursor_px;
        let (item_hit, title_hit, over_surface) = {
            let Some(gpu) = self.frame.gpu() else {
                return false;
            };
            (
                gpu.pipeline.menubar_item_at(px, py),
                gpu.pipeline.menubar_title_at(px, py),
                gpu.pipeline.over_menu_surface(px, py),
            )
        };
        if let Some((menu, item)) = item_hit {
            crate::menubar::set_open(None);
            let action = {
                let menus = crate::menu::roster();
                menus.get(menu).and_then(|menu| {
                    crate::menu::dropdown_action(menu, item, self.document.active_is_markdown())
                })
            };
            if let Some(action) = action {
                let exited = self.apply(action, false, exit, crate::stats::Door::Menu);
                if exited {
                    return true;
                }
            }
            self.sync_view(true);
            return true;
        }
        if let Some(i) = title_hit {
            crate::menubar::toggle_open(i);
            self.workspace_state.dismiss_pickers();
            self.sync_view(true);
            return true;
        }
        if crate::menubar::open_menu().is_some() {
            crate::menubar::set_open(None);
            self.sync_view(true);
            return true;
        }
        // 4. A press on the bar's own dead strip: swallow (never a caret move beneath it).
        over_surface
    }
}

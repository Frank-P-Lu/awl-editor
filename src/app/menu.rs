//! The live App's half of the native macOS MENU BAR (`crate::menu` owns the
//! pure roster/routing table + the muda construction; this file is the
//! App-specific wiring — routing a fired menu item's id into the SAME
//! `App::apply` seam every keypress uses). Native macOS only
//! (`cfg(target_os = "macos")`); see `crate::menu`'s module doc for the full
//! design-law + accelerator/Quit decisions.
//!
//! **Edit menu correctness note (why routed items, not muda's predefined
//! Cut/Copy/Paste/Undo/Redo):** muda's `PredefinedMenuItem::cut/copy/paste/
//! select_all/undo/redo` work by sending AppKit selectors (`cut:`, `copy:`,
//! …) up the RESPONDER CHAIN to the key window's `firstResponder` — the
//! mechanism a standard `NSTextView` implements for free. awl's document view
//! is a raw wgpu-rendered `NSView` (via winit) that implements none of those
//! selectors, so a predefined item would validate/fire against nothing and
//! silently no-op. Routing Edit's items through the SAME id → `Action` table
//! every other menu uses instead (`Action::Undo`/`CopyRegion`/`KillRegion`/
//! `Yank`/`SelectAll`, all already fired via clipboard mirroring in
//! `App::apply` — see `actions.rs`'s module doc) is both the ONLY choice that
//! actually works against this app's view and the one consistent with the
//! module's "every item fires an existing catalog Action" law. The "free
//! correctness win" the mac-citizen brief names is satisfied a different way
//! than muda's out-of-the-box predefined items: simply having a populated
//! Edit menu (regardless of how its items dispatch) is what lets macOS offer
//! its Edit-menu-anchored text services (the Character Viewer / Emoji &
//! Symbols item, Services menu entries) at all — a structural presence
//! requirement, not a responder-chain one.
use super::*;

#[cfg(target_os = "macos")]
impl App {
    /// A menu item fired (posted via `EventLoopProxy::send_event`, so this
    /// always runs on the normal winit thread — the same cross-thread-safety
    /// shape as `handle_daemon_event`). Resolves `id` through `crate::menu`'s
    /// ONE routing table and fires it through the SAME `App::apply` seam a
    /// keypress uses (`shift: false` — a menu click carries no modifier-hold
    /// concept); an id the table doesn't own (a predefined item muda itself
    /// handled, or a stray event) is a silent no-op, never a panic.
    ///
    /// A menu event arrives via `user_event`, NOT via `window_event`, so — like
    /// the daemon handler (the closest reference: also `user_event`-borne, also
    /// changes state and must paint) — it does NOT ride the keyboard/mouse
    /// handlers' trailing `sync_view` + `request_redraw`. So MIRROR the keyboard
    /// path's exact post-`apply` work here (`on_keyboard_input`, `app/input/keys.rs`):
    /// `sync_view(true)` rebuilds the ViewState the pipeline draws, and
    /// The transition's typed render requests own the sync + redraw, exactly
    /// as on the keyboard door; this handler adds no trailing repaint path.
    pub(super) fn handle_menu_event(&mut self, id: String, exit: &dyn schedule::Exit) {
        if crate::menu::resolve(&id)
            .is_some_and(|action| self.reject_menu_without_document(&action))
        {
            return;
        }
        // A row `crate::menu::NATIVE_PANEL_IDS` claims for the platform opens a
        // real AppKit panel instead of dispatching its routed action — the macOS
        // convention winning over the in-app overlay, and dodging that repaint
        // path entirely. The keyboard chords and the Cmd-P palette rows for the
        // same commands are UNCHANGED on every platform — only the MENU's rows
        // are redirected.
        if crate::menu::opens_native_panel(&id) {
            self.run_native_panel(&id, exit);
            return;
        }
        if let Some(action) = crate::menu::resolve(&id) {
            // MENU door: a click in the macOS menu bar (a SLOW discovery surface) —
            // attributed to `Door::Menu` in the silent usage ledger.
            self.apply(action, false, exit, crate::stats::Door::Menu);
        }
    }

    /// Answer one `NATIVE_PANEL_IDS` row with its AppKit panel. The id → panel
    /// routing goes through `crate::menu::resolve`'s existing table rather than a
    /// second id list, so a row can only reach a panel here if the roster already
    /// resolves it to a catalog action.
    ///
    /// ⚠️ **EVERY PANEL HERE IS APPLICATION-MODAL** — `runModal` blocks the
    /// process main thread until a human closes it. The export door carries the
    /// surface gate inside `export_via_platform_panel` (its doc says why); the
    /// open door predates it and is reachable only from a real `NSMenu` click,
    /// which cannot happen without a window.
    ///
    /// An id the table claims but this match does not place falls back to the
    /// routed action, so a roster edit degrades to the in-app surface rather than
    /// to nothing.
    fn run_native_panel(&mut self, id: &str, exit: &dyn schedule::Exit) {
        use crate::export::Format;
        use crate::keymap::Action;
        let format = match crate::menu::resolve(id) {
            // File ▸ "Browse files…" → `NSOpenPanel`. On OK it loads the chosen
            // path through the SAME `load_path` every open uses (which itself
            // syncs); then paint, per the post-`apply` pattern above. Cancel /
            // off-main-thread is a calm no-op.
            Some(Action::OpenBrowse) => {
                if let Some(path) =
                    crate::mac_chrome::pick_file_to_open(Some(&self.project_location.root))
                {
                    self.apply_file_choice(Some(path));
                    self.request_frame();
                }
                return;
            }
            Some(Action::ExportWord) => Some(Format::Docx),
            Some(Action::ExportHtml) => Some(Format::Html),
            Some(Action::ExportPdf) => Some(Format::Pdf),
            _ => None,
        };
        match format {
            // The write, the notice and the Finder reveal are the ordinary export
            // path — only the destination came from somewhere else.
            Some(format) => {
                if self.export_via_platform_panel(format) {
                    self.request_frame();
                }
            }
            None => {
                if let Some(action) = crate::menu::resolve(id) {
                    self.apply(action, false, exit, crate::stats::Door::Menu);
                }
            }
        }
    }
}

impl App {
    #[cfg(target_os = "macos")]
    pub(super) fn install_native_menu(
        &mut self,
        proxy: winit::event_loop::EventLoopProxy<AwlEvent>,
    ) {
        self._menu_bar = Some(crate::menu::install(
            proxy,
            AwlEvent::Menu,
            self.document.active_is_markdown(),
        ));
    }

    pub(super) fn sync_menu_context_and_gpu_absent(&self) -> bool {
        #[cfg(target_os = "macos")]
        if let Some(menu) = self._menu_bar.as_ref() {
            menu.set_markdown_enabled(self.document.active_is_markdown());
        }
        self.frame.gpu().is_none()
    }
}

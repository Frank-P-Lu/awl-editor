//! Live winit application lifecycle callbacks.

use super::*;

impl ApplicationHandler<AwlEvent> for App {
    /// A daemon event or (macOS only) a fired menu item, posted by their
    /// respective source (the daemon's accept-loop thread / muda's global
    /// event handler) via `EventLoopProxy::send_event` — always runs on this,
    /// the normal winit thread. A no-op on wasm (there is no `AwlEvent`
    /// variant to construct there).
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: AwlEvent) {
        #[cfg(not(target_arch = "wasm32"))]
        match _event {
            #[cfg(not(feature = "mas"))]
            AwlEvent::Daemon(e) => self.handle_daemon_event(e),
            #[cfg(target_os = "macos")]
            AwlEvent::Menu(id) => self.handle_menu_event(id, _event_loop),
            AwlEvent::Probe(e) => self.handle_probe_event(_event_loop, e),
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        if self.recovery_window.is_some() {
            self.rebuild_gpu(event_loop, "graphics resumed");
            return;
        }
        // THE PURE title string (`app::files::window_title`) — same owner
        // `App::update_title` uses on every later open/switch/theme-cycle, so
        // the very first frame's window title already names the document (and
        // the active world) rather than starting bare and waiting for the
        // first `update_title()` call to catch up.
        let title = files::window_title(
            self.document.buffer().path(),
            self.document.buffer().is_unnamed_fresh(),
            crate::theme::active().name,
            self.is_document_dirty(),
        );
        // MINIMUM window size, tied to the font metrics so the window can never be
        // dragged below roughly ONE readable line. Width = ~30 columns at the default
        // advance plus the side insets; height = a handful of lines plus the top inset.
        // Below this the responsive page column would have nothing left to show, so we
        // stop the drag here (LOGICAL px, so it scales with the monitor's DPI).
        const MIN_COLS: f32 = 30.0;
        const MIN_LINES: f32 = 8.0;
        let min_w = MIN_COLS * render::CHAR_WIDTH + 2.0 * render::TEXT_LEFT;
        let min_h = MIN_LINES * render::LINE_HEIGHT + 2.0 * render::TEXT_TOP;
        // NATIVE ONLY: pin a fixed opening size (1200x800 logical px — also the
        // capture harness's own default canvas, so `--screenshot` stays
        // byte-identical). On the WEB this must NOT be set: winit's web backend
        // maps `with_inner_size` straight onto an INLINE `style.width`/
        // `style.height` on the `<canvas>` (`web_sys::set_canvas_size`), which
        // permanently pins the element at that pixel size and overrides
        // index.html's responsive `width:100vw;height:100vh` CSS outright — a
        // viewport under 1200x800 then clips unreachably (`body{overflow:
        // hidden}`). Leaving `inner_size` unset means winit only ever writes
        // `min-width`/`min-height` (a floor, not a pin) and the canvas keeps its
        // CSS-driven size; winit's web backend installs a `ResizeObserver` on the
        // canvas unconditionally (`window_target.rs`'s `on_resize_scale`, wired
        // regardless of whether `inner_size` was ever set) that fires
        // `WindowEvent::Resized` on every CSS/viewport size change — the SAME
        // generic `Resized` arm below already re-syncs the layout on that event,
        // so no bespoke web resize plumbing (no `ResizeObserver` of our own) is
        // needed; winit already tracks the browser viewport for us.
        // SESSION RESTORE (native only): a previous session's window FRAME wins
        // over the fixed default, RE-CLAMPED here against the CURRENTLY connected
        // screens (`Self::apply_session_restore` already loaded + stashed it in
        // `self.restored_window`, but screens can change between quit and this
        // very relaunch — a disconnected external monitor must never strand the
        // window off every visible display). `None` (no session, kill-switch
        // off, or first-ever launch) falls back to the pre-existing fixed
        // 1200x800 default, so a plain `--screenshot` and a fresh install are
        // both unaffected.
        #[cfg(not(target_arch = "wasm32"))]
        let attrs = {
            let attrs = Window::default_attributes()
                .with_min_inner_size(LogicalSize::new(min_w, min_h))
                .with_title(if self.soak.is_some() {
                    "Awl GPU probe — keep visible".to_string()
                } else {
                    title
                })
                .with_visible(true);
            // LIVE PROBE: a small, corner-anchored, DETERMINISTIC window
            // (`crate::probe::PROBE_LOGICAL_*`). Overrides any restored session
            // frame — a probe run is isolated (temp HOME) and must land in a
            // known small corner, not wherever the last real window happened to
            // sit. Anchored near the top-left, clear of the menu bar.
            // ALWAYS-ON-TOP is the occlusion cure: a non-activating (Prohibited)
            // window never comes to front, so it would otherwise sit OCCLUDED
            // behind the user's windows and wgpu would skip every present (the
            // occlusion tripwire) — leaving the harness blind to the very
            // present-race it exists to catch. `WindowLevel::AlwaysOnTop` floats
            // it above other windows so it stays unoccluded and presents fire,
            // WITHOUT making it key (window LEVEL is z-order, not focus — verified
            // FOCUS-GAINED stays 0). Small + cornered keeps the always-on-top
            // window out of the way.
            if crate::probe::live_active() {
                attrs
                    .with_inner_size(LogicalSize::new(
                        crate::probe::PROBE_LOGICAL_W,
                        crate::probe::PROBE_LOGICAL_H,
                    ))
                    .with_position(winit::dpi::LogicalPosition::new(48.0, 64.0))
                    // `with_active(false)` → winit shows the window via
                    // `orderFront` instead of `makeKeyAndOrderFront`, so it never
                    // becomes the KEY window (no keyboard-focus theft). Paired with
                    // the Prohibited policy + `activate_ignoring_other_apps(false)`
                    // in `crate::app::run`. Only the probe opts out of focus; a
                    // normal launch keeps the default active window.
                    .with_active(false)
            } else {
                match self.restored_window {
                    Some(frame) => {
                        let screens: Vec<crate::session::ScreenRect> = event_loop
                            .available_monitors()
                            .map(|m| {
                                let pos = m.position();
                                let size = m.size();
                                crate::session::ScreenRect {
                                    x: pos.x,
                                    y: pos.y,
                                    width: size.width,
                                    height: size.height,
                                }
                            })
                            .collect();
                        let clamped = crate::session::clamp_frame_to_screens(frame, &screens);
                        attrs
                            .with_inner_size(winit::dpi::PhysicalSize::new(
                                clamped.width,
                                clamped.height,
                            ))
                            .with_position(winit::dpi::PhysicalPosition::new(clamped.x, clamped.y))
                    }
                    None => attrs.with_inner_size(LogicalSize::new(1200.0, 800.0)),
                }
            }
        };
        #[cfg(target_arch = "wasm32")]
        let attrs = Window::default_attributes()
            .with_min_inner_size(LogicalSize::new(min_w, min_h))
            .with_title(title);
        #[cfg(target_arch = "wasm32")]
        let attrs = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("awl-canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            attrs.with_canvas(canvas)
        };
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        self.recovery_window = Some(window.clone());
        // Ask the platform to deliver IME events so CJK (Japanese) composition
        // works: without this, WindowEvent::Ime is never sent and the user can
        // only type raw ASCII. Safe to call unconditionally; platforms without an
        // IME simply never emit the events.
        window.set_ime_allowed(true);
        let display_handle = event_loop.owned_display_handle();

        // NATIVE: the main thread is free to block on GPU init (pollster), so the
        // GPU is ready synchronously and we finish init inline.
        #[cfg(not(target_arch = "wasm32"))]
        match pollster::block_on(Gpu::new(window, display_handle)) {
            Ok(gpu) => {
                self.gpu = Some(gpu);
                self.gpu_lifecycle = GpuLifecycle::Active { oom_skips: 0 };
                self.on_gpu_ready();
                // NATIVE MACOS MENU BAR: install now that the window (and
                // therefore NSApp) exists — `Menu::init_for_nsapp` and the
                // root `Menu`'s own construction both require the real
                // process main thread, which `resumed()` always runs on.
                // `menu_proxy` is `take()`n so a later `resumed()` call (the
                // `gpu.is_some()` guard at the top already prevents that
                // today) could never double-install. The returned `Menu` is
                // STORED in `self._menu_bar`, never just dropped — see that
                // field's doc + `crate::menu::install`'s doc for the
                // use-after-free this fixes (every native `NSMenuItem` keeps
                // a raw, non-retaining pointer into this value's Rc chain).
                #[cfg(target_os = "macos")]
                if let Some(proxy) = self.menu_proxy.take() {
                    self.install_native_menu(proxy);
                    // TEMPLATE ICONS: mark every routed item's NSImage a template
                    // image so AppKit tints it to the current appearance's label
                    // ink (and the correct on-highlight tint) instead of the
                    // pre-baked flat gray `menu_icons.rs` draws — must run AFTER
                    // `install` has handed the real NSMenu tree to AppKit.
                    crate::mac_chrome::mark_menu_icons_as_templates();
                    crate::app_icon::adopt(&crate::theme::active());
                }
            }
            Err(e) => {
                eprintln!("failed to init render state: {e}");
                self.set_sticky_notice("graphics unavailable — closing safely");
                event_loop.exit();
            }
        }

        // WASM: the browser main thread CANNOT block, so adapter/device request is
        // an async that we drive on the microtask queue via `spawn_local`. The
        // finished GPU is parked in a shared slot; the trailing `request_redraw`
        // wakes `window_event`, which installs it and runs `on_gpu_ready` on the
        // first frame. (The event-loop borrow can't cross the await, hence the slot.)
        #[cfg(target_arch = "wasm32")]
        {
            self.gpu_lifecycle = GpuLifecycle::Rebuilding;
            let slot = self.gpu_pending.clone();
            let win = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match Gpu::new(window, display_handle).await {
                    Ok(gpu) => {
                        *slot.borrow_mut() = Some(Ok(gpu));
                        super::redraw::request_window(&win);
                    }
                    Err(e) => {
                        *slot.borrow_mut() = Some(Err(e.to_string()));
                        super::redraw::request_window(&win);
                    }
                }
            });
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.gpu = None;
        self.gpu_lifecycle = GpuLifecycle::Suspended;
        self.last_frame = None;
        self.input_stamp = None;
        self.resize_settle_at = None;
        self.move_settle_at = None;
        self.crossing_settle_at = None;
        self.crossing_teardown_pending = false;
        self.present_sync_on = false;
        self.present_sync_valid = false;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // WASM: install the GPU the async init parked in the shared slot (its
        // trailing `request_redraw` is what delivered us here). The first frame
        // after init lands here with `gpu` still `None` but the slot full.
        #[cfg(target_arch = "wasm32")]
        if self.gpu.is_none() {
            let pending = self.gpu_pending.borrow_mut().take();
            if let Some(result) = pending {
                match result {
                    Ok(gpu) => {
                        self.gpu = Some(gpu);
                        self.gpu_lifecycle = GpuLifecycle::Active { oom_skips: 0 };
                        self.on_gpu_ready();
                    }
                    Err(e) => {
                        log::error!("failed to rebuild render state: {e}");
                        self.set_sticky_notice("graphics could not recover — closing safely");
                        event_loop.exit();
                    }
                }
            }
        }
        if self.gpu.is_none() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(true) => self.on_focus_gained(),
            WindowEvent::Focused(false) => self.on_focus_lost(),
            WindowEvent::Occluded(occluded) => self.on_occluded(occluded),
            WindowEvent::Resized(size) => self.on_resized(event_loop, size),
            WindowEvent::Moved(position) => self.on_moved(position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(scale_factor);
            }
            WindowEvent::ModifiersChanged(m) => self.on_modifiers_changed(m),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_input(event_loop, state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(delta),
            WindowEvent::Ime(ime) => self.on_ime(ime),
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(event_loop, event),
            WindowEvent::RedrawRequested => self.on_redraw_requested(event_loop),
            _ => {}
        }
    }

    /// The event loop is exiting (quit / window closed): flush any pending note
    /// save — and the document autosave / scratch stash — so nothing typed right
    /// before quit is lost. The final safety net of the robust-autosave guarantee.
    /// Also the daemon's clean-shutdown door: flush every outstanding `--wait`
    /// connection + unlink the socket special file (native only).
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.flush_note();
        self.autosave_flush();
        // SESSION RESTORE: the final safety net, mirroring the autosave flush
        // right above it (native only; kill-switch gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.session_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush();
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        self.daemon_shutdown();
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::flight_active()
            && let Some(dist) = crate::probe::latency_distribution()
        {
            crate::probe::trace(format_args!("movement-latency distribution: {dist}"));
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // The full scheduling body (every debounce / settle deadline + the
        // ambient tick + GPU retries) lives in `app/schedule.rs`; a trait impl
        // can't span files, so this method is a thin delegate to the inherent
        // `App::about_to_wait_impl` moved there. `ActiveEventLoop` is the live
        // `Scheduler` sink (a headless `RecordingScheduler` is the other, driven by
        // `step_scheduling`), so the SAME body runs under the virtual-clock harness.
        self.about_to_wait_impl(event_loop);
        // The GPU SOAK drive runs LAST (its historical position at the end of the
        // scheduling body) but OUTSIDE it: it needs the real `&ActiveEventLoop`
        // (resizes the recovery window, sets its own control flow) and always runs on
        // real time, so it never belongs on the clock-steppable path. No-ops unless a
        // `--soak-gpu` run is active.
        #[cfg(not(target_arch = "wasm32"))]
        self.drive_gpu_soak(event_loop);
    }
}

//! Browser clipboard transport. Only a trusted paste event supplies text;
//! key gestures never fall back to the editor's previous kill-ring value.

use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};

use super::AwlEvent;

pub struct Clipboard {
    pub(super) allow_gesture: Rc<Cell<bool>>,
    proxy: Option<winit::event_loop::EventLoopProxy<AwlEvent>>,
    canvas: Option<web_sys::HtmlCanvasElement>,
    listeners: Vec<(&'static str, Closure<dyn FnMut(web_sys::Event)>)>,
}

impl Clipboard {
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            allow_gesture: Rc::new(Cell::new(false)),
            proxy: None,
            canvas: None,
            listeners: Vec::new(),
        })
    }

    pub(super) fn set_proxy(&mut self, proxy: winit::event_loop::EventLoopProxy<AwlEvent>) {
        self.proxy = Some(proxy);
    }

    pub(super) fn install(&mut self, canvas: web_sys::HtmlCanvasElement) {
        if self.canvas.is_some() {
            return;
        }
        let Some(proxy) = self.proxy.clone() else {
            return;
        };
        let allowed = self.allow_gesture.clone();
        let pending = Rc::new(Cell::new(0_u64));
        let key_pending = pending.clone();
        let key_proxy = proxy.clone();
        let key = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(key) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
                return;
            };
            let native = match crate::convention::Convention::current() {
                crate::convention::Convention::Mac => key.meta_key() && !key.ctrl_key(),
                crate::convention::Convention::Linux => key.ctrl_key() && !key.meta_key(),
            };
            if !event.is_trusted()
                || !allowed.get()
                || !native
                || key.alt_key()
                || key.shift_key()
                || !key.key().eq_ignore_ascii_case("v")
            {
                return;
            }
            // Capture runs before winit's bubble listener. Keep the browser
            // default paste gesture and suppress the duplicate internal Yank.
            event.stop_immediate_propagation();
            let token = key_pending.get().wrapping_add(1).max(1);
            key_pending.set(token);
            let waiting = key_pending.clone();
            let proxy = key_proxy.clone();
            let timeout = Closure::once_into_js(move || {
                if waiting.get() == token {
                    waiting.set(token.wrapping_add(1));
                    let _ = proxy.send_event(AwlEvent::BrowserPaste(Err(())));
                }
            });
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout.unchecked_ref(),
                    250,
                );
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        let paste_canvas = canvas.clone();
        let paste = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if !event.is_trusted() || !canvas_focused(&paste_canvas) {
                return;
            }
            let Some(paste) = event.dyn_ref::<web_sys::ClipboardEvent>() else {
                return;
            };
            event.prevent_default();
            event.stop_immediate_propagation();
            pending.set(pending.get().wrapping_add(1));
            let text = paste.clipboard_data().ok_or(()).and_then(|data| {
                if !data
                    .types()
                    .iter()
                    .any(|kind| kind.as_string().as_deref() == Some("text/plain"))
                {
                    return Err(());
                }
                data.get_data("text/plain").map_err(|_| ())
            });
            let _ = proxy.send_event(AwlEvent::BrowserPaste(text));
        }) as Box<dyn FnMut(web_sys::Event)>);
        for (name, callback) in [("keydown", key), ("paste", paste)] {
            if canvas
                .add_event_listener_with_callback_and_bool(
                    name,
                    callback.as_ref().unchecked_ref(),
                    true,
                )
                .is_ok()
            {
                self.listeners.push((name, callback));
            }
        }
        self.canvas = Some(canvas);
    }

    pub(super) fn focused(&self) -> bool {
        self.canvas.as_ref().is_some_and(canvas_focused)
    }

    pub fn set_text(&mut self, text: String) -> Result<(), &'static str> {
        let Some(window) = web_sys::window() else {
            return Err("no browser window");
        };
        let promise = window.navigator().clipboard().write_text(&text);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
        Ok(())
    }

    pub fn get_text(&mut self) -> Result<String, &'static str> {
        Err("use a trusted browser paste event")
    }
}

fn canvas_focused(canvas: &web_sys::HtmlCanvasElement) -> bool {
    web_sys::window()
        .and_then(|window| window.document())
        .filter(|document| document.has_focus().unwrap_or(false))
        .and_then(|document| document.active_element())
        .is_some_and(|active| active == canvas.clone().unchecked_into::<web_sys::Element>())
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        if let Some(canvas) = &self.canvas {
            for (name, callback) in &self.listeners {
                let _ = canvas.remove_event_listener_with_callback_and_bool(
                    name,
                    callback.as_ref().unchecked_ref(),
                    true,
                );
            }
        }
    }
}

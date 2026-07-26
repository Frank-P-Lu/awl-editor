//! PER-WORLD APP ICONS — the pre-rendered macOS icon set and the one door that
//! adopts one into the live Dock.
//!
//! **The lockup.** `aw` plus an ordinary lowercase `l` in the world's own
//! display face, at the same size and on the same baseline, with a deliberately
//! FAKE logo-cursor painted behind the `l` — never awl's live Block/Morph/I-beam
//! caret renderer, and it never moves. Three cursor shapes exist and there is no
//! fourth ([`crate::theme::IconCursor`]); which one a world wears is declared on
//! the world itself, so a new world cannot compile without choosing. The colours
//! are the world's own `base_100` (ground), `base_content` (`aw`), `primary`
//! (the cursor) and `primary_content` (the `l`) — nothing here holds a hex.
//!
//! **Everything is rendered AHEAD OF TIME.** `scripts/export-icons.sh` runs a
//! pinned, offline web compositor once; `awl --pack-icns` then cuts each world's
//! tiles into an `.icns` and writes both the committed assets and this module's
//! [`embedded`] table. An ordinary `cargo build`, `cargo test`, and the shipping
//! app invoke NO browser and NO wgpu renderer — they only ever read PNG bytes
//! that pipeline already produced. See `scripts/icons/README.md`.
//!
//! **Two consumers, deliberately different.**
//!   * FINDER / the bundle gets ONE canonical `Awl.icns` (the DEFAULT world's —
//!     see [`canonical_world`]), copied into `Contents/Resources/` and named by
//!     `CFBundleIconFile` in `scripts/package-macos.sh`. It never changes at
//!     runtime; a `.app`'s file icon is a property of the bundle, not of the
//!     session.
//!   * The live DOCK / app-switcher image follows the ACTIVE world, through
//!     [`adopt`].
//!
//! **THE DOCK NEVER CHURNS ON A HOVER.** [`adopt`] is called from exactly two
//! places, both of them settled states: once at live startup (after the sticky
//! theme has been restored from `config.toml`) and once inside `App::apply`'s
//! `theme_committed` arm — the same guard that decides whether the sticky
//! preference is written at all. The theme picker's live preview runs through
//! `retint_theme_preview`, which re-tints pipelines and reshapes text and has no
//! route to this module: arrowing or sweeping the pointer through eighteen
//! worlds moves the Dock zero times, because the preview path structurally
//! cannot reach the door, not because it politely declines to. `app_icon_law.rs`
//! pins that with a counter across a full preview sweep.

pub mod icns;

// The generated `include_bytes!` table is macOS-only: it is ~1.4 MB of Dock
// image that a Linux or wasm build has no API to hand it (the Linux launcher
// icon and the web favicon are their own later rounds). The LAW TESTS read the
// committed files off disk instead, so they sweep the whole roster on every
// platform, and one macOS-only test ties the embedded bytes back to those files.
#[cfg(target_os = "macos")]
mod embedded;

use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::theme::{DEFAULT_THEME, THEMES, Theme};

/// Where the committed per-world icons live, relative to the repo root.
pub const WORLD_ICON_DIR: &str = "assets/macos/world";

/// The canonical bundle icon `scripts/package-macos.sh` wires into
/// `CFBundleIconFile`, relative to the repo root.
pub const CANONICAL_ICNS: &str = "assets/macos/Awl.icns";

/// The world whose icon IS the canonical bundle icon: the DEFAULT world, i.e.
/// what a first launch actually looks like. Derived from [`DEFAULT_THEME`] so
/// retargeting the default retargets the bundle icon with it, rather than
/// leaving Finder showing a world nobody starts in.
pub fn canonical_world() -> &'static Theme {
    &THEMES[DEFAULT_THEME]
}

/// The last world whose icon was adopted into the Dock, or `None` if none has
/// been. The observable half of [`adopt`]: the AppKit call itself is live-only
/// (a headless or off-main-thread process has no Dock tile), but the ADOPTION
/// is recorded here either way, so the "a hover never churns the Dock" law is
/// testable without a Dock.
static ADOPTED: RwLock<Option<&'static str>> = RwLock::new(None);

/// How many times [`adopt`] has run this process. The law test's counter: a
/// preview sweep must not move it.
static ADOPTIONS: AtomicU32 = AtomicU32::new(0);

/// The world currently showing in the Dock, as far as this process knows.
/// Read by the Dock-churn law (`app/tests/dock_icon.rs`); the live app only
/// ever WRITES the icon, never asks itself which one it wrote.
#[allow(dead_code)]
pub fn adopted() -> Option<&'static str> {
    *ADOPTED.read().unwrap_or_else(|e| e.into_inner())
}

/// How many adoptions have happened this process — the counter that makes "a
/// preview never churns the Dock" an observable property rather than a claim.
/// Law-test surface only.
#[allow(dead_code)]
pub fn adoptions() -> u32 {
    ADOPTIONS.load(Ordering::Relaxed)
}

/// Reset the adoption record — `cfg(test)` only, so a law test can start from a
/// known count. Takes no lock of its own: the caller holds
/// [`crate::testlock::serial`], as every `cfg(test)` global writer in this
/// crate does.
#[cfg(test)]
pub(crate) fn reset_adoptions_for_test() {
    *ADOPTED.write().unwrap_or_else(|e| e.into_inner()) = None;
    ADOPTIONS.store(0, Ordering::Relaxed);
}

/// This world's pre-rendered `.icns` bytes, or `None` when the world has no
/// embedded image (which the bijection law makes impossible for a shipped
/// world, and which is exactly what a non-macOS build always sees).
pub fn icns_for(world: &str) -> Option<&'static [u8]> {
    #[cfg(target_os = "macos")]
    {
        embedded::WORLD_ICNS
            .iter()
            .find(|(name, _)| *name == world)
            .map(|(_, bytes)| *bytes)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = world;
        None
    }
}

/// Every world the embedded table carries, in table order. Empty off macOS.
/// Law-test surface (the bijection sweep); nothing in the running app needs it.
#[allow(dead_code)]
pub fn embedded_worlds() -> Vec<&'static str> {
    #[cfg(target_os = "macos")]
    {
        embedded::WORLD_ICNS.iter().map(|(n, _)| *n).collect()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

/// ADOPT this world's pre-rendered icon as the live Dock / app-switcher image.
///
/// THE ONLY DOOR. Call it from a SETTLED state and nowhere else: live startup
/// once the sticky theme has been restored, and a theme picker COMMIT. Never
/// from a preview — see the module doc.
///
/// Records the adoption unconditionally (so the law tests can see it) and then
/// hands the bytes to AppKit, which is live-only: off the main thread, off
/// macOS, or in a headless capture the AppKit half is a calm no-op and only the
/// record moves. Returns whether an image was actually handed over.
pub fn adopt(theme: &Theme) -> bool {
    ADOPTIONS.fetch_add(1, Ordering::Relaxed);
    *ADOPTED.write().unwrap_or_else(|e| e.into_inner()) = Some(theme.name);
    match icns_for(theme.name) {
        Some(bytes) => {
            #[cfg(target_os = "macos")]
            {
                crate::mac_chrome::set_dock_icon(bytes)
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = bytes;
                false
            }
        }
        None => false,
    }
}

/// THE PACK STEP (`awl --pack-icns`), the offline pipeline's second half.
///
/// Reads the tiles the web compositor already rendered and writes, for every
/// SHIPPED world, `assets/macos/world/<World>.icns`; copies the canonical
/// world's file to `assets/macos/Awl.icns`; and regenerates
/// `src/app_icon/embedded.rs`.
///
/// The embedded table is GENERATED rather than hand-listed for the usual
/// reason: `include_bytes!` needs a literal path, so a hand-written table would
/// be a second roster free to drift from `THEMES`. Writing it here means the
/// same command that produces an asset also declares it, and the bijection law
/// then catches a table that was never regenerated.
///
/// Returns the (world, byte length) pairs written, for the caller to print.
#[cfg(not(target_arch = "wasm32"))]
pub fn pack_all(
    tiles_dir: &std::path::Path,
    root: &std::path::Path,
) -> anyhow::Result<Vec<(&'static str, usize)>> {
    let out_dir = root.join(WORLD_ICON_DIR);
    std::fs::create_dir_all(&out_dir)?;
    let mut written = Vec::new();
    for theme in THEMES.iter() {
        let bytes = icns::pack_world(tiles_dir, theme)?;
        std::fs::write(out_dir.join(format!("{}.icns", theme.name)), &bytes)?;
        written.push((theme.name, bytes.len()));
    }
    let canonical = canonical_world();
    let canonical_bytes = std::fs::read(out_dir.join(format!("{}.icns", canonical.name)))?;
    std::fs::write(root.join(CANONICAL_ICNS), &canonical_bytes)?;
    std::fs::write(root.join("src/app_icon/embedded.rs"), embedded_source())?;
    Ok(written)
}

/// The text of the generated [`embedded`] module — one `include_bytes!` per
/// SHIPPED world, in `THEMES` order.
#[cfg(not(target_arch = "wasm32"))]
fn embedded_source() -> String {
    let mut s = String::new();
    s.push_str(
        "//! GENERATED by `awl --pack-icns` — do not hand-edit.\n\
         //!\n\
         //! One entry per world in `theme::THEMES` order, each `include_bytes!`ing\n\
         //! the `.icns` the offline exporter cut for that world. Regenerate with\n\
         //! `scripts/export-icons.sh` (which runs the pack step); the bijection law\n\
         //! in `app_icon/tests.rs` fails if this file is stale.\n\n\
         /// Every shipped world's pre-rendered app icon: `(world name, .icns bytes)`.\n\
         pub const WORLD_ICNS: &[(&str, &[u8])] = &[\n",
    );
    for theme in THEMES.iter() {
        s.push_str(&format!(
            "    ({:?}, include_bytes!(\"../../{WORLD_ICON_DIR}/{}.icns\")),\n",
            theme.name, theme.name
        ));
    }
    s.push_str("];\n");
    s
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

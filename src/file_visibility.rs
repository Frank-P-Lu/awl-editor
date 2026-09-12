//! FILE VISIBILITY — the ONE sticky picker-listing switch, replacing
//! the retired standalone "Show hidden files" toggle (`Action::ToggleHiddenFiles`,
//! Cmd-Shift-`.`, the per-overlay `OverlayState::show_hidden` flag).
//!
//! Two states:
//!   * **Text** (default) — the Browse file picker lists normal, non-hidden
//!     files whose *names* do not identify a known non-text format, plus every
//!     folder. Unfamiliar, extensionless, and misleading names remain offered
//!     and can still be refused on open.
//!   * **All** — also reveals dotfiles and known non-text filename hints. The
//!     actual open decision remains [`crate::openable`]'s content-validation gate.
//!
//! A process-global [`AtomicBool`] (DEFAULT OFF = Text), mirroring the
//! `page`/`spell`/`nits` sticky-toggle pattern: the Settings menu's "File
//! visibility" row, the sticky config pref, and `overlay::state::refilter`'s
//! display filter all read this ONE place. Reached through the Settings menu
//! only (no dedicated chord) — a picker-local ephemeral toggle was the old
//! shape being retired; this is a sticky, app-wide preference instead.

use crate::toggle::Toggle;

/// The listing-only answer shared by Files and Browse. It is intentionally a
/// pure name-and-entry-kind hint: no listing or selection path may infer an
/// openability verdict from file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PresentationHint {
    Folder,
    Candidate,
    KnownNonText { label: String },
}

impl PresentationHint {
    pub(crate) fn secondary(&self) -> String {
        match self {
            Self::KnownNonText { label } => label.clone(),
            Self::Folder | Self::Candidate => String::new(),
        }
    }

    pub(crate) fn text_visible(&self) -> bool {
        !matches!(self, Self::KnownNonText { .. })
    }
}

/// Produce a presentation hint from an entry's leaf name and kind. This is a
/// filename convention, not an allow-list and not a capability check.
pub(crate) fn presentation_hint(name: &str, is_dir: bool) -> PresentationHint {
    if is_dir {
        return PresentationHint::Folder;
    }
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .map(str::to_ascii_lowercase);
    let known_non_text = matches!(
        extension.as_deref(),
        Some(
            "7z" | "avi"
                | "bin"
                | "bmp"
                | "bz2"
                | "class"
                | "dmg"
                | "doc"
                | "docx"
                | "exe"
                | "gif"
                | "gz"
                | "heic"
                | "icns"
                | "ico"
                | "iso"
                | "jar"
                | "jpeg"
                | "jpg"
                | "m4a"
                | "mkv"
                | "mov"
                | "mp3"
                | "mp4"
                | "odp"
                | "ods"
                | "odt"
                | "pdf"
                | "png"
                | "ppt"
                | "pptx"
                | "rar"
                | "so"
                | "tar"
                | "tiff"
                | "wav"
                | "webm"
                | "webp"
                | "xls"
                | "xlsx"
                | "xz"
                | "zip"
        )
    );
    match known_non_text {
        true => PresentationHint::KnownNonText {
            label: extension.expect("known extension").to_ascii_uppercase(),
        },
        false => PresentationHint::Candidate,
    }
}

/// Whether "All" is active (dotfiles + unsupported files both revealed).
/// DEFAULT `false` (Text) — the calm, curated default a new install opens to.
/// The value this flag carries on a fresh install, before any config or
/// settings write — the ONE owner of that fact, read both by the static
/// below and by the generated reference (`settings::toggle_default`).
pub(crate) const FILE_VISIBILITY_DEFAULT: bool = false;
static ALL_ON: Toggle = Toggle::new(FILE_VISIBILITY_DEFAULT);

/// True when "All" visibility is active.
pub fn all_on() -> bool {
    ALL_ON.on()
}

/// Set the live global directly (config load / capture `--config` seeding).
pub fn set_all_on(on: bool) {
    ALL_ON.set(on);
}

/// Flip the global, returning the NEW value — the Settings menu's "File
/// visibility" toggle row rides the generic bool mechanism
/// (`App::setting_toggle`), which reads-then-writes itself, so this is
/// test-only sugar (offered for symmetry with `page::toggle`/`spell::toggle`).
#[allow(dead_code)]
pub fn toggle() -> bool {
    ALL_ON.toggle()
}

/// The setting row's calm VALUE-cell word: `"Text"` / `"All"`.
pub fn label() -> &'static str {
    if all_on() { "All" } else { "Text" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_hints_are_pure_and_never_promise_content_readiness() {
        assert_eq!(presentation_hint("folder", true), PresentationHint::Folder);
        assert_eq!(
            presentation_hint("notes.md", false),
            PresentationHint::Candidate
        );
        assert_eq!(
            presentation_hint("README", false),
            PresentationHint::Candidate
        );
        assert_eq!(
            presentation_hint("mystery.xyzzy", false),
            PresentationHint::Candidate
        );
        assert_eq!(
            presentation_hint("movie.MP4", false),
            PresentationHint::KnownNonText {
                label: "MP4".to_string()
            }
        );
    }

    #[test]
    fn defaults_to_text_and_toggles_both_ways() {
        let _g = crate::testlock::serial();
        let before = all_on();
        set_all_on(false);
        assert_eq!(label(), "Text");
        assert!(toggle(), "Text -> All");
        assert_eq!(label(), "All");
        assert!(all_on());
        assert!(!toggle(), "All -> Text");
        assert_eq!(label(), "Text");
        set_all_on(before); // never leak into a sibling test
    }
}

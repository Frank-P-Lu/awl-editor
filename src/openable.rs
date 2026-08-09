//! src/openable.rs — the ONE CAPABILITY OWNER: "is this path openable
//! as awl-editable TEXT?"
//!
//! Every door that can turn a user-named path into the ACTIVE buffer — a
//! picker Enter (Browse/Goto, `App::load_path`), a CLI/OS-open launch
//! argument (`App::new`), and the single-instance daemon's `open` handoff
//! (`App::handle_daemon_event`, itself routed through `App::load_path`) —
//! asks THIS module the same question before touching `Buffer::from_file`, so
//! a binary file can never slip into the rope. Before this module existed,
//! `Buffer::from_file` swallowed a decode failure and returned an EMPTY
//! buffer STILL BOUND to that path (`fs.rs`'s `read_to_string` errors on
//! invalid UTF-8, and the `Err` arm there falls back to `Rope::new()`) — so
//! opening a PNG silently produced a phantom empty document that a later
//! Cmd-S would happily use to TRUNCATE the real file to nothing. [`classify`]
//! closes that hole at the door, before any buffer/root state changes.
//!
//! NOT an extension allow-list: a recognized prose/code extension
//! (`.rs`/`.md`/`.env`/…) is always openable, but so is an EXTENSIONLESS or
//! entirely unfamiliar-extension file, PROVIDED its bytes actually decode as
//! text — the bytes decide, never the name. [`crate::file_visibility`] is the
//! separate, purely-cosmetic sibling: it decides what the Browse picker
//! LISTS, never what a door is willing to OPEN — seeing a row in the "All"
//! listing and being able to open it are two different questions, and this
//! module owns only the second one.

use std::path::Path;

/// The verdict [`classify`] returns for a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Openable {
    /// Decodable text — or the path is missing/unreadable, in which case
    /// there is nothing to refuse (mirrors `Buffer::from_file`'s own
    /// missing-file leniency: a not-yet-created path is always "openable",
    /// it just starts empty).
    Text,
    /// NOT decodable text — a binary/unsupported file. `label` is the
    /// concise TYPE word for the calm refusal message ([`Openable::refusal_message`]):
    /// the extension uppercased ("PNG", "ZIP"), or the generic "Binary" for
    /// an extensionless or unfamiliar-extension binary.
    Unsupported { label: String },
}

impl Openable {
    /// True for [`Openable::Text`]. Test-only today (every production door
    /// asks [`Openable::refusal_message`] instead, since it also carries the
    /// calm wording); kept as a real predicate so a future non-refusal
    /// consumer has one door, mirroring `commands::Command::available_on`'s
    /// own "kept as a real predicate" precedent.
    #[allow(dead_code)]
    pub fn is_text(&self) -> bool {
        matches!(self, Openable::Text)
    }

    /// The calm refusal line an invoked Unsupported row/door reports —
    /// `"PNG · not editable in awl"` (equivalent per type). `None` for
    /// [`Openable::Text`] (nothing to refuse).
    pub fn refusal_message(&self) -> Option<String> {
        match self {
            Openable::Unsupported { label } => Some(format!("{label} \u{b7} not editable in awl")),
            Openable::Text => None,
        }
    }
}

/// A concise TYPE label for `path`'s refusal message — the extension,
/// uppercased ("PNG", "ZIP", "MP4"), or the generic "Binary" for an
/// extensionless (or unrecognized-extension) binary file. Presentation only —
/// never a second decode gate; [`classify`] already decided Unsupported by
/// the time this is consulted.
pub fn type_label(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .map(|e| e.to_ascii_uppercase())
        .unwrap_or_else(|| "Binary".to_string())
}

/// THE decode heuristic: `bytes` reads as awl-editable text iff it has no
/// embedded NUL (the standard binary tripwire most editors + `git`/`grep`
/// use — a real text file, in any encoding awl actually supports, never
/// contains one) AND is valid UTF-8 (the only encoding `Buffer`/the rope
/// ever holds — see `fs.rs`'s module doc: awl only ever writes UTF-8 rope
/// text). Pure.
fn looks_like_text(bytes: &[u8]) -> bool {
    !bytes.contains(&0) && std::str::from_utf8(bytes).is_ok()
}

/// THE decision every door asks before it lets `path` become (or stay) the
/// active document: openable text, or refused? Routes through the same
/// [`crate::fs::FileSystem`] seam every other read uses (native disk in
/// production, `InMemoryFs`/`WebFs` under test/wasm/the scenario sandbox), so
/// this never touches the real disk from a headless test.
///
/// A MISSING or UNREADABLE path is [`Openable::Text`] — there is nothing to
/// refuse; the caller's existing missing-file handling (an empty buffer bound
/// to the path, ready for its first Cmd-S — mirroring mg) is unaffected. An
/// EMPTY file is `Text` too (zero bytes disqualify nothing). Otherwise the
/// full byte content decides via [`looks_like_text`].
///
/// Deliberately reads the WHOLE file rather than a bounded prefix: opening it
/// (on a `Text` verdict) was always going to read the whole thing anyway —
/// see `Buffer::from_file`/`fs::NativeFs::read_to_string` — so this adds no
/// new full-file read on the accept path. The one place this trades a little
/// eagerness for simplicity is the Browse LISTING (`overlay::build::browse_level`),
/// which classifies every FILE in ONE directory level up front so it can
/// label/filter the row without a second read on open; that's bounded by a
/// single directory's entry count, not the whole project (a TASTE call,
/// logged here rather than silently accepted: a single directory holding one
/// enormous unfamiliar-extension file is the one case this reads more than
/// strictly necessary for a listing).
pub fn classify(path: &Path) -> Openable {
    match crate::fs::active().read(path) {
        Ok(bytes) if bytes.is_empty() || looks_like_text(&bytes) => Openable::Text,
        Ok(_) => Openable::Unsupported {
            label: type_label(path),
        },
        Err(_) => Openable::Text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{FileSystem, InMemoryFs};
    use std::sync::Arc;

    fn mem_with(path: &str, bytes: &[u8]) -> InMemoryFs {
        let mem = InMemoryFs::new();
        mem.write(Path::new(path), bytes).unwrap();
        mem
    }

    #[test]
    fn recognized_prose_and_code_extensions_are_text() {
        let mem = mem_with("/p/main.rs", b"fn main() {}\n");
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(classify(Path::new("/p/main.rs")), Openable::Text);
        });
    }

    #[test]
    fn extensionless_and_unfamiliar_extension_text_is_openable_not_an_allow_list() {
        // NOT an extension allow-list: an extensionless file, and a totally
        // unfamiliar extension, both stay Text as long as the BYTES decode.
        let mem = InMemoryFs::new();
        mem.write(Path::new("/p/README"), b"hello\n").unwrap();
        mem.write(Path::new("/p/notes.xyzzy"), b"plain prose\n")
            .unwrap();
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(classify(Path::new("/p/README")), Openable::Text);
            assert_eq!(classify(Path::new("/p/notes.xyzzy")), Openable::Text);
        });
    }

    #[test]
    fn a_nul_byte_refuses_regardless_of_extension() {
        // A real PNG signature: high bytes + an embedded NUL.
        let bytes = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00];
        let mem = mem_with("/p/pic.png", &bytes);
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(
                classify(Path::new("/p/pic.png")),
                Openable::Unsupported {
                    label: "PNG".to_string()
                }
            );
        });
    }

    #[test]
    fn invalid_utf8_without_a_nul_still_refuses() {
        // Latin-1 bytes (0xE9 = 'é') that are not valid UTF-8 on their own.
        let bytes = [0xE9, 0xE9, 0xE9];
        let mem = mem_with("/p/data.bin", &bytes);
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(
                classify(Path::new("/p/data.bin")),
                Openable::Unsupported {
                    label: "BIN".to_string()
                }
            );
        });
    }

    #[test]
    fn an_extensionless_binary_labels_generic_binary() {
        let bytes = [0x00, 0x01, 0x02, 0x03];
        let mem = mem_with("/p/data", &bytes);
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(
                classify(Path::new("/p/data")),
                Openable::Unsupported {
                    label: "Binary".to_string()
                }
            );
        });
    }

    #[test]
    fn a_missing_or_empty_file_is_always_text() {
        let mem = InMemoryFs::new();
        mem.write(Path::new("/p/empty.png"), b"").unwrap();
        crate::fs::with_fs(Arc::new(mem), || {
            assert_eq!(
                classify(Path::new("/p/nope.png")),
                Openable::Text,
                "missing: nothing to refuse"
            );
            assert_eq!(
                classify(Path::new("/p/empty.png")),
                Openable::Text,
                "empty: nothing disqualifying"
            );
        });
    }

    #[test]
    fn refusal_message_names_the_type_and_is_calm() {
        let bytes = [0x00, 0x01];
        let mem = mem_with("/p/movie.mp4", &bytes);
        crate::fs::with_fs(Arc::new(mem), || {
            let verdict = classify(Path::new("/p/movie.mp4"));
            assert_eq!(
                verdict.refusal_message().as_deref(),
                Some("MP4 \u{b7} not editable in awl")
            );
        });
        assert_eq!(Openable::Text.refusal_message(), None);
    }
}

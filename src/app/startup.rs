pub(super) fn keymap(
    keys: &[(String, Vec<String>)],
    keep: &[String],
    linux_emacs_meta: bool,
) -> crate::keymap::KeymapState {
    crate::keymap::KeymapState::with_overrides_and_keep(keys, keep, linux_emacs_meta)
}

/// Read the persistent scratch stash into a buffer + baseline — the SCRATCH
/// RESTORE a no-argument `App::new` performs, and the live "Open scratch"
/// door's exact counterpart, so the two doors can never disagree on what
/// counts as saved. A corrupt stash is preserved to a `.corrupt-*` sibling
/// before falling back to a blank scratch, identically either way.
pub(super) fn scratch_buffer_from_stash() -> (crate::buffer::Buffer, crate::external::Seen) {
    let stash = crate::fs::scratch_stash_path();
    let mut unavailable = false;
    let buffer = match crate::openable::read_text(&stash) {
        Ok(s) if !s.is_empty() => crate::buffer::Buffer::from_str(&s),
        Ok(_) => crate::buffer::Buffer::scratch(), // present but empty: nothing to preserve
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound && crate::openable::is_missing(&stash) =>
        {
            crate::buffer::Buffer::scratch()
        }
        Err(_) => {
            unavailable = true;
            if let Ok(raw) = crate::fs::active().read(&stash) {
                crate::durable::preserve_corrupt(&stash, &raw);
            }
            crate::buffer::Buffer::scratch()
        }
    };
    let baseline = if unavailable {
        crate::external::Seen::Unavailable
    } else {
        crate::external::Seen::at(&stash)
    };
    (buffer, baseline)
}

/// A preflight refusal names unsupported types; the actual load remains fallible
/// so a later decode/access failure cannot become an empty path-bound document.
pub(super) struct LaunchFile {
    pub(super) loaded: Option<(crate::buffer::Buffer, crate::external::Seen)>,
    pub(super) refusal: Option<String>,
}

pub(super) fn load_launch_file(file: Option<std::path::PathBuf>) -> LaunchFile {
    let Some(path) = file else {
        return LaunchFile {
            loaded: None,
            refusal: None,
        };
    };
    if let Some(message) = crate::openable::classify(&path).refusal_message() {
        return LaunchFile {
            loaded: None,
            refusal: Some(message),
        };
    }
    match crate::buffer::Buffer::open_file(&path) {
        Ok(loaded) => LaunchFile {
            loaded: Some(loaded),
            refusal: None,
        },
        Err(_) => LaunchFile {
            loaded: None,
            refusal: Some("This file cannot be read — opening scratch instead".into()),
        },
    }
}

//! THE HOME REDACTION — the last thing that happens to a capture artifact
//! before it reaches disk.
//!
//! WHAT LEAKS. A sidecar is not written by hand, so no rule about what an agent
//! may *type* into a tracked file reaches it: a lane only has to CAPTURE. Every
//! ordinary capture carries at least three absolute filesystem locations —
//! `project.root` (an unseeded root is `fs::current_dir()`), `project.workspace`
//! (flag > config > `root.parent()`), and `project.default_folder` (which falls
//! back to `$HOME/notes` whether or not anything is configured) — and a driven
//! picker adds `overlay.browse_dir`, a buffer swap adds `buffers.active`, and an
//! embedded image adds `images[].path`. On a developer machine every one of
//! those is rooted at the account's home directory, and the repo is public.
//!
//! WHY IT RUNS OVER THE FINISHED TEXT, NOT PER FIELD. Per-field relativising
//! covers the fields someone remembered to route; this pass covers the field
//! nobody has written yet. `write_sidecar` is the single choke point every
//! capture door funnels through (the same property `capture::tests::
//! serialization_law` already leans on), so one call there sanitises the whole
//! artifact and a new path-bearing block cannot opt out of it — which is the
//! difference between preventing the leak and noticing it.
//!
//! WHAT A DEBUGGING HUMAN LOSES. Only the account name. `$HOME` becomes `~` and
//! nothing else about the path changes, so a sidecar still says exactly which
//! directory it listed: `~/code/awl` is as diagnostic as the original, and a
//! path outside home is untouched entirely.
//!
//! WHAT IT DOES NOT REACH. The redaction rewrites PATHS, so it cannot help with
//! CONTENT read off the real filesystem — `overlay.items` for a picker pointed
//! at a real directory is a list of that directory's own entry names, and the
//! PNG beside it is a photograph of the same rows. Those stay leakable, and
//! their fix is a hermetic capture door (a seeded root and an explicit
//! `--config`), not a serializer.

use std::path::Path;

/// Redact this process's `$HOME` out of a finished capture artifact.
pub(crate) fn redact(text: &str) -> String {
    match crate::fs::home_dir() {
        Some(home) => redact_with_home(text, &home),
        None => text.to_string(),
    }
}

/// The pure half: rewrite every `home`-prefixed path in `text` to `~`.
///
/// A match must sit between two PATH BOUNDARIES. At its end, the next byte
/// cannot continue a path component — which admits the `/` of a longer path,
/// the closing `"` of a JSON string, whitespace in prose, and the end of the
/// text — so a sibling account whose name merely extends ours
/// (`/Users/frankenstein` beside `/Users/frank`) is left alone. At its start,
/// the previous byte can be neither a component character nor a separator, so an
/// unrelated absolute path that happens to END with the home string
/// (`/opt/Users/frank`) is not sliced into `/opt~`. Both halves are needed: the
/// second was missing from the first draft and its own law caught it.
///
/// A home too GENERIC to recognise is refused rather than applied: a relative
/// value, `/`, or a single-component root like `/root` cannot be told apart
/// from an ordinary path that happens to start the same way, and rewriting it
/// would corrupt the artifact instead of sanitising it. Refusing is reported by
/// [`is_redactable`] so a law can say which configuration it ran in.
pub(crate) fn redact_with_home(text: &str, home: &Path) -> String {
    let home = home.to_string_lossy();
    let home = home.strip_suffix('/').unwrap_or(&home);
    if !is_redactable(Path::new(home)) {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/'
            && text[i..].starts_with(home)
            && (i == 0 || !starts_inside_a_path(bytes[i - 1]))
            && !bytes
                .get(i + home.len())
                .copied()
                .is_some_and(continues_a_path_component)
        {
            out.push('~');
            i += home.len();
            continue;
        }
        // Copy one whole char: `i` must stay on a UTF-8 boundary for the slice
        // above, and a home path is not guaranteed to be ASCII.
        let c = text[i..].chars().next().expect("i is a char boundary");
        out.push(c);
        i += c.len_utf8();
    }
    out
}

/// Is this home path specific enough to strip from a text safely? Absolute, and
/// at least two components deep (`/Users/frank`, `/home/runner`) — the shapes a
/// real account home takes on both platforms awl builds for.
pub(crate) fn is_redactable(home: &Path) -> bool {
    let home = home.to_string_lossy();
    home.starts_with('/') && home.split('/').filter(|s| !s.is_empty()).count() >= 2
}

fn continues_a_path_component(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')
}

/// Does `b`, sitting immediately before a candidate match, mean the match is
/// really the TAIL of some longer path rather than a path of its own?
fn starts_inside_a_path(b: u8) -> bool {
    continues_a_path_component(b) || b == b'/'
}

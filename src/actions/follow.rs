//! FOLLOWING a link — the one seam between "where is this span pointing"
//! ([`crate::markdown::follow`], pure text) and "what does awl DO about it"
//! (a typed [`Effect`] carrying the resolved destination).
//!
//! Every door reaches this: the `Action::FollowLink` chord, the palette row,
//! the right-click card's Go-to row, and the modifier-click pointer gesture.
//! They share it rather than each deciding for themselves, which is what keeps
//! the pointer from opening something the keyboard would not.
//!
//! The three destinations map to three EFFECTS, not to three implementations:
//!
//! | destination | effect | performed by |
//! | --- | --- | --- |
//! | external URL | [`Effect::FollowLink`] | the live `App` alone — the OS
//!   opener handoff, Intercepted under a `--keys` capture |
//! | local document | [`Effect::OpenPathAtLine`] | both the live `App` and
//!   the headless replay, for real |
//! | footnote reference | [`Effect::JumpToLine`] | both, for real |
//! | in-document anchor | [`Effect::None`] | nobody — DEFERRED, not guessed |

use super::*;

/// The effect that follows whatever `byte` lands on in `buffer`, or
/// [`Effect::None`] when nothing there is followable.
///
/// Gated on `is_markdown` because the affordance is a promise the RENDER makes:
/// a plain `.rs`/`.txt` buffer draws no followable underline, so nothing in it
/// offers to be followed. One rule for the hairline and the gesture both.
pub fn follow_effect(buffer: &crate::buffer::Buffer, byte: usize) -> Effect {
    if !buffer.is_markdown() {
        return Effect::None;
    }
    let text = buffer.text();
    // A footnote reference activates through this same door but is NOT part of
    // the underline grammar — it wears the painted number ornament instead, and
    // its destination is a line in this document. Asked first, since the
    // reference's own bytes carry no followable span for the grammar to find.
    if let Some(line) = crate::markdown::footnote_target_at(&text, byte) {
        return Effect::JumpToLine(line);
    }
    let Some(hit) = crate::markdown::followable_at(&text, byte) else {
        return Effect::None;
    };
    match hit.dest {
        crate::markdown::Destination::External(url) => Effect::FollowLink(url),
        // The Live-Preview model's own move: a note linking a sibling note
        // opens IN awl, never through the OS opener (which would hand a `.md`
        // file to whatever the desktop thinks owns that extension). The path is
        // anchored on the DOCUMENT's directory and emitted absolute, so the
        // effect's own root-relative `join` is an identity on it.
        crate::markdown::Destination::Local(path) => {
            match crate::markdown::resolve_local(buffer.path(), &path) {
                Some(abs) => Effect::OpenPathAtLine {
                    path: abs.to_string_lossy().into_owned(),
                    line: 0,
                    col: 0,
                },
                // A path-less scratch buffer has no directory for a relative
                // link to mean anything against — the calm no-op, never a guess
                // at the process cwd.
                None => Effect::None,
            }
        }
        // DEFERRED, deliberately: a bare `#heading-anchor` has no in-document
        // jump yet. A no-op is the honest answer; opening the wrong thing is not.
        crate::markdown::Destination::InDocument(_) => Effect::None,
    }
}

#[cfg(test)]
mod tests;

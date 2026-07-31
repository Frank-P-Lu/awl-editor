//! CHORD + COMMAND-NAME TOKENS — the CONVENTION-TRUTHFUL SURFACES round's write
//! side for the starting docs (`samples/welcome.md`, `samples/tour.md`,
//! `GUIDE.md`), extended (docs-vs-catalog law round) to cover cited command
//! NAMES the same way.
//!
//! A literal chord glyph baked into a doc (`⌘P`) is a LIE the instant it's read
//! under a different convention or platform — a Linux visitor sees a mac glyph
//! that doesn't fire; a web visitor on a browser-reserved chord (New document,
//! Switch theme…) sees a chord that's silently eaten by the browser chrome
//! before the page ever gets it. `{{key:slug}}` is the fix: a token substituted
//! at the RIGHT moment for each surface (seed time for welcome/tour — see
//! `fs::seed_write_if_absent` — open time for GUIDE.md — see `App::open_guide` /
//! `main::run`'s headless arm) through the SAME truthful label owner every other
//! chord surface reads (`commands::resolved_native_label_truthful`), so a doc
//! can never show a chord that doesn't actually fire.
//!
//! `slug` is a catalog command's own config slug (`commands::slug`) for the vast
//! majority of `{{key:slug}}` tokens; [`SYNTHETIC`] covers the two chords that
//! are real, fixed, non-rebindable presses with NO catalog row at all (the
//! command palette's own dedicated Cmd-P, the held stats HUD's Option-Cmd-I) —
//! both are the most-taught chords in the onboarding docs, so they still need a
//! token even though `commands::COMMANDS` has no entry to hang it on.
//!
//! **`{{cmd:slug}}`** is the sibling convention for a CITED COMMAND NAME — text
//! like `"Widen page"` or `"Keep version…"` that names a palette command by
//! itself, not by its chord. A renamed or retired command leaves a literal
//! mention silently stale (the same "doc taught something that no longer
//! fires" failure mode the welcome-doc incident named, one axis over: a name
//! instead of a chord). `{{cmd:slug}}` substitutes the catalog's own current
//! display NAME for `slug` (convention/platform-independent — a command's name
//! doesn't vary by platform, unlike its chord), through the SAME render seam
//! as `{{key:}}`, so a doc author writes `{{cmd:widen_page}}` once and it
//! always reads exactly what the live palette calls that command. DOCS
//! CONVENTION: any specific command-name citation in `samples/welcome.md` /
//! `samples/tour.md` / `GUIDE.md` prose (outside the generated keys-reference
//! table, which is already whole-table catalog-verified — see `guide.rs`)
//! should be a `{{cmd:slug}}` token, not literal text — the law test
//! [`tests::every_key_token_in_the_starting_docs_resolves`] is what actually
//! enforces it (an unknown slug renders a loud `[[unknown-cmd:slug]]` marker
//! rather than vanishing, and fails that test).

use crate::commands::{self, Platform};
use crate::convention::Convention;

/// The token delimiters: `{{` ... `}}`, with the token KIND named by its
/// prefix immediately inside (`key:` a chord, `cmd:` a command name).
const OPEN: &str = "{{";
const CLOSE: &str = "}}";
const KEY_PREFIX: &str = "key:";
const CMD_PREFIX: &str = "cmd:";

/// Non-catalog synthetic slugs: `(slug, mac spec, linux spec)`, each spec in
/// the same terse form [`crate::keyspec::parse_chord`] accepts. See the module
/// doc for why these two exist outside the catalog.
const SYNTHETIC: &[(&str, &str, &str)] = &[
    // The command palette: its own dedicated Cmd-P/Ctrl-P, matched directly in
    // `keymap.rs::resolve` (never a catalog row, never rebindable via `[keys]`).
    ("command_palette", "Cmd-P", "C-p"),
    // The held stats HUD: Option-Cmd-I / Ctrl-Alt-I, matched directly in
    // `keymap.rs::resolve_named`'s `native && alt` arm (deliberately NOT a
    // catalog row — see `commands.rs`'s "Held stats HUD" doc: a discrete
    // palette selection has no key-release to dismiss a hold-only panel with).
    ("stats_hud", "Cmd-M-i", "C-M-i"),
];

/// Resolve `slug_want`'s chord LABEL for `convention`+`platform`: first the
/// catalog, through [`commands::resolved_native_label_truthful`] (the ONE
/// owner every other chord surface reads — a token can therefore never show a
/// chord that doesn't actually fire, web-reserved/Linux-displaced/web-alternate
/// included), then [`SYNTHETIC`] for the two dedicated chords with no catalog
/// row. `None` for an unknown slug.
pub fn key_token_label(
    slug_want: &str,
    convention: Convention,
    platform: Platform,
) -> Option<String> {
    key_token_spec(slug_want, convention, platform).map(|spec| match convention {
        Convention::Mac => crate::keyspec::mac_glyph_chord(&spec),
        Convention::Linux => crate::keyspec::linux_glyph_chord(&spec),
    })
}

/// [`key_token_label`]'s SPEC half — the terse chord
/// [`crate::keyspec::parse_chord`] accepts, before it becomes a ⌘ glyph or a
/// `Ctrl+` word. `None` for an unknown slug; `Some("")` for a slug that
/// resolves to NO chord on this convention/platform (Insert-link on Linux,
/// where `C-k` stays kill-line; a browser-reserved chord with no alternate).
///
/// Every reader of "what does this doc teach" goes through here, so the law
/// below can drive exactly the chord a reader is shown — never a second,
/// possibly-diverging derivation of it.
pub fn key_token_spec(
    slug_want: &str,
    convention: Convention,
    platform: Platform,
) -> Option<String> {
    if let Some(c) = commands::COMMANDS
        .iter()
        .find(|c| commands::slug(c.name) == slug_want)
    {
        return Some(commands::resolved_native_truthful(c, convention, platform));
    }
    SYNTHETIC
        .iter()
        .find(|(s, _, _)| *s == slug_want)
        .map(|(_, mac, linux)| match convention {
            Convention::Mac => (*mac).to_string(),
            Convention::Linux => (*linux).to_string(),
        })
}

/// Every [`SYNTHETIC`] chord's Mac-glyph LABEL (`"⌘P"`, `"⌘⌥I"`) — the
/// non-catalog half of "every valid Mac chord label", for a consumer (the
/// docs-vs-catalog law's HTML-surface check, `docs_catalog_law.rs`) that
/// needs the WHOLE valid set without duplicating [`SYNTHETIC`]'s two
/// hardcoded specs. Test-only: its one consumer is itself `cfg(test)`.
#[cfg(test)]
pub(crate) fn synthetic_mac_glyphs() -> Vec<String> {
    SYNTHETIC
        .iter()
        .map(|(_, mac, _)| crate::keyspec::mac_glyph_chord(mac))
        .collect()
}

/// Resolve `slug_want`'s command DISPLAY NAME straight from the live catalog
/// (`commands::COMMANDS`, keyed the same way `[keys]` rebinding is — via
/// [`commands::slug`]), for a `{{cmd:slug}}` token. `None` for an unknown slug
/// (a typo, or a command renamed/retired since the doc was written). Unlike
/// [`key_token_label`] this carries no convention/platform parameter — a
/// command's NAME doesn't vary by platform, only its chord does.
pub fn cmd_token_label(slug_want: &str) -> Option<String> {
    commands::COMMANDS
        .iter()
        .find(|c| commands::slug(c.name) == slug_want)
        .map(|c| c.name.to_string())
}

/// Replace every `{{key:slug}}` / `{{cmd:slug}}` token in `text` with
/// [`key_token_label`] / [`cmd_token_label`]'s resolved text for
/// `convention`+`platform`. An UNKNOWN slug is left as a visible
/// `[[unknown-key:slug]]` / `[[unknown-cmd:slug]]` marker — never panics,
/// never silently vanishes — so a typo'd token is obvious in the rendered doc
/// even outside the build-time law test
/// (`tests::every_key_token_in_the_starting_docs_resolves`) that actually
/// guards against one ever shipping. An unterminated `{{...` (missing `}}`)
/// is likewise left verbatim rather than eating the rest of the document, and
/// a `{{...}}` span whose inner text carries neither recognized prefix is left
/// as `[[unknown-token:...]]` (there is no third kind today, but a stray
/// unrecognized brace pair should still be loud, not silent).
pub fn render_key_tokens(text: &str, convention: Convention, platform: Platform) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];
        match after.find(CLOSE) {
            Some(end) => {
                let inner = &after[..end];
                if let Some(slug_want) = inner.strip_prefix(KEY_PREFIX) {
                    match key_token_label(slug_want, convention, platform) {
                        Some(label) => out.push_str(&label),
                        None => out.push_str(&format!("[[unknown-key:{slug_want}]]")),
                    }
                } else if let Some(slug_want) = inner.strip_prefix(CMD_PREFIX) {
                    match cmd_token_label(slug_want) {
                        Some(label) => out.push_str(&label),
                        None => out.push_str(&format!("[[unknown-cmd:{slug_want}]]")),
                    }
                } else {
                    out.push_str(&format!("[[unknown-token:{inner}]]"));
                }
                rest = &after[end + CLOSE.len()..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests;

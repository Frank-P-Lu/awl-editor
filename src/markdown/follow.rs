//! FOLLOWING a followable span — the one owner that turns "the caret/pointer is
//! here" into "and this is where it goes".
//!
//! The underline grammar ([`MdKind::is_followable`], drawn as
//! `render::rects::Bucket::LinkUnderline`) already draws one hairline under
//! every span a person can follow. This module is the other half of that
//! promise: given a document byte, which followable span covers it and what is
//! its RESOLVED destination. Both halves read the same predicate, so a span
//! cannot wear the hairline without answering here.
//!
//! PURE and total. Nothing is opened, spawned or read off disk — the caller
//! turns a [`Destination`] into the one typed `actions::Effect` that carries it
//! (`FollowLink` for the OS handoff, `OpenPathAtLine` for a document awl opens
//! itself), and only the live `App` performs the outward half.

use std::ops::Range;

use super::MdKind;
use super::spans::bare_url_split;

/// Where a followable span points, once resolved. The three arms are not a
/// taxonomy of URLs — they are the three DOORS awl has for a destination, so
/// the caller's match is over what it must do rather than over syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// Carries its own URI scheme (`https:`, `mailto:`, `file:`…) — the OS
    /// opener's job. An outward action at the user's explicit gesture, the same
    /// shape as the `$EDITOR` daemon handoff; awl still fetches nothing itself.
    External(String),
    /// A path to another local document — awl opens it ITSELF, the Live-Preview
    /// model's own move for a vault of notes linking each other. Relative as
    /// WRITTEN; [`resolve_local`] anchors it against the containing document's
    /// own directory, which is what markdown means by a relative link.
    Local(String),
    /// A destination the follow grammar deliberately does not route yet — a
    /// bare `#heading-anchor`. Recorded rather than silently dropped so the
    /// affordance can say "nothing to go to" instead of opening the wrong
    /// thing, and so the deferred case has a name to be built against.
    InDocument(String),
}

/// One followable span the pointer or caret landed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Followable {
    /// WHICH member of the underline grammar enrolled this — carried so a law
    /// (and a failure message) can name what it swept rather than assuming.
    pub kind: MdKind,
    /// The span's own document byte range.
    pub range: Range<usize>,
    /// The destination exactly as the document writes it.
    pub raw: String,
    /// The same destination, classified into the door that serves it.
    pub dest: Destination,
}

/// The followable span containing document byte `byte`, with its destination
/// resolved — the ONE lookup both the caret door (`Action::FollowLink`) and the
/// pointer door (the modifier-click gesture) ask.
///
/// Two probes, in order, and they cannot disagree because the second is a
/// strict WIDENING of the first:
///
/// 1. The underline grammar's own spans (`spans(text)` filtered by
///    [`MdKind::is_followable`]) — exactly the bytes that wear the hairline.
///    The innermost match wins, so a followable span nested in another resolves
///    to the one actually under the pointer.
/// 2. For a named link only, the whole structural `[text](url)` range — so a
///    caret sitting on a bracket or inside the `(url)` tail, which wears no
///    hairline of its own, still follows the link it is plainly part of. A bare
///    URL has no such outer range: its span already IS the whole match.
///
/// `None` when `byte` is in no followable span at all — the calm no-op a
/// modifier-click on a plain word produces.
pub fn followable_at(text: &str, byte: usize) -> Option<Followable> {
    let spans = super::spans(text);
    let best = spans
        .iter()
        .filter(|(range, kind)| kind.is_followable() && range.contains(&byte))
        .min_by_key(|(range, _)| range.len())
        .map(|(range, kind)| (range.clone(), *kind));
    if let Some((range, kind)) = best
        && let Some(raw) = raw_destination(text, &range, kind)
    {
        return Some(followable(kind, range, raw));
    }
    // The widening probe: a named link's brackets and `(url)` tail are
    // `Markup`, not `LinkText`, so they carry no hairline — but they are the
    // same link, and the caret door has always followed from there.
    let link = super::link_at_full(text, byte)?;
    Some(followable(MdKind::LinkText, link.start..link.end, link.url))
}

fn followable(kind: MdKind, range: Range<usize>, raw: String) -> Followable {
    let dest = classify(&raw);
    Followable {
        kind,
        range,
        raw,
        dest,
    }
}

/// The destination text a followable span of `kind` covering `range` points at.
/// NO WILDCARD over the followable members: a new one fails to compile here
/// until it says how its destination is read, which is the whole point of
/// letting [`MdKind::is_followable`] be the enrolment.
fn raw_destination(text: &str, range: &Range<usize>, kind: MdKind) -> Option<String> {
    match kind {
        // A bare URL's destination IS its own source text, scheme through tail.
        MdKind::BareUrlText => text.get(range.clone()).map(str::to_string),
        // A named link's visible label; the destination lives in the enclosing
        // `[label](dest)`, read through the same parse the label came from.
        MdKind::LinkText => super::link_at(text, range.start),
        _ => None,
    }
}

/// Classify one destination string into the door that serves it. The scheme
/// test is the generic URI shape (`ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
/// ":"`) rather than the two literal schemes bare-URL DETECTION accepts: a
/// named link may legitimately write `mailto:` or `file:`, and handing those to
/// the OS opener is what every other editor does. A Windows drive letter
/// (`C:\…`) is one character and so fails the two-or-more rule, landing in
/// [`Destination::Local`] where it belongs.
pub fn classify(raw: &str) -> Destination {
    let raw = raw.trim();
    if raw.starts_with('#') {
        return Destination::InDocument(raw.to_string());
    }
    if has_uri_scheme(raw) {
        return Destination::External(raw.to_string());
    }
    // A same-document fragment or a query rides on the path in markdown; awl
    // opens the FILE and defers the within-file half (see `InDocument`).
    let path = raw.split(['#', '?']).next().unwrap_or(raw);
    if path.is_empty() {
        return Destination::InDocument(raw.to_string());
    }
    Destination::Local(path.to_string())
}

fn has_uri_scheme(s: &str) -> bool {
    let Some(colon) = s.find(':') else {
        return false;
    };
    let scheme = &s[..colon];
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    // Two-or-more so a bare Windows drive letter stays a path, not a scheme.
    first.is_ascii_alphabetic()
        && scheme.len() >= 2
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Anchor a [`Destination::Local`] path against `doc`, the absolute path of the
/// document the link was written in — markdown's own rule, and the reason this
/// cannot be resolved against the project root: `notes/a/b.md` linking `c.md`
/// means `notes/a/c.md`, not `notes/c.md`. An absolute `path` is already its own
/// answer. `None` for a document with no path of its own (an unsaved scratch
/// buffer has no directory for "relative" to mean anything against), which is
/// the calm no-op the caller signals as `Effect::None`.
pub fn resolve_local(doc: Option<&std::path::Path>, path: &str) -> Option<std::path::PathBuf> {
    let candidate = std::path::Path::new(path);
    if candidate.is_absolute() {
        return Some(candidate.to_path_buf());
    }
    Some(doc?.parent()?.join(candidate))
}

/// How many characters of a destination the go-to affordance may show before it
/// stops being a label and starts being the URL flood the tamed render exists
/// to avoid.
const LABEL_BUDGET: usize = 40;

/// The GO-TO ROW'S LABEL for `dest` — the same TAMED authority the document
/// itself shows under the hairline, never the raw URL. Routed through
/// [`bare_url_split`], the one owner of where a bare URL's scheme ends and its
/// tail begins, so the card and the rendered line cannot disagree about what
/// the tame form of a URL is.
///
/// - `https://example.com/a/b?c` → `Go to example.com…`
/// - `https://example.com` → `Go to example.com` (no tail, no ellipsis — the
///   same "no path, no ellipsis promise" rule the render already follows)
/// - `mailto:x@y.z` → `Go to x@y.z`
/// - `../notes/plan.md` → `Go to ../notes/plan.md`
/// - `#section` → `Go to #section`
pub fn go_to_label(raw: &str) -> String {
    format!("Go to {}", tamed(&classify(raw)))
}

/// The tamed rendering of one destination — [`go_to_label`]'s payload, split
/// out so a law can assert the taming without parsing a sentence around it.
pub fn tamed(dest: &Destination) -> String {
    let shown = match dest {
        Destination::External(url) => {
            let (scheme, tail) = bare_url_split(url);
            if scheme.is_empty() {
                // A non-`http(s)` scheme (`mailto:`, `file:`) has no authority
                // to tame down to; show the payload past the colon, which is
                // the part a reader recognises.
                url.split_once(':').map_or(url.clone(), |(_, rest)| {
                    rest.trim_start_matches('/').to_string()
                })
            } else {
                let authority = &url[scheme.end..tail.as_ref().map_or(url.len(), |t| t.start)];
                match tail {
                    Some(_) => format!("{authority}…"),
                    None => authority.to_string(),
                }
            }
        }
        Destination::Local(path) => path.clone(),
        Destination::InDocument(frag) => frag.clone(),
    };
    ellipsize(&shown, LABEL_BUDGET)
}

fn ellipsize(s: &str, budget: usize) -> String {
    if s.chars().count() <= budget {
        return s.to_string();
    }
    let kept: String = s.chars().take(budget.saturating_sub(1)).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests;

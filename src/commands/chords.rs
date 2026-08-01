//! src/commands/chords.rs — THE CONVENTION + PLATFORM RESOLUTION of a
//! command's chord: what `Cmd-S` becomes on a Linux desktop, what it becomes in
//! a browser tab that has already claimed it, and what is left when the answer
//! is "nothing that fires".
//!
//! Lifted out of `commands.rs` whole (queue item 24) when the SPEC half of the
//! truthful resolver pushed that file past its recorded size mark. Nothing here
//! changed in the move except the extraction itself; the cluster was already
//! one subject, with `resolved_native` at its base and every label surface in
//! the codebase reading down from it.

use super::{COMMANDS, Command, Platform, slug};
use crate::convention::Convention;

// ── LINUX-NATIVE KEYMAP: convention-resolved slot 1 ────────────────────────────
//
// THE DATA DESIGN (chosen over per-convention chord COLUMNS): each catalog row
// keeps its ONE mac-flavored `native` string, unchanged — that stays the source
// of truth `bindings()`/`join_slots` read for the Mac baseline. A Linux label or
// dispatch NEVER reads a second stored column; instead it's a PURE, TOTAL
// TRANSLATION of that same string (`keyspec::translate_native_for_linux`, a plain
// Cmd→Ctrl modifier swap) with an EXPLICIT OVERRIDE table below for the handful of
// commands where that naive swap is WRONG. Why this over per-convention columns:
// (1) it keeps the catalog's ONE mac-native field as the single hand-maintained
// fact per command (no risk of the two columns drifting when a mac chord changes
// and the Linux column isn't updated to match); (2) the override table is a
// SHORT, auditable exceptions list rather than 60+ rows of mostly-identical data;
// (3) `keymap.rs`'s dispatch reuses the EXACT SAME override for the handful of
// commands whose action needs a genuinely different resolve-time chord (not just
// a translated label) — see `commands::LINUX_NATIVE_OVERRIDE`'s doc for why those
// three exist.
//
// THE OVERRIDE TABLE, keyed by catalog command NAME, holding the LITERAL Linux
// chord spec to use instead of the naive Cmd→Ctrl swap:
//   - "Line start" / "Line end": mac native is Cmd-Left/Right; naively swapping
//     Super→Control would collide with Ctrl-Left/Right, which the keymap ALREADY
//     binds to word motion (`resolve_named`'s `alt || ctrl` arm, convention-
//     agnostic) — so the Linux-native chord is plain `Home`/`End` instead (no
//     modifier needed; `resolve_named`'s unconditional Home/End arms already fire
//     LineStart/LineEnd on every convention, so no keymap change is needed here —
//     only the LABEL differs from the naive swap).
//   - "Document start" / "Document end": mac native is Cmd-Up/Down; the Linux
//     convention for buffer start/end is Ctrl-Home/Ctrl-End (gedit/VS Code/GTK),
//     not the naively-translated Ctrl-Up/Down — `keymap.rs` gains a matching
//     `Convention::Linux`-gated `Ctrl-Home`/`Ctrl-End` arm (see its module doc).
const LINUX_NATIVE_OVERRIDE: &[(&str, &str)] = &[
    ("Line start", "Home"),
    ("Line end", "End"),
    ("Document start", "C-Home"),
    ("Document end", "C-End"),
];

/// The RESOLVED native chord spec for `c` under `convention` — Mac returns `c.native`
/// UNCHANGED (byte-identical to today, the hard law of this round); Linux consults
/// [`LINUX_NATIVE_OVERRIDE`] first, else falls back to the naive Cmd→Ctrl translation
/// (`keyspec::translate_native_for_linux`). Empty on either convention when the
/// command has no native slot to begin with. This is the ONE owner both `keymap.rs`'s
/// dispatch (for the handful of commands whose ACTION needs the resolved chord, not
/// just its label — via `[keys]`-style literal resolution) and every label surface
/// below route through.
pub fn resolved_native(c: &Command, convention: Convention) -> String {
    if c.native.trim().is_empty() {
        return String::new();
    }
    match convention {
        Convention::Mac => c.native.to_string(),
        Convention::Linux => LINUX_NATIVE_OVERRIDE
            .iter()
            .find(|(name, _)| *name == c.name)
            .map(|(_, chord)| chord.to_string())
            .unwrap_or_else(|| crate::keyspec::translate_native_for_linux(c.native)),
    }
}

/// The DISPLAY LABEL for `c`'s resolved native chord under `convention` — Mac glyphs
/// (`⌘S`) on [`Convention::Mac`], word labels (`Ctrl+S`) on [`Convention::Linux`].
/// `""` when the command has no native slot. THE ONE OWNER every label surface reads
/// (palette rows, the rebind menu, the in-app menubar hints, the hold-⌘ peek) — never
/// call [`crate::keyspec::mac_glyph_chord`] on a raw `c.native` directly outside this
/// function, or a Linux/web build would show a mac glyph under its own convention.
#[cfg(any(not(target_arch = "wasm32"), test))]
pub fn resolved_native_label(c: &Command, convention: Convention) -> String {
    let native = resolved_native(c, convention);
    if native.trim().is_empty() {
        return String::new();
    }
    match convention {
        Convention::Mac => crate::keyspec::mac_glyph_chord(&native),
        Convention::Linux => crate::keyspec::linux_glyph_chord(&native),
    }
}

/// THE WEB CHORD SANITY ROUND, Tier 2 — [`resolved_native_label`]'s TRUTHFUL
/// sibling: when `c`'s resolved native chord is a browser-reserved accelerator
/// ([`crate::webreserved::is_reserved`]) on `platform`, this shows the command's
/// [`WEB_ALTERNATE`] chord instead (see that table's doc — v2 of the web-chord
/// sanity round, closing the v1 "no replacement chord" gap), or `""` if it has
/// none; otherwise identical to [`resolved_native_label`]. `platform` is an
/// EXPLICIT parameter (not read from [`Platform::current`] internally) — the
/// same testability pattern [`Command::available_on`]/[`action_available`]
/// already use — so a native-run test can assert the WEB view directly by
/// passing [`Platform::Web`] without any cfg gymnastics; every real call site
/// passes [`Platform::current`]. The reserved check only ever fires on
/// [`Platform::Web`] — a native build's chords are never browser-shadowed, so
/// this is byte-identical to [`resolved_native_label`] on every native call
/// site. THE ONE OWNER of "is this command's native chord actually worth
/// showing" — [`join_slots_truthful`] (the two-slot palette/rebind label),
/// `menu::item_chord` (the awl-rendered menu bar's native-only column, which
/// shows on web too), and `keytoken::key_token_label` (the starting docs'
/// chord tokens) all route through it.
pub fn resolved_native_label_truthful(
    c: &Command,
    convention: Convention,
    platform: Platform,
) -> String {
    let spec = resolved_native_truthful(c, convention, platform);
    if spec.trim().is_empty() {
        return String::new();
    }
    match convention {
        Convention::Mac => crate::keyspec::mac_glyph_chord(&spec),
        Convention::Linux => crate::keyspec::linux_glyph_chord(&spec),
    }
}

/// [`resolved_native_label_truthful`]'s own SPEC half, split out so a caller
/// that needs to DISPATCH the chord — rather than print it — reads the same
/// answer the label does. `""` when there is no chord worth showing: the
/// command has no native slot at all, or its resolved chord is browser-reserved
/// on [`Platform::Web`] and it has no [`WEB_ALTERNATE`] to fall back to.
///
/// The split exists because a label alone cannot be verified. `keytoken`'s
/// starting-docs law drives every chord the welcome/tour/GUIDE actually teach
/// through a real [`crate::keymap::KeymapState`] and asserts it lands on the
/// command's own `Action` — which needs the terse spec `keyspec::parse_chord`
/// accepts, not the ⌘-glyph a human reads.
///
/// THE LINUX BUILT-IN KEEP TIER (queue item 24) is applied here, and it is the
/// tier this function was missing. `keymap::linux_builtin_keep()` (`["C-k"]`)
/// is UNCONDITIONAL on `Convention::Linux` — every flavour, every config — so
/// Insert-link's naively translated `C-k` never fires there; `C-k` is kill-line
/// (docs/config.md's tripwire). [`join_slots_truthful`] already knew this and
/// suppresses the label, which is why GUIDE's generated table correctly prints
/// an EMPTY Linux cell for Insert link. Every other reader — the two-slot
/// palette label's native half, and `keytoken`'s starting-doc tokens — reached
/// [`resolved_native_label_truthful`] directly and got `Ctrl+K` back: a chord
/// that resolves to kill-line, offered as the way to insert a link. A welcome
/// document is the worst place for that, so the floor moved into the shared
/// owner, where no reader can dodge it.
///
/// Only the UNCONDITIONAL builtin list is applied, never the config-dependent
/// `effective_linux_keep()`: a caller that knows the user's own keep list still
/// applies it on top ([`join_slots_truthful`] does), and a caller that does not
/// must never over-claim a suppression that some configs would not have.
pub fn resolved_native_truthful(c: &Command, convention: Convention, platform: Platform) -> String {
    let native = resolved_native(c, convention);
    if convention == Convention::Linux {
        let builtin: Vec<String> = crate::keymap::linux_builtin_keep()
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        if crate::keymap::linux_keeps_chord(&builtin, &native) {
            return String::new();
        }
    }
    let reserved =
        platform == Platform::Web && crate::webreserved::is_reserved(&native, convention);
    if reserved {
        return web_alternate_for(c, convention)
            .map(str::to_string)
            .unwrap_or_default();
    }
    native
}

const WEB_ALTERNATE: &[(&str, &str, &str)] = &[
    ("New document", "C-j", "M-n"),
    ("Switch theme…", "C-t", "M-t"),
];

pub(super) fn web_alternate_for(c: &Command, convention: Convention) -> Option<&'static str> {
    WEB_ALTERNATE
        .iter()
        .find(|(name, _, _)| *name == c.name)
        .map(|(_, mac, linux)| match convention {
            Convention::Mac => *mac,
            Convention::Linux => *linux,
        })
}

/// The config `[keys]`-shaped entries that wire every [`WEB_ALTERNATE`] chord
/// into REAL dispatch on [`Platform::Web`] — the keymap has no other seam for
/// "a chord outside the native/emacs static arms," so this reuses the SAME
/// override machinery a user's own `[keys]` line rides
/// (`KeymapState::apply_overrides`, fed from `App::new`'s keymap construction).
/// `existing` is the user's OWN config `[keys]` list — **config still trumps
/// everything**: a command the user has already rebound (by its slug) is
/// skipped here entirely, so their chosen chord is never shadowed by the
/// default alternate. `convention`/`platform` are EXPLICIT parameters,
/// mirroring [`resolved_native_label_truthful`]'s own testability pattern
/// (`Convention::current`/`Platform::current` can't be pinned from a plain
/// native test) — every real call site passes both `::current()`. Returns an
/// empty list on [`Platform::Native`], so a native build's keymap
/// construction is unaffected byte-for-byte.
pub fn web_alternate_keys(
    existing: &[(String, Vec<String>)],
    convention: Convention,
    platform: Platform,
) -> Vec<(String, Vec<String>)> {
    if platform != Platform::Web {
        return Vec::new();
    }
    COMMANDS
        .iter()
        .filter_map(|c| {
            let alt = web_alternate_for(c, convention)?;
            let want = slug(c.name);
            if existing.iter().any(|(name, _)| slug(name) == want) {
                return None; // a `[keys]` override already claims this command
            }
            Some((want, vec![alt.to_string()]))
        })
        .collect()
}

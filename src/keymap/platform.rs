use std::collections::HashSet;

use winit::keyboard::{Key, ModifiersState};

use super::binding::canon_key;
use super::{Chord, parse_binding};

pub(super) fn linux_keeps_chord_raw(
    keep: &HashSet<(Key, ModifiersState)>,
    chord_spec: &str,
) -> bool {
    match parse_binding(chord_spec) {
        Ok(Chord::Single(k, m)) => keep.contains(&(k, m)),
        _ => false,
    }
}

pub(super) fn linux_displaces_emacs_default_raw(
    emacs: &str,
    keep: &HashSet<(Key, ModifiersState)>,
) -> bool {
    let Some(first) = emacs.split_whitespace().next() else {
        return false;
    };
    let Ok((key, mods)) = crate::keyspec::parse_chord(first) else {
        return false;
    };
    if mods.state() != ModifiersState::CONTROL {
        return false;
    }
    let Key::Character(s) = &key else {
        return false;
    };
    s.chars().next().is_some_and(|c| {
        LINUX_DISPLACED_LETTERS.contains(&c.to_ascii_lowercase())
            && !keep.contains(&(canon_key(&key), mods.state()))
    })
}

/// The LETTERS the table above displaces (every `Ctrl-<letter>` whose native
/// meaning wins on [`Convention::Linux`]) — the ONE data owner both
/// `tests::linux_collision_table_matches_the_documented_displaced_list` (which
/// still separately pins EACH letter's resolved `Action`) and
/// [`linux_displaces_emacs_default`] (the LABEL-TRUTH half — is an emacs
/// default worth SHOWING under this convention) read, so the dispatch table and
/// the label truth can never silently drift apart. `k` is deliberately NOT
/// here — see `linux_builtin_keep()`'s doc for why Insert link's Ctrl-K is a
/// third, unconditionally-kept case rather than an ordinary displaced letter.
pub(crate) const LINUX_DISPLACED_LETTERS: &[char] = &[
    's', 'p', 'n', 'w', 'f', 'e', 'a', 'g', 'r', 'b', 'c', 'x', 'v',
];

/// THE INSERT-LINK-YIELDS-TO-KILL-LINE ROUND (settled — the user's own call:
/// "kill-line is too load-bearing for emacs hands to lose by default") — chords
/// that keep their EMACS meaning on [`Convention::Linux`] UNCONDITIONALLY,
/// independent of `linux_keep_emacs`/the `keymap` flavor preset. Currently just
/// `C-k` (Kill line survives Links v2's Cmd-K spend): unlike every letter in
/// [`LINUX_DISPLACED_LETTERS`] (which a user must opt BACK into via
/// `linux_keep_emacs`/`keymap = "emacs"` to keep), `C-k` never displaces at
/// all out of the box, on EITHER keymap flavor — the native Insert-link chord
/// simply has NO effective Linux binding by default (still one `[keys]
/// insert_link = "C-k"` line away for a Linux hand who explicitly wants the
/// trade — a `[keys]` override is consulted before this floor, same as every
/// other override).
///
/// Consumed from TWO structurally separate places that must agree (mirroring
/// how [`LINUX_DISPLACED_LETTERS`] itself already feeds both the dispatch
/// table and the label-truth functions): [`super::KeymapState::apply_linux_keep`]
/// seeds it UNCONDITIONALLY on every call (the dispatch half — a reload can
/// never clear it away) and [`crate::config::Config::effective_linux_keep`]
/// seeds it into the composed keep-list it returns (the label half —
/// `commands::join_slots_truthful` never touches `KeymapState` directly, so
/// it needs its own copy of the same guarantee). `Convention::Mac` never
/// consults `linux_keep` at all, so this is structurally inert there — Cmd-K
/// stays Insert link on Mac, unconditionally.
///
/// THE KEYMAP-DEFAULTS-AS-DATA ROUND: this is now a thin accessor over
/// [`crate::keymap_defaults::linux_builtin_keep`] (itself parsed once from
/// the embedded `assets/keymap-defaults.toml`'s `linux_builtin_keep` array)
/// rather than a literal `const` — the value (`["C-k"]`) is unchanged, only
/// where it lives moved, so every call site needed only `()` added.
pub(crate) fn linux_builtin_keep() -> &'static [&'static str] {
    crate::keymap_defaults::linux_builtin_keep()
}

/// THE WEB CHORD SANITY ROUND, Tier 3 — is `emacs` (a command's static slot-2
/// text, e.g. `"C-s"` or the `"C-c C-o"` prefix sequence) quietly DISPLACED under
/// [`Convention::Linux`]? Checks only the emacs default's FIRST key: a bare
/// (no Shift/Alt/Super) `Ctrl-<letter>` whose letter appears in
/// [`LINUX_DISPLACED_LETTERS`] is displaced — this covers both a single-chord
/// default (`"C-s"`) and a prefix sequence whose FIRST key is itself claimed
/// (`"C-c C-o"`: Ctrl-C now resolves straight to Copy, so the whole sequence
/// never arms). `false` for an empty/unparsable emacs slot, or a modified chord
/// (`"C-/"`, `"C-y"`) outside the displaced-letter set.
///
/// `keep` is the config `linux_keep_emacs` list (THE EMACS-HANDS-ON-LINUX
/// per-chord door) — a chord named there is NEVER displaced, regardless of
/// whether its letter is in [`LINUX_DISPLACED_LETTERS`] (checked via
/// [`linux_keeps_chord`], the SAME canonical-compare helper the label owner's
/// native-suppression half uses, so the two directions of this round's fix can
/// never disagree about what "kept" means). Pure — the label-truth owner
/// (`commands::join_slots_truthful`) is the only caller; mirrors the dispatch
/// collision table structurally, never re-derives it.
pub(crate) fn linux_displaces_emacs_default(emacs: &str, keep: &[String]) -> bool {
    let Some(first) = emacs.split_whitespace().next() else {
        return false;
    };
    let Ok((key, mods)) = crate::keyspec::parse_chord(first) else {
        return false;
    };
    if mods.state() != ModifiersState::CONTROL {
        return false; // must be a BARE Ctrl chord — no Shift/Alt/Super riders.
    }
    let Key::Character(s) = &key else {
        return false;
    };
    let letter_displaced = s
        .chars()
        .next()
        .is_some_and(|c| LINUX_DISPLACED_LETTERS.contains(&c.to_ascii_lowercase()));
    letter_displaced && !linux_keeps_chord(keep, first)
}

/// Is `chord_spec` (a raw chord string, e.g. `"C-f"` or a command's resolved
/// native chord like `"Ctrl-F"`) present in the LINUX KEEP-LIST `keep`, compared
/// CANONICALLY ([`crate::keyspec::canonical_binding`], so `"C-f"` == `"Ctrl-f"`
/// == `"Control-F"`)? `false` for an empty/unparsable `chord_spec` on EITHER
/// side. The ONE comparison both halves of the emacs-hands-on-Linux label fix
/// share: [`linux_displaces_emacs_default`] (does a kept chord stop displacing
/// the emacs default?) and `commands::join_slots_truthful`'s native-suppression
/// check (does a kept chord stop the NATIVE command from advertising it?) — so
/// the two directions can never quietly disagree about what "kept" means.
pub(crate) fn linux_keeps_chord(keep: &[String], chord_spec: &str) -> bool {
    let Some(want) = crate::keyspec::canonical_binding(chord_spec) else {
        return false;
    };
    keep.iter()
        .any(|k| crate::keyspec::canonical_binding(k).as_deref() == Some(want.as_str()))
}

enum_with_all! {
    /// THE KEYMAP FLAVOR ROUND — a config `keymap = "native" | "emacs"` PRESET,
    /// orthogonal to [`Convention`] (which decides whether slot 1 SPEAKS ⌘-chords
    /// or Ctrl-chords). `Native` (the default) is today's behavior byte-identical.
    /// `Emacs` widens the emacs-hands-on-Linux `linux_keep_emacs` PER-CHORD door
    /// (see [`super::KeymapState::apply_linux_keep`]/[`linux_keeps_chord`] above) into a
    /// whole-catalog PRESET: every chord [`LINUX_DISPLACED_LETTERS`] names keeps
    /// its emacs meaning, unioned with the user's own explicit `linux_keep_emacs`
    /// entries — see `crate::config::Config::effective_linux_keep`, THE ONE
    /// COMPOSITION OWNER (this module stays unaware of the config field entirely;
    /// it only ever sees the already-composed `keep` list `with_overrides_and_keep`/
    /// `apply_linux_keep` take). Inert on [`Convention::Mac`] structurally, same as
    /// `linux_keep_emacs` itself — no collisions exist there to keep.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum KeymapFlavor {
        #[default]
        Native,
        Emacs,
    }
}

impl KeymapFlavor {
    pub fn parse(s: &str) -> Option<KeymapFlavor> {
        match s.trim().to_ascii_lowercase().as_str() {
            "native" => Some(KeymapFlavor::Native),
            "emacs" => Some(KeymapFlavor::Emacs),
            _ => None,
        }
    }

    pub fn config_name(self) -> &'static str {
        match self {
            KeymapFlavor::Native => "native",
            KeymapFlavor::Emacs => "emacs",
        }
    }

    /// The "Keymap…" picker's PRIMARY column — plain language, never the bare
    /// config slug (`config_name`), so a reader who has never seen the word
    /// "emacs" still knows what they picked. Mirrors [`crate::caret::CaretMode::
    /// label`]'s split between a short primary label and a longer secondary
    /// [`Self::description`].
    pub fn label(self) -> &'static str {
        match self {
            KeymapFlavor::Native => "Standard",
            KeymapFlavor::Emacs => "Emacs",
        }
    }

    /// The picker's SECONDARY column: the concrete chords that differ, so the
    /// choice is legible without already knowing either convention by name.
    pub fn description(self) -> &'static str {
        match self {
            KeymapFlavor::Native => "Ctrl+C copies, Ctrl+V pastes",
            KeymapFlavor::Emacs => "Ctrl navigates (C-p, C-n, C-a…)",
        }
    }
}

/// The Emacs preset derives every `C-<letter>` directly from
/// [`LINUX_DISPLACED_LETTERS`], so the collision table and preset cannot drift.
/// `C-k` stays outside: `linux_builtin_keep()` covers it on either flavor, so it
/// unions both in regardless of which flavor is active.
pub fn linux_emacs_preset_keep() -> Vec<String> {
    LINUX_DISPLACED_LETTERS
        .iter()
        .map(|c| format!("C-{c}"))
        .collect()
}

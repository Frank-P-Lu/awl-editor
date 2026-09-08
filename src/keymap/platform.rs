use std::collections::HashSet;

use winit::keyboard::{Key, ModifiersState};

use super::binding::canon_key;
use super::{Action, Chord, parse_binding};
use crate::convention::Convention;

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

/// THE NATIVE-CLIPBOARD CARVE-OUT (user decision: the Omarchy/Hyprland compositor
/// forwards Super+C/V as Ctrl+C/V for the system clipboard, so those two letters
/// must survive the emacs preset even though [`LINUX_DISPLACED_LETTERS`] names
/// them) — the two letters [`crate::config::Config::effective_linux_keep`] must
/// never let the flavor PRESET'S OWN contribution claim, even though
/// [`linux_emacs_preset_keep`] still names them (so a user's own explicit
/// `linux_keep_emacs` entry for `"C-c"`/`"C-v"` is untouched by this — that
/// per-chord door is a deliberate ask, not the preset's blanket one). `C-x` is
/// deliberately NOT here: it carries Save/Open as the emacs prefix, and
/// excluding it would gut the flavor. Emacs hands keep Cut/Paste on their
/// existing `C-w`/`C-y` aliases regardless (`assets/keymap-defaults.toml`'s own
/// `cut`/`paste` emacs slots).
pub(crate) const NATIVE_CLIPBOARD_LETTERS: &[char] = &['c', 'v'];

/// Is `chord_spec` one of [`linux_emacs_preset_keep`]'s own `"C-<letter>"`
/// strings for a [`NATIVE_CLIPBOARD_LETTERS`] entry? Pure string match — the
/// preset always emits that exact canonical shape itself, so no parse/
/// canonicalize round-trip is needed. The ONE predicate
/// [`crate::config::Config::effective_linux_keep`] filters the preset's own
/// contribution through; never consulted for the user's `linux_keep_emacs`
/// list, which stays unfiltered.
pub(crate) fn linux_is_native_clipboard_chord(chord_spec: &str) -> bool {
    NATIVE_CLIPBOARD_LETTERS
        .iter()
        .any(|c| chord_spec.eq_ignore_ascii_case(&format!("C-{c}")))
}

/// THE CLASSIC META LAYER (user decision) — Linux-only default `default_single`
/// bindings [`super::KeymapState::seed_defaults`] seeds ONLY when
/// [`super::KeymapState::linux_emacs_meta`] is set (i.e. `Convention::Linux`
/// AND `keymap = "emacs"`; inert on every other combination, including Mac
/// under the emacs flavor — Option keeps typing accented characters there, see
/// `resolve.rs`'s `is_meta_chord` doc). Every entry fires an EXISTING catalog
/// `Action`, the same way any other Meta chord in this list seeds one — `M-x`
/// is a Linux-emacs-flavor-SEEDED binding onto the command palette's own
/// `Action::OpenCommandPalette` (its catalog `emacs` slot is deliberately
/// empty; this is a second, flavor-owned door onto the SAME action, never a
/// parallel palette-open path). `M-Backspace` and `M-<`/`M->` use their
/// standard-Emacs spellings; `M-<`/`M->` need no Shift companion since `<`/`>`
/// are themselves the key tokens `parse_binding` matches.
pub(crate) const LINUX_EMACS_META_SEED: &[(&str, Action)] = &[
    ("M-x", Action::OpenCommandPalette),
    ("M-w", Action::CopyRegion),
    ("M-f", Action::ForwardWord),
    ("M-b", Action::BackwardWord),
    ("M-d", Action::DeleteWordForward),
    ("M-Backspace", Action::DeleteWordBackward),
    // THE SENTENCE-MOTION ROUND — the classic emacs `M-a`/`M-e`/`M-k` slots
    // (forward/backward-sentence, kill-sentence), the same shape as the
    // word-motion trio just above: an EXISTING catalog `Action`, seeded here
    // rather than a native/emacs two-slot default (there is no macOS-native
    // sentence-motion convention to double). See `buffer::sentence`'s module
    // doc for the boundary rule these fire.
    ("M-a", Action::BackwardSentence),
    ("M-e", Action::ForwardSentence),
    ("M-k", Action::DeleteSentenceForward),
    ("M-v", Action::PageScrollUp),
    ("M-<", Action::BufferStart),
    ("M->", Action::BufferEnd),
    // THE CLASSIC-CHORDS ROUND (user decision) — `M-%` joins this table rather
    // than `LINUX_EMACS_CLASSIC_SEED` below: it is a single Meta-modified key,
    // the exact shape every other entry here already is, not a `C-x`
    // continuation.
    ("M-%", Action::OpenReplace),
];

/// THE CLASSIC-CHORDS ROUND (user decision, Linux `keymap = "emacs"` only) —
/// the classic emacs `C-x` continuations that a Linux hand loses when the
/// displaced-letter cluster above claims their single-key native equivalents
/// (Save's Ctrl-S becomes isearch, Finish file's Ctrl-W becomes kill-region,
/// Select all's Ctrl-A becomes line-start): `C-x C-s` Save, `C-x C-f` Go to
/// (the classic emacs find-file binding), `C-x k` Finish file (kill-buffer's
/// own key), `C-x h` Select all (mark-whole-buffer's own key). Same gate as
/// [`LINUX_EMACS_META_SEED`] (this table is a SECOND entry in
/// [`active_seed_tables`]'s returned list, not a second gate or a second
/// consumption loop), same doc precedent: every entry fires an EXISTING
/// catalog `Action`. The prefix machinery, which-key panel, and `c_x`
/// override map all already existed for the identity round's now-empty
/// defaults — this table is data flowing through them, not new machinery.
pub(crate) const LINUX_EMACS_CLASSIC_SEED: &[(&str, Action)] = &[
    ("C-x C-s", Action::Save),
    ("C-x C-f", Action::OpenGoto),
    ("C-x k", Action::FinishBuffer),
    ("C-x h", Action::SelectAll),
];

/// Is the Linux emacs LAYER active — the one gate the KEY seed table below
/// reads (both [`super::state::KeymapState::seed_defaults`] and the
/// label-truth query share it). `Convention::Mac` is structurally inert
/// (Option keeps typing accented characters there, and ⌘ already carries
/// every native chord); the `native` flavor seeds nothing anywhere.
///
/// **Middle-click follow no longer shares this gate** (user decision,
/// widening it off the emacs flavor): it collided with nothing under
/// `native`, so [`active_follow_gestures`] offers it on EITHER flavor —
/// `the_pointer_gesture_layer_no_longer_shares_the_key_seed_gate` names the
/// two axes decoupled rather than leaving the old shared-gate claim to rot.
pub(crate) fn linux_emacs_layer(convention: Convention, flavor: KeymapFlavor) -> bool {
    convention == Convention::Linux && flavor == KeymapFlavor::Emacs
}

/// A MOUSE chord that follows a followable span. Deliberately its own type
/// rather than a `&str` in the seed tables above: the `[keys]` grammar for
/// COMMAND rebinds and every seed table are KEY chords, parsed by
/// `keyspec::parse_chord` into a `(Key, ModifiersState)` pair, and a mouse
/// button has no spelling there. That separation is also why the Linux
/// keep-list cannot collide with any gesture here — `Config::effective_linux_keep`
/// composes, and `linux_keeps_chord` compares, only strings that parse as key
/// chords. A follow gesture DOES have its own rebinding grammar
/// (`keyspec::parse_pointer_chord`, a deliberate sibling of `parse_chord`,
/// never a branch inside it) — see [`active_follow_gestures`]'s doc for the
/// override this type's `label` is generated for when a config line is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FollowGesture {
    /// Which physical button. Kept as a plain enum rather than winit's own so
    /// the roster stays a pure core fact the label surfaces can read.
    pub button: PointerButton,
    /// The modifiers that must be held, EXACTLY — a gesture with extra
    /// modifiers down is not this gesture.
    pub mods: ModifiersState,
    /// How the gesture is NAMED to a reader — carried ON the roster rather
    /// than composed at each surface, so the first label surface to want it
    /// reads [`active_follow_gestures`] the way every label surface already
    /// reads [`active_seed_tables`] through [`seeded_chords_for`]. A built-in
    /// default's label is a literal; an override's label is rendered once by
    /// `keyspec::format_pointer_chord` and leaked to `'static` (the same
    /// one-time-per-config-load cost `commands::COMMANDS`'s own splice pays).
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Middle,
    /// The right/secondary button. Spellable in the rebind grammar
    /// (`right-click`, per the decided grammar roster) so a `[keys] follow`
    /// line naming it parses and shows in the roster — but no default
    /// gesture uses it, and dispatch does not route a right press through
    /// [`follows_link`] at all: `app::input::mouse_button::on_mouse_input`
    /// claims every right press for the context-menu card before the follow
    /// predicate is ever asked. A `follow = "right-click"` override is
    /// therefore a named, left gap (documented in docs/config.md), not a
    /// silently broken promise.
    Secondary,
}

/// macOS: ⌘-click, the decided Mac gesture and the platform convention in every
/// Mac editor. Ctrl-click deliberately has NO entry here — macOS itself spends
/// Ctrl-click as the secondary click, so claiming it would fight the OS.
const MAC_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Primary,
    mods: ModifiersState::SUPER,
    label: "Cmd-Click",
}];

/// Linux, BOTH flavors: Ctrl-click. The platform convention in every editor and
/// browser, and a MOUSE chord — so it collides with none of the `C-c`/`C-x`
/// text-chord rules, which govern key chords alone. Super-click is not offered:
/// the compositor usually owns Super.
const LINUX_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Primary,
    mods: ModifiersState::CONTROL,
    label: "Ctrl-Click",
}];

/// Linux, BOTH flavors (widened by user decision off its original
/// `keymap = "emacs"`-only gate: middle-click collided with nothing under
/// `native`, and the flavor gate existed only to keep the platform convention
/// plain) — middle-click (mouse-2). Free to claim on either flavor: awl
/// implements no X11 primary-selection paste, so nothing else wants mouse-2.
const LINUX_MIDDLE_FOLLOW: &[FollowGesture] = &[FollowGesture {
    button: PointerButton::Middle,
    mods: ModifiersState::empty(),
    label: "Middle-Click",
}];

/// THE FOLLOW GESTURE'S ONE SELECTION POINT — every mouse chord that follows a
/// followable span under `convention`, given any `[keys] follow` override
/// already loaded off `Config` (raw config strings, never pre-parsed — this
/// function owns the ONE parse). Both dispatch (the pointer press,
/// `app::input::mouse_button::press_follow_gesture`) and every label surface
/// read THIS, so an override and the built-in default can never drift apart.
///
/// `overrides` REPLACES the convention's default roster wholesale when it
/// parses to at least one valid gesture — unlike an ordinary `[keys]`
/// command rebind (additive: both the configured chord AND the default still
/// fire), because the follow roster is a small fixed SET, not a per-command
/// native/emacs pair, so "add to it" has no natural meaning. An entry that
/// [`crate::keyspec::parse_pointer_chord`] cannot spell is reported to
/// stderr NAMING the `[keys] follow` line and dropped — the same shape a bad
/// key chord already gets (`KeymapState::apply_overrides`'s own "keeping
/// default" note) — while any other valid entry in the line still takes
/// over; an override with NO valid entry at all leaves the built-in default
/// in force, exactly as if `overrides` were empty.
///
/// No longer flavor-gated (the decided widening above made Linux's roster
/// flavor-INDEPENDENT), so this function does not take one — see
/// `linux_emacs_layer`'s doc for what still does.
pub(crate) fn active_follow_gestures(
    convention: Convention,
    overrides: &[String],
) -> Vec<FollowGesture> {
    parse_follow_overrides(overrides).unwrap_or_else(|| default_follow_gestures(convention))
}

fn default_follow_gestures(convention: Convention) -> Vec<FollowGesture> {
    match convention {
        Convention::Mac => MAC_FOLLOW.to_vec(),
        Convention::Linux => LINUX_FOLLOW
            .iter()
            .chain(LINUX_MIDDLE_FOLLOW)
            .copied()
            .collect(),
    }
}

/// Parse a `[keys] follow` line into the roster it becomes; `None` when
/// `overrides` is empty or every entry fails to parse (the built-in default
/// keeps firing in either case — see [`active_follow_gestures`]'s doc for the
/// full per-entry contract).
fn parse_follow_overrides(overrides: &[String]) -> Option<Vec<FollowGesture>> {
    let mut out = Vec::new();
    for spec in overrides {
        match crate::keyspec::parse_pointer_chord(spec) {
            Ok((button, mods)) => {
                let label: &'static str =
                    Box::leak(crate::keyspec::format_pointer_chord(button, mods).into_boxed_str());
                out.push(FollowGesture {
                    button,
                    mods,
                    label,
                });
            }
            Err(e) => {
                eprintln!("config [keys]: follow = {spec:?}: {e}; keeping default");
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Does pressing `button` with exactly `mods` held FOLLOW, under
/// `convention` and any `[keys] follow` override? The one predicate both the
/// press path and the hover-cursor affordance ask, so the pointing hand and
/// the click can never disagree about which chord follows.
pub(crate) fn follows_link(
    convention: Convention,
    overrides: &[String],
    button: PointerButton,
    mods: ModifiersState,
) -> bool {
    active_follow_gestures(convention, overrides)
        .iter()
        .any(|g| g.button == button && g.mods == mods)
}

/// THE LABEL-TRUTH ROUND — every SEED TABLE active under `convention`+`flavor`,
/// in the SAME shape [`super::state::KeymapState::seed_defaults`] loops over for
/// real dispatch. This is the ONE selection point both the dispatch half (via
/// [`super::state::KeymapState::seed_defaults`]) and the advertisement half
/// (every label surface, via [`seeded_chords_for`]) consult — so a seeded layer
/// added table is picked up by BOTH from a single edit, rather than the two
/// staying in sync by hand. Today this returns [`LINUX_EMACS_META_SEED`] and
/// [`LINUX_EMACS_CLASSIC_SEED`]; empty under
/// `Native` flavor or [`Convention::Mac`], where the whole layer is
/// structurally inert (Option keeps typing accented characters there).
pub(crate) fn active_seed_tables(
    convention: Convention,
    flavor: KeymapFlavor,
) -> &'static [&'static [(&'static str, Action)]] {
    if linux_emacs_layer(convention, flavor) {
        &[LINUX_EMACS_META_SEED, LINUX_EMACS_CLASSIC_SEED]
    } else {
        &[]
    }
}

/// THE SEEDED-CHORD QUERY every label surface reads instead of re-deriving the
/// roster: the chord SPECS (verbatim, terse form — `"M-x"`, never a display
/// glyph) that dispatch `action` under `convention`+`flavor`, straight off
/// [`active_seed_tables`]. Empty when no seed layer is active, or none of its
/// entries target this action. Entries from both active tables are retained in
/// table-then-entry order; the tables currently target distinct actions.
pub(crate) fn seeded_chords_for(
    action: &Action,
    convention: Convention,
    flavor: KeymapFlavor,
) -> Vec<&'static str> {
    active_seed_tables(convention, flavor)
        .iter()
        .flat_map(|table| table.iter())
        .filter(|(_, a)| a == action)
        .map(|(spec, _)| *spec)
        .collect()
}

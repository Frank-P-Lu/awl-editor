use anyhow::{Result, bail};
use winit::event::Modifiers;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

use crate::keymap::{Action, KeymapState};

#[derive(Clone, Debug, PartialEq)]
pub struct Chord {
    pub spec: String,
    pub key: Key,
    pub mods: Modifiers,
}

/// Parse a whole `--keys` spec into its CHORD stream — STRUCTURAL validation
/// only (modifier prefixes + key tokens; an unrecognized token is a clear
/// `anyhow::Error`, never a panic). No keymap is consulted here: resolution
/// happens chord-by-chord inside the replay loop (see [`ChordResolver`]),
/// interleaved with the search guard, exactly like live key dispatch.
pub fn parse_chords(spec: &str) -> Result<Vec<Chord>> {
    spec.split_whitespace()
        .map(|tok| {
            let (key, mods) = parse_chord(tok)?;
            Ok(Chord {
                spec: tok.to_string(),
                key,
                mods,
            })
        })
        .collect()
}

/// The stateful chord→action resolver the headless replay drives: ONE
/// persistent `KeymapState` across the whole stream (so `C-x` prefix sequences
/// and config rebinds compose exactly as live), plus the STRICT refusals —
/// which moved HERE from parse time when the search guard made resolution
/// replay-state-dependent (a chord the open search panel consumes — `M-c`, a
/// bare `C-x` — never reaches the keymap and must not be judged "unbound").
/// Two action kinds are dropped (`Ok(None)`) because they carry no work for
/// the apply seam: `Ignore` (unbound combos, permissive's silent drop) and
/// `BeginPrefix` (the first half of a `C-x` sequence, whose only job is to
/// flip the keymap's prefix state). Under STRICT, an unbound chord or a
/// non-cancel chord dangling off an armed prefix is an error NAMING that
/// exact chord (`Esc`/`C-g` stay a legal explicit prefix-cancel), so a typo'd
/// scenario aborts instead of replaying a fiction.
pub struct ChordResolver<'a> {
    km: &'a mut KeymapState,
    strict: bool,
    pending_prefix: Option<String>,
}

impl<'a> ChordResolver<'a> {
    pub fn new(km: &'a mut KeymapState, strict: bool) -> Self {
        Self {
            km,
            strict,
            pending_prefix: None,
        }
    }

    pub fn resolve(&mut self, chord: &Chord) -> Result<Option<Action>> {
        let action = self.km.resolve(&chord.key, &chord.mods);
        if self.strict {
            if matches!(action, Action::Ignore) {
                bail!(
                    "strict replay: chord {:?} is unbound (resolves to no action)",
                    chord.spec
                );
            }
            if let Some(pfx) = &self.pending_prefix
                && matches!(action, Action::Cancel)
                && !is_explicit_cancel(&chord.key, chord.mods.state())
            {
                bail!(
                    "strict replay: chord {:?} does not complete the {pfx:?} prefix (unbound sequence)",
                    chord.spec
                );
            }
        }
        self.pending_prefix = matches!(action, Action::BeginPrefix).then(|| chord.spec.clone());
        Ok((!matches!(action, Action::Ignore | Action::BeginPrefix)).then_some(action))
    }
}

#[allow(dead_code)]
pub fn parse_keys(spec: &str) -> Result<Vec<Action>> {
    parse_keys_through(spec, KeymapState::new())
}

/// TEST-ONLY: like [`parse_keys`], but resolve through a keymap PINNED to
/// `convention` rather than the ambient [`crate::convention::Convention::current`]
/// — so a test with a hardcoded MAC-literal spec (`"Cmd-S-h"`, `"s-p"`, a bare
/// `"C-n"`/`"C-x"` whose letter Linux's collision table displaces, …) stays
/// CONVENTION-PROOF: it resolves identically regardless of which convention
/// happens to be ambient when the test runs (a dev Mac vs. CI's linux runner —
/// see `keymap.rs`'s collision-table doc for the displacement this sidesteps).
/// Every real (non-test) caller — the actual `--keys` CLI replay — correctly
/// wants the ambient convention via [`parse_keys`] / the replay keymap; this
/// pinned sibling exists only so hardcoded-literal unit tests can say exactly
/// which convention they mean.
#[cfg(test)]
pub(crate) fn parse_keys_pinned(
    spec: &str,
    convention: crate::convention::Convention,
) -> Result<Vec<Action>> {
    parse_keys_through(spec, KeymapState::new_with_convention(convention))
}

/// Shared core: resolve every chord in `spec` through one persistent `km` (so C-x
/// prefix state and any config rebinds compose across the sequence).
fn parse_keys_through(spec: &str, km: KeymapState) -> Result<Vec<Action>> {
    parse_keys_mode(spec, km, false)
}

fn parse_keys_mode(spec: &str, mut km: KeymapState, strict: bool) -> Result<Vec<Action>> {
    let chords = parse_chords(spec)?;
    let mut resolver = ChordResolver::new(&mut km, strict);
    let mut actions = Vec::new();
    for chord in &chords {
        if let Some(action) = resolver.resolve(chord)? {
            actions.push(action);
        }
    }
    Ok(actions)
}

fn is_explicit_cancel(key: &Key, mods: ModifiersState) -> bool {
    match key {
        Key::Named(NamedKey::Escape) => true,
        Key::Character(s) => {
            s.as_str().eq_ignore_ascii_case("g") && mods.contains(ModifiersState::CONTROL)
        }
        _ => false,
    }
}

/// WORD-form modifier prefixes (case-insensitive), an alternative to the terse
/// single-letter `C-`/`M-`/`S-`/`s-` spellings so a macOS-native binding reads as
/// `Cmd-S` / `Option-f`. Each entry carries its trailing '-'; the longest natural
/// spellings come first but matching is unambiguous (each begins distinctly).
const WORD_MODS: &[(&str, ModifiersState)] = &[
    ("cmd-", ModifiersState::SUPER),
    ("super-", ModifiersState::SUPER),
    ("ctrl-", ModifiersState::CONTROL),
    ("control-", ModifiersState::CONTROL),
    ("option-", ModifiersState::ALT),
    ("opt-", ModifiersState::ALT),
    ("alt-", ModifiersState::ALT),
    ("meta-", ModifiersState::ALT),
    ("shift-", ModifiersState::SHIFT),
];

/// Strip leading modifier prefixes shared by [`parse_chord`] and
/// [`parse_pointer_chord`] — the SAME two spellings, greedily and
/// order-independently (modifiers are just bitflags): the terse single-letter
/// "<m>-" form (`C-`, `M-`, `S-`, `s-`) AND the macOS-friendly WORD form
/// (`Cmd-`, `Option-`, ...). The word form is tried first so `Cmd-S` reads as
/// Super+`S` rather than as the literal letters. A bare "-" (or a 1-char
/// remainder) is never consumed as a prefix, so the trailing TOKEN always
/// survives — a key for [`parse_chord`], a `click`/`middle-click`/`right-click`
/// word for [`parse_pointer_chord`]. Only the trailing token's own grammar
/// differs between the two callers; the modifier vocabulary is one owner.
fn strip_modifier_prefixes(chord: &str) -> (ModifiersState, &str) {
    let mut rest = chord;
    let mut state = ModifiersState::empty();
    loop {
        if let Some((pfx, flag)) = WORD_MODS.iter().find(|(pfx, _)| {
            rest.len() > pfx.len()
                && rest
                    .get(..pfx.len())
                    .is_some_and(|h| h.eq_ignore_ascii_case(pfx))
        }) {
            state |= *flag;
            rest = &rest[pfx.len()..];
            continue;
        }
        let bytes = rest.as_bytes();
        if rest.len() >= 2 && bytes[1] == b'-' {
            let flag = match bytes[0] {
                b'C' => Some(ModifiersState::CONTROL),
                b'M' => Some(ModifiersState::ALT), // Meta == Alt (Option on mac)
                b'S' => Some(ModifiersState::SHIFT),
                b's' => Some(ModifiersState::SUPER), // Super == Cmd
                _ => None,
            };
            if let Some(f) = flag {
                state |= f;
                rest = &rest[2..];
                continue;
            }
        }
        break;
    }
    (state, rest)
}

pub fn parse_chord(chord: &str) -> Result<(Key, Modifiers)> {
    let (state, rest) = strip_modifier_prefixes(chord);

    if rest.is_empty() {
        bail!("empty key in chord {chord:?}");
    }

    let key = parse_key_token(rest, chord)?;
    Ok((key, Modifiers::from(state)))
}

/// THE MOUSE CHORD GRAMMAR (a deliberate SIBLING to [`parse_chord`], never a
/// branch inside it): `[keys] follow = "…"` rebinds the follow gesture roster
/// (`keymap::platform::active_follow_gestures`) through this parser, which
/// spells exactly three trailing tokens — `click` (the primary button),
/// `middle-click`, `right-click` — behind the SAME modifier prefixes a key
/// chord accepts ([`strip_modifier_prefixes`], terse `C-`/`M-`/`S-`/`s-` or
/// the word form `Cmd-`/`Option-`/…). Kept structurally separate from
/// [`parse_chord`]/[`canonical_binding`] on purpose: the Linux keep-list is
/// composed and compared through THOSE two, so a mouse chord having no
/// spelling there is what keeps it forever uncollidable with the `C-c`/`C-x`
/// rules (`keymap::tests::the_linux_keep_list_holds_only_key_chords_so_no_mouse_chord_can_collide`).
/// Case-insensitive on the trailing word, matching every other token this
/// grammar's sibling already treats case-insensitively.
pub fn parse_pointer_chord(spec: &str) -> Result<(crate::keymap::PointerButton, ModifiersState)> {
    let (state, rest) = strip_modifier_prefixes(spec);
    let button = match rest.to_ascii_lowercase().as_str() {
        "click" => crate::keymap::PointerButton::Primary,
        "middle-click" => crate::keymap::PointerButton::Middle,
        "right-click" => crate::keymap::PointerButton::Secondary,
        _ => bail!(
            "unrecognized pointer gesture {rest:?} in {spec:?} \
             (expected click, middle-click, or right-click)"
        ),
    };
    Ok((button, state))
}

/// Render a parsed pointer chord back to its CANONICAL spelling — the terse
/// modifier letters [`format_chord`] already uses, then the button word. The
/// one owner every override-derived [`crate::keymap::platform::FollowGesture`]
/// label reads (`Box::leak`, the SAME one-time-per-config-load cost
/// `commands::COMMANDS`'s own splice already pays), so a user's `C-click` and
/// `Ctrl-Click` compose to the same reported label without a second table.
pub fn format_pointer_chord(button: crate::keymap::PointerButton, mods: ModifiersState) -> String {
    let mut s = String::new();
    if mods.contains(ModifiersState::CONTROL) {
        s.push_str("C-");
    }
    if mods.contains(ModifiersState::ALT) {
        s.push_str("M-");
    }
    if mods.contains(ModifiersState::SHIFT) {
        s.push_str("S-");
    }
    if mods.contains(ModifiersState::SUPER) {
        s.push_str("s-");
    }
    s.push_str(match button {
        crate::keymap::PointerButton::Primary => "Click",
        crate::keymap::PointerButton::Middle => "Middle-Click",
        crate::keymap::PointerButton::Secondary => "Right-Click",
    });
    s
}

pub fn format_chord(key: &Key, mods: ModifiersState) -> String {
    let mut s = String::new();
    if mods.contains(ModifiersState::CONTROL) {
        s.push_str("C-");
    }
    if mods.contains(ModifiersState::ALT) {
        s.push_str("M-");
    }
    if mods.contains(ModifiersState::SHIFT) {
        s.push_str("S-");
    }
    if mods.contains(ModifiersState::SUPER) {
        s.push_str("s-");
    }
    s.push_str(&key_token(key));
    s
}

/// Format a chord spec with mac MODIFIER GLYPHS concatenated, no dashes — the
/// NATIVE (macOS) slot's display form: Cmd→⌘ (U+2318), Shift→⇧ (U+21E7),
/// Option/Meta→⌥ (U+2325), Ctrl→⌃ (U+2303), then the key (single letters
/// upper-cased so `Cmd-S` reads `⌘S`). `"Cmd-S-o"` → `"⌘⇧O"`, `"Cmd-F"` → `"⌘F"`,
/// `"Cmd-M-f"` → `"⌘⌥F"`. A whitespace-separated SEQUENCE formats each chord and
/// re-joins with a space; a token that fails to parse passes through verbatim. The
/// EMACS slot does NOT use this — it keeps its terse `C-`/`M-` text.
pub fn mac_glyph_chord(spec: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for tok in spec.split_whitespace() {
        match parse_chord(tok) {
            Ok((key, mods)) => out.push(mac_glyph_token(&key, mods.state())),
            Err(_) => out.push(tok.to_string()),
        }
    }
    out.join(" ")
}

fn mac_glyph_token(key: &Key, mods: ModifiersState) -> String {
    let mut s = String::new();
    if mods.contains(ModifiersState::SUPER) {
        s.push('\u{2318}'); // ⌘ Command
    }
    if mods.contains(ModifiersState::SHIFT) {
        s.push('\u{21E7}'); // ⇧ Shift
    }
    if mods.contains(ModifiersState::ALT) {
        s.push('\u{2325}'); // ⌥ Option
    }
    if mods.contains(ModifiersState::CONTROL) {
        s.push('\u{2303}'); // ⌃ Control
    }
    s.push_str(&mac_key_token(key));
    s
}

fn mac_key_token(key: &Key) -> String {
    if let Key::Character(s) = key {
        let mut chars = s.chars();
        if let (Some(c), None) = (chars.next(), chars.next())
            && c.is_ascii_alphabetic()
        {
            return c.to_ascii_uppercase().to_string();
        }
    }
    key_token(key)
}

fn key_token(key: &Key) -> String {
    match key {
        Key::Named(named) => match named {
            NamedKey::ArrowLeft => "Left",
            NamedKey::ArrowRight => "Right",
            NamedKey::ArrowUp => "Up",
            NamedKey::ArrowDown => "Down",
            NamedKey::Home => "Home",
            NamedKey::End => "End",
            NamedKey::PageUp => "PageUp",
            NamedKey::PageDown => "PageDown",
            NamedKey::Enter => "Enter",
            NamedKey::Tab => "Tab",
            NamedKey::Backspace => "Backspace",
            NamedKey::Delete => "Delete",
            NamedKey::Space => "Space",
            NamedKey::Escape => "Esc",
            _ => "?",
        }
        .to_string(),
        Key::Character(s) => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if c.is_ascii_alphabetic() => c.to_ascii_lowercase().to_string(),
                _ => s.to_string(),
            }
        }
        _ => String::new(),
    }
}

/// LITERAL TEXT → its chord stream, for a storyboard's `type` step: each char
/// becomes exactly the chord a `--keys` spec would spell for it (a bare
/// printable self-insert; whitespace via the NAMED keys — `Space` / `Enter` /
/// `Tab` — since a spec token cannot hold a literal space). Routed through
/// [`parse_chord`] so the token→key mapping has ONE owner and a typed char is
/// byte-for-byte the chord the replay loop already understands (the search
/// guard consumes it while the panel is open, the keymap self-inserts it
/// otherwise). A char `parse_chord` cannot express (none known — every single
/// char is a legal token) surfaces as its clear error rather than a skip.
pub fn text_chords(text: &str) -> Result<Vec<Chord>> {
    text.chars()
        .map(|ch| {
            let tok = match ch {
                ' ' => "Space".to_string(),
                '\n' => "Enter".to_string(),
                '\t' => "Tab".to_string(),
                c => c.to_string(),
            };
            let (key, mods) = parse_chord(&tok)?;
            Ok(Chord {
                spec: tok,
                key,
                mods,
            })
        })
        .collect()
}

/// NAIVE Mac→Linux chord TRANSLATION: swap SUPER for CONTROL in every token's
/// modifiers (leaving ALT/SHIFT untouched), re-emitting the terse canonical form.
/// This is the DEFAULT half of the convention-resolution data design (see
/// `commands::resolved_native`'s doc for the full story): most native chords are
/// a plain Cmd→Ctrl swap (`Cmd-S` → `C-s`), so a per-command override table only
/// needs entries for the handful where that swap is WRONG (word motion, line/doc
/// start-end — see `commands::LINUX_NATIVE_OVERRIDE`). A token that fails to parse
/// passes through verbatim (never panics), mirroring [`mac_glyph_chord`]'s own
/// tolerance. Pure — no convention/global read; the caller decides WHEN to use it.
pub fn translate_native_for_linux(mac_spec: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for tok in mac_spec.split_whitespace() {
        match parse_chord(tok) {
            Ok((key, mods)) => {
                let mut state = mods.state();
                if state.contains(ModifiersState::SUPER) {
                    state.remove(ModifiersState::SUPER);
                    state.insert(ModifiersState::CONTROL);
                }
                out.push(format_chord(&key, state));
            }
            Err(_) => out.push(tok.to_string()),
        }
    }
    out.join(" ")
}

/// Format a chord spec as a LINUX/GTK-style label — `"Ctrl+Shift+P"`, modifiers
/// joined with `+` (Ctrl, Alt, Shift, Super — in that fixed order, matching how
/// GNOME/GTK apps present accelerators) and the key upper-cased for a single
/// letter, mirroring [`mac_glyph_chord`]'s structure but with WORD labels instead
/// of Apple's modifier glyphs (Linux/GTK conventionally has none). A token that
/// fails to parse passes through verbatim. This is the LINUX-convention sibling of
/// [`mac_glyph_chord`] — the two are the only doors [`crate::commands`]'s resolved
/// label owner calls.
pub fn linux_glyph_chord(spec: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for tok in spec.split_whitespace() {
        match parse_chord(tok) {
            Ok((key, mods)) => out.push(linux_glyph_token(&key, mods.state())),
            Err(_) => out.push(tok.to_string()),
        }
    }
    out.join(" ")
}

/// THE UNDO CHORD, AS PROSE THE USER READS. `Undo` has no palette command and so
/// no [`crate::commands`] row to resolve a label from — but a sentence telling
/// someone how to take a change back must name a key that fires on the convention
/// they are on. `keyspec::tests::the_taught_undo_chord_is_the_one_that_fires`
/// proves these two specs do, through the real keymap.
pub fn undo_chord_label() -> String {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => mac_glyph_chord(UNDO_SPEC_MAC),
        crate::convention::Convention::Linux => linux_glyph_chord(UNDO_SPEC_LINUX),
    }
}

/// `Undo`'s default chord per convention — the spec [`undo_chord_label`] renders
/// and the law resolves.
pub(crate) const UNDO_SPEC_MAC: &str = "s-z";
pub(crate) const UNDO_SPEC_LINUX: &str = "C-/";

/// THE FIND/REPLACE PANEL'S OWN FIXED CHORDS, AS PROSE THE USER READS.
///
/// Every one of these is consumed directly by `search::keys::intercept` while
/// the panel is up — a raw-key interception with no catalog row and no
/// `[keys]` rebinding door, the same shape [`undo_chord_label`] already solved
/// for `Undo`. A doc string baked into `render/chrome/panel.rs` as a literal
/// `"\u{2318}\u{2325}c case"` is wrong the instant it is read on Linux (bare
/// `Alt-c` there, never `Super+Alt+c` — a Linux hand has no reason to hold a
/// Super/Windows key for this), so each fixed chord gets a mac/linux SPEC pair
/// resolved through the SAME glyph renderer every catalog chord uses
/// (`mac_glyph_chord`/`linux_glyph_chord`), gated on [`crate::convention::Convention::current`]
/// exactly like the undo label above. `panel_chords::tests` (in
/// `render/chrome/panel.rs`) proves each spec is the chord that actually fires,
/// through `search::keys::intercept` directly (these never reach the keymap, so
/// `KeymapState::resolve` cannot see them).
pub(crate) struct PanelChordSpec {
    pub mac: &'static str,
    pub linux: &'static str,
}

impl PanelChordSpec {
    /// The label for the RUNNING build's own convention — what the panel
    /// actually draws.
    pub(crate) fn label(&self) -> String {
        self.label_for(crate::convention::Convention::current())
    }

    /// The label for an EXPLICIT convention, so a law can compare both
    /// without mutating the process-global `Convention::current()`.
    pub(crate) fn label_for(&self, convention: crate::convention::Convention) -> String {
        match convention {
            crate::convention::Convention::Mac => mac_glyph_chord(self.mac),
            crate::convention::Convention::Linux => linux_glyph_chord(self.linux),
        }
    }
}

/// Enter, unmodified: replace the current match and advance (or, with no
/// replace row up, accept and close).
pub(crate) const PANEL_REPLACE_NEXT: PanelChordSpec = PanelChordSpec {
    mac: "Enter",
    linux: "Enter",
};
/// Super-Enter: replace every match. The SAME modifier slot on both
/// conventions (this app's one Cmd/Super slot, not a Linux-specific
/// Ctrl-Enter alternate) — `search::keys::intercept_character`'s Enter arm
/// gates on `ModifiersState::SUPER` alone, so the physical key is Cmd on mac
/// and the Super/Windows key on Linux.
pub(crate) const PANEL_REPLACE_ALL: PanelChordSpec = PanelChordSpec {
    mac: "s-Enter",
    linux: "s-Enter",
};
/// Tab, unmodified: switch focus between the find and replace fields.
pub(crate) const PANEL_SWITCH_FIELD: PanelChordSpec = PanelChordSpec {
    mac: "Tab",
    linux: "Tab",
};
/// Esc, unmodified: close the panel.
pub(crate) const PANEL_CLOSE: PanelChordSpec = PanelChordSpec {
    mac: "Esc",
    linux: "Esc",
};
/// Case-sensitivity toggle: a genuinely DIFFERENT spec per convention, not
/// just a different render of one spec — Option-c alone composes to 'ç' on
/// macOS (see `search::keys::intercept_character`'s own doc), so the
/// mac-reachable door holds Super down too; the Linux door is bare Alt-c.
pub(crate) const PANEL_MATCH_CASE: PanelChordSpec = PanelChordSpec {
    mac: "s-M-c",
    linux: "M-c",
};

/// One chord as a Linux/GTK label: `"Ctrl+Shift+P"`. Helper for [`linux_glyph_chord`].
fn linux_glyph_token(key: &Key, mods: ModifiersState) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if mods.contains(ModifiersState::CONTROL) {
        parts.push("Ctrl");
    }
    if mods.contains(ModifiersState::ALT) {
        parts.push("Alt");
    }
    if mods.contains(ModifiersState::SHIFT) {
        parts.push("Shift");
    }
    if mods.contains(ModifiersState::SUPER) {
        parts.push("Super");
    }
    let mut s = parts.join("+");
    if !s.is_empty() {
        s.push('+');
    }
    s.push_str(&mac_key_token(key)); // same "single letter upper-cased" rule
    s
}

pub fn canonical_binding(spec: &str) -> Option<String> {
    let mut out: Vec<String> = Vec::new();
    for tok in spec.split_whitespace() {
        let (key, mods) = parse_chord(tok).ok()?;
        out.push(format_chord(&key, mods.state()));
    }
    if out.is_empty() {
        return None;
    }
    Some(out.join(" "))
}

/// Map the key token (after modifier stripping) to a winit `Key`. Named keys are
/// matched case-insensitively against a fixed table; otherwise the token must be
/// exactly one character, passed through verbatim as a `Character`.
fn parse_key_token(tok: &str, chord: &str) -> Result<Key> {
    if let Some(named) = named_key(tok) {
        return Ok(Key::Named(named));
    }
    if tok.chars().count() != 1 {
        bail!("unrecognized key {tok:?} in chord {chord:?} (not a named key or single char)");
    }
    Ok(Key::Character(SmolStr::new(tok)))
}

fn named_key(tok: &str) -> Option<NamedKey> {
    let lower = tok.to_ascii_lowercase();
    Some(match lower.as_str() {
        "left" => NamedKey::ArrowLeft,
        "right" => NamedKey::ArrowRight,
        "up" => NamedKey::ArrowUp,
        "down" => NamedKey::ArrowDown,
        "home" => NamedKey::Home,
        "end" => NamedKey::End,
        "pageup" | "pgup" => NamedKey::PageUp,
        "pagedown" | "pgdn" | "pagedn" => NamedKey::PageDown,
        "enter" | "return" | "ret" => NamedKey::Enter,
        "tab" => NamedKey::Tab,
        "backspace" | "del" => NamedKey::Backspace,
        "delete" => NamedKey::Delete,
        "space" | "spc" => NamedKey::Space,
        "esc" | "escape" => NamedKey::Escape,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;

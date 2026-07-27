//! Hold-modifier shortcut peek state. It is cancelled before any real chord runs.
//!
//! **Determinism:** the peek is a HELD, clocked, live-only surface exactly like the
//! stats HUD. The live App pushes the personalized [`PeekRow`]s (its ledger's
//! graduation candidates) into the pipeline every `sync_view`; a headless capture
//! never does, so the pipeline's rows stay empty and the pure [`starter_rows`]
//! fallback renders — the `--peek` capture flag summons the SETTLED card showing the
//! curated STARTER SIX, byte-stable across machines. A default capture (not summoned)
//! draws nothing and is byte-identical.

/// How long the active convention's bare ARMING modifier ([`arming_modifier`] — ⌘ on
/// Mac, Ctrl on Linux) must be held — alone and uninterrupted — before the shortcut
/// peek summons. A TASTE constant (flagged for live tuning, named like
/// `THEME_FONT_DEBOUNCE` / `HOLD_PEEK_MS`): ~600ms is long enough that a fast native
/// chord (⌘S/⌘⇧P on Mac; `C-f`/`C-s`/… on Linux, where the SAME modifier also carries
/// the emacs nav layer) is always a press-and-release well under it (never flickering
/// the card), short enough that a deliberate "what were the shortcuts again?" pause
/// lands it promptly. LIVE-ONLY feel.
pub const HOLD_PEEK_MS: u64 = 600;

#[cfg(not(target_arch = "wasm32"))]
pub const PEEK_ROWS: usize = 6;

/// Whether the shortcut-peek card is drawn. DEFAULT OFF: the calm room shows no card
/// until the active convention's bare arming modifier ([`arming_modifier`]) is held
/// past [`HOLD_PEEK_MS`] (the live gesture) or the `--peek` capture flag forces it.
/// The shared summoned-card flag mechanism (see [`crate::card::CardFlag`]) — the peek
/// shares the FLAG, but not the modal any-key dismiss (it closes when the hold
/// breaks, via [`PeekArm`]).
static PEEK: crate::card::CardFlag = crate::card::CardFlag::new();

pub fn peek_open() -> bool {
    PEEK.is_open()
}

/// Open or close the card explicitly. The live App calls this from [`PeekArm`]'s
/// transitions (open on the hold completing, close on any cancellation); the `--peek`
/// flag passes `true` for a settled capture.
pub fn set_open(open: bool) {
    PEEK.set_open(open);
}

/// ONE peek row: a chord GLYPH figure (`"⌘O"`) over/beside its command NAME caption
/// (`"Go to file"`). The type system's ink × size — the chord rides content ink at
/// BODY size (the figure), the name faint ink at LABEL size (the caption) — NEVER amber
/// (the caret's alone). Owned strings so the App can hand personalized rows across the
/// pipeline seam by value, exactly like `HudStats`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeekRow {
    /// The macOS modifier-glyph chord, e.g. `"⌘O"` (`keyspec::mac_glyph_chord`).
    pub chord: String,
    pub name: String,
}

/// The curated STARTER SIX — the shortcuts a fresh install shows before the ledger has
/// learned anything, so the card is NEVER empty. The four IDENTITY-ROUND doors
/// (⌘O go-to-file, ⌘⇧P switch-project, ⌘P command-palette — this last one is NOT a
/// catalog command, hence the hand-spelled spec below rather than a catalog lookup —
/// ⌘F find) plus ⌘S save and ⌘T switch-theme: the six a new user most wants in their
/// fingers. Kept as (mac-flavored chord-spec, name) pairs; NONE of the six needs
/// `commands::LINUX_NATIVE_OVERRIDE`'s exceptions (that table only covers line/doc
/// start-end and word motion), so the naive Cmd→Ctrl translation
/// (`keyspec::translate_native_for_linux`) is correct here without consulting it.
const STARTER: &[(&str, &str)] = &[
    ("Cmd-O", "Go to file"),
    ("Cmd-S-p", "Switch project"),
    ("Cmd-P", "Command palette"),
    ("Cmd-F", "Find"),
    ("Cmd-S", "Save"),
    ("Cmd-T", "Switch theme"),
];

/// The curated STARTER SIX as [`PeekRow`]s (chords resolved + glyphified per the
/// ACTIVE convention). The pure fallback shown when the ledger has no personalized
/// rows yet — on a fresh install (live) AND in a headless `--peek` capture (no live
/// ledger), so the two agree byte-for-byte. [`starter_rows_for`] is the explicit-
/// convention sibling a test pins directly.
pub fn starter_rows() -> Vec<PeekRow> {
    starter_rows_for(crate::convention::Convention::current())
}

pub fn starter_rows_for(convention: crate::convention::Convention) -> Vec<PeekRow> {
    STARTER
        .iter()
        .map(|(spec, name)| {
            let chord = match convention {
                crate::convention::Convention::Mac => crate::keyspec::mac_glyph_chord(spec),
                crate::convention::Convention::Linux => crate::keyspec::linux_glyph_chord(
                    &crate::keyspec::translate_native_for_linux(spec),
                ),
            };
            PeekRow {
                chord,
                name: name.to_string(),
            }
        })
        .collect()
}

/// The rows the peek card actually renders: the App-pushed personalized `rows` when it
/// has any, else the curated [`starter_rows`]. ONE owner of the empty→starter fallback,
/// shared by the pixels (`render/chrome/hud.rs`) and the sidecar (`capture/sidecar.rs`)
/// so they can never disagree. An empty slice (a capture never pushed, or a fresh-install
/// ledger with no candidates) folds to the starter six.
pub fn rows_or_starter(rows: &[PeekRow]) -> Vec<PeekRow> {
    if rows.is_empty() {
        starter_rows()
    } else {
        rows.to_vec()
    }
}

/// Which SINGLE modifier ARMS the hold-⌘ shortcut peek, resolved per the active
/// keyboard [`crate::convention::Convention`] — the ONE owner every arm/cancel call
/// site reads (the live App's `ModifiersChanged` wiring,
/// `app/input/keys.rs::on_modifiers_changed`), so the mapping can never scatter into
/// separate checks. [`crate::convention::Convention::Mac`] arms on bare ⌘ (Super) —
/// the gesture every macOS user already makes hunting for a shortcut, byte-identical
/// to the pre-convention-round behavior. [`crate::convention::Convention::Linux`]
/// arms on bare Ctrl instead: Super belongs to the COMPOSITOR there (window-manager
/// gestures — e.g. Omarchy/Hyprland's own Super-driven bindings), so arming on it
/// would pop the card uninvited mid-WM-gesture; Ctrl is also the modifier the native
/// chords THIS CARD TEACHES already live on under `Convention::Linux` (see
/// [`starter_rows`] / `commands::resolved_native`), so holding it to "peek" is the
/// exact same gesture as Mac's ⌘, just on the modifier that convention's chords
/// actually use.
pub fn arming_modifier(
    convention: crate::convention::Convention,
) -> winit::keyboard::ModifiersState {
    match convention {
        crate::convention::Convention::Mac => winit::keyboard::ModifiersState::SUPER,
        crate::convention::Convention::Linux => winit::keyboard::ModifiersState::CONTROL,
    }
}

/// Is `mods` EXACTLY [`arming_modifier`]`(convention)` alone — the only state that
/// ARMS the hold-⌘ shortcut peek? Any other modifier state (the arming modifier plus
/// another, or none of it at all) is either the start of a real chord or no hold in
/// progress, so it never arms. Pure — the live App calls this with
/// [`crate::convention::Convention::current`]; a test pins either convention
/// explicitly (mirrors `KeymapState::new_with_convention`), so the arming law is
/// verifiable without depending on the ambient build target or `AWL_CONVENTION_FORCE`.
pub fn is_bare_arming_modifier(
    mods: winit::keyboard::ModifiersState,
    convention: crate::convention::Convention,
) -> bool {
    mods == arming_modifier(convention)
}

/// Whether the shortcut peek may ARM or STAY OPEN, given whether a ZOOM gesture is
/// currently IN FLIGHT (the sticky-zoom debounce window — `App::zoom_in_flight`,
/// backed by `zoom_persist_at` — is open). Zoom is the one gesture where the user
/// holds the arming modifier (⌘ on Mac / Ctrl on Linux) PRECISELY to change what they
/// are looking at: the card and its frosted backdrop would obscure exactly the text
/// being resized to read. So a zoom in flight SUPPRESSES the peek — a bare-modifier
/// hold never arms, and a pending/open card is put down — and it may RE-ARM only once
/// the zoom SETTLES (the debounce clears, `zoom_in_flight` falls back to `false`).
/// Pure (gesture state → panel visibility), so the whole suppression decision is
/// unit-testable without a clock or window; the live App consults it at the arming
/// seam (`on_modifiers_changed`) and the summon seam (`about_to_wait`'s peek timer),
/// and dismisses an already-open card the instant a zoom step lands (a wheel-zoom
/// notch feeds [`PeekStimulus::Interrupt`]).
pub fn peek_allowed(zoom_in_flight: bool) -> bool {
    !zoom_in_flight
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PeekArm {
    #[default]
    Idle,
    /// The convention's bare arming modifier went down alone (⌘ on Mac, Ctrl on
    /// Linux — [`arming_modifier`]); the hold timer is running. When it elapses
    /// (still `Pending`), the card summons.
    Pending,
    Open,
}

/// A stimulus fed to the [`PeekArm`] machine. The live App maps its raw input events to
/// these: `ModifiersChanged` → [`ArmAlone`]/[`ArmBroken`] (whether the mods are exactly
/// the convention's bare arming modifier — [`is_bare_arming_modifier`] /
/// [`arming_modifier`]: ⌘ on Mac, Ctrl on Linux), a key press past the lone-modifier
/// filter → [`KeyJoined`], a mouse press / focus loss → [`Interrupt`], and the
/// hold-timer deadline → [`Elapsed`].
///
/// [`ArmAlone`]: PeekStimulus::ArmAlone
/// [`ArmBroken`]: PeekStimulus::ArmBroken
/// [`KeyJoined`]: PeekStimulus::KeyJoined
/// [`Interrupt`]: PeekStimulus::Interrupt
/// [`Elapsed`]: PeekStimulus::Elapsed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeekStimulus {
    /// Modifiers became EXACTLY the convention's bare arming modifier alone (⌘ alone
    /// on Mac, Ctrl alone on Linux) — the only state that arms.
    ArmAlone,
    ArmBroken,
    /// A non-modifier key press joined the arming modifier (⌘S, ⌘⇧P's letter, Cmd-I on
    /// Mac; `C-f`, `C-s`, … on Linux) — a real chord is forming, so the peek
    /// cancels/closes instantly. THE CRUX: this is what keeps a native chord — ⌘S on
    /// Mac, or a Ctrl-chord on Linux (where the emacs nav layer AND the Linux-native
    /// layer both live on the SAME arming modifier) — from ever flickering the card.
    KeyJoined,
    Interrupt,
    Elapsed,
}

impl PeekArm {
    pub fn next(self, stim: PeekStimulus) -> PeekArm {
        use PeekArm::*;
        use PeekStimulus::*;
        match (self, stim) {
            (Idle, ArmAlone) => Pending,
            (Idle, _) => Idle,
            (Pending, Elapsed) => Open,
            (Pending, ArmAlone) => Pending,
            (Pending, ArmBroken | KeyJoined | Interrupt) => Idle,
            (Open, ArmAlone | Elapsed) => Open,
            (Open, ArmBroken | KeyJoined | Interrupt) => Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PeekArm::*;
    use PeekStimulus::*;

    #[test]
    fn defaults_closed() {
        let _g = crate::testlock::serial();
        set_open(false);
        assert!(!peek_open(), "the shortcut-peek card is closed by default");
    }

    #[test]
    fn set_open_drives_the_flag() {
        let _g = crate::testlock::serial();
        set_open(false);
        set_open(true);
        assert!(peek_open());
        set_open(false);
        assert!(!peek_open());
    }

    #[test]
    fn bare_arm_modifier_arms_then_timer_opens() {
        assert_eq!(Idle.next(ArmAlone), Pending);
        assert_eq!(Pending.next(Elapsed), Open);
        assert_eq!(Pending.next(ArmAlone), Pending);
        assert_eq!(Open.next(ArmAlone), Open);
        assert_eq!(Open.next(Elapsed), Open);
    }

    #[test]
    fn a_second_key_kills_a_pending_peek() {
        // THE CRUX: the arming modifier down (Pending), then a letter joins → the
        // peek is cancelled before it can ever flicker. A native chord — ⌘S on Mac,
        // C-f/C-s on Linux (where the SAME Ctrl modifier also carries the emacs nav
        // layer) — must feel like a plain chord, never a flickered card.
        let armed = Idle.next(ArmAlone);
        assert_eq!(armed, Pending);
        assert_eq!(
            armed.next(KeyJoined),
            Idle,
            "a joined key kills the pending peek"
        );
    }

    #[test]
    fn releasing_the_arm_modifier_before_the_threshold_shows_nothing() {
        // Press the arming modifier then release it before the hold elapses:
        // Pending → Idle, the card never opened.
        let armed = Idle.next(ArmAlone);
        assert_eq!(
            armed.next(ArmBroken),
            Idle,
            "release before threshold = nothing"
        );
    }

    #[test]
    fn a_click_or_blur_cancels_pending_and_closes_open() {
        assert_eq!(Pending.next(Interrupt), Idle);
        assert_eq!(Open.next(Interrupt), Idle);
    }

    #[test]
    fn an_open_peek_closes_when_the_arm_modifier_lifts() {
        assert_eq!(Open.next(ArmBroken), Idle);
        assert_eq!(Open.next(KeyJoined), Idle);
    }

    #[test]
    fn peek_allowed_only_when_no_zoom_in_flight() {
        assert!(
            peek_allowed(false),
            "no zoom in flight → the peek may arm/stay open"
        );
        assert!(!peek_allowed(true), "a zoom in flight suppresses the peek");
    }

    #[test]
    fn zoom_in_flight_gates_an_arm_into_idle_via_arm_broken() {
        // The live seam turns a bare-arming-modifier ModifiersChanged into `ArmAlone`
        // only when `peek_allowed(zoom_in_flight)`; else it feeds `ArmBroken`. Model
        // both edges purely: with a zoom in flight the arm never reaches `Pending`, and
        // a card already up is put down.
        let zoom_in_flight = true;
        let stim = if peek_allowed(zoom_in_flight) {
            ArmAlone
        } else {
            ArmBroken
        };
        assert_eq!(
            stim, ArmBroken,
            "a zoom in flight downgrades the arm to a cancel"
        );
        assert_eq!(Idle.next(stim), Idle, "so a fresh hold never arms mid-zoom");
        assert_eq!(Pending.next(stim), Idle, "and a pending hold is cancelled");
        assert_eq!(Open.next(stim), Idle, "and an open card is closed");
        let settled = if peek_allowed(false) {
            ArmAlone
        } else {
            ArmBroken
        };
        assert_eq!(settled, ArmAlone);
        assert_eq!(
            Idle.next(settled),
            Pending,
            "after settle a bare hold arms again"
        );
    }

    #[test]
    fn zoom_in_flight_suppresses_the_summon_at_the_timer_seam() {
        let elapsed_stim = |zoom_in_flight: bool| {
            if peek_allowed(zoom_in_flight) {
                Elapsed
            } else {
                ArmBroken
            }
        };
        assert_eq!(
            Pending.next(elapsed_stim(false)),
            Open,
            "no zoom: the pause opens the card"
        );
        assert_eq!(
            Pending.next(elapsed_stim(true)),
            Idle,
            "zoom in flight: the pause is suppressed"
        );
    }

    #[test]
    fn a_zoom_step_closes_an_open_card_via_interrupt() {
        assert_eq!(Open.next(Interrupt), Idle);
        assert_eq!(Pending.next(Interrupt), Idle);
    }

    #[test]
    fn a_stray_elapsed_while_idle_is_inert() {
        assert_eq!(Idle.next(Elapsed), Idle);
        assert_eq!(Open.next(Elapsed), Open);
    }

    // ---- THE ARMING-MODIFIER CONVENTION FIX --------------------------------------
    //
    // The bug this round fixes: the peek used to arm on the PHYSICAL Super/⌘ key on
    // EVERY platform — correct on Mac, wrong on Linux, where Super belongs to the
    // compositor (window-manager gestures, e.g. Omarchy/Hyprland) and the native
    // chords the peek itself TEACHES live on Ctrl. `arming_modifier` is the ONE
    // owner every arm/cancel call site now reads (the live App's
    // `on_modifiers_changed` wiring in `app/input/keys.rs`), so the mapping can
    // never scatter into separate checks.

    #[test]
    fn arming_modifier_is_super_on_mac_ctrl_on_linux() {
        use crate::convention::Convention;
        use winit::keyboard::ModifiersState;
        assert_eq!(arming_modifier(Convention::Mac), ModifiersState::SUPER);
        assert_eq!(arming_modifier(Convention::Linux), ModifiersState::CONTROL);
    }

    #[test]
    fn mac_convention_arms_only_on_bare_super_byte_identical_to_the_pre_round_behavior() {
        // Pinned explicitly (mirrors `KeymapState::new_with_convention`) rather than
        // reading the ambient `Convention::current()`, so this law holds regardless
        // of which convention the test binary happens to run under (ambient, or
        // forced via `AWL_CONVENTION_FORCE=linux`).
        use crate::convention::Convention;
        use winit::keyboard::ModifiersState;
        let c = Convention::Mac;
        assert!(
            is_bare_arming_modifier(ModifiersState::SUPER, c),
            "bare ⌘ alone arms on Mac"
        );
        assert!(!is_bare_arming_modifier(
            ModifiersState::SUPER | ModifiersState::SHIFT,
            c
        ));
        assert!(!is_bare_arming_modifier(
            ModifiersState::SUPER | ModifiersState::CONTROL,
            c
        ));
        assert!(!is_bare_arming_modifier(ModifiersState::empty(), c));
        assert!(!is_bare_arming_modifier(ModifiersState::SHIFT, c));
        assert!(
            !is_bare_arming_modifier(ModifiersState::CONTROL, c),
            "bare Ctrl is inert on Mac, unchanged"
        );
    }

    #[test]
    fn linux_convention_arms_only_on_bare_ctrl_super_is_inert() {
        // Interplay check (b): Super no longer arms anything under Linux convention —
        // it's the compositor's (Omarchy/Hyprland window-manager gestures), never
        // awl's. The live wiring only ever feeds `ArmAlone` when
        // `is_bare_arming_modifier` says so, so a bare Super hold under Linux
        // convention never even reaches `PeekArm::Pending` — there is no back door.
        use crate::convention::Convention;
        use winit::keyboard::ModifiersState;
        let c = Convention::Linux;
        assert!(
            is_bare_arming_modifier(ModifiersState::CONTROL, c),
            "bare Ctrl alone arms on Linux"
        );
        assert!(!is_bare_arming_modifier(
            ModifiersState::CONTROL | ModifiersState::SHIFT,
            c
        ));
        assert!(!is_bare_arming_modifier(
            ModifiersState::CONTROL | ModifiersState::ALT,
            c
        ));
        assert!(!is_bare_arming_modifier(ModifiersState::empty(), c));
        assert!(
            !is_bare_arming_modifier(ModifiersState::SUPER, c),
            "Super is inert on Linux — the compositor owns it"
        );
        assert!(!is_bare_arming_modifier(
            ModifiersState::SUPER | ModifiersState::SHIFT,
            c
        ));
    }

    #[test]
    fn linux_bare_ctrl_hold_arms_and_a_joined_key_cancels_into_the_ordinary_chord() {
        // Interplay check (a): under Linux convention, Ctrl is BOTH the peek's
        // arming modifier AND the modifier every ordinary Ctrl-chord lives on — the
        // emacs nav layer (`C-f`, `C-n`, …) AND the Linux-native layer (`C-s` save,
        // …) both sit on Ctrl. Proves the two never collide: the peek only arms on
        // Ctrl held ALONE, and the instant a letter joins — forming a REAL chord —
        // `KeyJoined` cancels the pending peek before it can ever flicker, exactly
        // mirroring Mac's ⌘S. The joined key itself is free to resolve through the
        // real keymap untouched; this machine only ever decides whether the CARD
        // shows, never whether the chord fires.
        use crate::convention::Convention;
        use winit::keyboard::ModifiersState;
        let c = Convention::Linux;
        assert!(is_bare_arming_modifier(ModifiersState::CONTROL, c));
        let armed = Idle.next(ArmAlone);
        assert_eq!(
            armed, Pending,
            "bare Ctrl alone arms a pending peek on Linux"
        );
        assert_eq!(
            armed.next(KeyJoined),
            Idle,
            "a joined key (C-f's 'f', C-s's 's', …) kills the pending peek before it flickers"
        );
    }

    #[test]
    fn starter_rows_are_the_curated_six_glyphified() {
        let rows = starter_rows();
        assert_eq!(rows.len(), 6, "the curated starter six");
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "Go to file",
                "Switch project",
                "Command palette",
                "Find",
                "Save",
                "Switch theme"
            ],
        );
        // CONVENTION-PARAMETRIC chord check: independently glyphify each STARTER
        // mac-form spec through the SAME two pure `keyspec` resolvers `starter_rows`
        // itself calls, per the ACTIVE convention — so the expectation holds
        // whichever convention is ambient (`Convention::Mac` on a dev Mac,
        // `Convention::Linux` on CI's linux runner via the real `cfg(target_os)`
        // path or the `AWL_CONVENTION_FORCE` dev knob), rather than hardcoding the
        // mac-only glyph form.
        let specs = ["Cmd-O", "Cmd-S-p", "Cmd-P", "Cmd-F", "Cmd-S", "Cmd-T"];
        let convention = crate::convention::Convention::current();
        for (row, spec) in rows.iter().zip(specs) {
            let expected = match convention {
                crate::convention::Convention::Mac => crate::keyspec::mac_glyph_chord(spec),
                crate::convention::Convention::Linux => crate::keyspec::linux_glyph_chord(
                    &crate::keyspec::translate_native_for_linux(spec),
                ),
            };
            assert_eq!(
                row.chord, expected,
                "{spec}: chord must glyphify per the active convention"
            );
        }
    }

    #[test]
    fn starter_rows_read_ctrl_form_chords_under_linux_mac_glyphs_under_mac() {
        // Interplay check (c), pinned EXPLICITLY via `starter_rows_for` (not
        // ambient — mirrors `KeymapState::new_with_convention`): the peek TEACHES
        // whatever fires, and its content already resolves per convention via the
        // truthful-label machinery this round didn't touch — this proves it with
        // literal expected strings on BOTH conventions, rather than re-deriving the
        // expectation through the same resolver `starter_rows_for` calls (which
        // would just be a tautology). The keymap FLAVOR setting (native/emacs)
        // never enters this path at all — only `Convention` does, confirming (c)'s
        // "the flavor setting does not change the trigger or the content" claim.
        use crate::convention::Convention;
        let mac = starter_rows_for(Convention::Mac);
        assert_eq!(mac[0].chord, "⌘O", "Go to file, Mac glyph form");
        assert_eq!(mac[1].chord, "⌘⇧P", "Switch project, Mac glyph form");
        assert_eq!(mac[4].chord, "⌘S", "Save, Mac glyph form");

        let linux = starter_rows_for(Convention::Linux);
        assert_eq!(linux[0].chord, "Ctrl+O", "Go to file, Linux word form");
        assert_eq!(
            linux[1].chord, "Ctrl+Shift+P",
            "Switch project, Linux word form"
        );
        assert_eq!(linux[4].chord, "Ctrl+S", "Save, Linux word form");
    }

    #[test]
    fn rows_or_starter_folds_empty_to_the_starter_six() {
        // A capture (never pushed) / fresh-install ledger (no candidates) → starter six.
        assert_eq!(rows_or_starter(&[]), starter_rows());
        let learned = vec![PeekRow {
            chord: "⌘;".into(),
            name: "Spell suggestions".into(),
        }];
        assert_eq!(rows_or_starter(&learned), learned);
    }
}

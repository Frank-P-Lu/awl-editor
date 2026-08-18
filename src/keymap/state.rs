use std::collections::HashMap;

use winit::keyboard::{Key, ModifiersState};

use crate::convention::Convention;

use super::platform::{
    linux_builtin_keep, linux_displaces_emacs_default_raw, linux_keeps_chord_raw,
};
use super::{Action, parse_binding};

pub enum Chord {
    Single(Key, ModifiersState),
    Cx(Key, ModifiersState),
    Cc(Key, ModifiersState),
}

pub struct KeymapState {
    pub(super) convention: Convention,
    pub(super) in_c_x: bool,
    pub(super) in_c_c: bool,
    pub(super) default_single: HashMap<(Key, ModifiersState), Action>,
    pub(super) default_c_x: HashMap<(Key, ModifiersState), Action>,
    pub(super) default_c_c: HashMap<(Key, ModifiersState), Action>,
    pub(super) override_single: HashMap<(Key, ModifiersState), Action>,
    pub(super) override_c_x: HashMap<(Key, ModifiersState), Action>,
    pub(super) override_c_c: HashMap<(Key, ModifiersState), Action>,
    /// THE EMACS-HANDS-ON-LINUX ROUND — the config `linux_keep_emacs` list, parsed
    /// into concrete `(key, mods)` chords: on [`Convention::Linux`], a chord in
    /// this set does NOT participate in the native-wins collision (see
    /// [`Self::linux_keeps`]) — its bare-control emacs meaning fires instead. Built
    /// by [`Self::apply_linux_keep`], consulted ONLY when `convention ==
    /// Convention::Linux` (so a Mac keymap can carry a non-empty set — e.g. a test
    /// exercising `apply_linux_keep` before switching convention — and it is still
    /// STRUCTURALLY inert there, matching "Mac convention ignores the key
    /// entirely"). NEVER truly empty by construction — [`Self::apply_linux_keep`]
    /// always seeds `linux_builtin_keep()` first (the insert-link-yields-to-
    /// kill-line floor), so an absent config keeps today's dispatch PLUS that one
    /// unconditional floor chord; every OTHER letter still needs an explicit
    /// `linux_keep_emacs`/`keymap = "emacs"` opt-in, unchanged.
    pub(super) linux_keep: std::collections::HashSet<(Key, ModifiersState)>,
    /// THE CLASSIC META LAYER gate — true only for `Convention::Linux` under
    /// `keymap = "emacs"` (set by [`Self::set_linux_emacs_meta`], the config
    /// layer's own boolean: this module stays unaware of `KeymapFlavor` itself,
    /// mirroring how `linux_keep` stays unaware of the config field that built
    /// it). Consulted ONLY inside [`Self::seed_defaults`], which re-checks
    /// `convention == Convention::Linux` itself — so a stray `true` set before a
    /// convention switch (or on `Convention::Mac`, where Option keeps typing
    /// accented characters) stays structurally inert, the same belt-and-
    /// suspenders shape `linux_keep`'s own doc describes.
    pub(super) linux_emacs_meta: bool,
}

impl Default for KeymapState {
    fn default() -> Self {
        let mut km = Self {
            convention: Convention::current(),
            in_c_x: false,
            in_c_c: false,
            default_single: HashMap::new(),
            default_c_x: HashMap::new(),
            default_c_c: HashMap::new(),
            override_single: HashMap::new(),
            override_c_x: HashMap::new(),
            override_c_c: HashMap::new(),
            linux_keep: std::collections::HashSet::new(),
            linux_emacs_meta: false,
        };
        // Seed the unconditional built-in keep floor (see `apply_linux_keep`'s
        // doc) — so even a `KeymapState` that never has `apply_linux_keep`
        // called on it (a bare `new`/`new_with_convention`, the shape most of
        // this module's own unit tests use) still carries the floor.
        km.apply_linux_keep(&[]);
        km
    }
}

impl KeymapState {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn new_with_convention(convention: Convention) -> Self {
        let mut km = Self {
            convention,
            ..Self::default()
        };
        km.seed_defaults();
        km
    }

    pub fn with_overrides(keys: &[(String, Vec<String>)]) -> Self {
        let mut km = Self::new();
        km.apply_overrides(keys);
        km
    }

    #[cfg(test)]
    pub fn with_overrides_and_convention(
        keys: &[(String, Vec<String>)],
        convention: Convention,
    ) -> Self {
        let mut km = Self::new_with_convention(convention);
        km.apply_overrides(keys);
        km
    }

    /// [`Self::with_overrides`], ALSO applying the config `linux_keep_emacs` list
    /// (see [`Self::apply_linux_keep`]) and the classic-Meta-layer gate (see
    /// [`Self::set_linux_emacs_meta`]) — the real production door every live/
    /// headless call site should use once it has a [`crate::config::Config`] in
    /// hand (`App::new`, the `--keys` replay keymap built in `main/args.rs`);
    /// `with_overrides` alone
    /// stays as the simpler door for the many call sites (mostly tests) that
    /// never touch the keep-list. `linux_emacs_meta` is set BEFORE
    /// `apply_linux_keep` runs so the one `seed_defaults()` it triggers already
    /// sees the correct gate — no double reseed.
    pub fn with_overrides_and_keep(
        keys: &[(String, Vec<String>)],
        keep: &[String],
        linux_emacs_meta: bool,
    ) -> Self {
        let mut km = Self::with_overrides(keys);
        km.linux_emacs_meta = linux_emacs_meta;
        km.apply_linux_keep(keep);
        km
    }

    /// Rebuild the catalog-default dispatch layer from the same resolved command
    /// slots every label surface reads. Platform collision policy stays here: on
    /// Linux a kept emacs chord suppresses its native claimant, while a displaced
    /// emacs chord is omitted. Duplicate effective defaults are an embedded-data
    /// bug and panic unless both rows intentionally resolve to the same action.
    fn seed_defaults(&mut self) {
        self.default_single.clear();
        self.default_c_x.clear();
        self.default_c_c.clear();

        for command in crate::commands::COMMANDS.iter() {
            let native = crate::commands::resolved_native(command, self.convention);
            let native_suppressed = self.convention == Convention::Linux
                && linux_keeps_chord_raw(&self.linux_keep, &native);
            let emacs_displaced = self.convention == Convention::Linux
                && linux_displaces_emacs_default_raw(command.emacs, &self.linux_keep);

            if !emacs_displaced {
                self.insert_default(command.emacs, command.action.clone(), command.name);
            }
            if !native_suppressed {
                self.insert_default(&native, command.action.clone(), command.name);
            }
        }

        let shifted: Vec<_> = self
            .default_single
            .iter()
            .filter(|((_, mods), action)| {
                !mods.contains(ModifiersState::SHIFT)
                    // Command palette's Shift companion is Open project — a
                    // bespoke, uncatalogued resolver arm (`resolve.rs`) that
                    // reads the SAME character key with Shift held and is only
                    // ever reached once `default_single` misses. Letting the
                    // general convenience duplication below claim that chord
                    // too would shadow the arm outright, since a `default_single`
                    // hit is checked first on every keypress.
                    && *action != &Action::OpenCommandPalette
            })
            .map(|((key, mods), action)| {
                ((key.clone(), *mods | ModifiersState::SHIFT), action.clone())
            })
            .collect();
        for (chord, action) in shifted {
            self.default_single.entry(chord).or_insert(action);
        }

        for command in crate::commands::COMMANDS.iter() {
            self.insert_control_super_variants(command.emacs, command.action.clone(), command.name);
            if command.native.starts_with("C-") {
                self.insert_control_super_variants(
                    command.native,
                    command.action.clone(),
                    command.name,
                );
            }
        }

        // THE CLASSIC META LAYER — seeded LAST, after the Shift-
        // convenience duplication above, so a Meta entry never grows its own
        // unrequested Shift companion the way an ordinary catalog default does.
        // Every entry fires an EXISTING catalog `Action`; see
        // `platform::LINUX_EMACS_META_SEED`'s doc for why `Convention::Mac`
        // never reaches this branch regardless of the gate's value.
        if self.convention == Convention::Linux && self.linux_emacs_meta {
            for (spec, action) in super::platform::LINUX_EMACS_META_SEED {
                self.insert_default(spec, action.clone(), "linux emacs Meta layer");
            }
        }
    }

    /// Flip the classic-Meta-layer gate and reseed — the Meta-layer
    /// sibling of [`Self::apply_linux_keep`], called right alongside it on every
    /// door that can change `keymap` flavor live (`App::apply_keymap_flavor`,
    /// config reload): both halves of a flavor flip land in the same reseed.
    /// Structurally inert off `Convention::Linux` (see the gate field's own doc
    /// above), so a caller may pass the raw `flavor == KeymapFlavor::Emacs` bool
    /// unconditionally, on either convention.
    pub fn set_linux_emacs_meta(&mut self, active: bool) {
        self.linux_emacs_meta = active;
        self.seed_defaults();
    }

    fn insert_default(&mut self, spec: &str, action: Action, name: &str) {
        if spec.trim().is_empty() {
            return;
        }
        let chord = parse_binding(spec).unwrap_or_else(|e| {
            panic!("assets/keymap-defaults.toml: {name:?} has invalid chord {spec:?}: {e}")
        });
        match chord {
            Chord::Single(k, m) => {
                insert_default_entry(
                    &mut self.default_single,
                    (k.clone(), m),
                    action.clone(),
                    name,
                    spec,
                );
            }
            Chord::Cx(k, m) => {
                insert_default_entry(&mut self.default_c_x, (k, m), action, name, spec);
            }
            Chord::Cc(k, m) => {
                insert_default_entry(&mut self.default_c_c, (k, m), action, name, spec);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn replace_defaults_for_test(&mut self, spec: &str, action: Action, name: &str) {
        self.default_single.clear();
        self.default_c_x.clear();
        self.default_c_c.clear();
        self.insert_default(spec, action, name);
    }

    fn insert_control_super_variants(&mut self, spec: &str, action: Action, name: &str) {
        if spec.trim().is_empty() {
            return;
        }
        let chord = parse_binding(spec).unwrap_or_else(|e| {
            panic!("assets/keymap-defaults.toml: {name:?} has invalid chord {spec:?}: {e}")
        });
        let add_super = |mods: ModifiersState| {
            mods.contains(ModifiersState::CONTROL)
                .then_some(mods | ModifiersState::SUPER)
        };
        match chord {
            Chord::Single(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_single
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_single
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
            Chord::Cx(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_c_x
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_c_x
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
            Chord::Cc(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_c_c
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_c_c
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
        }
    }

    /// Apply (or RE-apply, on a live config reload) the `[keys]` rebinds. Each entry
    /// maps an action NAME (the command-palette name, slugified) to a LIST of up to 2
    /// chords (slot 1 = native, slot 2 = emacs); each valid chord OVERRIDES that
    /// action's binding (additively — both the configured chords AND the default
    /// still fire). An unknown action or a bad chord is reported to stderr and
    /// SKIPPED, keeping the default — never a crash. Only the FIRST TWO chords of a
    /// list are honoured (the model is capped at 2). Clears any prior overrides first
    /// so a reload reflects exactly the current file.
    pub fn apply_overrides(&mut self, keys: &[(String, Vec<String>)]) {
        self.override_single.clear();
        self.override_c_x.clear();
        self.override_c_c.clear();
        for (name, chords) in keys {
            let Some(action) = crate::commands::action_for_name(name) else {
                eprintln!("config [keys]: unknown action {name:?}; ignored");
                continue;
            };
            for chord in chords.iter().take(2) {
                match parse_binding(chord) {
                    Ok(Chord::Single(k, m)) => {
                        self.override_single.insert((k, m), action.clone());
                    }
                    Ok(Chord::Cx(k, m)) => {
                        self.override_c_x.insert((k, m), action.clone());
                    }
                    Ok(Chord::Cc(k, m)) => {
                        self.override_c_c.insert((k, m), action.clone());
                    }
                    Err(e) => {
                        eprintln!("config [keys]: {name} = {chord:?}: {e}; keeping default");
                    }
                }
            }
        }
    }

    /// Apply (or RE-apply, on a live config reload) the `linux_keep_emacs` list —
    /// THE PER-CHORD DOOR the emacs-hands-on-Linux round adds: under
    /// [`Convention::Linux`], every chord named here is EXEMPTED from the
    /// native-wins collision (`native_down`'s displacement), so its bare-control
    /// emacs meaning keeps firing instead of the native chord that would
    /// otherwise claim that letter (see the module's collision-table doc). A
    /// chord is a plain SINGLE spec (`"C-f"`, no `C-x`/`C-c` prefix — the
    /// collision only ever touches single Ctrl-letter chords); a bad/unparseable
    /// entry, or one that isn't a single chord, is reported to stderr and
    /// SKIPPED (never a crash), mirroring [`Self::apply_overrides`]'s leniency.
    /// On [`Convention::Mac`] the list is parsed but the set stays
    /// consultable-yet-inert — [`Self::linux_keeps`] gates on convention too, so
    /// even a stray non-empty set can never fire there (belt + suspenders with
    /// the convention check at the call site in [`Self::resolve`]).
    ///
    /// THE INSERT-LINK-YIELDS-TO-KILL-LINE ROUND: clears any prior keep-set
    /// first (so a reload reflects exactly the current file), then ALWAYS
    /// re-seeds `linux_builtin_keep()` before layering `keep` on top — the
    /// built-in floor is UNREMOVABLE by this function, whether called with the
    /// full `Config::effective_linux_keep()` composition, a hand-rolled test
    /// list, or an empty one. This is what makes the floor real even for a
    /// caller (a bare unit test, `linux_emacs_preset_keep()` applied on its
    /// own) that never threads it through `Config` at all.
    pub fn apply_linux_keep(&mut self, keep: &[String]) {
        self.linux_keep.clear();
        for chord in linux_builtin_keep()
            .iter()
            .copied()
            .chain(keep.iter().map(String::as_str))
        {
            match parse_binding(chord) {
                Ok(Chord::Single(k, m)) => {
                    self.linux_keep.insert((k, m));
                }
                Ok(_) => {
                    eprintln!(
                        "config linux_keep_emacs: {chord:?}: only a single chord \
                         (no C-x/C-c prefix) is supported; ignored"
                    );
                }
                Err(e) => {
                    eprintln!("config linux_keep_emacs: {chord:?}: {e}; ignored");
                }
            }
        }
        self.seed_defaults();
    }
}

pub(super) fn insert_default_entry(
    map: &mut HashMap<(Key, ModifiersState), Action>,
    chord: (Key, ModifiersState),
    action: Action,
    name: &str,
    spec: &str,
) {
    if let Some(existing) = map.get(&chord) {
        assert_eq!(
            existing, &action,
            "assets/keymap-defaults.toml: conflicting effective default \
             {spec:?} for {name:?}: {existing:?} versus {action:?}"
        );
        return;
    }
    map.insert(chord, action);
}

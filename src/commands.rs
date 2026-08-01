use crate::convention::Convention;
use crate::facets::{Facet, FacetItem, FacetScheme};
use crate::keymap::Action;
use std::sync::Mutex;
mod catalog;
/// The convention/platform chord resolver — one subject, one file.
mod chords;
mod menu_section;
use catalog::COMMAND_SEED;
#[cfg(any(not(target_arch = "wasm32"), test))]
pub use chords::resolved_native_label;
pub use chords::{
    resolved_native, resolved_native_label_truthful, resolved_native_truthful, web_alternate_keys,
};
pub use menu_section::menu_section;
#[cfg(test)]
use menu_section::{EDIT_COMMANDS, FILE_COMMANDS, VIEW_COMMANDS};
pub struct Command {
    pub name: &'static str,
    pub action: Action,
    pub native: &'static str,
    pub emacs: &'static str,
    pub native_only: bool,
    pub web_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Native,
    Web,
}

impl Platform {
    pub fn current() -> Platform {
        if cfg!(target_arch = "wasm32") {
            Platform::Web
        } else {
            Platform::Native
        }
    }
}

impl Command {
    /// PURE predicate: is this command available on `platform`? `Native` excludes
    /// every `web_only` command (a native user has real files; the export escape
    /// hatch is pointless there); `Web` excludes every `native_only` command (a
    /// browser tab has no real disk / OS shell / daemon). The single owner every
    /// filtered view below routes through.
    pub fn available_on(&self, platform: Platform) -> bool {
        match platform {
            Platform::Native => !self.web_only,
            Platform::Web => !self.native_only,
        }
    }
}

/// Is the catalog command named `name` available on `platform`? Looks it up by
/// NAME (not corpus index) in the full, unfiltered catalog — the seam
/// [`crate::settings::COVERED_BY`] uses to decide whether a covered settings
/// row's covering command is actually reachable on this platform (so a settings
/// row can REAPPEAR in the palette union if its covering command is
/// platform-hidden, rather than the door being lost entirely). `false` for an
/// unknown name — never happens for a real `COVERED_BY` entry, guarded by
/// `settings::tests::every_covered_by_pair_names_a_real_row_and_a_real_command`.
pub fn available_by_name(name: &str, platform: Platform) -> bool {
    COMMANDS
        .iter()
        .find(|c| c.name == name)
        .is_some_and(|c| c.available_on(platform))
}

/// THE KEYMAP-DEFAULTS-AS-DATA ROUND (CLAUDE.md): the actual command catalog.
/// [`COMMAND_SEED`] above carries every command's NAME, ORDER, `Action`, and
/// platform scope (`native_only`/`web_only`) — hand-written code, unchanged
/// by this round — but its own `native`/`emacs` fields are unused
/// placeholders (always `""` in the literal). The REAL default chord values
/// are looked up ONCE by slug from the embedded `assets/keymap-defaults.toml`
/// ([`crate::keymap_defaults::command_defaults`]) and spliced in here — so a
/// default chord now exists in exactly ONE place (the TOML file), never
/// duplicated as a second literal in this array. `Box::leak` is a one-time,
/// ~80-entry startup cost (the whole `Vec` is memoized by `LazyLock`, so this
/// closure runs at most once per process) — it keeps `Command::native`/
/// `emacs`'s public field TYPE (`&'static str`) unchanged, so every existing
/// consumer (`c.native.trim()`, `COMMANDS.iter()`, `COMMANDS[i]`, …) needed no
/// edit beyond a handful of bare `for c in COMMANDS` loops (which cannot
/// desugar against a `LazyLock`'s owned `Vec` without an explicit `.iter()`,
/// unlike the retired `&'static [Command]` slice, which was `Copy`).
pub static COMMANDS: std::sync::LazyLock<Vec<Command>> = std::sync::LazyLock::new(|| {
    let defaults = crate::keymap_defaults::command_defaults();
    assert_eq!(
        defaults.len(),
        COMMAND_SEED.len(),
        "assets/keymap-defaults.toml must contain exactly one entry for every catalog command"
    );
    for key in defaults.keys() {
        assert!(
            COMMAND_SEED.iter().any(|seed| slug(seed.name) == *key),
            "assets/keymap-defaults.toml names unknown command slug {key:?}"
        );
    }
    COMMAND_SEED
        .iter()
        .map(|seed| {
            let seed_slug = slug(seed.name);
            let (native, emacs) = defaults
                .get(seed_slug.as_str())
                .cloned()
                .unwrap_or_else(|| {
                    panic!("assets/keymap-defaults.toml is missing catalog command {seed_slug:?}")
                });
            Command {
                name: seed.name,
                action: seed.action.clone(),
                native: Box::leak(native.into_boxed_str()),
                emacs: Box::leak(emacs.into_boxed_str()),
                native_only: seed.native_only,
                web_only: seed.web_only,
            }
        })
        .collect()
});

pub fn join_slots(native: &str, emacs: &str) -> String {
    let native_g = if native.trim().is_empty() {
        String::new()
    } else {
        crate::keyspec::mac_glyph_chord(native)
    };
    match (native_g.is_empty(), emacs.trim().is_empty()) {
        (false, false) => format!("{native_g} · {emacs}"),
        (false, true) => native_g,
        (true, false) => emacs.to_string(),
        (true, true) => String::new(),
    }
}

pub fn slug(name: &str) -> String {
    name.trim()
        .trim_end_matches('…')
        .trim()
        .to_ascii_lowercase()
        .replace(' ', "_")
}

pub fn action_for_name(name: &str) -> Option<Action> {
    let want = slug(name);
    COMMANDS
        .iter()
        .find(|c| slug(c.name) == want)
        .map(|c| c.action.clone())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn slug_for_action(action: &Action) -> Option<String> {
    COMMANDS
        .iter()
        .find(|c| &c.action == action)
        .map(|c| slug(c.name))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn has_native_chord(slug_want: &str) -> bool {
    COMMANDS
        .iter()
        .any(|c| slug(c.name) == slug_want && !c.native.trim().is_empty())
}

/// The DISCOVERABILITY row for a command `slug`: its NATIVE (macOS) chord as modifier
/// glyphs (`keyspec::mac_glyph_chord`) + its display name (ellipsis stripped), or `None`
/// when the slug is unknown OR palette-only (no native chord to teach). The shared
/// resolver behind BOTH the hold-⌘ peek's personalized rows ([`crate::peek::PeekRow`])
/// and the Keybindings footer's tip lines, so the two surfaces name a shortcut
/// identically. Called on the SLOW-DOOR graduation candidates the ledger ranks, every
/// one of which passed [`has_native_chord`], so the `None` arm is only the defensive
/// unknown-slug case.
///
/// Native-only, matching [`slug_for_action`]: called only from `app/stats.rs`.
#[cfg(not(target_arch = "wasm32"))]
pub fn peek_row_for_slug(slug_want: &str) -> Option<crate::peek::PeekRow> {
    let c = COMMANDS.iter().find(|c| slug(c.name) == slug_want)?;
    if c.native.trim().is_empty() {
        return None;
    }
    let chord = resolved_native_label(c, Convention::current());
    if chord.is_empty() {
        return None;
    }
    Some(crate::peek::PeekRow {
        chord,
        name: c.name.trim_end_matches('…').trim().to_string(),
    })
}

pub fn effective_bindings(keys: &[(String, Vec<String>)], keep: &[String]) -> Vec<String> {
    COMMANDS
        .iter()
        .map(|c| effective_binding_for(c, keys, keep, Platform::current()))
        .collect()
}

fn effective_binding_for(
    c: &Command,
    keys: &[(String, Vec<String>)],
    keep: &[String],
    platform: Platform,
) -> String {
    let convention = Convention::current();
    let chords = effective_chords(c, keys);
    if effective_is_override(c, keys) {
        // A `[keys]` override is CONVENTION-AGNOSTIC (taken literally on every
        // platform — the chord VALUE never gets Cmd→Ctrl translated), but its
        // DISPLAY GLYPHS still route through the ONE resolved label owner: slot 1
        // (index 0) is NATIVE → convention glyphs (mac ⌘ / Linux word labels);
        // slot 2+ is EMACS → terse text, matching the static `join_slots` rule.
        chords
            .iter()
            .enumerate()
            .map(|(i, ch)| {
                if i == 0 {
                    match convention {
                        Convention::Mac => crate::keyspec::mac_glyph_chord(ch),
                        Convention::Linux => crate::keyspec::linux_glyph_chord(ch),
                    }
                } else {
                    ch.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" · ")
    } else {
        join_slots_truthful(c, convention, platform, keep)
    }
}

/// THE WEB CHORD SANITY ROUND — THE LABEL-TRUTH OWNER for a command's STATIC
/// (non-override) two-slot label. Supersedes the old Mac-`join_slots` /
/// Linux-`join_slots_resolved` split with ONE function that joins `c`'s
/// resolved-native + emacs labels for `convention`, but DROPS either half that
/// would not actually fire:
///   - **Tier 2 (web-reserved):** the resolved native chord is a browser
///     accelerator no page can intercept ([`crate::webreserved::is_reserved`]) —
///     checked ONLY on [`Platform::Web`], since a native build's chords are
///     never browser-shadowed.
///   - **Tier 3 (Linux-displaced):** the static emacs default is quietly
///     DISPLACED by [`Convention::Linux`]'s collision table
///     ([`crate::keymap::linux_displaces_emacs_default`]) — checked on EITHER
///     platform, since the collision is a property of the DISPATCH TABLE (a
///     native Linux desktop build has it too), not of being on the web.
///   - **Tier 4 (emacs-hands-on-Linux — the `linux_keep_emacs` config, THE
///     PER-CHORD DOOR this round adds):** `keep` is the config
///     `linux_keep_emacs` list — chords a Linux hand asked to keep their emacs
///     meaning, suppressing that letter's NATIVE-WINS displacement for exactly
///     that chord (see `keymap.rs`'s `KeymapState::linux_keeps` — the SAME
///     `keep` list gates the real dispatch, so a label shown here can never
///     lie about what actually fires). This is TWO-SIDED, mirroring the
///     collision itself: (a) [`crate::keymap::linux_displaces_emacs_default`]
///     is now `keep`-aware — a kept chord is NOT displaced, so its emacs label
///     reappears; (b) the NATIVE command that used to claim that Linux chord
///     must stop advertising it (`native_suppressed` below) — a chord this
///     table shows must be the one that actually wins.
///
/// On `Convention::Mac` + `Platform::Native` (macOS native) NONE of the three
/// checks can ever fire (`Platform::Web` is false; `convention == Linux` is
/// false, so both the Tier-3 displacement AND the Tier-4 keep-list are
/// structurally inert — `keep` is ignored outright on Mac, by construction),
/// so this is BYTE-IDENTICAL to the old `join_slots(c.native, c.emacs)` there —
/// the hard law this round must not break (see
/// `tests::mac_native_label_truth_is_byte_identical_to_join_slots`).
fn join_slots_truthful(
    c: &Command,
    convention: Convention,
    platform: Platform,
    keep: &[String],
) -> String {
    let native_suppressed = convention == Convention::Linux
        && crate::keymap::linux_keeps_chord(keep, &resolved_native(c, convention));
    let native_label = if native_suppressed {
        String::new()
    } else {
        resolved_native_label_truthful(c, convention, platform)
    };

    let emacs_displaced = convention == Convention::Linux
        && crate::keymap::linux_displaces_emacs_default(c.emacs, keep);
    let emacs_label: &str = if emacs_displaced { "" } else { c.emacs };

    match (native_label.is_empty(), emacs_label.trim().is_empty()) {
        (false, false) => format!("{native_label} · {emacs_label}"),
        (false, true) => native_label,
        (true, false) => emacs_label.to_string(),
        (true, true) => String::new(),
    }
}

/// THE GUIDE'S GENERATED KEYS REFERENCE — the drift-proof source for the fenced
/// table between `<!-- GENERATED:keys-reference:BEGIN -->` /
/// `<!-- GENERATED:keys-reference:END -->` in `GUIDE.md`. Every catalog command,
/// its resolved DEFAULT (config-free) chord label under EACH convention — mac
/// glyphs on [`Convention::Mac`], Linux words on [`Convention::Linux`] — via the
/// SAME [`join_slots_truthful`] the palette itself reads (`Platform::Native`
/// throughout: both columns describe an OS convention, not the browser build, so
/// the web-reserved tier never fires here; the Linux-displaced tier DOES, since
/// that collision is a property of the dispatch table on ANY Linux build). The
/// LINUX column's `keep` list is [`crate::config::Config::empty`]'s
/// `effective_linux_keep()` — the DEFAULT, config-free composition (just
/// `keymap::linux_builtin_keep()`, under the default `native` flavor) — so a
/// command like Insert link, unbound on Linux out of the box, correctly shows
/// an empty Linux cell rather than a chord no default install would ever
/// actually honor. The LAW TEST living beside `GUIDE_MD` (`guide::tests::
/// generated_keys_reference_matches_catalog`) regenerates this and diffs it
/// byte-for-byte against the checked-in section — a catalog change (new
/// command, new default chord) fails that test until the doc is regenerated
/// and pasted back in. Regenerate with:
/// `cargo test --bin awl guide::tests::print_generated_keys_reference -- --ignored --nocapture`
#[cfg(test)]
pub(crate) fn generate_keys_reference_markdown() -> String {
    let mut out = String::new();
    out.push_str("| Command | macOS | Linux |\n");
    out.push_str("|---|---|---|\n");
    let default_linux_keep = crate::config::Config::empty().effective_linux_keep();
    for c in COMMANDS.iter() {
        let mac = join_slots_truthful(c, Convention::Mac, Platform::Native, &[]);
        let linux =
            join_slots_truthful(c, Convention::Linux, Platform::Native, &default_linux_keep);
        out.push_str(&format!("| {} | {mac} | {linux} |\n", c.name));
    }
    out
}

pub(crate) fn effective_chords(c: &Command, keys: &[(String, Vec<String>)]) -> Vec<String> {
    if let Some(over) = override_chords(c, keys) {
        return over;
    }
    [c.native, c.emacs]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .collect()
}

fn override_chords(c: &Command, keys: &[(String, Vec<String>)]) -> Option<Vec<String>> {
    keys.iter()
        .find(|(name, _)| slug(name) == slug(c.name) && action_for_name(name).is_some())
        .map(|(_, chords)| {
            chords
                .iter()
                .filter(|ch| crate::keymap::parse_binding(ch).is_ok())
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
}

fn effective_is_override(c: &Command, keys: &[(String, Vec<String>)]) -> bool {
    override_chords(c, keys).is_some()
}

/// CONFLICT check for the rebind menu: is `binding` already an effective chord of a
/// command OTHER than `exclude_slug`? Returns the conflicting command's display NAME
/// (the first match) so the menu can warn "already bound to X" before/while writing.
/// Bindings are compared CANONICALLY (`Cmd-S` == `s-s`), so equivalent spellings
/// clash; an unparseable `binding` never conflicts (returns `None`).
pub fn binding_conflict(
    binding: &str,
    exclude_slug: &str,
    keys: &[(String, Vec<String>)],
) -> Option<&'static str> {
    let want = crate::keyspec::canonical_binding(binding)?;
    COMMANDS
        .iter()
        .filter(|c| slug(c.name) != exclude_slug)
        .find(|c| {
            effective_chords(c, keys)
                .iter()
                .any(|ch| crate::keyspec::canonical_binding(ch).as_deref() == Some(want.as_str()))
        })
        .map(|c| c.name)
}

#[cfg(test)]
pub fn names() -> Vec<String> {
    COMMANDS.iter().map(|c| c.name.to_string()).collect()
}

#[allow(dead_code)]
pub fn bindings() -> Vec<String> {
    COMMANDS
        .iter()
        .map(|c| join_slots(c.native, c.emacs))
        .collect()
}

// ── PLATFORM-SCOPED COMMANDS: the ONE filtered view ────────────────────────────
//
// `COMMANDS` stays the raw, full catalog (every test that wants to enumerate every
// command — native or not — still reads it directly, or via `names()`/`bindings()`
// above, which are DELIBERATELY unfiltered so a native-run test can pin the FULL
// catalog). Every USER-FACING surface (the palette build, the rebind menu build, the
// palette's Enter/accept path, the rebind menu's Delete-to-reset + capture-prompt
// doors, which-key, and the awl-rendered + native menu bars) instead routes through
// `visible()` (and its `visible_*` siblings below) — the ONE narrowed view a command's
// `native_only` flag ever reaches through. A "corpus row index" downstream of
// `visible()` is an index into ITS OWN Vec, never into `COMMANDS` directly — that is
// what keeps a picker's displayed row and its Enter/accept action from ever drifting
// apart once some rows are hidden.

fn visible_indices_on(platform: Platform) -> Vec<usize> {
    COMMANDS
        .iter()
        .enumerate()
        .filter(|(_, c)| c.available_on(platform))
        .map(|(i, _)| i)
        .collect()
}

fn visible_on(platform: Platform) -> Vec<&'static Command> {
    visible_indices_on(platform)
        .into_iter()
        .map(|i| &COMMANDS[i])
        .collect()
}

pub fn visible() -> Vec<&'static Command> {
    visible_on(Platform::current())
}

pub fn visible_names() -> Vec<String> {
    visible().iter().map(|c| c.name.to_string()).collect()
}

/// The EFFECTIVE binding labels for [`visible`], parallel to [`visible_names`] — the
/// platform-filtered sibling of [`effective_bindings`], sharing its per-command body
/// (`effective_binding_for`) so the two can never compute a binding label differently.
pub fn visible_effective_bindings(keys: &[(String, Vec<String>)], keep: &[String]) -> Vec<String> {
    visible()
        .iter()
        .map(|c| effective_binding_for(c, keys, keep, Platform::current()))
        .collect()
}

/// The EFFECTIVE chord LISTS for [`visible`], parallel to [`visible_names`] — each
/// command's active chords (a valid config override, else the static native/emacs
/// slots), UN-joined and un-glyphified (empty slots dropped), narrowed to the
/// platform-visible set. This is what which-key (`crate::whichkey::continuations`)
/// derives its prefix rows from, so a hidden command's chord (if it happened to
/// start with a prefix) never surfaces as a continuation on web.
pub fn visible_effective_chord_lists(keys: &[(String, Vec<String>)]) -> Vec<Vec<String>> {
    visible()
        .iter()
        .map(|c| effective_chords(c, keys))
        .collect()
}

pub fn visible_action_of(corpus_i: usize) -> Action {
    visible()[corpus_i].action.clone()
}

pub fn visible_slug_of(corpus_i: usize) -> String {
    slug(visible()[corpus_i].name)
}

pub fn visible_name_of(corpus_i: usize) -> &'static str {
    visible()[corpus_i].name
}

/// The recently-run-command MRU ([`recent_indices`], catalog-index space), translated
/// into VISIBLE-CORPUS row indices — dropping any catalog index that isn't visible on
/// this platform (a hidden command, if somehow ever recorded, can never show as
/// "recent"). The one door that feeds a built `OverlayState.recent` (corpus-index
/// space), so a stale catalog index there can never point at the wrong visible row.
pub fn visible_recent_indices() -> Vec<usize> {
    let idx = visible_indices_on(Platform::current());
    recent_indices()
        .into_iter()
        .filter_map(|catalog_i| idx.iter().position(|&v| v == catalog_i))
        .collect()
}

/// RUNTIME-gated rows, parallel to [`visible`]/[`visible_names`] — `true` at index
/// `i` iff `visible()[i]` should be HIDDEN from selection right now for a reason
/// that is NOT the compile-time `Platform` axis (`native_only`/`web_only`) but a
/// live fact the caller gathers. Today exactly one row is runtime-gated: "Finish
/// file" (`Action::FinishBuffer`, C-x #) only makes sense mid a daemon `--wait`
/// round-trip (`crate::daemon`'s module doc) — with no terminal actively waiting
/// there is nothing to finish, so it stays out of the palette. `has_waiter` is the
/// ONE live fact the caller passes (the live App's `wait_conns`; always `false` in
/// the headless capture/replay path, which has no daemon at all — the daemon
/// capture gate — so a `--keys`/`--screenshot` palette is deterministically built
/// WITHOUT this row). A pure fn of that one bool: `has_waiter` true unmasks every
/// row (an empty mask, byte-identical to before this round existed); false masks
/// exactly the one `FinishBuffer` row. Consumed by `OverlayState::new_command`'s
/// `hidden` parameter, which `refilter` reads to drop masked rows from what's
/// SELECTABLE while leaving `corpus` itself (and every index into it that
/// `visible_action_of` relies on) untouched.
pub fn visible_hidden_mask(has_waiter: bool) -> Vec<bool> {
    visible()
        .iter()
        .map(|c| c.action == Action::FinishBuffer && !has_waiter)
        .collect()
}

/// The DISPATCH-time gate: is `action` available on `platform`? `true` for any action
/// with NO catalog entry (a motion / self-insert / non-catalog effect always fires, and
/// there is nothing to hide) and for a catalog action that IS available; `false` for a
/// `native_only` catalog action on `Web` OR a `web_only` catalog action on `Native`.
/// This is the BELT to `visible`'s BRACES: even if a chord is still configured/rebound
/// to fire a hidden command, or a stray `Effect::RunAction` re-dispatch names one
/// directly, this stops the actual mutation — hiding a picker row alone is not enough
/// (a keymap chord bypasses the picker entirely). Cheap: at most `COMMANDS.len()` (59)
/// enum comparisons, no allocation. (Was a Native short-circuit before `web_only`
/// existed — now a plain `available_on` lookup on both platforms, since a `web_only`
/// row must actually be gated on Native too.)
pub fn action_available(action: &Action, platform: Platform) -> bool {
    match COMMANDS.iter().find(|c| &c.action == action) {
        Some(c) => c.available_on(platform),
        None => true,
    }
}

const COMMAND_FACET_STRIP: [Facet; 5] = [
    Facet {
        label: "All",
        id: "all",
        sections: &[],
    },
    Facet {
        label: "File",
        id: "file",
        sections: &["File"],
    },
    Facet {
        label: "Edit",
        id: "edit",
        sections: &["Edit"],
    },
    Facet {
        label: "View",
        id: "view",
        sections: &["View"],
    },
    Facet {
        label: "Recent",
        id: "recent",
        sections: &["Recent"],
    },
];

fn command_bucket(item: FacetItem, lens_idx: usize) -> Option<&'static str> {
    match lens_idx {
        1 => (menu_section(item.accept) == Some("File")).then_some("File"),
        2 => (menu_section(item.accept) == Some("Edit")).then_some("Edit"),
        3 => (menu_section(item.accept) == Some("View")).then_some("View"),
        4 => item.recent.then_some("Recent"), // Recent
        _ => None,
    }
}

pub static COMMAND_FACETS: FacetScheme = FacetScheme {
    strip: &COMMAND_FACET_STRIP,
    bucket: command_bucket,
};

// ── Recently-run commands (an in-memory MRU, NOT persisted) ────────────────────
//
// The palette's Recent lens is sourced from a process-global MRU of catalog indices,
// recorded whenever a command is RUN from the palette. It is deliberately in-memory
// only (no disk store this round) — a fresh process starts empty, so a headless
// capture's Recent lens is inert (nothing recorded), honoring the determinism gate.
// Recording is LIVE-APP-ONLY ([`crate::app`]'s `Effect::RunAction` handler), never the
// shared/headless core, so the capture path never mutates this global.

const RECENT_CAP: usize = 12;

static RECENT: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// Record that the command dispatching `action` was just RUN (from the palette),
/// moving its catalog index to the front of the MRU (deduped, capped at
/// [`RECENT_CAP`]). A no-op for an `action` no catalog command carries. LIVE-ONLY:
/// called from the App's palette-run seam, never the headless replay.
pub fn record_recent(action: &Action) {
    let Some(i) = COMMANDS.iter().position(|c| &c.action == action) else {
        return;
    };
    if let Ok(mut mru) = RECENT.lock() {
        mru.retain(|&x| x != i);
        mru.insert(0, i);
        mru.truncate(RECENT_CAP);
    }
}

pub fn recent_indices() -> Vec<usize> {
    RECENT.lock().map(|m| m.clone()).unwrap_or_default()
}

#[cfg(test)]
pub fn clear_recent() {
    if let Ok(mut mru) = RECENT.lock() {
        mru.clear();
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod identity_snapshot {
    use super::*;

    #[test]
    #[ignore]
    fn print_full_catalog_snapshot() {
        for c in COMMANDS.iter() {
            println!(
                "{}|{:?}|{}|{}|{}|{}",
                c.name, c.action, c.native, c.emacs, c.native_only, c.web_only
            );
        }
    }
}

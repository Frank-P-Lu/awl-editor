use crate::convention::Convention;
use crate::facets::{Facet, FacetItem, FacetScheme};
use crate::keymap::Action;
use std::sync::Mutex;

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

static COMMAND_SEED: &[Command] = &[
    Command {
        name: "Go to file…",
        action: Action::OpenGoto,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Switch project…",
        action: Action::OpenProject,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Recent projects…",
        action: Action::OpenRecentProjects,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Browse files…",
        action: Action::OpenBrowse,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Go to heading…",
        action: Action::OpenOutline,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Spell suggestions…",
        action: Action::OpenSpellSuggest,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Version history…",
        action: Action::OpenHistory,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Compare with version…",
        action: Action::CompareVersion,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Clean unused assets…",
        action: Action::OpenAssetClean,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Keep version…",
        action: Action::KeepVersion,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Last file",
        action: Action::LastBuffer,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "New document",
        action: Action::NewDocument,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Move…",
        action: Action::MoveFile,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Rename note…",
        action: Action::OpenRenameNote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Duplicate note",
        action: Action::DuplicateNote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // FINISH FILE: the emacsclient "server-edit" convention — save, notify any daemon
    // `--wait` client, and switch to the previously-open file. The emacs `C-x #`
    // default is retired; Cmd-W is its native slot now (P5 of the keybinding
    // idiom audit — awl's closest analogue to "close the document": non-
    // destructive under stray muscle memory, since it saves rather than closes
    // anything). NATIVE-ONLY: the daemon handoff it notifies has no web analog.
    // See `crate::daemon`. (Action stays `FinishBuffer`.)
    Command {
        name: "Finish file",
        action: Action::FinishBuffer,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Follow link",
        action: Action::FollowLink,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Switch theme…",
        action: Action::OpenThemeMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Caret style…",
        action: Action::OpenCaretMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Dictionary…",
        action: Action::OpenDictionaryMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle spellcheck",
        action: Action::ToggleSpellcheck,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle caret style",
        action: Action::ToggleCaretMode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle page mode",
        action: Action::TogglePageMode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle writing nits",
        action: Action::ToggleWritingNits,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Widen page",
        action: Action::PageWider,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Narrow page",
        action: Action::PageNarrower,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Reset page width",
        action: Action::PageReset,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle debug",
        action: Action::ToggleDebug,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle outline",
        action: Action::ToggleOutline,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // FOLD SECTION: collapse/expand the markdown section under the caret (view state,
    // never file content). Default Cmd-. / C-c C-f; rebindable via config `[keys]`.
    Command {
        name: "Fold section",
        action: Action::ToggleFold,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Collapse other sections",
        action: Action::CollapseOtherSections,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle typewriter scroll",
        action: Action::ToggleTypewriter,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Toggle menu bar",
        action: Action::ToggleMenuBar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "About",
        action: Action::About,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Credits",
        action: Action::OpenCredits,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Guide",
        action: Action::OpenGuide,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Lifetime stats",
        action: Action::LifetimeStats,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Writing streaks",
        action: Action::WritingStreaks,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    // LINE ENDINGS: toggle the active file's on-disk ending (LF <-> CRLF). No default
    // chord — the palette IS its entry point (a rare command, like Settings/About); a
    // real `Action` (`ConvertLineEndings`), independently rebindable via `[keys]`.
    Command {
        name: "Line endings…",
        action: Action::ConvertLineEndings,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // ALIGN TABLE: re-pad the GFM table under the caret so its `|` line up (source
    // alignment, never a drawn grid). No default chord — the palette IS its entry
    // point (like Settings/About); a real `Action`, independently rebindable.
    Command {
        name: "Align table",
        action: Action::AlignTable,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Insert Date",
        action: Action::InsertDate,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // REPORT A PROBLEM: compose a mailto: link to the maintainer, with the
    // newest local crash log's path attached-by-name if one exists (never its
    // content — the crash-visibility privacy law). No default chord — the
    // palette IS its entry point (like Settings/About/Align table); a real
    // `Action`, independently rebindable via `[keys]`. `native_only: false` —
    // available on the web build too (the mailto composition is pure and
    // platform-agnostic; only the crash-log path lookup is native-only). See
    // `crashlog.rs`.
    Command {
        name: "Report a Problem",
        action: Action::ReportProblem,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Download file",
        action: Action::DownloadFile,
        native: "",
        emacs: "",
        native_only: false,
        web_only: true,
    },
    // CHECK FOR UPDATES: never a network fetch — records a LOCAL "last checked"
    // marker (best-effort, `updates::record_checked`) then hands off to the OS
    // browser at the site's own `/check?v=…` page, which does the actual version
    // comparison against its own `version.json` (see `updates.rs`). No default
    // chord — the palette IS its entry point (like Report a Problem/About). Uses
    // the SAME `Effect::FollowLink`-style OS-handoff seam `App::follow_link`
    // already provides. `native_only: true` — the web build updates by
    // deploy/refresh, so "checking" is meaningless there.
    Command {
        name: "Check for Updates",
        action: Action::CheckForUpdates,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Blockquote",
        action: Action::ToggleBlockquote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Bullet list",
        action: Action::ToggleBulletList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Numbered list",
        action: Action::ToggleNumberedList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Task list",
        action: Action::ToggleTaskList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Heading",
        action: Action::ToggleHeading,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Cycle heading",
        action: Action::HeadingCycle,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Code block",
        action: Action::ToggleCodeBlock,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Bold",
        action: Action::Bold,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Italic",
        action: Action::Italic,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Inline code",
        action: Action::InlineCode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Highlight",
        action: Action::Highlight,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Strikethrough",
        action: Action::Strikethrough,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Export as Word…",
        action: Action::ExportWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Export as HTML…",
        action: Action::ExportHtml,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Export as PDF…",
        action: Action::ExportPdf,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Insert link…",
        action: Action::InsertLink,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Save",
        action: Action::Save,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Quit",
        action: Action::Quit,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
    },
    Command {
        name: "Search forward",
        action: Action::SearchForward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Search backward",
        action: Action::SearchBackward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Find and replace…",
        action: Action::OpenReplace,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Undo",
        action: Action::Undo,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Redo",
        action: Action::Redo,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Copy",
        action: Action::CopyRegion,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Cut",
        action: Action::KillRegion,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Paste",
        action: Action::Yank,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Select all",
        action: Action::SelectAll,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Zoom in",
        action: Action::ZoomIn,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Zoom out",
        action: Action::ZoomOut,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Reset zoom",
        action: Action::ZoomReset,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // MOTION COMMANDS (user-decided 2026-07-10, superseding the original all-motions
    // exclusion — see the module doc): the curated NAVIGATION motions are catalog rows
    // so they show in Cmd-P + the Keybindings rebind menu and are REBINDABLE via
    // `[keys]`. The concrete ask this serves: reclaiming the retired Option-letter
    // word motion — `forward_word = ["M-Right", "M-f"]` / `backward_word =
    // ["M-Left", "M-b"]` — which macOS reserves for typing by DEFAULT (the platform
    // rule that retired the M-letter layer) but a config line may deliberately opt
    // back in. Each row shows its REAL default chord (both slots fire; a config
    // override is ADDITIVE); the emacs slots left empty by that retirement stay
    // empty for the user to fill — never re-shipped. Line start/end keep their
    // surviving bare-control second slots (C-a / C-e), now visible + teachable.
    Command {
        name: "Forward word",
        action: Action::ForwardWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Backward word",
        action: Action::BackwardWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Line start",
        action: Action::LineStart,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Line end",
        action: Action::LineEnd,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Document start",
        action: Action::BufferStart,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Document end",
        action: Action::BufferEnd,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Forward char",
        action: Action::ForwardChar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Backward char",
        action: Action::BackwardChar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Next line",
        action: Action::NextLine,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Previous line",
        action: Action::PreviousLine,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    // WORD-DELETE, the mutating siblings of the word MOTIONS above — catalog rows
    // so `[keys]` can reach them (`delete_word_forward = "M-d"` reclaims the
    // classic emacs kill-word; `delete_word_backward = "M-Backspace"`). Both slots
    // are EMPTY by default: the SURVIVING default chords (⌥⌫ back, ⌥Delete / C-⌫ /
    // C-Delete forward) are dispatched by `keymap::resolve_named`'s static
    // NamedKey arms, NOT a catalog chord — the SAME split the plain char/line
    // motions above use (arrows fire from static arms; the catalog row exists only
    // to show in Cmd-P + be rebindable). The retired M-letter emacs slot stays
    // empty for the user to fill, never re-shipped (the word-motion precedent).
    Command {
        name: "Delete word forward",
        action: Action::DeleteWordForward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Delete word backward",
        action: Action::DeleteWordBackward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Settings…",
        action: Action::OpenSettingsMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
    Command {
        name: "Keybindings…",
        action: Action::OpenKeybindings,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
    },
];

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
    let reserved = platform == Platform::Web
        && crate::webreserved::is_reserved(&resolved_native(c, convention), convention);
    if reserved {
        match web_alternate_for(c, convention) {
            Some(alt) => match convention {
                Convention::Mac => crate::keyspec::mac_glyph_chord(alt),
                Convention::Linux => crate::keyspec::linux_glyph_chord(alt),
            },
            None => String::new(),
        }
    } else {
        resolved_native_label(c, convention)
    }
}

const WEB_ALTERNATE: &[(&str, &str, &str)] = &[
    ("New document", "C-j", "M-n"),
    ("Switch theme…", "C-t", "M-t"),
];

fn web_alternate_for(c: &Command, convention: Convention) -> Option<&'static str> {
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

// ── The command palette's FACETING scheme (All · File · Edit · View · Recent) ──
//
// The Cmd-P palette is a faceting picker (see `crate::facets`): ←/→ regroup the flat
// catalog under a lens. File / Edit / View mirror the macOS menu bar's grouping;
// Recent lists the most-recently-run commands.
//
// SINGLE-OWNER NOTE (menu section): the task calls for reusing `menu.rs`'s section
// table so there is no second hand-maintained category map. `menu.rs` is, however,
// `#![cfg(target_os = "macos")]` — its `SECTIONS` cannot be referenced from this
// CROSS-PLATFORM palette code. So the SEMANTIC owner of "which menu section a command
// belongs to" lives HERE, in [`menu_section`] (compiled on every target), and the
// macOS `menu.rs` is checked AGAINST it by a drift-guard test
// (`menu::tests::routed_sections_match_command_section`), so the menu's File/Edit/View
// arrays and this owner can never silently disagree — one source of truth, guarded.

const FILE_COMMANDS: &[&str] = &[
    "New document",
    "Browse files…",
    "Switch project…",
    "Recent projects…",
    "Save",
    "Finish file",
    "Export as PDF…",
    "Export as Word…",
    "Export as HTML…",
];
const EDIT_COMMANDS: &[&str] = &["Undo", "Redo", "Cut", "Copy", "Paste", "Select all"];
const VIEW_COMMANDS: &[&str] = &[
    "Toggle page mode",
    "Switch theme…",
    "Zoom in",
    "Zoom out",
    "Reset zoom",
    "Toggle debug",
];

/// The menu SECTION (`"File"` / `"Edit"` / `"View"`) command `name` sits under, or
/// `None` for a command in no menu section (the App-menu About/Quit, or any command
/// not surfaced in the menu bar at all). The SINGLE owner of this mapping, consulted
/// by both the palette's File/Edit/View lenses (every platform) and the macOS menu's
/// own drift-guard test — see the module note above.
pub fn menu_section(name: &str) -> Option<&'static str> {
    if FILE_COMMANDS.contains(&name) {
        Some("File")
    } else if EDIT_COMMANDS.contains(&name) {
        Some("Edit")
    } else if VIEW_COMMANDS.contains(&name) {
        Some("View")
    } else {
        None
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
mod tests {
    use super::*;

    #[test]
    fn every_defaults_toml_slug_names_a_real_catalog_command() {
        for slug_in_file in crate::keymap_defaults::command_defaults().keys() {
            assert!(
                COMMAND_SEED.iter().any(|c| &slug(c.name) == slug_in_file),
                "assets/keymap-defaults.toml names {slug_in_file:?}, which is not a commands::COMMAND_SEED slug"
            );
        }
    }

    #[test]
    fn every_catalog_command_appears_in_the_defaults_toml_or_is_unbound() {
        let defaults = crate::keymap_defaults::command_defaults();
        for c in COMMAND_SEED.iter() {
            assert!(
                defaults.contains_key(&slug(c.name)),
                "{:?} (slug {:?}) has no entry in assets/keymap-defaults.toml — every catalog \
                 command must appear there, even if unbound (both slots empty)",
                c.name,
                slug(c.name)
            );
        }
    }

    #[test]
    fn defaults_toml_has_no_stale_slugs_and_no_duplicates() {
        let defaults = crate::keymap_defaults::command_defaults();
        assert_eq!(
            defaults.len(),
            COMMAND_SEED.len(),
            "assets/keymap-defaults.toml's entry count must equal the catalog's — an orphaned \
             or duplicated slug would slip past the pure set-membership checks alone"
        );
    }

    #[test]
    fn commands_splices_the_embedded_defaults_verbatim() {
        // THE SINGLE-SOURCE LAW, checked directly: `COMMANDS[i].native`/`.emacs`
        // is EXACTLY what `assets/keymap-defaults.toml` names for that command's
        // slug (never a residual literal from `COMMAND_SEED`, which carries only
        // `""` placeholders in both slots by construction).
        let defaults = crate::keymap_defaults::command_defaults();
        for c in COMMANDS.iter() {
            let (native, emacs) = defaults.get(&slug(c.name)).cloned().unwrap_or_default();
            assert_eq!(
                c.native, native,
                "{:?}'s native slot must come from the embedded defaults",
                c.name
            );
            assert_eq!(
                c.emacs, emacs,
                "{:?}'s emacs slot must come from the embedded defaults",
                c.name
            );
        }
    }

    #[test]
    fn command_seed_itself_carries_no_residual_chord_literals() {
        // A belt-and-suspenders structural check: `COMMAND_SEED`'s own
        // `native`/`emacs` fields (never read by anything but the `COMMANDS`
        // splice above) must stay blank placeholders — a stray literal chord
        // reintroduced there would silently be DISCARDED by the splice (which
        // always overwrites both fields), so this catches the authoring mistake
        // even though it would otherwise have zero runtime effect.
        for c in COMMAND_SEED.iter() {
            assert_eq!(
                c.native, "",
                "{:?}: COMMAND_SEED must not carry a literal native chord",
                c.name
            );
            assert_eq!(
                c.emacs, "",
                "{:?}: COMMAND_SEED must not carry a literal emacs chord",
                c.name
            );
        }
    }

    #[test]
    fn catalog_non_empty_and_named() {
        assert!(
            !COMMANDS.is_empty(),
            "the command catalog must list commands"
        );
        for c in COMMANDS.iter() {
            assert!(!c.name.trim().is_empty(), "command needs a display name");
        }
        const PALETTE_ONLY: &[&str] = &[
            "Keybindings…",
            "Caret style…",
            "Dictionary…",
            "Toggle spellcheck",
            "Toggle writing nits",
            "Reset page width",
            "About",
            "Credits",
            "Guide",
            "Lifetime stats",
            "Writing streaks",
            "Line endings…",
            "Align table",
            "Report a Problem",
            "Download file",
            "Check for Updates",
            "Recent projects…",
            "Go to heading…",
            "Toggle typewriter scroll",
            "Toggle menu bar",
            "Keep version…",
            "Clean unused assets…",
            "Compare with version…",
            "Browse files…",
            "Move…",
            "Rename note…",
            "Duplicate note",
            "Toggle page mode",
            "Toggle caret style",
            "Widen page",
            "Narrow page",
            "Toggle debug",
            "Delete word forward",
            "Delete word backward",
            "Blockquote",
            "Bullet list",
            "Numbered list",
            "Heading",
            "Cycle heading",
            "Code block",
            "Highlight",
            "Strikethrough",
            "Export as Word…",
            "Export as HTML…",
            "Export as PDF…",
        ];
        for c in COMMANDS.iter() {
            if !PALETTE_ONLY.contains(&c.name) {
                assert!(
                    !join_slots(c.native, c.emacs).is_empty(),
                    "command {} needs at least one binding slot",
                    c.name
                );
            }
        }
        assert_eq!(names().len(), COMMANDS.len());
        assert_eq!(bindings().len(), COMMANDS.len());
    }

    #[test]
    fn every_popover_button_fires_a_catalog_command() {
        for &b in crate::popover::ALL {
            let action = b.action();
            assert!(
                COMMANDS.iter().any(|c| c.action == action),
                "format popover button {b:?} fires {action:?}, which is not a catalog \
                 command — every popover button must route through an existing catalog Action"
            );
        }
    }

    #[test]
    fn command_facets_land_on_all_home_then_group_by_menu_section() {
        assert_eq!(COMMAND_FACETS.strip[0].id, "all");
        assert!(COMMAND_FACETS.strip[0].sections.is_empty());
        let ids: Vec<&str> = COMMAND_FACETS.strip.iter().map(|f| f.id).collect();
        assert_eq!(ids, vec!["all", "file", "edit", "view", "recent"]);
    }

    #[test]
    fn menu_section_buckets_known_commands() {
        assert_eq!(menu_section("Save"), Some("File"));
        assert_eq!(menu_section("New document"), Some("File"));
        assert_eq!(menu_section("Export as PDF…"), Some("File"));
        assert_eq!(menu_section("Export as Word…"), Some("File"));
        assert_eq!(menu_section("Export as HTML…"), Some("File"));
        assert_eq!(menu_section("Copy"), Some("Edit"));
        assert_eq!(menu_section("Select all"), Some("Edit"));
        assert_eq!(menu_section("Switch theme…"), Some("View"));
        assert_eq!(menu_section("Toggle debug"), Some("View"));
        assert_eq!(menu_section("Quit"), None);
        assert_eq!(menu_section("About"), None);
        assert_eq!(menu_section("Settings"), None);
        for name in FILE_COMMANDS
            .iter()
            .chain(EDIT_COMMANDS)
            .chain(VIEW_COMMANDS)
        {
            assert!(
                COMMANDS.iter().any(|c| &c.name == name),
                "menu-section name {name:?} is not a catalog command"
            );
        }
    }

    #[test]
    fn command_bucket_routes_each_lens() {
        assert_eq!(command_bucket(FacetItem::new("Save"), 1), Some("File"));
        assert_eq!(command_bucket(FacetItem::new("Copy"), 1), None); // Edit, not File
        assert_eq!(command_bucket(FacetItem::new("Copy"), 2), Some("Edit"));
        assert_eq!(
            command_bucket(FacetItem::new("Switch theme…"), 3),
            Some("View")
        );
        let mut recent = FacetItem::new("Undo");
        recent.recent = true;
        assert_eq!(command_bucket(recent, 4), Some("Recent"));
        assert_eq!(command_bucket(FacetItem::new("Undo"), 4), None); // not flagged
        // The All home (index 0) never groups.
        assert_eq!(command_bucket(FacetItem::new("Save"), 0), None);
    }

    #[test]
    fn recent_mru_records_newest_first_deduped_and_capped() {
        // RECENT is a process-wide global WRITER — take the ONE reentrant guard
        // so a parallel test can't interleave its clear/record/read (the
        // CLAUDE.md flake tripwire: every `cfg(test)` global writer acquires
        // `testlock::serial()`; with one lock there's no order to invert).
        let _l = crate::testlock::serial();
        clear_recent();
        assert!(recent_indices().is_empty(), "fresh process starts empty");
        record_recent(&Action::Undo);
        record_recent(&Action::Redo);
        record_recent(&Action::Undo); // re-run moves it to front, no dup
        let undo = COMMANDS
            .iter()
            .position(|c| c.action == Action::Undo)
            .unwrap();
        let redo = COMMANDS
            .iter()
            .position(|c| c.action == Action::Redo)
            .unwrap();
        assert_eq!(recent_indices(), vec![undo, redo]);
        clear_recent(); // leave no residue for other tests reading the global
    }

    #[test]
    fn action_for_name_matches_label_and_slug() {
        assert_eq!(action_for_name("Switch theme"), Some(Action::OpenThemeMenu));
        assert_eq!(action_for_name("switch_theme"), Some(Action::OpenThemeMenu));
        assert_eq!(action_for_name("go_to_file"), Some(Action::OpenGoto));
        assert_eq!(action_for_name("settings"), Some(Action::OpenSettingsMenu));
        assert_eq!(action_for_name("Toggle debug"), Some(Action::ToggleDebug));
        assert_eq!(action_for_name("toggle_debug"), Some(Action::ToggleDebug));
        assert_eq!(
            action_for_name("Toggle outline"),
            Some(Action::ToggleOutline)
        );
        assert_eq!(
            action_for_name("toggle_outline"),
            Some(Action::ToggleOutline)
        );
        assert_eq!(
            action_for_name("Toggle spellcheck"),
            Some(Action::ToggleSpellcheck)
        );
        assert_eq!(
            action_for_name("toggle_spellcheck"),
            Some(Action::ToggleSpellcheck)
        );
        // The held stats HUD is NOT a palette command — it is a momentary HOLD-to-peek, so
        // a discrete selection (with no key-release to dismiss it) would leave it stuck on.
        // It is summoned ONLY by the held Option-Cmd-I chord (`keymap.rs`), never
        // the catalog.
        assert_eq!(action_for_name("Stats HUD"), None);
        assert_eq!(action_for_name("stats_hud"), None);
        assert_eq!(action_for_name("nope"), None);
    }

    /// LAW (item 76): the "Notes"/two-desk-flip command is COMPLETELY gone —
    /// no catalog row named "Notes", no `Action`/`Effect` variant reachable
    /// through the rebinder by that name or the old `notes` slug. A future
    /// command literally named "Notes" (unlikely, but the point of a
    /// no-wildcard sweep is to never assume) would need a NEW, deliberate
    /// entry here — this test is grep-forced, not name-coincidental, since it
    /// also asserts the retired slug resolves to nothing.
    #[test]
    fn notes_project_flip_command_and_slug_are_fully_retired() {
        assert!(
            !COMMANDS.iter().any(|c| c.name == "Notes"),
            "no catalog row is named \"Notes\" (the retired two-desk flip)"
        );
        assert_eq!(action_for_name("Notes"), None);
        assert_eq!(
            action_for_name("notes"),
            None,
            "the retired [keys] rebind slug resolves to nothing"
        );
    }

    #[test]
    fn a_trailing_ellipsis_never_forks_a_config_key() {
        // THE ELLIPSIS GATE: the `…` picker suffix is DISPLAY-ONLY — `slug` strips it,
        // so a command shown as "Switch theme…" keys under exactly `switch_theme`, the
        // SAME key a `[keys]` entry or the menu-routing table derives. This law pins
        // that a `…` can never fork a second config key.
        for c in COMMANDS.iter() {
            let s = slug(c.name);
            assert!(
                !s.contains('…'),
                "{}: slug must not carry the ellipsis: {s:?}",
                c.name
            );
            let bare = c.name.trim_end_matches('…').trim();
            assert_eq!(
                slug(bare),
                s,
                "{}: bare and suffixed forms must slug the same",
                c.name
            );
            assert_eq!(
                action_for_name(c.name),
                Some(c.action.clone()),
                "{}: suffixed rebind",
                c.name
            );
            assert_eq!(
                action_for_name(bare),
                Some(c.action.clone()),
                "{}: bare rebind",
                c.name
            );
        }
        assert_eq!(slug("Switch theme…"), "switch_theme");
        assert_eq!(slug("Switch theme"), "switch_theme");
        assert_eq!(
            action_for_name("switch_theme…"),
            Some(Action::OpenThemeMenu)
        );
    }

    /// CONVENTION-PARAMETRIC glyph helper for these two tests: glyphify a literal
    /// chord SPEC (an override value, taken literally — never Cmd→Ctrl
    /// translated, per `effective_binding_for`'s own doc) through the SAME two
    /// pure resolvers it calls, for whichever convention is ambient.
    fn glyph(spec: &str) -> String {
        match Convention::current() {
            Convention::Mac => crate::keyspec::mac_glyph_chord(spec),
            Convention::Linux => crate::keyspec::linux_glyph_chord(spec),
        }
    }

    fn label_for(name: &str) -> String {
        let c = COMMANDS.iter().find(|c| c.name == name).unwrap();
        resolved_native_label(c, Convention::current())
    }

    #[test]
    fn effective_bindings_reflect_overrides() {
        // No config: effective == default labels — a MAC-ONLY invariant.
        // `bindings()`/`join_slots` is explicitly documented as "the Mac
        // baseline" (always mac glyphs, never convention-resolved), while
        // `effective_bindings` IS convention-resolved (`Convention::current()`
        // via `effective_binding_for`) — so the two agree only when the ambient
        // convention actually IS Mac; under Linux they correctly diverge (Ctrl
        // word labels vs. the mac-glyph baseline) BY DESIGN.
        if Convention::current() == Convention::Mac {
            assert_eq!(effective_bindings(&[], &[]), bindings());
        }
        let keys = vec![("switch_theme".to_string(), vec!["C-t".to_string()])];
        let eff = effective_bindings(&keys, &[]);
        let i = COMMANDS
            .iter()
            .position(|c| c.name == "Switch theme…")
            .unwrap();
        assert_eq!(eff[i], glyph("C-t"));
        let bad = vec![("switch_theme".to_string(), vec!["C-frobnicate".to_string()])];
        let eff = effective_bindings(&bad, &[]);
        assert_eq!(eff[i], label_for("Switch theme…"));
    }

    #[test]
    fn effective_bindings_show_both_slots() {
        let i = COMMANDS.iter().position(|c| c.name == "Save").unwrap();
        assert_eq!(bindings()[i], "⌘S");
        let z = COMMANDS.iter().position(|c| c.name == "Zoom in").unwrap();
        assert_eq!(bindings()[z], "⌘=");
        let g = COMMANDS
            .iter()
            .position(|c| c.name == "Go to file…")
            .unwrap();
        assert_eq!(bindings()[g], "⌘O");
        let cut = COMMANDS.iter().position(|c| c.name == "Cut").unwrap();
        assert_eq!(bindings()[cut], "⌘X · C-w");
        let s = COMMANDS.iter().position(|c| c.name == "Settings…").unwrap();
        assert_eq!(bindings()[s], "⌘,");
        let keys = vec![(
            "save".to_string(),
            vec!["Cmd-S".to_string(), "C-x C-s".to_string()],
        )];
        assert_eq!(
            effective_bindings(&keys, &[])[i],
            format!("{} · C-x C-s", glyph("Cmd-S"))
        );
        let mixed = vec![(
            "save".to_string(),
            vec!["Cmd-S".to_string(), "C-frobnicate".to_string()],
        )];
        assert_eq!(effective_bindings(&mixed, &[])[i], glyph("Cmd-S"));
    }

    #[test]
    fn settings_command_present() {
        assert!(
            COMMANDS
                .iter()
                .any(|c| c.action == Action::OpenSettingsMenu)
        );
    }

    #[test]
    fn line_endings_command_present_and_rebindable() {
        let c = COMMANDS
            .iter()
            .find(|c| c.name == "Line endings…")
            .expect("Line endings… must be in the catalog");
        assert_eq!(c.native, "");
        assert_eq!(c.emacs, "");
        assert_eq!(c.action, Action::ConvertLineEndings);
        assert_eq!(
            action_for_name("Line endings…"),
            Some(Action::ConvertLineEndings)
        );
        assert_eq!(
            action_for_name("line_endings"),
            Some(Action::ConvertLineEndings)
        );
    }

    #[test]
    fn follow_link_command_present_and_rebindable() {
        let c = COMMANDS
            .iter()
            .find(|c| c.name == "Follow link")
            .expect("Follow link must be in the catalog");
        assert_eq!(c.native, "");
        assert_eq!(c.emacs, "C-c C-o");
        assert_eq!(c.action, Action::FollowLink);
        assert_eq!(action_for_name("Follow link"), Some(Action::FollowLink));
        assert_eq!(action_for_name("follow_link"), Some(Action::FollowLink));
        // The default `C-c C-o` chord parses AND resolves to FollowLink through a
        // fresh MAC-convention keymap (the C-c prefix path) — the catalog/keymap
        // agreement sweep relies on this, pinned here explicitly too. Mac-pinned
        // deliberately: under `Convention::Linux`, bare Ctrl-C is displaced to
        // native Copy (`LINUX_DISPLACED_LETTERS` includes 'c'), so the `C-c`
        // prefix never arms there — that displacement is its own contract, see
        // `keymap.rs`'s collision table doc.
        assert!(crate::keymap::parse_binding("C-c C-o").is_ok());
        assert_eq!(
            resolve_chord_under("C-c C-o", Convention::Mac),
            Action::FollowLink
        );
    }

    #[test]
    fn report_problem_command_present_and_rebindable() {
        let c = COMMANDS
            .iter()
            .find(|c| c.name == "Report a Problem")
            .expect("Report a Problem must be in the catalog");
        assert_eq!(c.native, "");
        assert_eq!(c.emacs, "");
        assert_eq!(c.action, Action::ReportProblem);
        assert!(
            !c.native_only,
            "Report a Problem must be available on the web build too"
        );
        assert_eq!(
            action_for_name("Report a Problem"),
            Some(Action::ReportProblem)
        );
        assert_eq!(
            action_for_name("report_a_problem"),
            Some(Action::ReportProblem)
        );
    }

    #[test]
    fn check_for_updates_command_present_rebindable_and_native_only() {
        // "Check for Updates" is a real palette command (no default chord, like
        // Report a Problem/Settings/About) backed by `Action::CheckForUpdates`,
        // `native_only: true` (the web build updates by deploy, so a "check"
        // command is meaningless there — it must NOT appear in the web view),
        // and independently rebindable via `[keys] check_for_updates`.
        let c = COMMANDS
            .iter()
            .find(|c| c.name == "Check for Updates")
            .expect("Check for Updates must be in the catalog");
        assert_eq!(c.native, "");
        assert_eq!(c.emacs, "");
        assert_eq!(c.action, Action::CheckForUpdates);
        assert!(
            c.native_only,
            "Check for Updates must be hidden on the web build"
        );
        assert!(!c.available_on(Platform::Web));
        assert!(c.available_on(Platform::Native));
        assert_eq!(
            action_for_name("Check for Updates"),
            Some(Action::CheckForUpdates)
        );
        assert_eq!(
            action_for_name("check_for_updates"),
            Some(Action::CheckForUpdates)
        );
    }

    #[test]
    fn toggle_writing_nits_command_present_and_rebindable() {
        let c = COMMANDS
            .iter()
            .find(|c| c.name == "Toggle writing nits")
            .expect("the Toggle writing nits command must be in the catalog");
        assert_eq!(c.native, "");
        assert_eq!(c.emacs, "");
        assert_eq!(c.action, Action::ToggleWritingNits);
        assert_eq!(
            action_for_name("Toggle writing nits"),
            Some(Action::ToggleWritingNits)
        );
        assert_eq!(
            action_for_name("toggle_writing_nits"),
            Some(Action::ToggleWritingNits)
        );
    }

    #[test]
    fn clipboard_and_select_all_in_catalog_with_real_bindings() {
        let find = |name: &str| COMMANDS.iter().find(|c| c.name == name).unwrap();
        let copy = find("Copy");
        assert_eq!(copy.action, Action::CopyRegion);
        assert_eq!((copy.native, copy.emacs), ("Cmd-C", ""));
        let cut = find("Cut");
        assert_eq!(cut.action, Action::KillRegion);
        assert_eq!((cut.native, cut.emacs), ("Cmd-X", "C-w"));
        let paste = find("Paste");
        assert_eq!(paste.action, Action::Yank);
        assert_eq!((paste.native, paste.emacs), ("Cmd-V", "C-y"));
        let all = find("Select all");
        assert_eq!(all.action, Action::SelectAll);
        assert_eq!((all.native, all.emacs), ("Cmd-A", ""));
        assert_eq!(action_for_name("copy"), Some(Action::CopyRegion));
        assert_eq!(action_for_name("select_all"), Some(Action::SelectAll));
    }

    #[test]
    fn keybindings_command_present_and_rebindable() {
        assert!(COMMANDS.iter().any(|c| c.action == Action::OpenKeybindings));
        assert_eq!(
            action_for_name("Keybindings"),
            Some(Action::OpenKeybindings)
        );
        assert_eq!(
            action_for_name("keybindings"),
            Some(Action::OpenKeybindings)
        );
    }

    #[test]
    fn version_history_command_present_and_rebindable() {
        assert!(COMMANDS.iter().any(|c| c.action == Action::OpenHistory));
        assert_eq!(
            action_for_name("Version history…"),
            Some(Action::OpenHistory)
        );
        assert_eq!(
            action_for_name("version_history"),
            Some(Action::OpenHistory)
        );
        let cmd = COMMANDS
            .iter()
            .find(|c| c.action == Action::OpenHistory)
            .unwrap();
        assert_eq!(cmd.native, "Cmd-S-h");
    }

    #[test]
    fn keep_version_command_present_named_and_rebindable() {
        assert!(COMMANDS.iter().any(|c| c.action == Action::KeepVersion));
        assert_eq!(action_for_name("Keep version…"), Some(Action::KeepVersion));
        assert_eq!(action_for_name("Keep version"), Some(Action::KeepVersion));
        assert_eq!(action_for_name("keep_version"), Some(Action::KeepVersion));
        let cmd = COMMANDS
            .iter()
            .find(|c| c.action == Action::KeepVersion)
            .unwrap();
        assert_eq!(cmd.native, "", "palette-only — no default chord");
        assert_eq!(cmd.emacs, "");
    }

    #[test]
    fn binding_conflict_finds_canonical_clash() {
        assert_eq!(binding_conflict("C-s", "undo", &[]), Some("Search forward"));
        assert_eq!(
            binding_conflict("Ctrl-s", "undo", &[]),
            Some("Search forward")
        );
        assert_eq!(binding_conflict("C-s", "search_forward", &[]), None);
        assert_eq!(binding_conflict("C-j", "undo", &[]), None);
        let keys = vec![("save".to_string(), vec!["C-j".to_string()])];
        assert_eq!(binding_conflict("C-j", "undo", &keys), Some("Save"));
        // An unparseable spec never conflicts.
        assert_eq!(binding_conflict("C-frobnicate", "undo", &[]), None);
    }

    #[test]
    fn markdown_formatting_commands_are_all_present_named_and_rebindable() {
        let formatting: &[(&str, Action, &str)] = &[
            ("Blockquote", Action::ToggleBlockquote, ""),
            ("Bullet list", Action::ToggleBulletList, ""),
            ("Numbered list", Action::ToggleNumberedList, ""),
            ("Task list", Action::ToggleTaskList, "Cmd-S-l"),
            ("Heading", Action::ToggleHeading, ""),
            ("Code block", Action::ToggleCodeBlock, ""),
            ("Bold", Action::Bold, "Cmd-B"),
            ("Italic", Action::Italic, "Cmd-I"),
            ("Inline code", Action::InlineCode, "Cmd-E"),
            ("Highlight", Action::Highlight, ""),
            ("Strikethrough", Action::Strikethrough, ""),
        ];
        for (name, action, native) in formatting {
            let cmd = COMMANDS
                .iter()
                .find(|c| c.name == *name)
                .unwrap_or_else(|| panic!("formatting command {name:?} missing from catalog"));
            assert_eq!(&cmd.action, action, "{name}: catalog action");
            assert_eq!(cmd.native, *native, "{name}: native chord slot");
            assert_eq!(
                cmd.emacs, "",
                "{name}: emacs slot is left empty for the user"
            );
            assert_eq!(
                action_for_name(name),
                Some(action.clone()),
                "{name}: label rebind"
            );
            assert_eq!(
                action_for_name(&slug(name)),
                Some(action.clone()),
                "{name}: slug rebind"
            );
        }
        assert_eq!(binding_conflict("Cmd-B", "bold", &[]), None);
        assert_eq!(binding_conflict("Cmd-I", "italic", &[]), None);
        assert_eq!(binding_conflict("Cmd-E", "inline_code", &[]), None);
        assert_eq!(binding_conflict("Cmd-S-l", "task_list", &[]), None);
        let eff = effective_bindings(&[], &[]);
        let bold = COMMANDS.iter().position(|c| c.name == "Bold").unwrap();
        let ital = COMMANDS.iter().position(|c| c.name == "Italic").unwrap();
        let code = COMMANDS
            .iter()
            .position(|c| c.name == "Inline code")
            .unwrap();
        let task = COMMANDS.iter().position(|c| c.name == "Task list").unwrap();
        let convention = Convention::current();
        assert_eq!(
            eff[bold],
            resolved_native_label(&COMMANDS[bold], convention)
        );
        assert_eq!(
            eff[ital],
            resolved_native_label(&COMMANDS[ital], convention)
        );
        assert_eq!(
            eff[code],
            resolved_native_label(&COMMANDS[code], convention)
        );
        assert_eq!(
            eff[task],
            resolved_native_label(&COMMANDS[task], convention)
        );
    }

    #[test]
    fn links_v2_command_is_present_named_and_rebindable() {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.name == "Insert link…")
            .expect("Insert link… missing from catalog");
        assert_eq!(cmd.action, Action::InsertLink);
        assert_eq!(cmd.native, "Cmd-K");
        assert_eq!(cmd.emacs, "");
        assert!(!cmd.native_only, "Insert link… is available on web too");
        assert_eq!(action_for_name("Insert link…"), Some(Action::InsertLink));
        assert_eq!(
            action_for_name(&slug("Insert link…")),
            Some(Action::InsertLink)
        );
        assert_eq!(binding_conflict("Cmd-K", "insert_link", &[]), None);
    }

    fn resolve_chord_under(spec: &str, convention: Convention) -> Action {
        let mut km = crate::keymap::KeymapState::new_with_convention(convention);
        let mut last = Action::Ignore;
        for tok in spec.split_whitespace() {
            let (key, mods) = crate::keyspec::parse_chord(tok)
                .unwrap_or_else(|e| panic!("catalog chord {spec:?} failed to parse: {e}"));
            last = km.resolve(&key, &mods);
        }
        last
    }

    #[test]
    fn catalog_and_keymap_agree_on_every_default_chord() {
        // THE AGREEMENT SWEEP: the catalog's binding labels and the keymap's
        // dispatch are now SEEDED FROM ONE SOURCE (assets/keymap-defaults.toml).
        // On the chord-VALUE axis this loop is therefore a round-trip — it can
        // no longer catch a wrong default chord, because dispatch and
        // expectation read the same parse. What it STILL genuinely verifies is
        // the SEED-TO-DISPATCH ROUND-TRIP (every seeded slot actually reaches
        // `resolve` and fires its command, so `[keys]` can always address it)
        // PLUS the hand-written Linux POLICY layer below (translation, override,
        // displacement, keep) which is NOT seeded from the TOML. The VALUE
        // oracle — "this specific command resolves to this specific chord" — is
        // the checked-in literal snapshots
        // (`keymap::tests::mac_convention_is_byte_identical_to_the_pre_round_table`
        // and `keymap::tests::catalog_chord_snapshot_is_frozen`), NOT this sweep.
        //
        // CONVENTION-PROOF (per-convention, not just whichever is ambient):
        // `c.native` is always stored in MAC-LITERAL form ("Cmd-O") — under
        // `Convention::Linux` the chord that ACTUALLY fires is the one
        // `commands::resolved_native` computes (a translated/overridden Ctrl
        // chord, per `LINUX_NATIVE_OVERRIDE`/`translate_native_for_linux`), so the
        // native half is checked by resolving THAT translated chord under each
        // convention in turn — never the literal mac string against a Linux
        // keymap (which would never fire native_down at all, see
        // `KeymapState::native_down`'s Super-vs-Ctrl split). The emacs half is
        // OS-agnostic text ("C-s") and is checked directly under BOTH
        // conventions, EXCEPT where `keymap::linux_displaces_emacs_default` says
        // Linux's native layer displaces it (`LINUX_DISPLACED_LETTERS`) — that
        // displacement is its own exhaustively law-tested contract
        // (`keymap::tests::linux_collision_table_matches_the_documented_displaced_list`),
        // not something this sweep should re-assert. SYMMETRICALLY, the NATIVE
        // half skips a chord the DEFAULT (config-free) Linux keep-list holds
        // back (`keymap::linux_builtin_keep()` — Insert link's Ctrl-K, which
        // yields to kill-line out of the box; the insert-link-yields round) —
        // that non-firing is ITS own law-tested contract too
        // (`keymap::tests::out_of_the_box_linux_ctrl_k_is_kill_line_under_both_keymap_flavors`),
        // and the labels never advertise the chord there either
        // (`insert_link_has_no_visible_linux_binding_out_of_the_box_mac_shows_cmd_k`).
        let default_linux_keep = crate::config::Config::empty().effective_linux_keep();
        for c in COMMANDS.iter() {
            for convention in [Convention::Mac, Convention::Linux] {
                if !c.native.trim().is_empty() {
                    let resolved = resolved_native(c, convention);
                    let kept_back = convention == Convention::Linux
                        && crate::keymap::linux_keeps_chord(&default_linux_keep, &resolved);
                    if !resolved.trim().is_empty() && !kept_back {
                        assert!(
                            crate::keymap::parse_binding(&resolved).is_ok(),
                            "{}: {:?}'s resolved native chord {resolved:?} must parse via parse_binding",
                            c.name,
                            convention
                        );
                        assert_eq!(
                            resolve_chord_under(&resolved, convention),
                            c.action,
                            "{}: {:?}'s resolved native chord {resolved:?} must resolve to the catalog action",
                            c.name,
                            convention
                        );
                    }
                }
                if !c.emacs.trim().is_empty() {
                    assert!(
                        crate::keymap::parse_binding(c.emacs).is_ok(),
                        "{}: emacs default {:?} must parse via parse_binding",
                        c.name,
                        c.emacs
                    );
                    if convention == Convention::Linux
                        && crate::keymap::linux_displaces_emacs_default(c.emacs, &[])
                    {
                        continue; // displaced by native on Linux — covered by keymap.rs's own law test.
                    }
                    assert_eq!(
                        resolve_chord_under(c.emacs, convention),
                        c.action,
                        "{}: {:?}'s emacs default {:?} must resolve to the catalog action",
                        c.name,
                        convention,
                        c.emacs
                    );
                }
            }
            assert_eq!(
                action_for_name(&slug(c.name)),
                Some(c.action.clone()),
                "{}: slug round-trip through action_for_name",
                c.name
            );
        }
    }

    #[test]
    fn no_two_catalog_commands_share_a_default_chord() {
        // PAIRWISE default-chord conflicts, compared CANONICALLY through the same
        // `binding_conflict` the rebind menu gates on (so `Cmd-S` == `s-s`
        // spellings clash too). An INTENTIONALLY shared chord would be allow-
        // listed here as a (command, command) pair with a comment explaining the
        // share — today there are NONE, so the list is empty and every default
        // chord belongs to exactly one command.
        const INTENTIONALLY_SHARED: &[(&str, &str)] = &[];
        for c in COMMANDS.iter() {
            for chord in [c.native, c.emacs] {
                if chord.trim().is_empty() {
                    continue;
                }
                if let Some(other) = binding_conflict(chord, &slug(c.name), &[]) {
                    let allowlisted = INTENTIONALLY_SHARED.iter().any(|(a, b)| {
                        (*a == c.name && *b == other) || (*a == other && *b == c.name)
                    });
                    assert!(
                        allowlisted,
                        "default chord {chord:?} is bound to BOTH {:?} and {other:?} \
                         (not in the intentional-share allowlist)",
                        c.name
                    );
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn slug_for_action_and_has_native_chord_key_the_usage_ledger() {
        assert_eq!(
            slug_for_action(&Action::OpenGoto).as_deref(),
            Some("go_to_file")
        );
        assert_eq!(
            slug_for_action(&Action::OpenThemeMenu).as_deref(),
            Some("switch_theme")
        );
        assert_eq!(
            slug_for_action(&Action::ForwardChar),
            Some("forward_char".to_string())
        );
        assert_eq!(slug_for_action(&Action::InsertChar('x')), None);
        assert_eq!(slug_for_action(&Action::BeginPrefix), None);
        assert!(has_native_chord("go_to_file"), "Go to file… carries Cmd-O");
        assert!(has_native_chord("save"), "Save carries Cmd-S");
        assert!(
            has_native_chord("settings"),
            "Settings… now carries Cmd-, (P1)"
        );
        assert!(
            !has_native_chord("browse_files"),
            "Browse files… is palette-only"
        );
        assert!(!has_native_chord("about"), "About is palette-only");
        assert!(
            !has_native_chord("reset_page_width"),
            "Reset page width is palette-only"
        );
        assert!(!has_native_chord("no_such_command"), "unknown slug: false");
        assert!(has_native_chord(&slug_for_action(&Action::Save).unwrap()));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn peek_row_resolves_native_chord_and_name_or_none_for_palette_only() {
        assert_eq!(
            peek_row_for_slug("go_to_file"),
            Some(crate::peek::PeekRow {
                chord: label_for("Go to file…"),
                name: "Go to file".into()
            })
        );
        assert_eq!(
            peek_row_for_slug("switch_theme"),
            Some(crate::peek::PeekRow {
                chord: label_for("Switch theme…"),
                name: "Switch theme".into()
            })
        );
        // A palette-only command (no native chord to teach) → None, so it never
        // surfaces as a peek/footer row even if slow-door usage ranks it.
        assert_eq!(peek_row_for_slug("about"), None);
        assert_eq!(
            peek_row_for_slug("settings"),
            Some(crate::peek::PeekRow {
                chord: label_for("Settings…"),
                name: "Settings".into()
            })
        );
        assert_eq!(peek_row_for_slug("no_such_command"), None);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn catalog_motions_are_exactly_the_curated_navigation_set() {
        // THE MOTION SPLIT (user-decided 2026-07-10, superseding the original
        // all-motions exclusion; WIDENED by the emacs-hands-on-Linux round to the
        // last four bare-control nav motions — char forward/back, line up/down —
        // so `[keys]` can finally rebind C-f/C-b/C-n/C-p at all). Every motion
        // `Action::is_motion` names is now a catalog row (palette-visible +
        // rebindable); the split that remains is self-insertion, which never
        // enters the catalog. Kept as a NO-WILDCARD-style completeness sweep
        // (rather than deleting it now that the split is "all of them") so a
        // FUTURE motion added to `is_motion` without a matching catalog row still
        // fails this test loudly, exactly like before.
        const NAVIGATION_MOTIONS: &[Action] = &[
            Action::ForwardChar,
            Action::BackwardChar,
            Action::NextLine,
            Action::PreviousLine,
            Action::ForwardWord,
            Action::BackwardWord,
            Action::LineStart,
            Action::LineEnd,
            Action::BufferStart,
            Action::BufferEnd,
        ];
        for c in COMMANDS.iter() {
            if c.action.is_motion() {
                assert!(
                    NAVIGATION_MOTIONS.contains(&c.action),
                    "{}: a motion outside the curated navigation set entered the catalog",
                    c.name
                );
            }
            assert!(
                !matches!(c.action, Action::InsertChar(_)),
                "{} self-inserts; excluded",
                c.name
            );
        }
        for m in NAVIGATION_MOTIONS {
            assert!(
                COMMANDS.iter().any(|c| &c.action == m),
                "curated navigation motion {m:?} missing from the catalog"
            );
        }
        for m in NAVIGATION_MOTIONS {
            assert!(
                m.is_motion(),
                "{m:?} listed as a navigation motion but is_motion() is false"
            );
        }
    }

    #[test]
    fn motion_commands_are_all_present_named_and_rebindable() {
        let motions: &[(&str, Action, &str, &str)] = &[
            ("Forward word", Action::ForwardWord, "M-Right", ""),
            ("Backward word", Action::BackwardWord, "M-Left", ""),
            ("Line start", Action::LineStart, "Cmd-Left", "C-a"),
            ("Line end", Action::LineEnd, "Cmd-Right", "C-e"),
            ("Document start", Action::BufferStart, "Cmd-Up", ""),
            ("Document end", Action::BufferEnd, "Cmd-Down", ""),
        ];
        for (name, action, native, emacs) in motions {
            let cmd = COMMANDS
                .iter()
                .find(|c| c.name == *name)
                .unwrap_or_else(|| panic!("motion command {name:?} missing from catalog"));
            assert_eq!(&cmd.action, action, "{name}: catalog action");
            assert_eq!(cmd.native, *native, "{name}: native chord slot");
            assert_eq!(cmd.emacs, *emacs, "{name}: emacs chord slot");
            assert_eq!(
                action_for_name(name),
                Some(action.clone()),
                "{name}: label rebind"
            );
            assert_eq!(
                action_for_name(&slug(name)),
                Some(action.clone()),
                "{name}: slug rebind"
            );
        }
        for spec in ["M-f", "M-b"] {
            assert!(
                crate::keymap::parse_binding(spec).is_ok(),
                "{spec:?} must parse"
            );
        }
        assert_eq!(binding_conflict("M-f", "forward_word", &[]), None);
        assert_eq!(binding_conflict("M-b", "backward_word", &[]), None);
        let keys = vec![("forward_word".to_string(), vec!["M-f".to_string()])];
        let i = COMMANDS
            .iter()
            .position(|c| c.name == "Forward word")
            .unwrap();
        assert_eq!(effective_bindings(&keys, &[])[i], glyph("M-f"));
    }

    #[test]
    fn word_delete_commands_are_catalog_rows_and_rebindable() {
        let deletes: &[(&str, Action)] = &[
            ("Delete word forward", Action::DeleteWordForward),
            ("Delete word backward", Action::DeleteWordBackward),
        ];
        for (name, action) in deletes {
            let cmd = COMMANDS
                .iter()
                .find(|c| c.name == *name)
                .unwrap_or_else(|| panic!("word-delete command {name:?} missing from catalog"));
            assert_eq!(&cmd.action, action, "{name}: catalog action");
            assert_eq!(cmd.native, "", "{name}: native slot empty by default");
            assert_eq!(cmd.emacs, "", "{name}: emacs slot empty by default");
            assert_eq!(
                action_for_name(name),
                Some(action.clone()),
                "{name}: label rebind"
            );
            assert_eq!(
                action_for_name(&slug(name)),
                Some(action.clone()),
                "{name}: slug rebind"
            );
        }
        assert_eq!(
            action_for_name("delete_word_forward"),
            Some(Action::DeleteWordForward)
        );
        assert_eq!(
            action_for_name("delete_word_backward"),
            Some(Action::DeleteWordBackward)
        );
        assert!(
            crate::keymap::parse_binding("M-d").is_ok(),
            "M-d must parse"
        );
        assert_eq!(binding_conflict("M-d", "delete_word_forward", &[]), None);
        let keys = vec![("delete_word_forward".to_string(), vec!["M-d".to_string()])];
        let i = COMMANDS
            .iter()
            .position(|c| c.name == "Delete word forward")
            .unwrap();
        assert_eq!(effective_bindings(&keys, &[])[i], glyph("M-d"));
    }

    const HIDE_ON_WEB: &[&str] = &[
        "Quit",
        "Finish file",
        "Version history…",
        "Compare with version…",
        "Keep version…",
        "Lifetime stats",
        "Writing streaks",
        "Clean unused assets…",
        "Recent projects…",
        "Check for Updates",
        "Export as PDF…",
    ];

    const HIDE_ON_NATIVE: &[&str] = &["Download file"];

    #[test]
    fn hide_list_is_exactly_the_native_only_commands() {
        let flagged: std::collections::HashSet<&str> = COMMANDS
            .iter()
            .filter(|c| c.native_only)
            .map(|c| c.name)
            .collect();
        let listed: std::collections::HashSet<&str> = HIDE_ON_WEB.iter().copied().collect();
        assert_eq!(
            flagged, listed,
            "native_only flags and the hide list must match exactly"
        );
    }

    #[test]
    fn inverse_hide_list_is_exactly_the_web_only_commands() {
        let flagged: std::collections::HashSet<&str> = COMMANDS
            .iter()
            .filter(|c| c.web_only)
            .map(|c| c.name)
            .collect();
        let listed: std::collections::HashSet<&str> = HIDE_ON_NATIVE.iter().copied().collect();
        assert_eq!(
            flagged, listed,
            "web_only flags and the inverse hide list must match exactly"
        );
    }

    #[test]
    fn no_command_is_flagged_unavailable_on_both_platforms() {
        for c in COMMANDS.iter() {
            assert!(
                !(c.native_only && c.web_only),
                "{}: native_only and web_only can never both be true (available nowhere)",
                c.name
            );
        }
    }

    #[test]
    fn web_only_commands_are_unavailable_on_native_available_on_web() {
        for name in HIDE_ON_NATIVE {
            let c = COMMANDS
                .iter()
                .find(|c| &c.name == name)
                .unwrap_or_else(|| panic!("{name}: missing"));
            assert!(
                !c.available_on(Platform::Native),
                "{name}: must be hidden on native"
            );
            assert!(
                c.available_on(Platform::Web),
                "{name}: must stay available on web"
            );
        }
    }

    #[test]
    fn hide_listed_commands_are_unavailable_on_web_available_on_native() {
        for name in HIDE_ON_WEB {
            let c = COMMANDS
                .iter()
                .find(|c| &c.name == name)
                .unwrap_or_else(|| panic!("{name}: missing"));
            assert!(
                !c.available_on(Platform::Web),
                "{name}: must be hidden on web"
            );
            assert!(
                c.available_on(Platform::Native),
                "{name}: must stay available natively"
            );
        }
    }

    #[test]
    fn every_other_command_is_available_on_both_platforms() {
        for c in COMMANDS.iter() {
            if HIDE_ON_WEB.contains(&c.name) || HIDE_ON_NATIVE.contains(&c.name) {
                continue;
            }
            assert!(
                c.available_on(Platform::Web),
                "{}: unexpectedly hidden on web",
                c.name
            );
            assert!(
                c.available_on(Platform::Native),
                "{}: unexpectedly hidden on native",
                c.name
            );
        }
    }

    #[test]
    fn platform_current_is_native_under_a_native_test_binary() {
        // `cargo test` is never a wasm32 target, so `Platform::current()` reads
        // Native here — the compiled-platform door and the explicit-platform door
        // agree on THIS binary by construction.
        assert_eq!(Platform::current(), Platform::Native);
    }

    #[test]
    fn visible_on_native_drops_exactly_the_inverse_hide_list_and_nothing_else() {
        let native = visible_on(Platform::Native);
        assert_eq!(native.len(), COMMANDS.len() - HIDE_ON_NATIVE.len());
        // Order is otherwise preserved exactly (filtering, never reordering).
        let expected: Vec<&str> = COMMANDS
            .iter()
            .map(|c| c.name)
            .filter(|n| !HIDE_ON_NATIVE.contains(n))
            .collect();
        let actual: Vec<&str> = native.iter().map(|c| c.name).collect();
        assert_eq!(
            actual, expected,
            "native visible() must preserve catalog order exactly"
        );
        for name in HIDE_ON_NATIVE {
            assert!(
                !native.iter().any(|c| &c.name == name),
                "{name}: leaked into the native view"
            );
        }
        assert_eq!(visible().len(), visible_on(Platform::Native).len());
    }

    #[test]
    fn visible_on_web_drops_exactly_the_hide_list_and_nothing_else() {
        let web = visible_on(Platform::Web);
        assert_eq!(web.len(), COMMANDS.len() - HIDE_ON_WEB.len());
        for c in &web {
            assert!(
                !HIDE_ON_WEB.contains(&c.name),
                "{}: should have been hidden on web",
                c.name
            );
        }
        for name in HIDE_ON_WEB {
            assert!(
                !web.iter().any(|c| &c.name == name),
                "{name}: leaked into the web view"
            );
        }
        for name in HIDE_ON_NATIVE {
            assert!(
                web.iter().any(|c| &c.name == name),
                "{name}: missing from the web view"
            );
        }
    }

    /// INDEX-COHERENCE LAW for the filtered palette/rebind-menu corpus: for every
    /// row `i` in `visible()`, `visible_action_of(i)` / `visible_slug_of(i)` /
    /// `visible_name_of(i)` all name THAT SAME row's command — never a raw
    /// `COMMANDS[i]` (which would silently mis-map once rows are hidden). Checked on
    /// both platforms explicitly (`visible_on`), not just the native-compiled
    /// `visible()`, so the web-filtered corpus's own index coherence is pinned too.
    #[test]
    fn visible_corpus_index_coherence_holds_on_both_platforms() {
        for platform in [Platform::Native, Platform::Web] {
            let filtered = visible_on(platform);
            let names: Vec<String> = filtered.iter().map(|c| c.name.to_string()).collect();
            let actions: Vec<Action> = filtered.iter().map(|c| c.action.clone()).collect();
            for (i, (name, action)) in names.iter().zip(actions.iter()).enumerate() {
                let c = filtered[i];
                assert_eq!(
                    &c.name.to_string(),
                    name,
                    "row {i}: name must match its own filtered slot"
                );
                assert_eq!(
                    &c.action, action,
                    "row {i}: action must match its own filtered slot"
                );
            }
        }
        let corpus = visible();
        for (i, command) in corpus.iter().enumerate() {
            assert_eq!(
                visible_action_of(i),
                command.action,
                "row {i}: visible_action_of drift"
            );
            assert_eq!(
                visible_slug_of(i),
                slug(command.name),
                "row {i}: visible_slug_of drift"
            );
            assert_eq!(
                visible_name_of(i),
                corpus[i].name,
                "row {i}: visible_name_of drift"
            );
        }
    }

    #[test]
    fn visible_names_and_bindings_are_parallel_and_match_visible() {
        let corpus = visible();
        let names = visible_names();
        let binds = visible_effective_bindings(&[], &[]);
        assert_eq!(names.len(), corpus.len());
        assert_eq!(binds.len(), corpus.len());
        for (i, c) in corpus.iter().enumerate() {
            assert_eq!(names[i], c.name);
        }
    }

    #[test]
    fn visible_hidden_mask_gates_finish_buffer_on_the_live_waiter_fact_alone() {
        let corpus = visible();
        let idx = corpus
            .iter()
            .position(|c| c.action == Action::FinishBuffer)
            .expect("FinishBuffer is a real catalog row");

        let mask_no_waiter = visible_hidden_mask(false);
        assert_eq!(
            mask_no_waiter.len(),
            corpus.len(),
            "mask is parallel to visible()"
        );
        assert!(
            mask_no_waiter[idx],
            "FinishBuffer must be hidden with no waiter"
        );
        assert_eq!(
            mask_no_waiter.iter().filter(|&&h| h).count(),
            1,
            "exactly one row (FinishBuffer) is ever runtime-hidden"
        );

        let mask_waiting = visible_hidden_mask(true);
        assert!(
            !mask_waiting[idx],
            "FinishBuffer must show while a waiter is active"
        );
        assert!(
            mask_waiting.iter().all(|&h| !h),
            "no OTHER row is ever runtime-gated"
        );
    }

    #[test]
    fn action_available_gates_hidden_actions_only_on_web() {
        assert!(!action_available(&Action::Quit, Platform::Web));
        assert!(action_available(&Action::Quit, Platform::Native));
        assert!(!action_available(&Action::FinishBuffer, Platform::Web));
        assert!(action_available(&Action::OpenKeybindings, Platform::Web));
        assert!(action_available(&Action::Save, Platform::Web));
        assert!(action_available(&Action::Save, Platform::Native));
        assert!(action_available(&Action::ForwardChar, Platform::Web));
        assert!(action_available(&Action::InsertChar('x'), Platform::Web));
    }

    #[test]
    fn visible_recent_indices_drops_hidden_catalog_entries_and_translates_the_rest() {
        // RECENT is a process-wide global WRITER — take the ONE reentrant guard
        // (see the sibling test above; the CLAUDE.md flake tripwire).
        let _l = crate::testlock::serial();
        clear_recent();
        record_recent(&Action::Undo);
        record_recent(&Action::Quit); // a hidden-on-web command
        record_recent(&Action::Redo);
        let vis = visible_recent_indices();
        assert_eq!(vis.len(), 3);
        let corpus = visible();
        let redo_row = corpus
            .iter()
            .position(|c| c.action == Action::Redo)
            .unwrap();
        assert_eq!(vis[0], redo_row, "most-recent-first order preserved");
        clear_recent();
    }

    /// THE HARD LAW: on `Convention::Mac` + `Platform::Native` (a plain macOS
    /// native build) neither Tier 2 (web-reserved) nor Tier 3 (Linux-displaced)
    /// can ever fire, so [`join_slots_truthful`] must be BYTE-IDENTICAL to the
    /// pre-round `join_slots(c.native, c.emacs)` for EVERY catalog command.
    #[test]
    fn mac_native_label_truth_is_byte_identical_to_join_slots() {
        for c in COMMANDS.iter() {
            assert_eq!(
                join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
                join_slots(c.native, c.emacs),
                "{} diverged from the pre-round Mac-native label",
                c.name
            );
        }
    }

    #[test]
    fn web_reserved_native_chord_shows_its_web_alternate() {
        let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
        let switch_theme = COMMANDS.iter().find(|c| c.name == "Switch theme…").unwrap();
        for c in [new_document, switch_theme] {
            assert_eq!(
                c.emacs.trim(),
                "",
                "{} must have no emacs slot for this test's claim",
                c.name
            );
            for convention in [Convention::Mac, Convention::Linux] {
                let label = resolved_native_label_truthful(c, convention, Platform::Web);
                assert!(
                    !label.is_empty(),
                    "{}: web alternate must not be blank ({convention:?})",
                    c.name
                );
                assert_ne!(
                    label,
                    resolved_native_label(c, convention),
                    "{}: the web label must be the ALTERNATE, not the (reserved) native one",
                    c.name
                );
                assert_eq!(
                    join_slots_truthful(c, convention, Platform::Web, &[]),
                    label
                );
                assert_eq!(
                    resolved_native_label_truthful(c, convention, Platform::Native),
                    resolved_native_label(c, convention)
                );
            }
        }
    }

    #[test]
    fn web_alternate_labels_are_convention_keyed() {
        let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
        let switch_theme = COMMANDS.iter().find(|c| c.name == "Switch theme…").unwrap();
        assert_eq!(
            resolved_native_label_truthful(new_document, Convention::Mac, Platform::Web),
            "\u{2303}J"
        );
        assert_eq!(
            resolved_native_label_truthful(switch_theme, Convention::Mac, Platform::Web),
            "\u{2303}T"
        );
        assert_eq!(
            resolved_native_label_truthful(new_document, Convention::Linux, Platform::Web),
            "Alt+N"
        );
        assert_eq!(
            resolved_native_label_truthful(switch_theme, Convention::Linux, Platform::Web),
            "Alt+T"
        );
    }

    #[test]
    fn exactly_new_note_and_switch_theme_are_web_reserved_and_available() {
        let mut hit: Vec<&str> = COMMANDS
            .iter()
            .filter(|c| c.available_on(Platform::Web))
            .filter(|c| {
                [Convention::Mac, Convention::Linux]
                    .iter()
                    .any(|conv| crate::webreserved::is_reserved(&resolved_native(c, *conv), *conv))
            })
            .map(|c| c.name)
            .collect();
        hit.sort_unstable();
        assert_eq!(hit, vec!["New document", "Switch theme…"]);
    }

    #[test]
    fn web_alternate_keys_is_inert_on_native_and_populated_on_web() {
        assert_eq!(
            web_alternate_keys(&[], Convention::Mac, Platform::Native),
            Vec::new()
        );
        let mut on_web = web_alternate_keys(&[], Convention::Mac, Platform::Web);
        on_web.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            on_web,
            vec![
                ("new_document".to_string(), vec!["C-j".to_string()]),
                ("switch_theme".to_string(), vec!["C-t".to_string()])
            ]
        );
        let mut on_web_linux = web_alternate_keys(&[], Convention::Linux, Platform::Web);
        on_web_linux.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            on_web_linux,
            vec![
                ("new_document".to_string(), vec!["M-n".to_string()]),
                ("switch_theme".to_string(), vec!["M-t".to_string()])
            ]
        );
    }

    /// **Config still trumps everything:** a user `[keys]` entry for "New
    /// note" suppresses ITS web alternate entirely (the user's own chosen
    /// chord is never shadowed), while "Switch theme…"'s alternate — untouched
    /// by the user's config — still appears.
    #[test]
    fn web_alternate_keys_skips_a_command_the_user_has_already_rebound() {
        let existing = vec![("new_document".to_string(), vec!["C-x C-n".to_string()])];
        let on_web = web_alternate_keys(&existing, Convention::Mac, Platform::Web);
        assert!(
            !on_web.iter().any(|(name, _)| name == "new_document"),
            "user's own new_document rebind must not be shadowed"
        );
        assert!(
            on_web.iter().any(|(name, _)| name == "switch_theme"),
            "switch_theme's alternate is still added"
        );
    }

    #[test]
    fn web_alternate_keys_dispatch_the_real_action_on_web() {
        let keys = web_alternate_keys(&[], Convention::Mac, Platform::Web);
        let mut km = crate::keymap::KeymapState::with_overrides(&keys);
        let (key, mods) = crate::keyspec::parse_chord("C-j").expect("C-j parses");
        assert_eq!(km.resolve(&key, &mods), Action::NewDocument);
        let (key, mods) = crate::keyspec::parse_chord("C-t").expect("C-t parses");
        assert_eq!(km.resolve(&key, &mods), Action::OpenThemeMenu);
    }

    /// TIER 2, the fallback half: a SYNTHETIC command whose native chord is
    /// web-reserved but which ALSO carries a surviving emacs slot falls back
    /// to that slot on the web — never a blank label when a truthful door
    /// remains.
    #[test]
    fn web_reserved_native_chord_falls_back_to_a_surviving_emacs_slot() {
        let synthetic = Command {
            name: "Synthetic",
            action: Action::Ignore,
            native: "Cmd-N",
            emacs: "C-k",
            native_only: false,
            web_only: false,
        };
        assert_eq!(
            join_slots_truthful(&synthetic, Convention::Mac, Platform::Web, &[]),
            "C-k"
        );
        assert_eq!(
            join_slots_truthful(&synthetic, Convention::Linux, Platform::Web, &[]),
            "C-k"
        );
        assert_eq!(
            join_slots_truthful(&synthetic, Convention::Mac, Platform::Native, &[]),
            "⌘N · C-k"
        );
    }

    #[test]
    fn linux_web_reserved_uses_the_ctrl_translated_form() {
        let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
        assert_eq!(resolved_native(new_document, Convention::Linux), "C-n");
        assert!(crate::webreserved::is_reserved("C-n", Convention::Linux));
        assert_eq!(
            resolved_native_label_truthful(new_document, Convention::Linux, Platform::Web),
            "Alt+N"
        );
    }

    /// TIER 3: "Search forward" (native Cmd-F, emacs `C-s`) under
    /// `Convention::Linux` — Ctrl-S is claimed by Save, so the emacs slot is
    /// displaced and must NOT appear in the joined label, on EITHER platform
    /// (the collision is a dispatch-table property, not a web-only one).
    #[test]
    fn linux_displaced_emacs_default_never_shown_on_either_platform() {
        let search = COMMANDS
            .iter()
            .find(|c| c.name == "Search forward")
            .unwrap();
        for platform in [Platform::Native, Platform::Web] {
            let label = join_slots_truthful(search, Convention::Linux, platform, &[]);
            assert_eq!(
                label, "Ctrl+F",
                "displaced C-s must not appear (platform {platform:?})"
            );
        }
        // Mac convention: the emacs slot is UNCHANGED (Ctrl never reads native
        // there), so the old joined form survives on both platforms.
        assert_eq!(
            join_slots_truthful(search, Convention::Mac, Platform::Native, &[]),
            "⌘F · C-s"
        );
    }

    /// TIER 3, the prefix-sequence edge case: "Follow link"'s emacs default is
    /// the two-key `"C-c C-o"` sequence — Ctrl-C now resolves straight to Copy
    /// on Linux, so the WHOLE sequence is displaced (never arms), and Follow
    /// link has no native slot either — the joined label goes fully blank.
    #[test]
    fn linux_displaces_a_prefix_sequence_by_its_first_key() {
        let follow = COMMANDS.iter().find(|c| c.name == "Follow link").unwrap();
        assert_eq!(follow.native.trim(), "");
        assert_eq!(follow.emacs, "C-c C-o");
        assert_eq!(
            join_slots_truthful(follow, Convention::Linux, Platform::Native, &[]),
            ""
        );
        assert_eq!(
            join_slots_truthful(follow, Convention::Mac, Platform::Native, &[]),
            "C-c C-o"
        );
    }

    /// TIER 3, the non-displaced control: "Undo"'s emacs slot `C-/` is a
    /// non-letter chord outside the displaced-letter set entirely, so it
    /// survives Linux exactly like Mac.
    #[test]
    fn non_displaced_emacs_default_survives_linux() {
        let undo = COMMANDS.iter().find(|c| c.name == "Undo").unwrap();
        assert_eq!(
            join_slots_truthful(undo, Convention::Linux, Platform::Native, &[]),
            "Ctrl+Z · C-/"
        );
    }

    /// THE LABEL-TRUTH LAW, swept over the WHOLE catalog × every (convention,
    /// platform) pair: [`resolved_native_label_truthful`] is empty whenever
    /// [`crate::webreserved::is_reserved`] says so, and the joined label never
    /// contains a Linux-displaced emacs default as one of its `·`-separated
    /// tokens. A future catalog command that starts colliding fails THIS test
    /// until it is accounted for — the no-wildcard sweep the round's laws ask for.
    #[test]
    fn label_truth_law_holds_across_the_whole_catalog() {
        for c in COMMANDS.iter() {
            for convention in [Convention::Mac, Convention::Linux] {
                for platform in [Platform::Native, Platform::Web] {
                    let native_resolved = resolved_native(c, convention);
                    let reserved = platform == Platform::Web
                        && crate::webreserved::is_reserved(&native_resolved, convention);
                    if reserved {
                        let label = resolved_native_label_truthful(c, convention, platform);
                        let native_label = resolved_native_label(c, convention);
                        assert_ne!(
                            label, native_label,
                            "{}: reserved native chord {native_resolved:?} still shown verbatim ({convention:?}/{platform:?})",
                            c.name
                        );
                        // Either a web alternate (non-blank) or blank (no alternate defined) — but
                        // never the reserved native chord itself.
                        if let Some(alt) = web_alternate_for(c, convention) {
                            let expect = match convention {
                                Convention::Mac => crate::keyspec::mac_glyph_chord(alt),
                                Convention::Linux => crate::keyspec::linux_glyph_chord(alt),
                            };
                            assert_eq!(
                                label, expect,
                                "{}: web alternate label mismatch ({convention:?}/{platform:?})",
                                c.name
                            );
                        } else {
                            assert_eq!(
                                label, "",
                                "{}: no alternate defined, label should be blank ({convention:?}/{platform:?})",
                                c.name
                            );
                        }
                    }
                    let displaced = convention == Convention::Linux
                        && crate::keymap::linux_displaces_emacs_default(c.emacs, &[]);
                    if displaced {
                        let label = join_slots_truthful(c, convention, platform, &[]);
                        assert!(
                            !label.split(" · ").any(|tok| tok == c.emacs),
                            "{}: displaced emacs default {:?} still shown ({convention:?}/{platform:?}) — label was {label:?}",
                            c.name,
                            c.emacs
                        );
                    }
                }
            }
        }
    }

    /// TIER 4 (emacs-hands-on-Linux): "Forward char" (no native slot, emacs
    /// `C-f`) is normally Linux-DISPLACED by "Search forward"'s native Ctrl-F.
    /// A `linux_keep_emacs = ["C-f"]` config UN-displaces it (its emacs label
    /// reappears) AND suppresses "Search forward"'s own native label for that
    /// SAME chord — the two-sided fix, checked on both commands at once so
    /// they can never drift apart.
    #[test]
    fn linux_keep_emacs_restores_the_emacs_label_and_suppresses_the_native_one() {
        let keep = vec!["C-f".to_string()];
        let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
        let search = COMMANDS
            .iter()
            .find(|c| c.name == "Search forward")
            .unwrap();

        assert_eq!(
            join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &[]),
            ""
        );
        assert_eq!(
            join_slots_truthful(search, Convention::Linux, Platform::Native, &[]),
            "Ctrl+F"
        );

        // WITH the keep-list: Forward char shows its kept emacs chord; Search
        // forward's native Ctrl+F vanishes (it no longer actually fires there),
        // leaving only Search forward's OWN un-displaced... wait, C-s IS still
        // displaced by Save's native Ctrl-S (unrelated to this keep entry), so
        // Search forward's label goes fully blank — it has NO chord that fires
        // on Linux once C-f is given back to Forward char.
        assert_eq!(
            join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &keep),
            "C-f"
        );
        assert_eq!(
            join_slots_truthful(search, Convention::Linux, Platform::Native, &keep),
            ""
        );

        assert_eq!(
            join_slots_truthful(forward_char, Convention::Mac, Platform::Native, &keep),
            join_slots_truthful(forward_char, Convention::Mac, Platform::Native, &[]),
        );
        assert_eq!(
            join_slots_truthful(search, Convention::Mac, Platform::Native, &keep),
            join_slots_truthful(search, Convention::Mac, Platform::Native, &[]),
        );
    }

    #[test]
    fn linux_keep_emacs_is_a_per_chord_door_not_a_policy_flip() {
        let keep = vec!["C-f".to_string()];
        let next_line = COMMANDS.iter().find(|c| c.name == "Next line").unwrap();
        assert_eq!(
            join_slots_truthful(next_line, Convention::Linux, Platform::Native, &keep),
            ""
        );
        let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
        assert_eq!(
            join_slots_truthful(new_document, Convention::Linux, Platform::Native, &keep),
            "Ctrl+N"
        );
    }

    /// `effective_bindings`/`visible_effective_bindings` (the palette/rebind-menu
    /// doors) thread the keep-list all the way through — not just the pure
    /// `join_slots_truthful` unit.
    #[test]
    fn effective_bindings_reflects_the_linux_keep_emacs_list() {
        if Convention::current() != Convention::Linux {
            return;
        }
        let keep = vec!["C-f".to_string()];
        let i = COMMANDS
            .iter()
            .position(|c| c.name == "Forward char")
            .unwrap();
        assert_eq!(effective_bindings(&[], &[])[i], "");
        assert_eq!(effective_bindings(&[], &keep)[i], "C-f");
    }

    #[test]
    fn linux_keep_emacs_is_inert_on_mac_for_the_whole_catalog() {
        let keep = vec![
            "C-f".to_string(),
            "C-b".to_string(),
            "C-n".to_string(),
            "C-p".to_string(),
        ];
        for c in COMMANDS.iter() {
            assert_eq!(
                join_slots_truthful(c, Convention::Mac, Platform::Native, &keep),
                join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
                "{}: linux_keep_emacs must be inert on Mac",
                c.name
            );
        }
    }

    /// TIER 4, WHOLE-PRESET FLAVOR: the same two-sided label fix
    /// [`linux_keep_emacs_restores_the_emacs_label_and_suppresses_the_native_one`]
    /// exercises for a hand-picked `["C-f"]`, now exercised for the FULL emacs
    /// flavor preset (`keymap::linux_emacs_preset_keep`) — "Forward char" gets
    /// its emacs `C-f` label back and "Search forward" loses the native
    /// `Ctrl+F` claim it would otherwise show. UNLIKE the hand-picked case,
    /// "Search forward" does NOT go blank here: its OWN emacs default (`C-s`)
    /// is ALSO in the whole preset (the letter `s` is displaced too, by
    /// Save's native Ctrl-S), so Save's native claim is suppressed right back
    /// and Search forward's bare `C-s` reappears — the whole-preset's actual
    /// shape, every displaced letter reverting to its own emacs owner at once.
    #[test]
    fn keymap_flavor_emacs_preset_restores_labels_two_sided() {
        let preset = crate::keymap::linux_emacs_preset_keep();
        let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
        let search = COMMANDS
            .iter()
            .find(|c| c.name == "Search forward")
            .unwrap();
        let save = COMMANDS.iter().find(|c| c.name == "Save").unwrap();
        assert_eq!(
            join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &preset),
            "C-f"
        );
        assert_eq!(
            join_slots_truthful(search, Convention::Linux, Platform::Native, &preset),
            "C-s"
        );
        assert_eq!(
            join_slots_truthful(save, Convention::Linux, Platform::Native, &preset),
            ""
        );
    }

    #[test]
    fn keymap_flavor_emacs_preset_is_inert_on_mac_for_the_whole_catalog() {
        let preset = crate::keymap::linux_emacs_preset_keep();
        for c in COMMANDS.iter() {
            assert_eq!(
                join_slots_truthful(c, Convention::Mac, Platform::Native, &preset),
                join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
                "{}: the emacs keymap flavor must be inert on Mac",
                c.name
            );
        }
    }

    /// `Config::effective_linux_keep` is the ONE composition owner both dispatch
    /// (`keymap.rs`) and labels (`join_slots_truthful`, via this module) read —
    /// pinning that a `keymap = "emacs"` config produces the SAME label as
    /// passing the preset directly, so the two can never drift.
    #[test]
    fn config_effective_linux_keep_feeds_join_slots_truthful_identically_to_the_bare_preset() {
        let mut cfg = crate::config::Config::empty();
        cfg.keymap = Some("emacs".to_string());
        let via_config = cfg.effective_linux_keep();
        let bare_preset = crate::keymap::linux_emacs_preset_keep();
        let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
        assert_eq!(
            join_slots_truthful(
                forward_char,
                Convention::Linux,
                Platform::Native,
                &via_config
            ),
            join_slots_truthful(
                forward_char,
                Convention::Linux,
                Platform::Native,
                &bare_preset
            ),
        );
    }

    /// HARD LAW (b): Insert link's VISIBLE effective binding is EMPTY on Linux —
    /// out of the box, no user config, under BOTH keymap flavors — while Mac
    /// still shows Cmd-K (the `keymap` flavor is a Linux-only concept; Mac's
    /// label is unaffected regardless). Drives the SAME `Config::
    /// effective_linux_keep()` composition the live palette/rebind-menu read,
    /// so a label surface can never advertise a Linux chord that dispatch (see
    /// `keymap::tests::out_of_the_box_linux_ctrl_k_is_kill_line_under_both_
    /// keymap_flavors`) would never actually honor.
    #[test]
    fn insert_link_has_no_visible_linux_binding_out_of_the_box_mac_shows_cmd_k() {
        let insert_link = COMMANDS.iter().find(|c| c.name == "Insert link…").unwrap();
        for flavor in ["native", "emacs"] {
            let mut cfg = crate::config::Config::empty();
            cfg.keymap = Some(flavor.to_string());
            let keep = cfg.effective_linux_keep();
            assert_eq!(
                join_slots_truthful(insert_link, Convention::Linux, Platform::Native, &keep),
                "",
                "Insert link must show no Linux chord out of the box under keymap={flavor:?}"
            );
            assert_eq!(
                join_slots_truthful(insert_link, Convention::Mac, Platform::Native, &keep),
                "⌘K",
                "Mac must still show Cmd-K under keymap={flavor:?} (the keep list is Linux-only)"
            );
        }
    }

    #[test]
    fn effective_linux_keep_builtin_floor_is_inert_on_mac_for_the_whole_catalog() {
        for flavor in ["native", "emacs"] {
            let mut cfg = crate::config::Config::empty();
            cfg.keymap = Some(flavor.to_string());
            let keep = cfg.effective_linux_keep();
            for c in COMMANDS.iter() {
                assert_eq!(
                    join_slots_truthful(c, Convention::Mac, Platform::Native, &keep),
                    join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
                    "{}: the built-in keep floor must be inert on Mac (keymap={flavor:?})",
                    c.name
                );
            }
        }
    }
}

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

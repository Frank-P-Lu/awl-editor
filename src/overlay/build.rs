//! Build picker and navigator [`OverlayState`] values from caller-gathered data.

use super::{OverlayKind, OverlayState};
use std::path::Path;

/// Author the folder half of Go-to's destination roster from the configured
/// workspace and persisted folder MRU. The workspace itself and its direct
/// child folders are always known destinations; valid remembered folders may
/// live deeper. Paths are absolute so accepting a row has unambiguous identity.
pub fn goto_folder_roster(
    workspace: Option<&Path>,
    recent_folders: &[String],
) -> (Vec<(String, bool)>, Vec<String>) {
    let Some(workspace) = workspace else {
        return (Vec::new(), Vec::new());
    };
    let mut folders = vec![(
        workspace.to_string_lossy().to_string(),
        workspace.join(".git").is_dir(),
    )];
    for entry in crate::index::list_dir_level(workspace, None)
        .into_iter()
        .filter(|entry| entry.is_dir)
    {
        folders.push((
            workspace.join(entry.name).to_string_lossy().to_string(),
            entry.is_git,
        ));
    }
    let mut recent = Vec::new();
    for raw in recent_folders {
        let path = std::path::PathBuf::from(raw);
        if !path.is_dir() {
            continue;
        }
        let display = path.to_string_lossy().to_string();
        if !folders.iter().any(|(known, _)| known == &display) {
            folders.push((display.clone(), path.join(".git").is_dir()));
        }
        if !recent.contains(&display) {
            recent.push(display);
        }
    }
    (folders, recent)
}

/// The inputs the FLAT-picker overlay builder ([`build`]) needs, gathered by the
/// caller so the construction itself lives in ONE place (shared by the live App
/// and the headless `--keys` replay). The live-only GO-TO recency bits
/// (`goto_open` / `goto_recent` / `goto_times`) are filled by the App and left
/// EMPTY by the headless path, keeping the capture byte-stable. `config_keys`
/// feeds the command palette's EFFECTIVE bindings.
pub struct BuildCtx<'a> {
    /// The go-to corpus (root-relative paths), already recency-ordered when live.
    pub goto_corpus: Vec<String>,
    /// Corpus indices currently OPEN — ranking bias (live-only; empty headless).
    pub goto_open: Vec<usize>,
    /// Corpus indices recently opened — ranking bias (live-only; empty headless).
    pub goto_recent: Vec<usize>,
    /// Per-file "last edited" labels, parallel to `goto_corpus` (live-only; empty
    /// for a non-notes root AND in headless capture, for determinism).
    pub goto_times: Vec<String>,
    /// Config `[keys]` overrides → the command palette's effective binding column.
    pub config_keys: &'a [(String, Vec<String>)],
    /// Config `linux_keep_emacs` — the per-chord door that keeps a kept chord's
    /// emacs meaning showing (and suppresses the native label it would otherwise
    /// display) in the SAME effective binding column, under `Convention::Linux`
    /// only (see `commands::join_slots_truthful`'s Tier 4). Empty on Mac and on
    /// every headless capture that doesn't pass `--config`.
    pub config_linux_keep: &'a [String],
    /// The CURRENT buffer's markdown headings (depth-indented label + line) for
    /// Go-to's HEADINGS lens (the fold that retired the standalone Outline picker).
    /// Caller-gathered (it needs the live buffer text); EMPTY for a non-markdown
    /// buffer or one with no headings, so the Headings lens simply reads empty.
    pub goto_headings: Vec<(String, usize)>,
    /// Absolute folder destinations for Go-to, each with its git marker.
    pub goto_folders: Vec<(String, bool)>,
    /// Absolute folder MRU, newest-first. `attach_folders` enrols these into the
    /// shared Recent lens after the file MRU.
    pub goto_recent_folders: Vec<String>,
    /// The Cmd-`;` spell target — the misspelled word's corrections, its span, AND
    /// its current TEXT — resolved by the caller ONLY when the spell binding fired.
    /// `None` when the cursor isn't on a flagged word (or spell-check is off), so
    /// the summon no-ops. The word text builds the "Add '<word>' to dictionary" row
    /// label + rides the add-row accept effect ([`OverlayState::new_spell`]).
    // This public builder field mirrors the spell seam's suggestions, byte span, and source word.
    #[allow(clippy::type_complexity)]
    pub spell_target: Option<(Vec<String>, (usize, usize, usize), String)>,
    /// The HISTORY TIMELINE rows for the current file — [`crate::history::TimelineRow`]
    /// (when / which / counts / id), newest-first — resolved by the caller (via
    /// [`crate::history::timeline_rows`]) ONLY when the History binding fired. EMPTY
    /// otherwise AND when the file has no history yet; an empty list summons the calm
    /// "no history yet" row (History always opens; the Headings lens simply reads empty).
    pub history_entries: Vec<crate::history::TimelineRow>,
    /// The REFERENCE clock (millis) for the History picker's Today lens — `Some`
    /// live, `None` in the headless capture path (so the clock-relative lenses stay
    /// inert, the determinism gate).
    pub history_now: Option<u64>,
    /// The current session's start (millis) for the History picker's Session lens —
    /// `Some` live, `None` headless / untracked.
    pub history_session_start: Option<u64>,
    /// The config/project-derived VALUE inputs for the SETTINGS menu's secondary
    /// column ([`crate::settings::SettingsValues`]). The process-global settings
    /// (theme / page mode / caret / spell / markdown / nits) are read LIVE inside
    /// the readout, so only the config pieces are gathered by the caller — the live
    /// App from `self.config` + root + zoom, the headless replay from its `config`.
    /// Empty [`Default`] for a non-Settings summon (unused there).
    pub settings_values: crate::settings::SettingsValues,
    /// The ASSET CLEANER's scanned ORPHAN list ([`crate::assets::scan`]) — filled by
    /// the caller ONLY when the "Clean unused assets" binding fired (scanning the whole
    /// project tree is pure waste otherwise), EMPTY for every other summon. The live
    /// App AND the headless replay both fill it from the same scan over the
    /// [`crate::fs`] seam, so a `--keys` capture sees the real orphan list.
    pub assets: Vec<crate::assets::Orphan>,
    /// Is a daemon `--wait` client actively waiting on the CURRENT buffer right
    /// now (`crate::daemon`'s module doc, `App::wait_conns`)? The ONE live fact
    /// behind the Command palette's "Finish file" row visibility
    /// (`commands::visible_hidden_mask`) — `true` only on the live App, when the
    /// daemon exists AND some connection is parked waiting. Structurally `false`
    /// in the headless capture/replay path (which never imports `crate::daemon`
    /// at all — the daemon capture gate) and on wasm/`mas` (no daemon compiled),
    /// so a default palette build hides the row deterministically everywhere but
    /// a real `EDITOR=awl --wait` round-trip.
    pub row_gates: crate::commands::RowGates,
}

/// Build the SUMMONED overlay for a non-navigable picker kind (Goto / Theme /
/// Command, plus the buffer-scoped Spell) from the caller-gathered [`BuildCtx`].
/// Returns `None` for the navigable explorers (Browse / MoveDest / Project) —
/// those need a directory LEVEL, built by [`browse_level`] — and for an unresolved
/// Spell target, so those summons stay quiet no-ops. Shared by the live App
/// (`app.rs`) and the headless replay (`main.rs`) so both summon byte-identical
/// overlays.
pub fn build(kind: OverlayKind, ctx: &BuildCtx) -> Option<OverlayState> {
    match kind {
        // Go-to: the active project's file index. The open/recent tiers + the
        // relative "last edited" labels are caller-supplied (live-only; empty in
        // headless capture, so `set_times([])` is a no-op there).
        OverlayKind::Goto => {
            let mut ov = OverlayState::new(
                kind,
                ctx.goto_corpus.clone(),
                ctx.goto_open.clone(),
                ctx.goto_recent.clone(),
            );
            ov.set_times(ctx.goto_times.clone());
            // Fold the current doc's HEADINGS in as the Headings lens's corpus (the
            // retired Outline picker). Appended after the files; empty for a
            // non-markdown buffer (the lens then reads "no headings yet").
            ov.attach_headings(ctx.goto_headings.clone());
            ov.attach_folders(ctx.goto_folders.clone(), &ctx.goto_recent_folders);
            Some(ov)
        }
        // Theme picker: every world name + the active index (for revert). Built
        // from THEMES so it auto-extends as worlds are added.
        OverlayKind::Theme => {
            let names: Vec<String> = crate::theme::THEMES
                .iter()
                .map(|t| t.name.to_string())
                .collect();
            Some(OverlayState::new_theme(names, crate::theme::active_index()))
        }
        // Caret-style picker: the three looks + the active one (for revert). Built
        // from CaretMode::ALL so it auto-extends if a look is added.
        OverlayKind::Caret => Some(OverlayState::new_caret(crate::caret::mode())),
        // Dictionary picker: the three variants + the active one (pre-selected;
        // there is nothing to revert since nothing previews on move).
        OverlayKind::Dictionary => {
            Some(OverlayState::new_dictionary(crate::spell::active_variant()))
        }
        // CJK-priority language picker: the four languages + whichever currently
        // sits at the FRONT of the live ladder (pre-selected; nothing previews
        // on move, mirroring Dictionary).
        OverlayKind::CjkLang => Some(OverlayState::new_cjk_lang(
            crate::frontmatter::cjk_priority()
                .first()
                .copied()
                .unwrap_or(crate::frontmatter::Lang::Ja),
        )),
        // Date-format picker: the five formats, EACH row's primary text rendered
        // with `today` (live clock, or the fixed placeholder in a headless
        // capture — the SAME `today_ymd` the Settings "Date format" row previews,
        // so both surfaces agree), the active format pre-selected. Nothing
        // previews on move (the example dates ARE the preview), so no revert
        // bookkeeping — the Dictionary shape.
        OverlayKind::Date => Some(OverlayState::new_date(
            crate::dateformat::active_format(),
            ctx.settings_values.today_ymd,
        )),
        // Command palette: the PLATFORM-FILTERED command catalog
        // (`commands::visible()` — hides desktop-only commands on web; byte-identical
        // to the full catalog on native), each row showing its EFFECTIVE chord (config
        // `[keys]` rebinds included), so it teaches the live binding.
        OverlayKind::Command => {
            let mut ov = OverlayState::new_command(
                crate::commands::visible_names(),
                crate::commands::visible_effective_bindings(ctx.config_keys, ctx.config_linux_keep),
                // RUNTIME gate: "Finish file" only shows while a daemon `--wait`
                // client is actively waiting (see `BuildCtx::has_waiter`'s doc).
                crate::commands::visible_hidden_mask(ctx.row_gates),
            );
            // The Recent lens reads the in-memory recently-run MRU (empty in a fresh
            // process, so headless Recent is inert), translated into VISIBLE-CORPUS
            // indices (`visible_recent_indices`) so it can never point at a hidden row.
            ov.recent = crate::commands::visible_recent_indices();
            // THE UNION ROUND: the SETTINGS corpus joins the palette — appended after
            // the commands (mirrors Go-to's headings-after-files convention), so the
            // flat All lens fuzzy-ranks commands + settings together; typed settings
            // rows share the Settings browse category without a second command entry.
            // Same platform-filtered value readout the Settings menu itself
            // opens with, so a setting reached via the palette shows the identical
            // current-value secondary cell.
            //
            // ONE PALETTE DOOR PER DESTINATION (the union round's own follow-up fix):
            // `palette_names`/`palette_value_cells` are `visible_names`/
            // `visible_value_cells` MINUS every row whose covering command
            // (`settings::COVERED_BY`) is available on this platform — e.g. "Theme" is
            // excluded here because "Switch theme…" already opens the identical
            // `OverlayKind::Theme`, the exact door-duplication the user reported
            // ("what's the difference between the new theme option and the switch
            // theme option???"). A covered row stays fully reachable from the Settings
            // menu itself, which reads `visible_names`/`visible_value_cells` unfiltered.
            ov.attach_settings_rows(
                crate::settings::palette_rows(),
                crate::settings::palette_value_cells(&ctx.settings_values),
            );
            Some(ov)
        }
        // Rebind menu: the same platform-filtered command catalog + effective chords
        // as the palette, but opened in capture mode (Enter rebinds rather than runs).
        OverlayKind::Keybindings => Some(OverlayState::new_keybindings(
            crate::commands::visible_names(),
            crate::commands::visible_effective_bindings(ctx.config_keys, ctx.config_linux_keep),
        )),
        // Spell: the caller-resolved word target + its corrections. None when the
        // cursor isn't on a flagged word, so the summon no-ops.
        OverlayKind::Spell => ctx
            .spell_target
            .clone()
            .map(|(sugg, target, word)| OverlayState::new_spell(sugg, target, word)),
        // History: the caller-gathered timeline rows. ALWAYS summons: an empty list
        // becomes the calm "no history yet" row, so the picker never silently no-ops
        // on a file that simply hasn't been snapshotted yet.
        OverlayKind::History => Some(OverlayState::new_history(
            ctx.history_entries.clone(),
            ctx.history_now,
            ctx.history_session_start,
        )),
        // Settings menu: the flat settings corpus (display names) + each setting's
        // current VALUE in the secondary (binding) column, read via the settings
        // readout against the caller-gathered config/project values. It FACETS by
        // category (the scheme is registered), so it lands on the flat All home and
        // ←/→ step through the category lenses. Always summons.
        OverlayKind::Settings => {
            let mut ov = OverlayState::new(
                kind,
                crate::settings::visible_names(),
                Vec::new(),
                Vec::new(),
            );
            ov.set_secondaries(crate::settings::visible_value_cells(&ctx.settings_values));
            // The RAIL column beside the value text — the same gathered
            // values, read through the range-spec owner, so the thumb and the
            // number are one instant's truth.
            ov.set_range_cells(crate::settings::visible_range_cells(&ctx.settings_values));
            Some(ov)
        }
        // Asset cleaner: the caller-scanned orphan list. ALWAYS summons (like
        // History): an empty list becomes the calm "no unused assets" row.
        OverlayKind::Assets => Some(OverlayState::new_assets(ctx.assets.clone())),
        // Navigable explorers open via `browse_level` (they need a dir level).
        OverlayKind::Browse
        | OverlayKind::MoveDest
        | OverlayKind::ExportDest
        | OverlayKind::Project
        | OverlayKind::ProjectBrowse => None,
        // NOTES VERBS round: the Rename minibuffer is built directly at its
        // `Action::OpenRenameNote` apply_transition arm (`OverlayState::new_rename`) — it
        // needs only the buffer's own path, no caller-gathered context — so this
        // generic builder never constructs one. This arm exists for exhaustiveness.
        OverlayKind::Rename => None,
        // LINKS V2: the InsertLink minibuffer is built directly at its
        // `Action::InsertLink` apply_transition arm (`link::open_insert_link` →
        // `OverlayState::new_link_edit`) — it needs only the buffer's own
        // selection/cursor/text, no caller-gathered context — so this generic
        // builder never constructs one. This arm exists for exhaustiveness.
        OverlayKind::InsertLink => None,
        // NAMED SAVE POINTS: the Keep-version minibuffer is built directly at
        // its `Action::KeepVersion` apply_transition arm (`OverlayState::new_keep_name`)
        // — it needs no caller-gathered context at all (the prompt opens empty) —
        // so this generic builder never constructs one. Exhaustiveness arm.
        // The CONFLICT workspace is built from the App's own latched conflict
        // (`OverlayState::new_conflict`, at the `Effect::ReviewExternalChange`
        // arm) — the path and the disk text it carries are live-App facts this
        // shared builder has no access to, and no headless summon can invent.
        // Exhaustiveness arm.
        OverlayKind::Conflict => None,
        OverlayKind::KeepName | OverlayKind::Context => None,
    }
}

/// Build ONE directory LEVEL as a navigable overlay of the requested `kind`,
/// shared by the live App and the headless replay (parameterized by the caller's
/// roots so live + capture descend identically):
///   * `Project` navigates by ABSOLUTE path (`rel` IS the absolute dir; `None` =
///     start at `workspace`). Lists child FOLDERS only (git-marked) with the
///     synthetic accept-this-folder row on top, which reads as
///     [`crate::overlay::here_folder_label`] and names the level's own
///     directory. `None` when no workspace.
///   * `MoveDest` walks the ACTIVE root (`active_root`), listing FOLDERS only —
///     a document moves to a folder inside the SAME active folder it lives in.
///   * `ExportDest` is the same walk and the same folders-only listing: an export
///     lands IN a folder, so the two destination navigators differ in what they
///     put there, never in what they show.
///   * `ProjectBrowse` — the switch-project DOOR's navigator — walks the
///     WORKSPACE by absolute path, listing FOLDERS only, and cannot build a level
///     outside the workspace: that refusal IS the door's floor (see below).
///   * `Browse` walks the active root (`active_root`), listing files + folders.
///     `rel` is the root-relative level for the latter two (`None` = the root).
///
/// `recent_projects` is the persisted recent-PROJECTS MRU (absolute paths,
/// newest-first), resolved HERE ([`recent::resolve`]) and handed to
/// [`OverlayState::new_project`] as the roster's second route: a root that names
/// one of this level's children marks it, and a root that names anything else
/// enrols as its own whole-path row, which is how a project nested below a
/// direct child is findable at all. It is EMPTY for the other kinds (they have
/// no Recent lens) and in the headless replay (the determinism gate — recents is
/// live-only persisted state).
pub fn browse_level(
    kind: OverlayKind,
    rel: Option<String>,
    active_root: &Path,
    workspace: Option<&Path>,
    recent_projects: &[String],
) -> Option<OverlayState> {
    if kind == OverlayKind::Project {
        let dir = rel
            .clone()
            .or_else(|| workspace.map(|w| w.to_string_lossy().to_string()))?;
        let folders: Vec<(String, bool)> = crate::index::list_dir_level(Path::new(&dir), None)
            .into_iter()
            .filter(|e| e.is_dir)
            .map(|e| (e.name, e.is_git))
            .collect();
        // THE REMEMBERED ROWS BELONG TO THE LANDING, not to every level. The
        // flat switch-project picker never leaves the workspace, so every card
        // it builds is one. The only DEEPER levels are the Settings folder-VALUE
        // navigator's, and there a list of your recent projects would be rows
        // the directory in front of you does not contain — so they stop.
        //
        // That navigator's own LANDING is the workspace too, and it keeps them
        // deliberately: the key it is filling in wants a folder, and the folders
        // you have actually opened as projects are good answers to that. Unlike
        // the `Browse for folder…` door — which is withheld from it because a
        // descend would un-park the Settings surface — a remembered row descends
        // through `relevel`, which keeps the parked parent and the config key.
        //
        // Asked of the DIRECTORY rather than of `rel`, so ascending back to the
        // workspace is the same place, not a different moment.
        let at_landing = workspace.is_some_and(|w| Path::new(&dir) == w);
        let recent = match at_landing {
            true => recent::resolve(&dir, &folders, recent_projects),
            false => Vec::new(),
        };
        return Some(OverlayState::new_project(dir, folders, &recent));
    }
    // THE SWITCH-PROJECT DOOR'S LEVEL. Walks by ABSOLUTE path like `Project`
    // (the workspace is not inside the active root — it is usually above it),
    // and lists folders only: whatever you stop on becomes the project.
    //
    // THE WORKSPACE IS ITS FLOOR, and the floor is enforced HERE rather than in
    // the ascend arithmetic: a level outside the workspace simply cannot be
    // built, so `←`/`⌫` at the top find nothing to relevel into and the card
    // stands still. One gate then covers every way of naming a directory — the
    // ascend, a descend that leaves the tree, anything added later — instead of
    // a boundary test at each of them.
    if kind == OverlayKind::ProjectBrowse {
        let ws = workspace?;
        let dir = rel
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| ws.to_path_buf());
        if !dir.starts_with(ws) {
            return None;
        }
        let mut corpus = Vec::new();
        let mut git = Vec::new();
        let mut is_dir = Vec::new();
        for e in crate::index::list_dir_level(&dir, None)
            .into_iter()
            .filter(|e| e.is_dir)
        {
            corpus.push(e.name);
            git.push(e.is_git);
            is_dir.push(true);
        }
        return Some(OverlayState::new_marked(
            kind,
            corpus,
            git,
            is_dir,
            Vec::new(),
            Vec::new(),
            Some(dir.to_string_lossy().to_string()),
        ));
    }
    // The DESTINATION navigators (MoveDest, ExportDest) and Browse all walk the
    // active root; a destination lists folders only (something lands IN a
    // folder), Browse lists files + folders.
    let folders_only = matches!(kind, OverlayKind::MoveDest | OverlayKind::ExportDest);
    let level = crate::index::list_dir_level(active_root, rel.as_deref());
    // Browse alone classifies each FILE entry's openability up front
    // (bounded to ONE directory level — see `crate::openable::classify`'s doc
    // for why this is scoped here rather than the whole project's Goto index),
    // so `refilter`'s Text-mode filter can hide it and an "All" listing can
    // label it, with no second disk read on open.
    let dir_path = (kind == OverlayKind::Browse)
        .then(|| crate::index::resolve_dir_level(active_root, rel.as_deref()));
    let mut corpus = Vec::new();
    let mut git = Vec::new();
    let mut is_dir = Vec::new();
    let mut secondary = Vec::new();
    for e in &level {
        if folders_only && !e.is_dir {
            continue; // destinations are folders only
        }
        corpus.push(e.name.clone());
        git.push(e.is_git);
        is_dir.push(e.is_dir);
        let label = match (&dir_path, e.is_dir) {
            (Some(dir), false) => match crate::openable::classify(&dir.join(&e.name)) {
                crate::openable::Openable::Unsupported { label } => label,
                crate::openable::Openable::Text => String::new(),
            },
            _ => String::new(),
        };
        secondary.push(label);
    }
    let mut ov = OverlayState::new_marked(kind, corpus, git, is_dir, Vec::new(), Vec::new(), rel);
    if kind == OverlayKind::Browse {
        // `new_marked` already ran ONE `refilter()` before any row had its
        // secondary/type-label stamped (construction order), so a fresh Text-
        // mode summon would show every unsupported file until the NEXT
        // refilter — re-run it now that the labels are in place.
        ov.set_secondaries(secondary);
        ov.refilter();
    }
    Some(ov)
}

pub(in crate::overlay) mod recent;
mod rowdisplay;
pub(in crate::overlay) use rowdisplay::row_display;
pub use rowdisplay::{
    HERE_ACCEPT, HERE_LABEL, elide_directory_path, elide_path, here_folder_label, row_split,
};

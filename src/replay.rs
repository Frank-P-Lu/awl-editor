//! STRICT REPLAY TRUTHFULNESS — the ONE classification of every deferred
//! [`Effect`] the headless `--keys` replay can encounter, and the error/warning
//! surface both replay modes share.
//!
//! The headless replay (`main/run.rs::replay_keys`) drives the REAL
//! `apply_transition` seam, which returns filesystem/OS/window requests as
//! typed [`Effect`] values — and the capture caller can only honestly perform
//! SOME of them. This module names that honesty in data:
//!
//!   * [`EffectClass::Applied`] — the replay performs the effect FOR REAL
//!     (or the effect is a cosmetic one-shot whose settled frame is
//!     byte-identical by contract, so there is nothing unperformed to observe).
//!   * [`EffectClass::Intercepted`] — an EXTERNAL handoff (open a URL, compose
//!     a mailto:, move a file to the OS Trash, a DOM download) is OBSERVED and
//!     RECORDED — payload included — but deliberately not performed. Recording
//!     rides [`Intercept`], the seam the scenario trace (phase 5) consumes.
//!   * [`EffectClass::Unsupported`] — live-App-only work the replay cannot
//!     perform, whose skip leaves the session in a DIFFERENT state than live
//!     (a config write that never lands, a rename that never happens). The
//!     strict mode aborts on these, naming the exact action + effect.
//!
//! The dividing line between the last two: an INTERCEPTED effect's skip changes
//! nothing about subsequent in-app state (the handoff leaves the editor as-is),
//! while an UNSUPPORTED effect's skip silently diverges the session from what
//! the same keys would do live. Truthfulness means the strict runner refuses to
//! continue past a divergence rather than verify a fiction.
//!
//! [`classify_for`] is a NO-WILDCARD match over [`Effect`] (and, for
//! [`Effect::OverlayAccept`], over [`OverlayKind`]): a future variant fails to
//! compile here until someone consciously classifies it. `main/run.rs`'s
//! replay loop consults this classification; the two can only drift if a human
//! edits one without the other, which the bucket-pinning tests below guard.
//!
//! MODES ([`Mode`]): the legacy one-off `--keys` flag stays PERMISSIVE (never
//! aborts; warns on stderr and records, so existing captures keep working
//! byte-identically apart from stderr). STRICT is the scenario-runner default
//! the later phases plumb through — exposed today via the opt-in
//! `--strict-replay` flag on `--screenshot --keys`. Strict also refuses an
//! unbound chord or dangling prefix sequence at replay time
//! (`keyspec::ChordResolver` — resolution moved INTO the replay loop so the
//! search guard can consume a chord first; an unparseable token still errors
//! at parse time via `keyspec::parse_chords`) and a missing layout oracle
//! before the first key ([`missing_oracle_error`]), so a strict run's motion
//! verdicts always rode the real wrap geometry.

use crate::actions::Effect;
use crate::keymap::Action;
use crate::overlay::OverlayKind;
mod skip;
mod typed;
pub use skip::{SkippedEffect, permissive_skip};
use typed::{classify_buffer, classify_clipboard, classify_persistence, classify_settings, named};

/// How a replay treats the effects (and chords) it cannot honestly apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The legacy `--keys` door: never aborts. Crossing an Intercepted or
    /// Unsupported seam WARNS on stderr (and records the same line in the
    /// replay result, so the warning itself is testable).
    Permissive,
    /// The scenario-runner default: ABORT on any Unsupported effect, naming
    /// the exact action + effect ([`strict_error`]). Intercepted effects are
    /// recorded silently — observing a handoff without performing it IS the
    /// strict contract, not a compromise of it.
    Strict,
}

/// Filesystem authority handed explicitly to one headless replay. Ordinary
/// replay owns none. A strict scenario/storyboard may own an isolated sandbox
/// installed by its caller; the interpreter never infers this from global
/// filesystem state or from [`Mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemCapability {
    None,
    Isolated,
}

/// The truthfulness class of one [`Effect`] under headless replay. See the
/// module doc for the Applied / Intercepted / Unsupported dividing lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectClass {
    Applied,
    /// `detail` is the observed handoff payload (the URL, the trash-bound
    /// root-relative path, …) — empty when the effect carries none.
    Intercepted {
        detail: String,
    },
    /// `why` names the live-App-only work the replay cannot perform, for the
    /// strict error / permissive warning.
    Unsupported {
        why: &'static str,
    },
}

/// One classified effect: its stable snake_case `name` (used in errors,
/// warnings, and the phase-5 trace) plus its class.
pub struct Classified {
    pub name: &'static str,
    pub class: EffectClass,
}

/// One intercepted external handoff, recorded in replay order for the future
/// scenario trace: a stable effect name plus the observed payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intercept {
    /// The effect's stable name ([`classify_for`]'s `name`), e.g. `"follow_link"`.
    pub effect: &'static str,
    /// The observed handoff payload (URL / trash path / …); `""` when the
    /// effect carries none.
    pub detail: String,
}

/// Classify one [`Effect`] — the ONE owner of the Applied / Intercepted /
/// Unsupported truth, consulted by the replay loop in `main/run.rs`. A
/// NO-WILDCARD match: a future `Effect` or [`OverlayKind`] fails to compile
/// until it is consciously classified.
#[cfg(test)]
pub fn classify(effect: &Effect) -> Classified {
    classify_for(effect, FilesystemCapability::None)
}

/// Classify with the filesystem authority the caller explicitly owns.
pub fn classify_for(effect: &Effect, filesystem: FilesystemCapability) -> Classified {
    let c = |name, class| Classified { name, class };
    let applied = EffectClass::Applied;
    let unsupported = |why| EffectClass::Unsupported { why };
    match effect {
        // ── APPLIED: the replay performs these for real (see the matching
        // arms in `main/run.rs::replay_keys_mode` / `capture_screenshot`). ──
        Effect::None => c("none", applied),
        Effect::Buffer(buffer) => classify_buffer(buffer),
        Effect::RunAction(_) => c("run_action", applied),
        Effect::OverlayAccept(kind, _) => c("overlay_accept", accept_class(*kind)),
        Effect::JumpToLine(_) => c("jump_to_line", applied),
        Effect::Persistence(persistence) => classify_persistence(persistence, filesystem),
        Effect::Clipboard(clipboard) => classify_clipboard(clipboard),
        Effect::Daemon(crate::actions::DaemonEffect::NotifyFinished) => {
            intercepted("daemon_notify_finished", String::new())
        }
        Effect::Surface(surface) => match surface {
            crate::actions::SurfaceEffect::ShowAbout => c("show_about", applied),
            crate::actions::SurfaceEffect::OpenFileChooser => c(
                "open_file_chooser",
                unsupported(
                    "the platform file chooser is live-only; capture the resulting open separately",
                ),
            ),
            crate::actions::SurfaceEffect::OpenFolderChooser => c(
                "open_folder_chooser",
                unsupported(
                    "the platform folder chooser is live-only; capture the resulting rescope separately",
                ),
            ),
        },
        Effect::Notice(notice) => match notice {
            crate::actions::NoticeEffect::Toast(_) => c("notice_toast", applied),
            crate::actions::NoticeEffect::Sticky(_) => c("notice_sticky", applied),
            crate::actions::NoticeEffect::Clear => c("notice_clear", applied),
        },
        Effect::Render(render) => match render {
            crate::actions::RenderEffect::SyncView { .. } => c("sync_view", applied),
            crate::actions::RenderEffect::Reshape => c("reshape", applied),
            crate::actions::RenderEffect::ZoomChanged => c("zoom_changed", applied),
            crate::actions::RenderEffect::Redraw => c("redraw", applied),
            crate::actions::RenderEffect::EditStreak => c("edit_streak", applied),
        },
        // INSERT DATE: the headless replay performs the SAME insert live does
        // (against the fixed placeholder date instead of the real clock — see
        // `dateformat::CAPTURE_PLACEHOLDER_YMD`), so this is honestly Applied,
        // not a divergence.
        Effect::InsertDate => c("insert_date", applied),
        // Cosmetic caret one-shots: the underlying edit/copy already applied in
        // the core, and the flourish's settled frame is byte-identical BY
        // CONTRACT (each variant's own doc in `actions.rs`), so the skipped
        // animation is unobservable in any capture — Applied, not a gap.
        Effect::Recoil(_) => c("recoil", applied),
        Effect::TypeImpact => c("type_impact", applied),
        Effect::DeleteSquash => c("delete_squash", applied),
        Effect::Gulp => c("gulp", applied),
        Effect::LineLand => c("line_land", applied),
        Effect::CopyPulse => c("copy_pulse", applied),

        // ── INTERCEPTED: external handoffs, observed + recorded, safely not
        // performed — skipping them leaves the editor state exactly as live
        // (the handoff target is OUTSIDE the app). ──
        Effect::FollowLink(url) => intercepted("follow_link", url.clone()),
        Effect::ReportProblem => intercepted("report_problem", String::new()),
        Effect::DownloadFile => intercepted("download_file", String::new()),
        // The export renders the document + writes a sibling file (or a web
        // download) — a live-App-only external write the replay/capture safely
        // skips, leaving the editor state exactly as live. Recorded, not performed.
        Effect::Export(format, _) => intercepted("export", format.ext().to_string()),
        Effect::CheckForUpdates => intercepted("check_for_updates", String::new()),
        Effect::TrashAsset { rel } => intercepted("trash_asset", rel.clone()),

        // ── UNSUPPORTED: live-App-only work whose skip diverges the session
        // from what the same keys do live — strict replay aborts here. ──
        Effect::Quit => c(
            "quit",
            // Live exits the event loop; the replay has none and would keep
            // applying LATER keys past the "exit" — a real divergence. A
            // future scenario runner may promote this to a clean stop instead.
            unsupported("live exits the event loop; a replay would keep applying keys past it"),
        ),
        Effect::KeepVersion { .. } => c(
            "keep_version",
            unsupported(
                "pinning (and naming) writes the local-history store, gated off the capture path",
            ),
        ),
        Effect::AddToDictionary(_) => c(
            "add_to_dictionary",
            unsupported(
                "silencing the word + appending it to the personal-dictionary file are live-App-only; the squiggle would not clear",
            ),
        ),
        Effect::RebindCommit { .. } => c(
            "rebind_commit",
            unsupported(
                "the config write + live keymap reload are live-App-only; the binding would not take effect",
            ),
        ),
        Effect::RebindReset { .. } => c(
            "rebind_reset",
            unsupported(
                "the config write + live keymap reload are live-App-only; the reset would not take effect",
            ),
        ),
        // Settings persistence gets the same capability grant as
        // `save`/`finish_save`, classified in `typed::classify_settings` to keep
        // this match within its line budget.
        Effect::SettingToggle { key } => classify_settings(
            "setting_toggle",
            "flipping the live global + persisting it are live-App-only; \
             the setting would not change",
            filesystem,
            Some(key),
        ),
        Effect::SettingValueCommit { .. } => classify_settings(
            "setting_value_commit",
            "parse-clamp-apply-persist is live-App-only; the value would not take effect",
            filesystem,
            None,
        ),
        Effect::SettingPathPick { .. } => classify_settings(
            "setting_path_pick",
            "the config folder-key write is live-App-only; the path would not take effect",
            filesystem,
            None,
        ),
        // A RANGE row's step: unlike its Toggle/Value siblings above, the
        // VALUE CHANGE ITSELF already happened in the shared core (`apply_transition`
        // stepped `ActionCtx::zoom` through the range spec and mirrored the row's
        // readout + thumb), so the replay session observes exactly what live does —
        // the SAME reason the Theme/Caret/Date accepts are Applied. What the replay
        // skips is the live tail (a metric reflow it has no pipeline for, and the
        // sticky config write the capture path is structurally free of), neither of
        // which is observable in a capture. Applied, not a gap.
        Effect::SettingRangeStep { .. } => c("setting_range_step", applied),
        Effect::RenameNoteCommit { .. } => c(
            "rename_note_commit",
            unsupported("the disk rename is live-App-only; the buffer would keep its old path"),
        ),
        Effect::DuplicateNote => c(
            "duplicate_note",
            unsupported("the sibling copy + buffer swap are live-App-only"),
        ),
    }
}

fn intercepted(name: &'static str, detail: String) -> Classified {
    named(name, EffectClass::Intercepted { detail })
}

/// The per-[`OverlayKind`] class of an [`Effect::OverlayAccept`] — accepts are
/// the one effect whose truthfulness depends on WHICH picker emitted it. A
/// NO-WILDCARD match, mirroring [`OverlayKind::accept_disposition`]'s own law:
/// a future kind fails to compile until it declares whether its accept is
/// honestly applied headlessly.
fn accept_class(kind: OverlayKind) -> EffectClass {
    match kind {
        // Applied for real: Goto drives the multi-buffer registry switch inline in the
        // replay loop; Project re-roots (whole sidecar block re-derived from the accepted
        // root by the ONE builder `run::project_info`; the session's OWN
        // root/workspace/corpus re-scope through `ReplaySession::resync_project_location`,
        // so a chord applied after the accept reads the new tree too) and
        // History restores, both in `capture_screenshot`'s accept stage; Theme / Caret /
        // Dictionary / CjkLang / Date set their process-global CORE-level, so the replay
        // observes them exactly as live (`actions/overlay_nav.rs`).
        OverlayKind::Goto
        | OverlayKind::Project
        | OverlayKind::History
        | OverlayKind::Theme
        | OverlayKind::Caret
        | OverlayKind::Dictionary
        | OverlayKind::CjkLang
        | OverlayKind::Date => EffectClass::Applied,
        // The note move (mkdir + rename under the notes root) is live-App-only;
        // headlessly the buffer keeps its old path — a divergence.
        OverlayKind::MoveDest => EffectClass::Unsupported {
            why: "the note move (mkdir + rename) is live-App-only; the buffer would keep its old path",
        },
        // These pickers ride their own effects or core-internal edits rather than
        // an accept (Browse re-routes files through Goto; Command runs via
        // `RunAction`; Spell edits in the core; Keybindings/Settings/Assets/
        // Rename/InsertLink — see `actions/overlay_nav.rs`). The CONFLICT
        // workspace is the one member here that CAN reach the generic accept
        // fallthrough — `⇧↵` on one of its rows emits `OverlayAccept(Conflict,
        // <row label>)`, which the live App answers with a deliberate no-op — so
        // Unsupported is the honest classification rather than a defensive one:
        // a strict replay must abort naming it instead of pretending it settled a
        // conflict it cannot see. The rest are Unsupported so a NEW emission
        // aborts loudly rather than silently passing.
        OverlayKind::Browse
        | OverlayKind::Command
        | OverlayKind::Spell
        | OverlayKind::Keybindings
        | OverlayKind::Settings
        | OverlayKind::Assets
        | OverlayKind::Rename
        | OverlayKind::InsertLink
        | OverlayKind::KeepName
        | OverlayKind::Conflict
        // The export DESTINATION navigator rides `Effect::Export` (already
        // classified Intercepted) rather than an accept, exactly as Browse rides
        // Goto's — the folder it chose is a component of that effect, not a
        // separate acceptance.
        | OverlayKind::ExportDest
        // The switch-project DOOR's navigator emits its answer AS `Project`
        // (Applied above) — one owner of the switch, whichever door reached it —
        // so an accept never carries THIS kind.
        | OverlayKind::ProjectBrowse
        | OverlayKind::Context => EffectClass::Unsupported {
            why: "this picker is not expected to emit an accept effect; classify it in replay::accept_class before strict replay can pass it",
        },
    }
}

/// The permissive `--keys` warning line for a non-Applied effect — `None` for
/// Applied (the common case warns about nothing). The ONE owner of the warning
/// wording: `main/run.rs` prints exactly this string to stderr AND records it
/// in the replay result, so tests pin the same text users see.
pub fn warn_line(action: &Action, c: &Classified) -> Option<String> {
    match &c.class {
        EffectClass::Applied => None,
        EffectClass::Intercepted { detail } => {
            let payload = if detail.is_empty() {
                String::new()
            } else {
                format!(" ({detail})")
            };
            Some(format!(
                "--keys replay: intercepted `{}`{payload} from action {:?} — recorded, not performed",
                c.name, action
            ))
        }
        EffectClass::Unsupported { why } => Some(format!(
            "--keys replay: skipped unsupported effect `{}` from action {:?} — {}",
            c.name, action, why
        )),
    }
}

/// The strict-mode abort for an Unsupported effect: names the exact action AND
/// effect (the spec's contract), plus the live-App-only reason. Only ever
/// built for [`EffectClass::Unsupported`].
pub fn strict_error(action: &Action, c: &Classified) -> anyhow::Error {
    let why = match &c.class {
        EffectClass::Unsupported { why } => why,
        // `main/run.rs` only calls this on the Unsupported arm; a misuse still
        // produces an honest (if less specific) error rather than a panic.
        EffectClass::Applied | EffectClass::Intercepted { .. } => "not an unsupported effect",
    };
    anyhow::anyhow!(
        "strict replay: unsupported effect `{}` from action {:?} — {}",
        c.name,
        action,
        why
    )
}

/// The strict-mode abort for a missing LAYOUT ORACLE: without the offscreen
/// shaped pipeline (no GPU adapter), visual-line motion silently falls back to
/// LOGICAL lines — fine for the permissive door, a fiction the strict runner
/// refuses to verify against.
pub fn missing_oracle_error() -> anyhow::Error {
    anyhow::anyhow!(
        "strict replay: layout oracle unavailable (no GPU adapter) — \
     visual-line motion would fall back to logical lines instead of the shaped wrap geometry"
    )
}

#[cfg(test)]
mod tests;

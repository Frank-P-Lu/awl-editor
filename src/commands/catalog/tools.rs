//! The catalog's non-editing APP DOORS: the problem report, the web-only
//! download, the update check, and the scratch summon door. Peeled off
//! `navigation.rs`'s tail purely to keep both files under the ~500-line
//! ceiling — `catalog.rs` chains
//! `navigation ++ tools ++ editing`, so the corpus index and palette display
//! order are byte-for-byte what they were when these three sat at the end of
//! `navigation.rs`.

use super::Command;
use crate::keymap::Action;

pub(super) static COMMANDS: &[Command] = &[
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
        description: Some(
            "Compose a `mailto:` bug report, attaching the newest crash log's path if one exists.",
        ),
    },
    Command {
        name: "Download file",
        action: Action::DownloadFile,
        native: "",
        emacs: "",
        native_only: false,
        web_only: true,
        description: Some(
            "Download the buffer's text as a file — the web export, since there is no real disk.",
        ),
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
        description: Some(
            "Record a last-checked marker and open the site's version-check page in the browser.",
        ),
    },
    // OPEN SCRATCH: the in-session door back to the persistent scratch
    // surface once it has been closed (Finish file / a stack-row close both
    // just close it now — the autosave engine's own stash already holds the
    // text, and this reads it back). No default chord — the palette IS its
    // entry point, like Report a Problem/About.
    Command {
        name: "Open scratch",
        action: Action::OpenScratch,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the persistent scratch surface, restoring it if it was closed."),
    },
];

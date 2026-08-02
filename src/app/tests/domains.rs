//! ITEM 172 STRUCTURAL GATES — the `App` ownership map, as executable data.
//!
//! `docs/app-domains.md` is the prose map. This file is the part a reviewer
//! cannot forget to read: it parses `src/app.rs`'s own `pub struct App`
//! declaration at test time and asserts that
//!
//!  1. every field is classified into exactly one [`Domain`] (so a new field
//!     fails the suite until its owner is chosen consciously), and
//!  2. the ROOT struct does not grow (so "add a field to `App`" stops being
//!     the path of least resistance), and
//!  3. a domain already EXTRACTED into an owner type has zero fields left on
//!     root `App` — the compile-time removal item 172 asks for, asserted from
//!     the other side so a re-added field cannot quietly rejoin the struct.
//!
//! The match in [`Domain::describe`] is deliberately WILDCARD-FREE: adding a
//! `Domain` variant (item 173 will) fails to COMPILE until the new owner is
//! described, and the roster sweep below then forces it to be exercised. A
//! `_ => ..` arm here would let a whole domain join the map unexamined, which
//! is the exact failure mode CLAUDE.md's four green-law-over-real-defect
//! stories share.

/// The state-ownership domains of `App`. One variant per owner in
/// `docs/app-domains.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Domain {
    /// The summoned-UI layer ladder — EXTRACTED (`app::workspace::WorkspaceState`).
    WorkspaceState,
    /// App-global save feedback + the autosave debounce — EXTRACTED
    /// (`app::persistence::PersistenceRuntime`).
    PersistenceRuntime,
    /// Persisted configuration plus CLI/default-folder location policy.
    ConfigurationRuntime,
    /// The active document slot, the registry, the checker.
    DocumentSession,
    /// Raw keyboard/pointer/gesture state (`app::input::InputRuntime`).
    InputRuntime,
    /// "Where am I working": root, project, indexes, MRUs.
    ProjectLocation,
    /// GPU/presentation lifecycle, frame state, and every settle deadline.
    FrameRuntime,
    /// Process/OS handles and one-shot startup handoffs. `App` genuinely is
    /// their lifecycle; these stay.
    HostLifecycle,
}

/// Whether this domain has already been migrated to an owner type. An
/// extracted domain must own ZERO root-`App` fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Extraction {
    /// Migrated: the fields live on the owner type, not on `App`.
    Extracted,
    /// Mapped but not yet migrated: the fields are still root-`App` fields.
    OnRootApp,
}

/// Root-`App` fields still owned by [`Domain::HostLifecycle`].
const HOST_LIFECYCLE: &[&str] = &[
    "clipboard",
    "clipboard_last_written",
    "soak",
    "soak_recovery_pending",
    "soak_passed",
    "probe_ready",
    "daemon_socket_path",
    "wait_conns",
    "menu_proxy",
    "_menu_bar",
    "restored_window",
    "pending_crash",
    "stats",
    "stats_origin",
    "stats_last_input_ms",
    "stats_last_caret_xy",
    "stats_last_cursor",
    "stats_dirty",
    "streaks",
    "streaks_baseline",
    "streaks_dirty",
];

impl Domain {
    /// Every domain, in declaration order. Paired with the no-wildcard match
    /// below: a new variant must be added here to be swept, and must be
    /// described there to compile.
    pub(crate) const ROSTER: &'static [Domain] = &[
        Domain::WorkspaceState,
        Domain::PersistenceRuntime,
        Domain::ConfigurationRuntime,
        Domain::DocumentSession,
        Domain::InputRuntime,
        Domain::ProjectLocation,
        Domain::FrameRuntime,
        Domain::HostLifecycle,
    ];

    /// The ONE root-`App` field that HOLDS this domain's owner type, once the
    /// domain has been extracted — `None` while the domain's state is still
    /// loose fields on `App`. Exactly one handle per extracted domain: two
    /// handles for one domain would mean the owner had been split, which is the
    /// coupling this item removes wearing a tidier hat.
    ///
    /// NO WILDCARD ARM.
    pub(crate) fn owner_handle(self) -> Option<&'static str> {
        match self {
            Domain::WorkspaceState => Some("workspace_state"),
            Domain::PersistenceRuntime => Some("persistence"),
            Domain::ConfigurationRuntime => Some("config"),
            Domain::ProjectLocation => Some("project_location"),
            Domain::InputRuntime => Some("input"),
            Domain::DocumentSession => Some("document"),
            Domain::FrameRuntime => Some("frame"),
            Domain::HostLifecycle => None,
        }
    }

    /// The domain's extraction status and the root-`App` fields it still owns.
    /// NO WILDCARD ARM — see this module's doc.
    pub(crate) fn describe(self) -> (Extraction, &'static [&'static str]) {
        match self {
            // ── EXTRACTED ────────────────────────────────────────────────
            // `overlay` / `search` / `popover_open` now live behind
            // `WorkspaceState`'s transitions; the precedence ladder that used
            // to be five hand-written conjunctions is `WorkspaceState::layer`.
            Domain::WorkspaceState => (Extraction::Extracted, &[]),
            // `autosave_dirty_at` / `autosave_saved_version` /
            // `autosave_last_ok` / `last_saved_ok` / `title_dirty` now live
            // behind `PersistenceRuntime`'s transitions; the debounce stamp and
            // the version it wrote are ONE ledger, not two fields.
            Domain::PersistenceRuntime => (Extraction::Extracted, &[]),
            // `Config`, its CLI precedence inputs, and the default-folder
            // fallback are one runtime policy, held behind `App::config`.
            Domain::ConfigurationRuntime => (Extraction::Extracted, &[]),
            // The root, its derived project/index/workspace state, and both
            // project MRUs move together behind one location owner.
            Domain::ProjectLocation => (Extraction::Extracted, &[]),

            // ── MAPPED, STILL ON ROOT `App` ──────────────────────────────
            Domain::DocumentSession => (Extraction::Extracted, &[]),
            Domain::InputRuntime => (Extraction::Extracted, &[]),
            Domain::FrameRuntime => (Extraction::Extracted, &[]),
            Domain::HostLifecycle => (Extraction::OnRootApp, HOST_LIFECYCLE),
            // NO `_ =>` ARM. A new domain must be described here.
        }
    }
}

/// THE ROOT-`App` FIELD ROSTER, read out of `src/app.rs` itself.
///
/// Parsed rather than hand-listed on purpose: a hand-listed roster is a second
/// copy of the struct that drifts, and the whole point of this gate is that a
/// field CANNOT be added without the map noticing. Every `#[cfg]`-gated field
/// counts — the map is about ownership, not about which platform compiles it.
pub(crate) fn root_app_fields() -> Vec<String> {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app.rs");
    let text = std::fs::read_to_string(&src).expect("src/app.rs must be readable");
    let start = text
        .find("pub struct App {")
        .expect("src/app.rs must declare `pub struct App`");
    let body_start = start + text[start..].find('{').expect("struct body") + 1;
    let mut depth = 1usize;
    let mut end = body_start;
    for (i, c) in text[body_start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let mut fields = Vec::new();
    // Bracket depth across DECLARATION lines only. A doc comment is prose and
    // may carry any number of unbalanced `<` / `(` (a chord like `M-<`, an
    // arrow, an aside in parentheses that wraps to the next line) — counting
    // those truncated the parse at the first such line and left every gate in
    // this file vacuously green over an 83-field slice of a 105-field struct.
    // `the_root_app_field_parser_is_not_vacuous` is the law that caught it.
    let mut depth = 0i32;
    for raw in text[body_start..end].lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if depth == 0
            && let Some((name, _)) = line.split_once(':')
        {
            let name = name.trim();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                fields.push(name.to_string());
            }
        }
        depth += line.matches('<').count() as i32 - line.matches('>').count() as i32;
        depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
    }
    fields
}

/// Every field of root `App` is assigned to exactly one owner, and every owner
/// still claiming root-`App` fields claims only fields that are really there.
///
/// This is the gate against DIRECT CROSS-DOMAIN FIELD ACCESS from the field
/// side: privacy alone cannot express it (all 107 fields are visible to all
/// ~24 `impl App` blocks, because they are all descendants of `crate::app`),
/// so the enforceable version is "a field that belongs to an extracted domain
/// does not exist on `App`" plus "a field that belongs to a mapped domain is
/// declared as belonging to it".
#[test]
fn every_root_app_field_has_exactly_one_owner() {
    let fields = root_app_fields();
    let mut claimed: std::collections::BTreeMap<&str, Vec<Domain>> = Default::default();
    for domain in Domain::ROSTER {
        let (_, owned) = domain.describe();
        for f in owned {
            claimed.entry(f).or_default().push(*domain);
        }
        // The owner HANDLE is itself a root-`App` field and must be classified
        // too, or extracting a domain would silently create an unowned field.
        if let Some(handle) = domain.owner_handle() {
            claimed.entry(handle).or_default().push(*domain);
        }
    }

    // No field claimed twice — two owners for one field is the coupling this
    // item exists to remove, dressed up as a map.
    let doubled: Vec<_> = claimed
        .iter()
        .filter(|(_, ds)| ds.len() > 1)
        .map(|(f, ds)| format!("{f} claimed by {ds:?}"))
        .collect();
    assert!(
        doubled.is_empty(),
        "a root App field must have exactly ONE owner: {doubled:?}"
    );

    let unassigned: Vec<&String> = fields
        .iter()
        .filter(|f| !claimed.contains_key(f.as_str()))
        .collect();
    assert!(
        unassigned.is_empty(),
        "new root App field(s) with no owner: {unassigned:?} — assign each to a \
         Domain in src/app/tests/domains.rs (and prefer moving the state into an \
         existing owner over growing `App`); see docs/app-domains.md"
    );

    let phantom: Vec<&&str> = claimed
        .keys()
        .filter(|f| !fields.iter().any(|got| got == *f))
        .collect();
    assert!(
        phantom.is_empty(),
        "the ownership map claims root App field(s) that no longer exist: \
         {phantom:?} — remove them from src/app/tests/domains.rs (a renamed field \
         must be renamed here too, or the map silently stops guarding it)"
    );
}

/// An EXTRACTED domain owns zero root-`App` fields — the compile-time removal
/// of the old field access, asserted from the field side so a re-added field
/// cannot quietly rejoin the struct under a slightly different name.
///
/// The sweep is the ROSTER, not a hand-picked pair: the axis a new domain is
/// most likely to dodge is "was anyone checking THIS variant", so every variant
/// is visited and each one's status is exercised.
#[test]
fn an_extracted_domain_keeps_nothing_on_root_app() {
    let fields = root_app_fields();
    let mut extracted = 0usize;
    let mut on_root = 0usize;
    for domain in Domain::ROSTER {
        let (status, owned) = domain.describe();
        match status {
            Extraction::Extracted => {
                extracted += 1;
                let handle = domain.owner_handle().unwrap_or_else(|| {
                    panic!("{domain:?} is Extracted but names no owner-handle field on `App`")
                });
                assert!(
                    fields.iter().any(|f| f == handle),
                    "{domain:?} names owner handle `App::{handle}`, which is not a \
                     field of `App` — the map and the struct have drifted"
                );
                assert!(
                    owned.is_empty(),
                    "{domain:?} is marked Extracted but still claims root App \
                     fields {owned:?} — either finish the migration or mark it OnRootApp"
                );
                // The migrated names must be GONE from `App`, under any
                // spelling the domain used to own. Checked against the live
                // parse rather than a remembered list.
                for retired in retired_field_names(*domain) {
                    assert!(
                        !fields.iter().any(|f| f == retired),
                        "`App::{retired}` belongs to the extracted {domain:?} domain and \
                         must not be a root App field again — put the state behind that \
                         owner's transitions instead"
                    );
                }
            }
            Extraction::OnRootApp => {
                on_root += 1;
                assert_eq!(
                    domain.owner_handle(),
                    None,
                    "{domain:?} is still OnRootApp but already names an owner handle — \
                     mark it Extracted once its fields have moved"
                );
                assert!(
                    !owned.is_empty(),
                    "{domain:?} is mapped OnRootApp but claims no fields — if its \
                     state has moved, mark it Extracted"
                );
            }
        }
    }
    assert_eq!(
        extracted + on_root,
        Domain::ROSTER.len(),
        "every domain in the roster must be visited by this sweep"
    );
    assert!(
        extracted >= 2,
        "item 172 landed two owner extractions; a regression that folds either \
         back onto root `App` must fail here"
    );
}

/// The field names each extracted domain took OFF root `App`. Kept explicit
/// (the parse can only see what is there now, never what was removed), so a
/// re-added `overlay`/`search`/… fails by name rather than only inflating the
/// count.
#[cfg(test)]
fn retired_field_names(domain: Domain) -> &'static [&'static str] {
    match domain {
        Domain::WorkspaceState => &["overlay", "search", "popover_open"],
        Domain::PersistenceRuntime => &[
            "autosave_dirty_at",
            "autosave_saved_version",
            "autosave_last_ok",
            "last_saved_ok",
            "title_dirty",
        ],
        Domain::ConfigurationRuntime => &["default_folder", "cli_workspace", "cli_default_folder"],
        Domain::InputRuntime => &[
            "keymap",
            "mods",
            "prefix_pending_at",
            "whichkey_shown",
            "hud_key",
            "hud_mods",
            "peek_arm",
            "peek_armed_at",
            "pointer_hide",
            "cursor_px",
            "dragging",
            "drag_press_px",
            "drag_armed",
            "page_resizing",
            "page_resize_edge",
            "page_resize_anchor",
            "image_resizing",
            "range_drag",
            "cursor_icon",
            "drag_granularity",
            "last_click_time",
            "last_click_px",
            "click_count",
            "scroll_px_accum",
            "preedit",
            "ime_enabled",
            "scroll_sensitivity",
        ],
        Domain::ProjectLocation => &[
            "root",
            "project",
            "file_index",
            "workspace_root",
            "recent_projects",
            "recent_files",
        ],
        Domain::DocumentSession => &["active", "buffer_registry", "prev_file", "spell"],
        Domain::FrameRuntime => &[
            "gpu",
            "recovery_window",
            "gpu_lifecycle",
            "gpu_retry_at",
            "gpu_timeout_streak",
            "gpu_pending",
            "present_sync_on",
            "present_sync_valid",
            "dpi",
            "zoom",
            "zoom_reflow",
            "zoom_anchor",
            "theme_font_at",
            "theme_font_last_reshape_at",
            "theme_switch_at",
            "theme_settle",
            "theme_switches",
            "caret_edit_streaks",
            "caret_held",
            "caret_impact",
            "caret_recoil",
            "frame_costs",
            "debug_still",
            "redraw_count",
            "last_latency_ms",
            "input_stamp",
            "clock",
            "last_frame",
            "lava_tick_at",
            "resize_settle_at",
            "move_settle_at",
            "crossing_settle_at",
            "crossing_teardown_pending",
            "focused",
            "notice",
            "notice_kind",
            "notice_expires_at",
            "zoom_persist_at",
        ],
        Domain::HostLifecycle => &[],
    }
}

/// THE ROOT-`App` GROWTH RATCHET.
///
/// `App` is meant to become lifecycle composition, so its field count may go
/// DOWN freely and may not go UP. This is deliberately an equality assertion
/// rather than a `<=`: a slice that removes five fields and adds one back
/// under a new name nets out under an inequality, and that is precisely the
/// move the item forbids.
#[test]
fn root_app_does_not_grow() {
    // Root App is lifecycle composition: seven domain handles plus host state.
    const CEILING: usize = 28;
    let fields = root_app_fields();
    assert_eq!(
        fields.len(),
        CEILING,
        "root `App` field count changed ({} vs {CEILING}). Fields now: {fields:?}\n\
         GOING UP is the thing this gate exists to stop — move the state into an \
         owner (docs/app-domains.md) instead of adding a field here. GOING DOWN is \
         the goal: lower this ceiling in the same commit.",
        fields.len()
    );
}

/// Extracted owners are explicit handles. `Deref` would make their private
/// state look like App fields again and restore the cross-domain reach.
#[test]
fn app_does_not_deref_across_extracted_runtime_boundaries() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app");
    let mut source =
        std::fs::read_to_string(root.join("../app.rs")).expect("src/app.rs must be readable");
    for path in rust_files_beneath(&root) {
        source.push_str(&std::fs::read_to_string(path).expect("app source must be readable"));
    }
    let deref = ["De", "ref for App"].concat();
    let deref_mut = ["De", "refMut for App"].concat();
    let input_deref = ["De", "ref for InputRuntime"].concat();
    let input_deref_mut = ["De", "refMut for InputRuntime"].concat();
    let frame_deref = ["De", "ref for FrameRuntime"].concat();
    let frame_deref_mut = ["De", "refMut for FrameRuntime"].concat();
    assert!(
        !source.contains(&deref)
            && !source.contains(&deref_mut)
            && !source.contains(&input_deref)
            && !source.contains(&input_deref_mut)
            && !source.contains(&frame_deref)
            && !source.contains(&frame_deref_mut),
        "App and its owner handles must not Deref/DerefMut across domain boundaries; \
         spell the owner explicitly"
    );
}

fn visible_impl_method_count(source: &str, owner: &str) -> usize {
    let marker = format!("impl {owner} {{");
    let mut rest = source;
    let mut count = 0;
    while let Some(start) = rest.find(&marker) {
        let body = &rest[start + marker.len()..];
        let mut depth = 1usize;
        let mut end = body.len();
        for (index, byte) in body.bytes().enumerate() {
            match byte {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = index;
                        break;
                    }
                }
                _ => {}
            }
        }
        count += body[..end].matches("pub(in crate::app) fn ").count();
        rest = &body[end.saturating_add(1)..];
    }
    count
}

#[test]
fn frame_runtime_api_does_not_regrow_into_a_field_bag() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app/frame");
    let mut count = 0;
    for file in ["mod.rs", "poll.rs", "presentation.rs", "surface.rs"] {
        let source = std::fs::read_to_string(root.join(file)).expect("frame source");
        count += visible_impl_method_count(&source, "FrameRuntime");
    }
    const CEILING: usize = 79;
    assert_eq!(
        count, CEILING,
        "FrameRuntime's visible API changed ({count} vs {CEILING}). A larger API \
         recreates root App's field bag behind one-field accessors; add a named \
         transition or typed read snapshot. When the API shrinks, lower this ratchet."
    );
}

#[derive(Clone, Copy)]
enum InputConsumer {
    Apply,
    Document,
    Open,
    RangeSettings,
    Rebind,
    Settings,
    ContextMenu,
    Drags,
    Ime,
    Keys,
    Mouse,
    Press,
    Probe,
    Schedule,
    Viewstate,
    Window,
}

impl InputConsumer {
    const ROSTER: &'static [Self] = &[
        Self::Apply,
        Self::Document,
        Self::Open,
        Self::RangeSettings,
        Self::Rebind,
        Self::Settings,
        Self::ContextMenu,
        Self::Drags,
        Self::Ime,
        Self::Keys,
        Self::Mouse,
        Self::Press,
        Self::Probe,
        Self::Schedule,
        Self::Viewstate,
        Self::Window,
    ];

    /// Production consumers of `App::input`. No wildcard arm: a new roster
    /// member cannot join without choosing whether it is inside the owner.
    fn path_and_reach(self) -> (&'static str, bool) {
        match self {
            Self::Apply => ("src/app/apply.rs", false),
            Self::Document => ("src/app/files/document.rs", false),
            Self::Open => ("src/app/files/open.rs", false),
            Self::RangeSettings => ("src/app/files/range_settings.rs", false),
            Self::Rebind => ("src/app/files/rebind.rs", false),
            Self::Settings => ("src/app/files/settings.rs", false),
            Self::ContextMenu => ("src/app/input/context_menu.rs", true),
            Self::Drags => ("src/app/input/drags.rs", true),
            Self::Ime => ("src/app/input/ime.rs", true),
            Self::Keys => ("src/app/input/keys.rs", true),
            Self::Mouse => ("src/app/input/mouse.rs", true),
            Self::Press => ("src/app/press.rs", false),
            Self::Probe => ("src/app/probe.rs", false),
            Self::Schedule => ("src/app/schedule.rs", false),
            Self::Viewstate => ("src/app/viewstate.rs", false),
            Self::Window => ("src/app/window.rs", false),
        }
    }
}

/// KeyboardInput and PointerInput are the only raw state projections. Every
/// sibling consumer must use an InputRuntime observation or transition.
#[test]
fn input_substates_are_private_and_every_consumer_is_swept() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input = std::fs::read_to_string(repo.join("src/app/input/mod.rs"))
        .expect("input owner source must be readable");
    assert_eq!(
        struct_fields(&input, "InputRuntime"),
        ["keyboard", "pointer"],
        "InputRuntime must contain exactly the two coherent substates"
    );
    assert_eq!(
        struct_fields(&input, "KeyboardInput"),
        [
            "keymap",
            "mods",
            "prefix_pending_at",
            "whichkey_shown",
            "hud_key",
            "hud_mods",
            "peek_arm",
            "peek_armed_at",
            "preedit",
            "ime_enabled",
        ],
        "keyboard state roster drifted"
    );
    assert_eq!(
        struct_fields(&input, "PointerInput"),
        [
            "pointer_hide",
            "cursor_px",
            "dragging",
            "drag_press_px",
            "drag_armed",
            "page_resizing",
            "page_resize_edge",
            "page_resize_anchor",
            "image_resizing",
            "range_drag",
            "cursor_icon",
            "drag_granularity",
            "last_click_time",
            "last_click_px",
            "click_count",
            "scroll_px_accum",
            "scroll_sensitivity",
        ],
        "pointer state roster drifted"
    );

    let mut roster_paths = std::collections::BTreeSet::new();
    for consumer in InputConsumer::ROSTER {
        let (relative, inside_owner) = consumer.path_and_reach();
        roster_paths.insert(relative.to_string());
        let source = std::fs::read_to_string(repo.join(relative))
            .unwrap_or_else(|e| panic!("{relative} must be readable: {e}"));
        assert!(
            source.contains("self.input."),
            "{relative} is a vacuous consumer"
        );
        let has_raw_reach =
            source.contains("self.input.keyboard") || source.contains("self.input.pointer");
        assert_eq!(
            has_raw_reach, inside_owner,
            "{relative} has the wrong InputRuntime reach; only app/input children \
             may project a substate"
        );
    }

    let mut discovered = std::collections::BTreeSet::new();
    for path in rust_files_beneath(&repo.join("src/app")) {
        let relative = path
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if relative.contains("/tests/") || relative.ends_with("/input/tests.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("app source must be readable");
        if source.contains("self.input.") {
            discovered.insert(relative);
        }
    }
    assert_eq!(
        discovered, roster_paths,
        "the no-wildcard input-consumer roster must cover every production consumer"
    );
}

fn rust_files_beneath(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).expect("app source directory must be readable") {
            let path = entry.expect("source entry must be readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn struct_fields<'a>(source: &'a str, name: &str) -> Vec<&'a str> {
    let marker = format!("struct {name} {{");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {marker}"));
    let body = &source[start + marker.len()..];
    let end = body
        .find('\n')
        .and_then(|_| body.find('}'))
        .expect("struct body");
    body[..end]
        .lines()
        .filter_map(|line| line.trim().split_once(':').map(|(field, _)| field.trim()))
        .collect()
}

/// Sanity: the parser reads the real struct, not an empty list. A silently
/// empty parse would make all three gates above vacuously green — the
/// classic way a structural law stops guarding anything.
#[test]
fn the_root_app_field_parser_is_not_vacuous() {
    let fields = root_app_fields();
    assert!(
        fields.len() > 20,
        "parsed only {} field(s) out of src/app.rs — the parser lost the struct \
         body and every gate in this file just went vacuous: {fields:?}",
        fields.len()
    );
    // Spot-check three fields of very different declaration shapes: a plain
    // one, a `#[cfg]`-gated one, and one whose type carries a generic
    // parameter list (the bracket-depth tracking).
    for expect in ["input", "wait_conns", "document"] {
        assert!(
            fields.iter().any(|f| f == expect),
            "the parser missed `{expect}`, so its declaration shape is not covered: {fields:?}"
        );
    }
    // And nothing that is NOT a field leaked in.
    for reject in ["match", "self", "if", "impl"] {
        assert!(
            !fields.iter().any(|f| f == reject),
            "the parser invented a field `{reject}`: {fields:?}"
        );
    }
}

// ── THE SUMMONED-LAYER BYPASS COUNT ─────────────────────────────────────
//
// `WorkspaceState` hands out two escape hatches from its own ladder, each
// justified in its own doc, and each is only justified while it has ONE call
// site:
//
//  - `popover_summon_bit()` — the raw summon flag, ladder-free, for the
//    cursor-icon composition's byte-identity.
//  - `core_slots()` — `&mut` on both slots, for the shared `ActionCtx` seam
//    for the shared action core. Production has exactly two: `App::apply`'s `run_action_core`
//    and its palette re-dispatch's `stamp_return_to`; the live search-key
//    intercept is the third, since `search::keys::intercept` is the seam
//    shared verbatim with the headless replay.
//
// COUNTING, not merely locating: CLAUDE.md's tripwire is that a
// needle-locating audit stays green forever while a second copy lives happily
// beside it. The needles are assembled at runtime so this file's own text
// cannot match them.
/// The files that NAME the two escape hatches without calling them: the owner
/// module (declaration + doc) and this law's own prose.
#[cfg(test)]
const DECLARING_FILES: &[&str] = &[
    "app/workspace/mod.rs",
    "app/workspace/tests.rs",
    "app/tests/domains.rs",
];

#[test]
fn the_summoned_layer_bypasses_have_the_call_sites_they_claim() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let raw_bit = ["popover_summon", "_bit("].concat();
    let mut bit_hits: std::collections::BTreeMap<String, usize> = Default::default();
    super::source_audit::scan_dir_collapsed(&root, &root, &raw_bit, &mut bit_hits);
    // Declaration + doc-reference in `app/workspace.rs`, the one consumer in
    // `app/input/mouse.rs`, and the summon-gate law's assertions.
    assert_eq!(
        bit_hits.get("app/input/mouse.rs"),
        Some(&1),
        "the raw popover summon bit must have exactly ONE consumer \
         (`sync_cursor_icon`); every other reader asks `popover_holds_attention`. \
         Found: {bit_hits:?}"
    );
    // Two files legitimately NAME the method without consuming it: its own
    // declaration + doc (`app/workspace.rs`), and this law's own prose. Both
    // are excluded by name rather than by a substring dodge, so a needle that
    // stops matching is a loud failure instead of a silent one.
    let outside: Vec<&String> = bit_hits
        .keys()
        .filter(|f| !DECLARING_FILES.contains(&f.as_str()) && f.as_str() != "app/input/mouse.rs")
        .collect();
    assert!(
        outside.is_empty(),
        "the raw popover summon bit leaked outside its one consumer: {outside:?}"
    );

    let slots = ["core_slo", "ts()"].concat();
    let mut slot_hits: std::collections::BTreeMap<String, usize> = Default::default();
    super::source_audit::scan_dir_collapsed(&root, &root, &slots, &mut slot_hits);
    let mut prod: Vec<(&str, usize)> = slot_hits
        .iter()
        .filter(|(f, _)| !DECLARING_FILES.contains(&f.as_str()))
        .map(|(f, n)| (f.as_str(), *n))
        .collect();
    prod.sort();
    assert_eq!(
        prod,
        vec![
            // `run_action_core` (1) + the palette `RunAction` re-dispatch's
            // `stamp_return_to` (1).
            ("app/apply.rs", 2),
            // The live search-key intercept — the seam shared verbatim with the
            // headless `--keys` replay.
            ("app/input/keys.rs", 1),
        ],
        "the summoned-layer slot lend must stay confined to the shared action-core \
         seam. A new site means something outside `actions::apply_transition` is \
         opening or closing a picker — route it through an `Action` instead."
    );
}

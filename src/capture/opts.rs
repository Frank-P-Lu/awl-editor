//! Public deterministic capture options and read-only metadata blocks.

/// Deterministic overrides for the verification hooks. All default to the
/// byte-stable baseline (the authored zoom default, cursor-follow scroll, no selection), so a
/// plain `--screenshot` is unaffected. Each field is applied verbatim into the
/// render snapshot, letting a reviewer capture a selection / zoom / semantic-scroll still
/// as a reproducible PNG.
/// MULTI-BUFFER registry snapshot for the sidecar `buffers` block: how many
/// buffers a `--keys` replay left open (the active one + anything still
/// backgrounded — see `crate::buffers::BufferRegistry`), and the active
/// buffer's identity (its path, or the literal `"scratch"`).
#[derive(Clone)]
pub struct BuffersInfo {
    pub open: usize,
    pub active: String,
}

/// THE WRITER'S DIFF (`crate::prosediff`): read-only STATE of an active prose-diff
/// view for the sidecar `diff` block — reported only when the capture harness
/// rendered a diff (`AWL_DIFF_OLD`/`AWL_DIFF_NEW`), so an agent can verify "am I
/// looking at a diff, and does it carry the deletions / insertions / moves / folds
/// I expect". APPEARANCE (the struck region is muted, the wash is present) is
/// asserted over the PNG's pixels, per the sidecar-vs-appearance tripwire — these
/// counts are a state oracle only. `active` is always true when present (the field
/// is `None` for every ordinary capture, so a plain `--screenshot` omits the block).
#[derive(Clone)]
pub struct DiffInfo {
    pub active: bool,
    /// The transcript's title (the diff view's heading — "Comparing versions").
    pub label: String,
    /// Paragraphs shown STRUCK whole (a coalesced rewrite's old side / a deletion).
    pub struck: usize,
    /// Paragraphs shown WASHED whole (a coalesced rewrite's new side / an insertion).
    pub washed: usize,
    /// Paragraphs edited IN PLACE (inline word/sentence segments).
    pub modified: usize,
    /// Relocated paragraphs, shown once at their new location.
    pub moved: usize,
    /// Folded unchanged-stretch rows (`⋯ N paragraphs unchanged ⋯`).
    pub folds: usize,
}

/// Read-only project metadata for the sidecar `project` block (`--root`-derived).
#[derive(Clone)]
pub struct ProjectInfo {
    pub root: std::path::PathBuf,
    pub name: String,
    pub branch: Option<String>,
    pub dirty: bool,
    /// The EFFECTIVE default_folder (flag > config > `~/notes`) — the FIRST-RUN
    /// fallback only (not the active folder), surfaced so a
    /// `--config`-driven launch's configured folder is verifiable from the sidecar
    /// with no flags. `None` (timeline/held paths) -> JSON null.
    pub default_folder: Option<std::path::PathBuf>,
    /// The EFFECTIVE workspace (flag > config > root.parent). `None` -> JSON null.
    pub workspace: Option<std::path::PathBuf>,
    /// THE KEYMAP FLAVOR ROUND — the EFFECTIVE keymap flavor's config NAME
    /// (`"native"`/`"emacs"`, see `crate::keymap::KeymapFlavor::config_name`),
    /// so a `--config`-driven launch's `keymap = "emacs"` is verifiable from the
    /// sidecar with no flags, mirroring `default_folder`/`workspace` above. Every
    /// construction site defaults it to `"native"` (the built-in default),
    /// keeping a plain capture with no `--config` byte-identical.
    pub keymap_flavor: &'static str,
}

/// Summoned-overlay state for the sidecar `overlay` block. Populated when a
/// `--keys` replay left the go-to / switch overlay open (or when it accepted).
#[derive(Clone)]
pub struct OverlayInfo {
    pub active: bool,
    pub mode: &'static str,
    /// The alignment this overlay
    /// FROZE at summon ([`crate::overlay::OverlayState::align`]), carried verbatim so
    /// a single-frame `--keys` capture places the card at the SAME anchor the live
    /// picker held (and a preview-crossing capture holds it, exactly like live). Fed
    /// into `ViewState::overlay_align`; the `AWL_OVERLAY_ALIGN` capture knob is what
    /// this reflects for the audition gallery's right-aligned shots.
    pub align: crate::theme::CardAnchor,
    pub query: String,
    pub items: Vec<String>,
    /// EMPTY STATE: the shared calm message shown when NO rows match (empty corpus →
    /// per-kind "no history yet"/"no suggestions"/…; a query that matched nothing →
    /// "no matches"), or `None` when there ARE rows. From the one owner
    /// [`crate::overlay::OverlayState::empty_notice`]; emitted as `overlay.empty`.
    pub empty: Option<String>,
    /// Command palette only: binding labels parallel to `items` (each command's
    /// current chord). Empty for every other mode; emitted as a parallel array so
    /// the palette's binding column is verifiable from the sidecar.
    pub bindings: Vec<String>,
    /// SETTINGS menu only: the RAIL FRACTION (0..1) parallel to `items` —
    /// `Some(frac)` for a `SettingKind::Range` row, `None` for an ordinary row, and
    /// the whole vec EMPTY for every other mode. Emitted as `overlay.ranges`, so a
    /// `--keys` step of the rail is assertable from the sidecar (the row's `bindings`
    /// cell carries the same value as TEXT — the two must agree). From the one owner
    /// [`crate::overlay::OverlayState::item_range_fracs`], which derives it through
    /// the range spec from the row's own quantized step.
    pub ranges: Vec<Option<f32>>,
    /// Project / Browse pickers only: a dim `"git"` tag parallel to `items` for each
    /// row that is itself a git repo (`""` otherwise); EMPTY when no row is a git repo.
    /// From the one owner [`crate::overlay::OverlayState::item_git_tags`]; emitted as a
    /// parallel array (`overlay.git`) so the repo tags are verifiable from the sidecar.
    pub git: Vec<String>,
    pub selected_index: usize,
    /// The per-kind control-hint line drawn dim at the foot of the card (e.g.
    /// "↵ select   → open   ← up" for switch-project), formatted by the one shared
    /// [`crate::overlay::format_hint`] owner. Surfaced to the sidecar so the
    /// discoverability hint is agent-verifiable.
    pub hint: String,
    /// Browse only: the root-relative directory the current level lists (`None` =
    /// the root). Surfaced so a `--keys` descend/ascend is verifiable; emitted as
    /// JSON null for the goto/switch modes.
    pub browse_dir: Option<String>,
    /// Spell picker only: the misspelled word's `(line, start_col, end_col)` CHAR span,
    /// so the capture path can anchor the contextual float panel AT the word (and the
    /// sidecar can report it). `None` for every other mode.
    pub spell_target: Option<(usize, usize, usize)>,
    /// Context-menu click anchor in physical pixels; `None` for every other card.
    pub context_anchor: Option<(f32, f32)>,
    /// Keybindings rebind menu only: the active CAPTURE sub-state (the command being
    /// rebound, the phase, the KEY/CHORD mode, and the combos captured so far), or
    /// `None` while browsing the list / for every other mode. Emitted as the sidecar
    /// `overlay.capture` block so the rebind flow is agent-verifiable.
    pub capture: Option<CaptureInfo>,
    /// Keybindings menu only: the transient NOTICE line (conflict / saved / reset).
    /// Empty otherwise; emitted as `overlay.notice`.
    pub notice: String,
    /// FACETING pickers (Go-to / Browse / Switch-project / Command / History /
    /// Settings): the ACTIVE lens name (e.g. `"recent"`/`"file"`/`"session"`/`"all"`),
    /// or `None` for a flat, non-faceting picker (the theme picker is flat — its lens
    /// strip was retired 2026-07-15). Emitted as `overlay.lens` so a `--keys` lens
    /// switch is verifiable.
    pub lens: Option<&'static str>,
    /// FACETING pickers only: the lens STRIP — each lens label + a flag marking the
    /// active one. Drives the rendered strip; emitted as `overlay.lens_strip`. Empty
    /// for a flat picker.
    pub lens_strip: Vec<(String, bool)>,
    /// FACETING pickers only: the SECTION label per `items` row (parallel), so the
    /// grouping is drawable + assertable. Empty for a flat picker / the All lens.
    pub sections: Vec<String>,
    /// The SUBJECT of the read-only comparison the capture is previewing in the
    /// document: a restore id on a timeline, the conflicted file's path on a
    /// conflict workspace (paired with [`CaptureOpts::preview_text`] — the
    /// previewed text is the transcript, not the raw version). `None` for every
    /// mode that shows no comparison and for the empty-state row. Emitted as
    /// `overlay.preview_id`.
    pub preview_id: Option<String>,
    /// WHICH READ-ONLY VIEW of that subject
    /// ([`crate::overlay::ComparisonView::tag`] — `"diff"` / `"mine"` /
    /// `"theirs"`), or `None` when there is no comparison. Emitted as
    /// `overlay.preview_view`.
    ///
    /// It is a separate key from `preview_id` because it is a separate fact, and
    /// because it is the ONE fact that distinguishes the three views a conflict
    /// workspace offers of ONE subject — a sidecar carrying the subject alone
    /// could not tell them apart, which is precisely the cache collision
    /// [`crate::overlay::ComparisonRequest::cache_key`] exists to prevent, read
    /// back out at the oracle end.
    pub preview_view: Option<&'static str>,
    /// Is this card drawn as a SUMMONED WORKSPACE (the viewport, with a
    /// navigation rail beside its content) rather than a contextual card? Emitted
    /// as `overlay.workspace`. When it is true, `lens_strip` IS the rail — the
    /// same category data, stood on its end — so there is no second array to read.
    pub workspace: bool,
    /// Does the summoned surface's DETAIL stage hold the keyboard? For History's
    /// diff-as-preview that is the diff panel (↑/↓ scroll it; its border
    /// strengthens); for Settings it is the content pane
    /// beside the rail. One fact, one field: emitted as `overlay.detail_focus`,
    /// and false for every card with no second region.
    pub detail_focus: bool,
    /// DIFF-AS-PREVIEW (History only): the diff panel's scroll in VISUAL ROWS
    /// (PgUp/PgDn / panel ↑/↓ / the wheel over the page). Emitted as
    /// `overlay.diff_scroll`; always 0 elsewhere.
    pub diff_scroll: usize,
    /// File pickers only (go-to / browse): whether dot-prefixed entries (AND,
    /// unsupported/binary Browse rows) are REVEALED. Mirrors the
    /// STICKY `crate::file_visibility::all_on()` global (Settings -> Files ->
    /// "File visibility") gated on `kind.hides_dotfiles()`, NOT a per-picker
    /// flag any more (the old `Cmd-Shift-.` toggle +
    /// `OverlayState::show_hidden`) — so the OBSERVABLE CONTRACT is unchanged:
    /// `false` for a non-file picker (Theme/Settings/…), `items` already
    /// reflects the filtering either way. Emitted as `overlay.show_hidden`.
    pub show_hidden: bool,
    /// BREADCRUMB: the summoning overlay's mode string (`"settings"` / `"command"`) to
    /// re-summon when THIS picker POPS (Esc / value-pick), or `None` for a top-level
    /// summon that closes to the buffer. From [`crate::overlay::OverlayState::return_to`]
    /// via its `as_str`; emitted as `overlay.return_to` so a `--keys` breadcrumb chain
    /// (palette → theme → Esc → palette) is assertable straight from the sidecar.
    pub return_to: Option<&'static str>,
    /// THE OVERLAY-TITLES ROUND: this picker's short, lowercase self-announcement
    /// ([`crate::overlay::OverlayKind::title`]) — the same text the render path
    /// draws as a quiet prefix on the input line. Emitted as `overlay.title` so the
    /// destination of a palette→picker route is agent-verifiable straight from the
    /// sidecar (a picker with no title-carrying render surface still reports it
    /// here — the law is "every kind names itself", not "every kind draws it").
    pub title: &'static str,
}

pub(super) fn spell_target_json(target: Option<(usize, usize, usize)>) -> String {
    target
        .map(|(l, s, e)| format!("[{l}, {s}, {e}]"))
        .unwrap_or_else(|| "null".into())
}

pub(super) fn context_anchor_json(anchor: Option<(f32, f32)>) -> String {
    anchor
        .map(|(x, y)| format!("[{x}, {y}]"))
        .unwrap_or_else(|| "null".into())
}

/// The Keybindings menu's capture sub-state for the sidecar `overlay.capture` block.
#[derive(Clone)]
pub struct CaptureInfo {
    /// The command being rebound (display name).
    pub command: String,
    /// The phase: `"choose"` (Key vs Chord) / `"recording"` / `"confirm"`.
    pub stage: &'static str,
    /// `true` while capturing a CHORD sequence, `false` for a single KEY.
    pub chord_mode: bool,
    /// The combos captured so far (each a canonical chord spec).
    pub captured: Vec<String>,
    /// The dim prompt line the card shows for this phase.
    pub prompt: String,
}

/// WHICH EDITOR PRODUCED THIS SIDECAR — the tier, per
/// `docs/harness-reach.md`, that the capture was driven at. Two sidecars with
/// identical fields can mean different things: a tier-1 capture replays the
/// SHARED CORE and skips the live-`App`-only effects it owns no capability for
/// (they show up in `replay_skips`), while a tier-2 capture drives a real `App`
/// and performs them for real. An oracle that cannot say which it was is an
/// oracle a reader has to guess about, so the sidecar says so itself.
// `LiveApp` is constructed only by the native-only `--screenshot-app` mode; the
// field itself, and `Replay`, are carried on every target.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CaptureDriver {
    /// The SHARED CORE (`actions::apply_transition`) — every `--screenshot`,
    /// `--keys`, `--storyboard`, `--capture-timeline`/`-held` and
    /// `--screenshot-frames` capture. Tier 1.
    #[default]
    Replay,
    /// A real headless live [`crate::app::App`] (`--screenshot-app`), driven by
    /// real chords through `dispatch_pressed_key` → the keymap → `App::apply`.
    /// Tier 2 — nothing is skipped, because nothing is stood in for.
    LiveApp,
}

impl CaptureDriver {
    /// The sidecar's own `driver` string — the tier that drove the frame.
    /// Hyphenated, matching the flag that selects it, so a reader can grep one
    /// for the other.
    pub fn as_str(self) -> &'static str {
        match self {
            CaptureDriver::Replay => "replay",
            CaptureDriver::LiveApp => "live-app",
        }
    }
}

#[derive(Clone, Default)]
pub struct CaptureOpts {
    /// Zoom factor (`None` = [`crate::range::ZOOM`]'s authored default).
    pub zoom: Option<f32>,
    /// Explicit semantic document top (None = cursor-follow default).  A
    /// fixed-point offset makes sub-row geometry reproducible in capture.
    pub scroll: Option<crate::render::ScrollPos>,
    /// Selection as ((l0,c0),(l1,c1)) in line/col (None = no selection).
    pub selection: Option<((usize, usize), (usize, usize))>,
    /// Synthetic IME preedit (composition) string to render at the cursor for the
    /// IME verify path (None/empty = no composition). Drawn underlined via the
    /// same Advanced-shaping path as the live IME overlay; never enters the
    /// buffer, so the capture stays deterministic.
    pub preedit: Option<String>,
    /// Live isearch query to render the panel + highlights deterministically
    /// (None = no search). Matches are computed against the loaded buffer.
    pub search: Option<String>,
    /// Case-sensitive toggle for the headless search (default false).
    pub search_case_sensitive: bool,
    /// REPLACE mode revealed on the search panel (default false). A `--keys`
    /// replay of Cmd-R / Tab / Cmd-Option-F opens it through the SAME
    /// interception seam the live panel uses (`crate::search::keys`), so this
    /// is verifiable from the capture.
    pub search_replace_active: bool,
    /// The replacement string — typed headlessly through the shared search-key
    /// seam (a `--keys "Cmd-r <needle> Tab <text>"` replay fills it; the old
    /// always-empty "isearch-input gap" is retired).
    pub search_replacement: String,
    /// Whether typing currently edits the REPLACEMENT field (vs. the query) —
    /// a replayed Tab/Cmd-R focus move folds in here so the panel's focused
    /// row + the sidecar's `editing_replacement` reflect it (default false).
    pub search_editing_replacement: bool,
    /// The active project (`--root`-derived) for the sidecar `project` block.
    /// None (default) -> `project: null` so a plain `--screenshot` is unchanged.
    pub project: Option<ProjectInfo>,
    /// The summoned overlay state for the sidecar `overlay` block. None ->
    /// overlay inactive.
    pub overlay: Option<OverlayInfo>,
    /// PHYSICAL canvas dimensions for this run (`--capture-size WxH`). `None` =
    /// the byte-stable default [`super::CANVAS_WIDTH`]x[`super::CANVAS_HEIGHT`]
    /// (1200x800), so a plain `--screenshot` is unchanged. Lets a capture render at
    /// the REAL window size so size-dependent layout bugs (e.g. the page
    /// right-margin) are visible.
    pub canvas: Option<(u32, u32)>,
    /// Display DPI `scale_factor` fed to the renderer metrics (`--capture-dpi N`).
    /// `None` = 1.0 (today's implied capture scale, a no-op via `set_dpi`'s guard),
    /// so the no-flag path stays byte-identical. A 2400x1600 canvas at dpi 2.0
    /// renders like a 1200x800 LOGICAL retina window (text + column geometry scale
    /// exactly like the live retina app).
    pub dpi: Option<f32>,
    /// The WHICH-KEY panel's `(key, command-name)` rows to render (`--whichkey`), or
    /// `None` (default) so a plain `--screenshot` draws no panel and stays
    /// byte-identical. Populated in `run.rs` from the command catalog + config when the
    /// `--whichkey` force-global is set, so the capture shows the SETTLED summoned panel
    /// deterministically (the live 500ms pause is windowed; only the shown STATE is
    /// captured here).
    pub whichkey: Option<Vec<(String, String)>>,
    /// HISTORY timeline live preview: the CONTENT of the version the still-open
    /// History overlay's highlighted row resolves to (paired with the
    /// `OverlayInfo::preview_id`). Folded over the render snapshot's `text`
    /// BEFORE the scroll math, exactly like the live preview — the capture then
    /// shows THAT VERSION in the document itself, and the sidecar `text` reports
    /// it (assertable). `None` (default) = no preview, so a plain `--screenshot`
    /// is unchanged. Populated in `run.rs` from the replay's open overlay.
    pub preview_text: Option<String>,
    /// MULTI-BUFFER registry snapshot for the sidecar `buffers` block. `None`
    /// (default) means "derive it from the loaded buffer alone" (`open: 1`,
    /// `active` = its path or `"scratch"`) — so a plain `--screenshot` needs no
    /// wiring, and every test/caller that never touches multi-buffer state
    /// gets a sensible default. Populated in `run.rs`'s main capture path from
    /// the replay's registry count.
    pub buffers: Option<BuffersInfo>,
    /// Unsupported live-only effects a permissive `--keys` replay skipped, in
    /// replay order. Empty for an ordinary capture and for a replay that crossed
    /// no unsupported seam.
    pub replay_skips: Vec<crate::replay::SkippedEffect>,
    /// THE WRITER'S DIFF: read-only STATE of an active prose-diff view for the
    /// sidecar `diff` block. `None` (default) for every ordinary capture — the block
    /// is omitted, so a plain `--screenshot` is byte-identical. Populated only by the
    /// capture harness's env-gated diff render (`AWL_DIFF_OLD`/`AWL_DIFF_NEW`).
    pub diff: Option<DiffInfo>,
    /// Is the captured document holding an UNRESOLVED external change? Drives
    /// the page-mode gutter's persistent `changed elsewhere` affordance and is
    /// emitted as `gutter.changed`. Only a `driver: "live-app"` capture can ever
    /// set it — see `run::CaptureSubject::changed_elsewhere`.
    pub gutter_changed: bool,
    /// THE CALM NOTICE the driven editor is showing, with its kind — `None` when
    /// there is none, which is every capture that raises no notice-bearing
    /// effect (so those stay byte-identical).
    ///
    /// This slot exists because the notice channel was invisible to every capture
    /// door for as long as it existed: `CaptureOpts` had nowhere to put it, so a
    /// live `App` that had genuinely raised "saved" produced a PNG byte-identical
    /// to one that had not — while the SAME sidecar's `semantic` block announced
    /// that notice to a screen reader. One artifact, two answers. Fed by
    /// [`crate::run::CaptureSubject::notice`] from whichever driver is running;
    /// emitted as the top-level `notice` block.
    pub notice: Option<(String, crate::actions::NoticeKind)>,
    /// Explicit passive pending-crash state for the About-card capture law.
    /// False by default; ordinary/headless captures never inspect ambient crash
    /// files and remain deterministic.
    pub pending_crash: bool,
    /// WHICH DRIVER produced this capture ([`CaptureDriver`]) — emitted as the
    /// top-level `driver` field. Defaults to `Replay`, so every existing capture
    /// path is unchanged; only `--screenshot-app` sets `LiveApp`.
    pub driver: CaptureDriver,
    /// The exact renderer-independent semantic tree owned by a live `App`.
    /// Replay captures leave this `None`; `--screenshot-app` embeds the same
    /// value `--semantic-json` and AccessKit consume.
    pub semantic: Option<crate::semantic::SemanticSnapshot>,
    /// FORCE the format popover over the current `selection` (the deterministic
    /// in-test equivalent of the CLI's `AWL_POPOVER` env probe — the live summon is
    /// a mouse gesture the headless path has no pointer for). Default false, so an
    /// ordinary capture is byte-identical; the `popover.rs` card-fits law sets it
    /// to render the toolbar without racing a process-global env var.
    pub force_popover: bool,
}

impl CaptureOpts {
    /// The `semantic` sidecar field: the live-App semantic tree serialized
    /// verbatim, or JSON `null`. Lives beside the field rather than in the
    /// writer so the one place that knows the field also knows its absence.
    pub(super) fn semantic_json(&self) -> String {
        self.semantic
            .as_ref()
            .map(|snapshot| serde_json::to_string(snapshot).expect("semantic snapshot serializes"))
            .unwrap_or_else(|| "null".to_string())
    }
}

enum_with_all! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OverlayKind {
        Goto,
        Project,
        ProjectBrowse,
        Browse,
        Theme,
        Caret,
        Dictionary,
        CjkLang,
        Date,
        MoveDest,
        Command,
        Spell,
        Keybindings,
        History,
        Conflict,
        Settings,
        Assets,
        Rename,
        InsertLink,
        KeepName,
        Context,
        ExportDest,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptDisposition {
    Navigate,
    ValuePick,
    StayOpen,
}

impl OverlayKind {
    pub fn from_mode(mode: &str) -> Option<OverlayKind> {
        Self::ALL.iter().copied().find(|k| k.as_str() == mode)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            OverlayKind::Goto => "goto",
            OverlayKind::Project => "switch",
            OverlayKind::ProjectBrowse => "project_browse",
            OverlayKind::Browse => "browse",
            OverlayKind::Theme => "theme",
            OverlayKind::Caret => "caret",
            OverlayKind::Dictionary => "dictionary",
            OverlayKind::CjkLang => "cjk_lang",
            OverlayKind::Date => "date",
            OverlayKind::MoveDest => "move",
            OverlayKind::Command => "command",
            OverlayKind::Spell => "spell",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "history",
            OverlayKind::Conflict => "conflict",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "assets",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert_link",
            OverlayKind::KeepName => "keep_version",
            OverlayKind::Context => "context",
            OverlayKind::ExportDest => "export_dest",
        }
    }

    pub fn accept_disposition(self) -> AcceptDisposition {
        use AcceptDisposition::*;
        match self {
            OverlayKind::Goto
            | OverlayKind::Browse
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Spell
            | OverlayKind::History
            | OverlayKind::Command
            | OverlayKind::Context => Navigate,
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date => ValuePick,
            OverlayKind::Assets
            | OverlayKind::Keybindings
            | OverlayKind::Settings
            | OverlayKind::Conflict => StayOpen,
            OverlayKind::Rename => Navigate,
            OverlayKind::InsertLink => Navigate,
            OverlayKind::KeepName => Navigate,
        }
    }

    #[allow(dead_code)] // consumed only by overlay::tests's runtime roster sweep today.
    pub fn row_meta_roster(self) -> &'static [super::RowMetaTag] {
        use super::RowMetaTag::*;
        match self {
            OverlayKind::Goto => &[GotoFile, GotoHeading],
            OverlayKind::Command => &[Plain, CommandHidden, CommandSetting],
            OverlayKind::Context => &[Plain],
            OverlayKind::Spell => &[Plain, SpellAdd],
            OverlayKind::History => &[History],
            OverlayKind::Conflict => &[Plain],
            OverlayKind::Project => &[Plain, ProjectDoor],
            OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Keybindings
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName => &[Plain],
        }
    }

    /// PREVIEWS THE LIVE DOCUMENT: does moving the highlight in this picker
    /// repaint the page BEHIND the card, before anything is committed?
    ///
    /// The ONE owner of that question, and the property
    /// [`Self::keeps_backdrop_crisp`] spends — the two are asserted EQUAL over
    /// [`Self::ALL`], so a kind cannot earn the frost exemption without declaring
    /// the audition that pays for it, or declare the audition and inherit frost
    /// over the very thing its rows are showing.
    /// `actions::overlay_nav::preview_overlay` gates on this before it mutates
    /// anything, and the accept path asks it to decide which Enter is a KEEP of an
    /// already-live value rather than a fresh apply.
    ///
    /// NOT this, though all three are [`AcceptDisposition::ValuePick`]s: a card
    /// whose rows preview INSIDE THEMSELVES — the date formats render today's
    /// date, the dictionary and CJK pickers pre-select the live value — because
    /// nothing behind the card changes as the highlight moves, so frost costs
    /// them nothing. Nor the version timeline, whose highlighted row substitutes
    /// a snapshot into the workspace's own comparison region rather than into the
    /// page behind its card.
    ///
    /// Exhaustive rather than `matches!`: a new picker that auditions a value
    /// answers here, which is what forces the frost decision next door.
    pub fn previews_live_document(self) -> bool {
        match self {
            OverlayKind::Theme | OverlayKind::Caret => true,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest => false,
        }
    }

    /// THE CRISP-BACKDROP SET: does this card leave the DOCUMENT behind it
    /// unfrosted? Exhaustive rather than `matches!`, because the answer is a
    /// composition decision and a new kind must make it here instead of
    /// inheriting the frost.
    ///
    /// A card frosts what it covers (DESIGN §5: a summoned surface recedes the
    /// room). The exception is earned by exactly one property, and it has its own
    /// owner: [`Self::previews_live_document`] — the theme picker repaints the
    /// page under itself, the caret picker poses the real caret — where frosting
    /// would blur the very thing the row is showing you. The two predicates are
    /// asserted EQUAL over [`Self::ALL`], not merely nested, so neither can drift
    /// past the other; being an [`AcceptDisposition::ValuePick`] is necessary but
    /// nowhere near sufficient (three value-pickers preview inside their own rows
    /// and want the frost). A comparison is NOT this either: it composites inside
    /// the workspace's own content region, so what sits behind its card is the
    /// user's untouched document — a quiet backdrop, which is exactly what frost
    /// is for.
    ///
    /// Read by the LIVE door (`App::sync_view`) and by the CAPTURE door
    /// (`capture::modes::settled_viewstate`, which arrives holding a serialized
    /// mode string and resolves it through [`Self::from_mode`]) — one owner, so
    /// a headless frame cannot disagree with the running editor about frost.
    pub fn keeps_backdrop_crisp(self) -> bool {
        match self {
            OverlayKind::Theme | OverlayKind::Caret => true,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest => false,
        }
    }

    /// A DESTINATION NAVIGATOR: a folders-only walk whose accept names the
    /// folder you stopped on. Every member shares the entire navigation grammar
    /// — `→` descends, `←` ascends, `↵` takes the highlighted folder — and they
    /// differ only in WHAT lands there and WHICH tree they walk, which is why
    /// every navigation site asks this instead of naming one kind and growing a
    /// second branch later.
    ///
    /// The two `Dest` members walk the ACTIVE ROOT and put something in the
    /// folder; [`Self::ProjectBrowse`] walks the WORKSPACE by absolute path and
    /// makes the folder the project. Only the first two take a typed name that
    /// does not exist yet (`actions::overlay_nav::dest_value`'s `allow_new`): a
    /// move creates the folder it names, and there is nothing to switch to in a
    /// folder that isn't there.
    pub fn is_folder_destination(self) -> bool {
        matches!(
            self,
            OverlayKind::MoveDest | OverlayKind::ExportDest | OverlayKind::ProjectBrowse
        )
    }

    /// THIS KIND IS BUILT FROM A DIRECTORY LEVEL, so [`super::build`] cannot
    /// make one — [`super::browse_level`] does, from a path the caller supplies.
    /// The ONE owner of that split: a resume rebuilds a parked parent through
    /// whichever builder can answer for its kind
    /// (`actions::overlay_nav::resume_rebuild`), and a parked explorer handed
    /// only `build` resolves to `None` and drops the whole journey to the editor
    /// instead of coming back.
    ///
    /// Exhaustive rather than `matches!`: a new explorer answers here, or it is
    /// unresumable in a way nothing reports.
    pub fn needs_dir_level(self) -> bool {
        match self {
            OverlayKind::Browse
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse => true,
            OverlayKind::Goto
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context => false,
        }
    }

    /// THE FLAT SWITCH-PROJECT PICKER'S ONE DOOR — the label of the row that
    /// summons [`Self::ProjectBrowse`]. Its `…` is the promise of a further
    /// surface (`menu::ellipsis_law`'s subject, read here on a picker row rather
    /// than a menu item), and `actions::tests::project_door` holds the promise:
    /// the row opens a card, it never switches on the spot.
    ///
    /// The row is identified by [`super::RowMeta::ProjectDoor`] everywhere it is
    /// acted on, never by this string, so the wording stays only a wording.
    pub const BROWSE_DOOR_LABEL: &'static str = "Browse for folder…";

    pub fn hides_dotfiles(self) -> bool {
        matches!(
            self,
            OverlayKind::Goto
                | OverlayKind::Browse
                | OverlayKind::MoveDest
                | OverlayKind::ExportDest
                | OverlayKind::Project
                | OverlayKind::ProjectBrowse
        )
    }

    pub const MAX_SUGGESTIONS: usize = 5;

    pub fn window_rows(self) -> usize {
        match self {
            OverlayKind::Spell => Self::MAX_SUGGESTIONS + 1,
            OverlayKind::Context => 8,
            OverlayKind::Theme => crate::theme::THEMES.len(),
            // The workspace canvas, not this roster, bounds Settings rows.
            OverlayKind::Settings => crate::settings::SETTINGS.len(),
            _ => 12,
        }
    }
    pub fn hint_actions(self) -> Vec<HintAction> {
        let mut actions = vec![HintAction {
            glyph: "type",
            label: "to filter",
        }];
        actions.extend(self.kind_actions());
        actions
    }
    fn kind_actions(self) -> Vec<HintAction> {
        let enter = |label| HintAction {
            glyph: "\u{21B5}",
            label,
        };
        let key = |glyph, label| HintAction { glyph, label };
        match self {
            OverlayKind::Project => vec![
                enter("select"),
                key(ARROWS_LR, "lens"),
                key("\u{232B}", "up"),
            ],
            OverlayKind::MoveDest => vec![
                enter("move here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            // THE EXPORT DESTINATION. The same folder-navigator grammar as the
            // move destination above, and the same three keys — the verb in the
            // accept cell is the only difference, because the only difference is
            // what lands in the folder you stop on.
            OverlayKind::ExportDest => vec![
                enter("export here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            // THE SWITCH-PROJECT DOOR. The same three keys as the two
            // destinations above, because it is the same walk — only the verb
            // in the accept cell differs, and here the folder you stop on
            // becomes the project.
            OverlayKind::ProjectBrowse => vec![
                enter("switch here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            OverlayKind::Browse => {
                vec![enter("open"), key(ARROWS_LR, "lens"), key("\u{232B}", "up")]
            }
            OverlayKind::Goto => vec![enter("open"), key(ARROWS_LR, "lens")],
            OverlayKind::Theme => vec![enter("keep"), key("esc", "revert")],
            OverlayKind::Caret => vec![enter("apply")],
            OverlayKind::Dictionary => vec![enter("apply")],
            OverlayKind::CjkLang => vec![enter("apply")],
            OverlayKind::Date => vec![enter("apply")],
            OverlayKind::Command => super::command_hint_actions(),
            OverlayKind::Spell => vec![enter("replace")],
            OverlayKind::Context => vec![enter("choose"), key("esc", "close")],
            OverlayKind::Keybindings => {
                vec![enter("rebind"), key("del", "reset"), key("esc", "close")]
            }
            // THE THREE READ-ONLY VIEWS. `↵` is the workspace grammar's own
            // "into the content" (`workspace_nav::rows_primary_intercept`) —
            // which is what you want when a version is longer than the pane. It
            // commits nothing: neither resolution is reachable by pressing a key
            // on a page of prose. `esc` names the OUTCOME rather than the key's
            // usual meaning, because leaving here resolves nothing.
            OverlayKind::Conflict => vec![enter("read"), key("esc", "keep editing")],
            // Restore is deliberately gated behind ⇧↵.
            OverlayKind::History => vec![
                enter("compare"),
                key("\u{21E7}\u{21B5}", "restore"),
                key(ARROWS_LR, "lens"),
            ],
            // The rows pane. `esc` does not go back — it leaves — so the Back is
            // a key of its own, appended by `foot_hint` from the one owner that
            // knows which key is free (`OverlayState::detail_back`), never a
            // fact about the KIND; `esc close` stays on the rail's line, because
            // a fifth cell overruns the card on a narrow Bars world. AND NO
            // `←/→` CELL: on this workspace those keys are the region seam's
            // (`detail_left_returns`) unless a Range row's value rail owns them.
            OverlayKind::Settings => vec![enter("edit")],
            OverlayKind::Assets => vec![enter("trash"), key("esc", "close")],
            OverlayKind::Rename => vec![enter("rename"), key("esc", "cancel")],
            OverlayKind::InsertLink => vec![enter("insert link"), key("esc", "cancel")],
            OverlayKind::KeepName => vec![enter("keep"), key("esc", "cancel")],
        }
    }
    pub fn hint(self) -> String {
        format_hint(&self.hint_actions())
    }
    /// [`OverlayKind::Project`]'s line MINUS the `⌫ up` cell — the flat
    /// switch-project picker's own statement, for the one card shape that
    /// draws as this kind but never ascends
    /// (`actions::overlay_nav::overlay_intercept`'s `DeleteBackward`
    /// arm, gated on `Bind::Path` rather than on the kind). Every other cell
    /// stays: `↵ select` and the lens strip are true of both features this
    /// kind draws as, only the ascend affordance is not.
    ///
    /// [`super::OverlayState::foot_hint_scoped`] is the only caller, and only
    /// when its `bind` says the card is the flat feature — the Settings
    /// folder-VALUE picker (`Bind::Path`) keeps [`Self::hint`] verbatim, and
    /// so does the roster sweep in `overlay::tests::hints`, which has no
    /// instance to ask and so states the kind's general (ascending) grammar.
    pub(crate) fn project_flat_hint(self) -> String {
        debug_assert_eq!(self, OverlayKind::Project, "no other kind is bind-scoped");
        let actions: Vec<HintAction> = self
            .hint_actions()
            .into_iter()
            .filter(|a| a.glyph != super::workspace::ERASE_GLYPH)
            .collect();
        format_hint(&actions)
    }
    /// The RANGE-row line's CELLS — [`Self::hint_actions`] with the `←/→` cell
    /// re-labelled for the row's own rail. Split out from
    /// [`Self::range_row_hint`] so `foot_hint` can append the workspace's
    /// derived Back cell to it exactly as it does to an ordinary detail line,
    /// rather than formatting a sentence and then trying to edit the string.
    pub fn range_row_actions(self) -> Vec<HintAction> {
        let mut actions = self.hint_actions();
        match actions.iter_mut().find(|a| a.glyph == ARROWS_LR) {
            Some(cell) => cell.label = RANGE_LR_LABEL,
            None => actions.push(HintAction {
                glyph: ARROWS_LR,
                label: RANGE_LR_LABEL,
            }),
        }
        actions
    }

    pub fn range_row_hint(self) -> String {
        format_hint(&self.range_row_actions())
    }

    pub fn empty_corpus_message(self) -> &'static str {
        match self {
            OverlayKind::History => "no history yet",
            // Unreachable in practice: the workspace is only summoned with a
            // conflict open, and it always carries its three fixed views.
            OverlayKind::Conflict => "nothing to review",
            OverlayKind::Spell => "no suggestions",
            OverlayKind::Browse => "this folder is empty",
            OverlayKind::Goto | OverlayKind::Project | OverlayKind::MoveDest => "no files here",
            // A destination list holds FOLDERS only, so "no files here" would name
            // the wrong absence.
            OverlayKind::ExportDest | OverlayKind::ProjectBrowse => "no folders here",
            OverlayKind::Assets => "no unused assets",
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Command
            | OverlayKind::Keybindings
            | OverlayKind::Settings => "no matches",
            OverlayKind::Rename => "no matches",
            OverlayKind::InsertLink => "no matches",
            OverlayKind::KeepName => "no matches",
            OverlayKind::Context => "no actions",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            OverlayKind::Goto => "go to",
            OverlayKind::Project => "switch project",
            // The DOOR's own card. Names the errand you are on rather than the
            // surface you left, so the title and the row you pressed read as one
            // thing.
            OverlayKind::ProjectBrowse => "browse for folder",
            OverlayKind::Browse => "browse",
            OverlayKind::Theme => "themes",
            OverlayKind::Caret => "caret style",
            OverlayKind::MoveDest => "move note",
            // Names the QUESTION the list answers rather than the verb that
            // brought you here: every row is a folder, and reading the title
            // into the highlighted row completes the sentence.
            OverlayKind::ExportDest => "export to",
            OverlayKind::Dictionary => "dictionary",
            OverlayKind::CjkLang => "ambiguous cjk",
            OverlayKind::Date => "date format",
            OverlayKind::Command => "commands",
            OverlayKind::Spell => "spelling",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "version history",
            // The SAME words the persistent gutter affordance uses, so the thing
            // you noticed and the place you review it read as one thing.
            OverlayKind::Conflict => "changed elsewhere",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "unused assets",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert link",
            OverlayKind::KeepName => "keep version",
            OverlayKind::Context => "context menu",
        }
    }

    pub fn row_path_splits(self) -> bool {
        match self {
            OverlayKind::InsertLink => true,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::KeepName
            | OverlayKind::Context => false,
        }
    }

    pub fn draws_title_prefix(self) -> bool {
        !matches!(
            self,
            OverlayKind::Rename
                | OverlayKind::InsertLink
                | OverlayKind::KeepName
                | OverlayKind::Context
        )
    }

    pub const SETTINGS_MARKER_PREFIX: &'static str = "§ ";

    pub const HEADING_MARKER_PREFIX: &'static str = "❡ ";

    pub fn empty_lens_message(self, lens: &str) -> Option<&'static str> {
        match (self, lens) {
            (OverlayKind::Goto, "recent") => Some("no recent files yet"),
            (OverlayKind::Goto, "headings") => Some("no headings yet"),
            (OverlayKind::Project, "recent") => Some("no recent projects yet"),
            (_, "all") => None,
            _ => Some("nothing here"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintAction {
    pub glyph: &'static str,
    pub label: &'static str,
}

pub const HINT_SEP: &str = "   ";
pub const ARROWS_LR: &str = "\u{2190}/\u{2192}";
/// The vertical arrow pair. Shared hint vocabulary: a workspace's PRIMARY list
/// advertises a vertical step as its headline key, and so does a comparison,
/// whose `\u{2191}/\u{2193}` scrolls the transcript.
pub const ARROWS_UD: &str = "\u{2191}/\u{2193}";
pub const RANGE_LR_LABEL: &str = "adjust";
pub const PIN_TAG: &str = "pinned";

pub fn format_hint(actions: &[HintAction]) -> String {
    actions
        .iter()
        .map(|a| format!("{} {}", a.glyph, a.label))
        .collect::<Vec<_>>()
        .join(HINT_SEP)
}

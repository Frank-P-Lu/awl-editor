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
        Keymap,
        MoveDest,
        Command,
        Spell,
        Keybindings,
        History,
        Conflict,
        Credits,
        Settings,
        Assets,
        Rename,
        InsertLink,
        KeepName,
        Context,
        ExportDest,
        TableDims,
        SearchFolder,
        UserWords,
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
            OverlayKind::Keymap => "keymap",
            OverlayKind::MoveDest => "move",
            OverlayKind::Command => "command",
            OverlayKind::Spell => "spell",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "history",
            OverlayKind::Conflict => "conflict",
            OverlayKind::Credits => "credits",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "assets",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert_link",
            OverlayKind::KeepName => "keep_version",
            OverlayKind::Context => "context",
            OverlayKind::ExportDest => "export_dest",
            OverlayKind::TableDims => "table_dims",
            OverlayKind::SearchFolder => "search_folder",
            OverlayKind::UserWords => "user_words",
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
            | OverlayKind::Context
            | OverlayKind::SearchFolder => Navigate,
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap => ValuePick,
            OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Keybindings
            | OverlayKind::Settings
            | OverlayKind::Conflict
            // Read-only, and `↵` on its content pane does nothing — the same
            // "nothing to commit" shape as Conflict.
            | OverlayKind::Credits => StayOpen,
            OverlayKind::Rename => Navigate,
            OverlayKind::InsertLink => Navigate,
            OverlayKind::KeepName => Navigate,
            OverlayKind::TableDims => Navigate,
        }
    }

    #[allow(dead_code)] // consumed only by overlay::tests's runtime roster sweep today.
    pub fn row_meta_roster(self) -> &'static [super::RowMetaTag] {
        use super::RowMetaTag::*;
        match self {
            OverlayKind::Goto => &[GotoFile, GotoHeading, GotoLine, GotoFolder, FolderChooser],
            OverlayKind::Command => &[Plain, CommandHidden, CommandSetting],
            OverlayKind::Context => &[Plain],
            OverlayKind::Spell => &[Plain, SpellAdd],
            OverlayKind::History => &[History],
            OverlayKind::Conflict => &[Plain],
            OverlayKind::Credits => &[Plain],
            OverlayKind::Project => &[Plain, ProjectDoor],
            OverlayKind::MoveDest => &[Plain, MoveHere, NewFolder],
            OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::ExportDest
            | OverlayKind::Keybindings
            | OverlayKind::Settings
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::UserWords
            | OverlayKind::TableDims => &[Plain],
            OverlayKind::Assets => &[Asset],
            OverlayKind::SearchFolder => &[SearchHit],
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
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest
            | OverlayKind::TableDims
            | OverlayKind::UserWords
            | OverlayKind::SearchFolder => false,
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
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest
            | OverlayKind::TableDims
            | OverlayKind::UserWords
            | OverlayKind::SearchFolder => false,
        }
    }

    /// A DESTINATION NAVIGATOR: a folders-only walk. Every member shares the
    /// `→` descends / `←` ascends grammar, and they differ only in WHAT lands
    /// in the folder you stop on and WHICH tree they walk, which is why every
    /// navigation site asks this instead of naming one kind and growing a
    /// second branch later.
    ///
    /// `↵` no longer reads identically across the family: `ExportDest` and
    /// `ProjectBrowse` still take the highlighted folder directly
    /// (`actions::overlay_nav::dest_value`). `MoveDest` does not — its
    /// contextual `Move here`/`New folder…` rows (`RowMeta::MoveHere`/
    /// `NewFolder`) make `↵` on a FOLDER row descend instead, mirroring `→`,
    /// so the primary "commit here" verb needs its own reachable row rather
    /// than living in the ambiguity of "nothing else is highlighted"
    /// (`actions::overlay_nav::accept_move_dest`).
    ///
    /// The two `Dest` members walk the ACTIVE ROOT and put something in the
    /// folder; [`Self::ProjectBrowse`] walks the WORKSPACE by absolute path and
    /// makes the folder the project. `ExportDest` alone still takes a typed
    /// name that does not exist yet through `dest_value`'s `allow_new` — an
    /// export creates the folder it names; `MoveDest`'s create-a-folder door
    /// is its own `New folder…` row instead, and `ProjectBrowse` has neither,
    /// because there is nothing to switch to in a folder that isn't there.
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
            | OverlayKind::Keymap
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::UserWords
            | OverlayKind::SearchFolder => false,
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
    pub fn empty_corpus_message(self) -> &'static str {
        match self {
            OverlayKind::History => "no history yet",
            // Unreachable in practice: the workspace is only summoned with a
            // conflict open, and it always carries its three fixed views.
            OverlayKind::Conflict => "nothing to review",
            // Unreachable in practice: `new_credits` always carries its one
            // fixed row.
            OverlayKind::Credits => "nothing to review",
            OverlayKind::Spell => "no suggestions",
            OverlayKind::Browse => "this folder is empty",
            OverlayKind::Goto | OverlayKind::Project | OverlayKind::MoveDest => "no files here",
            // A destination list holds FOLDERS only, so "no files here" would name
            // the wrong absence.
            OverlayKind::ExportDest | OverlayKind::ProjectBrowse => "no folders here",
            OverlayKind::Assets => "no unused assets",
            // Names the ONE door that fills this list, because the file it
            // mirrors is not otherwise visible anywhere in the product.
            OverlayKind::UserWords => "no added words — add one from the spell card",
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::Command
            | OverlayKind::Keybindings
            | OverlayKind::Settings => "no matches",
            OverlayKind::Rename => "no matches",
            OverlayKind::InsertLink => "no matches",
            OverlayKind::KeepName => "no matches",
            OverlayKind::Context => "no actions",
            // Unreachable in practice: the picker carries no candidate row
            // list at all -- see `TableDimsEdit`'s own doc.
            OverlayKind::TableDims => "no matches",
            // Reads the same at rest (nothing typed yet) and after a query
            // that finds nothing -- both are "no matches", the same message
            // every other query-driven picker in this list gives.
            OverlayKind::SearchFolder => "no matches",
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
            OverlayKind::Keymap => "keymap",
            OverlayKind::Command => "commands",
            OverlayKind::Spell => "spelling",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "version history",
            // The SAME words the persistent gutter affordance uses, so the thing
            // you noticed and the place you review it read as one thing.
            OverlayKind::Conflict => "changed elsewhere",
            OverlayKind::Credits => "credits",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "unused assets",
            OverlayKind::UserWords => "personal dictionary",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert link",
            OverlayKind::KeepName => "keep version",
            OverlayKind::Context => "context menu",
            OverlayKind::TableDims => "insert table",
            OverlayKind::SearchFolder => "search in folder",
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
            | OverlayKind::Keymap
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::UserWords
            // The match highlight rides its own dedicated per-row byte-range
            // field (`RowMeta::SearchHit`'s `hl_start`/`hl_end`, threaded
            // through `ViewState::overlay_match_highlights`), never this
            // path-split figure/ground mechanism -- a snippet's split point
            // is a match position, not a directory boundary, and free-form
            // prose may itself contain a `/` that this mechanism's generic
            // fallback would misread as one.
            | OverlayKind::SearchFolder => false,
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

    /// **DOES TYPING ON THIS CARD FILTER ANYTHING?**
    ///
    /// Every picker's head line is a search field: a typed character narrows the
    /// rows and the caret parked at its end says so. One kind has no rows to
    /// narrow. Credits' "list" is a single fixed row that NAMES the document
    /// beside it (`OverlayState::new_credits`) — there is nothing to choose, so a
    /// query can only ever hide the one row and leave the reader on `no matches`
    /// while the prose it named is still on screen.
    ///
    /// The answer is read by BOTH ends of the field, so what is advertised and
    /// what acts are one fact rather than two that agree today: the query's own
    /// growth door refuses to accept characters ([`OverlayState::push`]) and the
    /// renderer draws no caret on the head line
    /// (`ViewState::overlay_query_field`). A card that answers `false` still
    /// draws its head line — the title is what that line is FOR here — it simply
    /// stops pretending to be a field.
    ///
    /// Wildcard-free: a new kind must say whether it can be searched.
    pub fn offers_query(self) -> bool {
        match self {
            OverlayKind::Credits => false,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::SearchFolder
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings => true,
        }
    }

    pub const SETTINGS_MARKER_PREFIX: &'static str = "§ ";

    pub const HEADING_MARKER_PREFIX: &'static str = "❡ ";

    pub fn empty_lens_message(self, lens: &str) -> Option<&'static str> {
        match (self, lens) {
            (OverlayKind::Goto, "files") => Some("no files here"),
            (OverlayKind::Goto, "headings") => Some("no headings yet"),
            (OverlayKind::Goto, "folders") => Some("no folders here"),
            (OverlayKind::Goto, "recent") => Some("no recent destinations"),
            (OverlayKind::Project, "recent") => Some("no recent projects yet"),
            (_, "all") => None,
            _ => Some("nothing here"),
        }
    }
}

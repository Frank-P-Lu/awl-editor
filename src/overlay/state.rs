use super::{
    Capture, KeepEdit, LinkEdit, OverlayKind, OverlayRow, PIN_TAG, RangeCell, RenameEdit, RowMeta,
    TableDimsEdit, ValueEdit,
};
use crate::textbox::TextBox;
use std::sync::Arc;

/// Immutable summon-time display material for a faceted picker's stable hug
/// measurement. An `Arc` is the corpus identity, so ordinary view syncs do not
/// rebuild or clone a complete corpus.
#[derive(Debug)]
pub struct HugRoster {
    pub primary: Vec<String>,
    pub secondary: Vec<String>,
}

pub use super::add_to_dictionary_label;

#[derive(Debug, Clone)]
pub struct OverlayState {
    pub kind: OverlayKind,
    pub align: crate::theme::CardAnchor,
    pub query: TextBox,
    pub rows: Vec<OverlayRow>,
    pub open: Vec<usize>,
    pub recent: Vec<usize>,
    pub items: Vec<usize>,
    pub selected: usize,
    pub scroll: usize,
    pub browse_dir: Option<String>,
    /// WHAT A CANCEL MUST UNDO while this card auditions something live; the
    /// lifecycle owns the payload and the revert (`journey::Audition`).
    pub audition: super::Audition,
    pub spell_target: Option<(usize, usize, usize)>,
    pub add_word: Option<String>,
    pub capture: Option<Capture>,
    pub notice: String,
    pub facet_lens: usize,
    pub facet_now: Option<u64>,
    pub facet_session_start: Option<u64>,
    pub item_sections: Vec<String>,
    pub value_edit: Option<ValueEdit>,
    pub rename_edit: Option<RenameEdit>,
    pub link_edit: Option<LinkEdit>,
    pub keep_edit: Option<KeepEdit>,
    /// The INSERT-TABLE dimension picker's live sculpted state, `Some` only
    /// on an [`OverlayKind::TableDims`] card. Unlike its `*_edit` siblings
    /// this card carries no candidate row list at all -- see
    /// [`TableDimsEdit`]'s own doc for why.
    pub table_dims: Option<TableDimsEdit>,
    pub detail_focus: bool,
    pub diff_scroll: usize,
    pub last_hover_px: Option<(f32, f32)>,
    pub context_actions: Vec<crate::keymap::Action>,
    /// The named working-set buffer a contextual filename card was summoned
    /// over. Ordinary context cards leave this absent.
    pub context_buffer: Option<crate::buffers::BufferKey>,
    pub context_anchor: Option<(f32, f32)>,
    /// THE CONFLICT WORKSPACE'S OWN SUBJECT ([`super::ConflictSubject`]), or
    /// `None` for every other kind.
    pub conflict: Option<super::ConflictSubject>,
    /// WHICH FORMAT the destination navigator is finding a folder for — set on
    /// the [`OverlayKind::ExportDest`] card the export action opens, `None` on
    /// every other kind.
    ///
    /// It lives on the card because the format is chosen by the ACTION and read
    /// by the ACCEPT, with any number of descends and ascends in between; and it
    /// survives those because [`super::Journey::relevel`] carries it onto the
    /// next level, the one seam a navigator is rebuilt at. That is the same
    /// mechanism [`super::Bind::Path`] uses for the config key a Settings folder
    /// picker is filling in — and the reason both are payloads rather than a
    /// field each rebuild site has to remember.
    pub export_format: Option<crate::export::Format>,
    /// A Save a Copy request reuses the folders-only export destination card;
    /// this payload selects its distinct write on accept.
    pub save_copy: bool,
    pub save_copy_dest: Option<String>,
    /// Go to Line's one fact: the destination buffer's total line count,
    /// gathered once at build time (`attach_line_jump`). `0` is the inert
    /// default every non-`Goto`/bare-constructed picker carries -- "no
    /// buffer known", so the line-jump row never offers a target
    /// (`OverlayState::goto_line_target`).
    pub goto_line_count: usize,
    /// The file Move is finding a destination for. The DIRECTORY LEVEL can't
    /// know this -- only the summon did -- so `title()` reads it to name the
    /// errand ("move welcome.md") instead of the generic kind title, and it
    /// survives every descend/ascend via `carry_level_payload_from`. `None`
    /// on every non-`MoveDest` card.
    pub move_filename: Option<String>,
    pub(super) hug_roster: Option<Arc<HugRoster>>,
}

impl OverlayState {
    /// The visible errand for this card. Save a Copy reuses the folder and
    /// filename mechanisms, but its payload changes what the user is doing;
    /// Move names the file it is finding a destination for the same way.
    ///
    /// Once standing anywhere but the level it opened at, the current
    /// ROOT-RELATIVE destination folds into the title too (`"move welcome.md
    /// to notes/drafts/"`) — the one place a descended destination navigator
    /// says where it is standing, since [`Self::browse_dir`] is otherwise a
    /// fact only the sidecar could see. Unchanged at the level it opened at:
    /// `browse_dir` is `None` there for [`OverlayKind::MoveDest`] and
    /// [`OverlayKind::ExportDest`] (both walk the active root by a
    /// root-relative `rel`, [`crate::overlay::browse_level`]'s doc), so the
    /// suffix has nothing to append until a real descend happens.
    pub fn title(&self) -> String {
        if self.save_copy && self.kind == OverlayKind::ExportDest {
            self.with_browse_dir_suffix("save a copy to".to_string())
        } else if self.save_copy_dest.is_some() && self.rename_edit.is_some() {
            "save a copy as".to_string()
        } else if let Some(name) = self
            .move_filename
            .as_deref()
            .filter(|_| self.kind == OverlayKind::MoveDest)
        {
            self.move_dest_title(name)
        } else if self.kind == OverlayKind::ExportDest {
            self.with_browse_dir_suffix(self.kind.title().to_string())
        } else {
            self.kind.title().to_string()
        }
    }

    /// `title`'s composition for [`OverlayKind::MoveDest`]: `"move {name}"`
    /// at the level it opened at, `"move {name} to {dir}/"` once descended —
    /// `to` belongs to Move's own phrasing (it names an action, not a
    /// question), unlike [`OverlayKind::ExportDest`]'s `"export to"`, whose
    /// title already ends in the word a plain append would double.
    fn move_dest_title(&self, name: &str) -> String {
        match self.browse_dir_display() {
            Some(dir) => format!("move {name} to {dir}"),
            None => format!("move {name}"),
        }
    }

    /// Append the current destination folder to `base`, unchanged when
    /// [`Self::browse_dir_display`] has nothing to show. `base` is assumed to
    /// already read naturally with a folder after it (`"export to"`, `"save a
    /// copy to"`) — every caller's own title already ends in a preposition,
    /// so this never doubles one.
    fn with_browse_dir_suffix(&self, base: String) -> String {
        match self.browse_dir_display() {
            Some(dir) => format!("{base} {dir}"),
            None => base,
        }
    }

    /// The destination folder `title` shows, root-relative with a trailing
    /// `/` (`"notes/drafts/"`), or `None` at the level the card opened at.
    ///
    /// [`OverlayKind::MoveDest`]/[`OverlayKind::ExportDest`] walk the active
    /// root by a root-relative `rel`, so [`Self::browse_dir`] IS that string
    /// already (`crate::actions::overlay_nav::join_browse`'s callers) —
    /// `None` at the root, never `Some("")`. Every other kind (including the
    /// two ABSOLUTE-path walkers, `Project`/`ProjectBrowse`) shows nothing:
    /// their `browse_dir` is a whole directory rather than a root-relative
    /// fragment, and folding that into this title would need the workspace
    /// baseline to relativize against, which this card does not carry.
    fn browse_dir_display(&self) -> Option<String> {
        if !matches!(self.kind, OverlayKind::MoveDest | OverlayKind::ExportDest) {
            return None;
        }
        let dir = self.browse_dir.as_deref()?;
        if dir.is_empty() {
            return None;
        }
        Some(format!("{dir}/"))
    }

    pub fn new(
        kind: OverlayKind,
        corpus: Vec<String>,
        open: Vec<usize>,
        recent: Vec<usize>,
    ) -> Self {
        let n = corpus.len();
        Self::new_marked(
            kind,
            corpus,
            vec![false; n],
            vec![false; n],
            open,
            recent,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_marked(
        kind: OverlayKind,
        corpus: Vec<String>,
        git: Vec<bool>,
        is_dir: Vec<bool>,
        open: Vec<usize>,
        recent: Vec<usize>,
        browse_dir: Option<String>,
    ) -> Self {
        let rows: Vec<OverlayRow> = corpus
            .into_iter()
            .zip(git)
            .zip(is_dir)
            .map(|((accept, git), is_dir)| {
                let mut row = OverlayRow::plain(accept);
                row.git = git;
                row.is_dir = is_dir;
                if kind == OverlayKind::Goto {
                    row.meta = RowMeta::GotoFile {
                        time: String::new(),
                    };
                }
                row
            })
            .collect();
        let mut s = Self {
            kind,
            align: crate::render::effective_card_anchor(),
            query: TextBox::new(),
            rows,
            open,
            recent,
            items: Vec::new(),
            selected: 0,
            scroll: 0,
            browse_dir,
            audition: super::Audition::None,
            spell_target: None,
            add_word: None,
            capture: None,
            notice: String::new(),
            facet_lens: 0,
            facet_now: None,
            facet_session_start: None,
            item_sections: Vec::new(),
            value_edit: None,
            rename_edit: None,
            link_edit: None,
            keep_edit: None,
            table_dims: None,
            detail_focus: false,
            diff_scroll: 0,
            last_hover_px: None,
            context_actions: Vec::new(),
            context_buffer: None,
            context_anchor: None,
            conflict: None,
            export_format: None,
            save_copy: false,
            save_copy_dest: None,
            goto_line_count: 0,
            move_filename: None,
            hug_roster: None,
        };
        s.refilter();
        s.refresh_hug_roster();
        s
    }
    /// CARRY the facts a directory LEVEL cannot know onto the next level — the
    /// one owner of the [`super::Journey::relevel`] hand-off (see its doc).
    /// Today that is exactly [`Self::export_format`]: the level supplier reads a
    /// folder, and only the summon knew which format was being exported.
    ///
    /// Deliberately NOT a blanket struct copy: a level rebuild must replace the
    /// rows, the query and the browse dir, which is the whole point of
    /// rebuilding. Anything added here is a conscious "the summon decided this,
    /// not the disk".
    pub fn carry_level_payload_from(&mut self, prev: &Self) {
        self.export_format = prev.export_format;
        self.save_copy = prev.save_copy;
        self.save_copy_dest = prev.save_copy_dest.clone();
        self.move_filename = prev.move_filename.clone();
    }

    pub fn accepts(&self) -> Vec<&str> {
        self.rows.iter().map(|r| r.accept.as_str()).collect()
    }

    pub fn set_secondaries(&mut self, secondaries: Vec<String>) {
        for (row, s) in self.rows.iter_mut().zip(secondaries) {
            row.secondary = s;
        }
        self.refresh_hug_roster();
    }

    pub fn set_range_cells(&mut self, cells: Vec<Option<RangeCell>>) {
        if cells.is_empty() {
            for row in self.rows.iter_mut() {
                row.range = None;
            }
            return;
        }
        for (row, c) in self.rows.iter_mut().zip(cells) {
            row.range = c;
        }
    }

    pub fn set_times(&mut self, times: Vec<String>) {
        for (i, row) in self.rows.iter_mut().enumerate() {
            row.meta = RowMeta::GotoFile {
                time: times.get(i).cloned().unwrap_or_default(),
            };
        }
        self.refresh_hug_roster();
    }

    pub fn new_theme(names: Vec<String>, active_index: usize) -> Self {
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Theme,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.audition = super::Audition::Theme {
            original: active_index,
        };
        s.facet_lens = 0;
        s.refilter();
        if let Some(pos) = s.items.iter().position(|&i| i == active_index) {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    pub fn new_caret(active: crate::caret::CaretMode) -> Self {
        let names: Vec<String> = crate::caret::CaretMode::ALL
            .iter()
            .map(|m| m.label().to_string())
            .collect();
        let descriptions: Vec<String> = crate::caret::CaretMode::ALL
            .iter()
            .map(|m| m.description().to_string())
            .collect();
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Caret,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(descriptions);
        s.audition = super::Audition::Caret {
            original: active,
            was_auto: crate::caret::is_auto(),
        };
        if let Some(active_index) = crate::caret::CaretMode::ALL
            .iter()
            .position(|&m| m == active)
            && let Some(pos) = s.items.iter().position(|&i| i == active_index)
        {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    pub fn new_dictionary(active: crate::spell::DictVariant) -> Self {
        let names: Vec<String> = crate::spell::DictVariant::ALL
            .iter()
            .map(|v| v.label().to_string())
            .collect();
        let descriptions: Vec<String> = crate::spell::DictVariant::ALL
            .iter()
            .map(|v| v.description().to_string())
            .collect();
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Dictionary,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(descriptions);
        if let Some(active_index) = crate::spell::DictVariant::ALL
            .iter()
            .position(|&v| v == active)
            && let Some(pos) = s.items.iter().position(|&i| i == active_index)
        {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    pub fn new_cjk_lang(active: crate::frontmatter::Lang) -> Self {
        let names: Vec<String> = crate::frontmatter::DEFAULT_CJK_PRIORITY
            .iter()
            .map(|l| l.label().to_string())
            .collect();
        let descriptions: Vec<String> = crate::frontmatter::DEFAULT_CJK_PRIORITY
            .iter()
            .map(|l| l.description().to_string())
            .collect();
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::CjkLang,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(descriptions);
        if let Some(active_index) = crate::frontmatter::DEFAULT_CJK_PRIORITY
            .iter()
            .position(|&l| l == active)
            && let Some(pos) = s.items.iter().position(|&i| i == active_index)
        {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    pub fn new_date(active: crate::dateformat::DateFormat, today: (i32, u32, u32)) -> Self {
        let (y, m, d) = today;
        let names: Vec<String> = crate::dateformat::DateFormat::ALL
            .iter()
            .map(|f| f.format(y, m, d))
            .collect();
        let descriptions: Vec<String> = crate::dateformat::DateFormat::ALL
            .iter()
            .map(|f| f.label().to_string())
            .collect();
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Date,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(descriptions);
        if let Some(active_index) = crate::dateformat::DateFormat::ALL
            .iter()
            .position(|&f| f == active)
            && let Some(pos) = s.items.iter().position(|&i| i == active_index)
        {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    /// The keymap-flavor picker: native/emacs, plain-language labels
    /// ([`crate::keymap::KeymapFlavor::label`]) with the concrete chord
    /// difference in the secondary column ([`crate::keymap::KeymapFlavor::
    /// description`]), the active flavor pre-selected. Nothing previews on
    /// move (`OverlayKind::previews_live_document` is `false` for `Keymap` —
    /// the Dictionary/CjkLang/Date shape, not Caret/Theme's), so — unlike
    /// those two — there is no [`super::Audition`] variant to revert on
    /// cancel: nothing changed live until Enter.
    pub fn new_keymap(active: crate::keymap::KeymapFlavor) -> Self {
        let names: Vec<String> = crate::keymap::KeymapFlavor::ALL
            .iter()
            .map(|f| f.label().to_string())
            .collect();
        let descriptions: Vec<String> = crate::keymap::KeymapFlavor::ALL
            .iter()
            .map(|f| f.description().to_string())
            .collect();
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Keymap,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(descriptions);
        if let Some(active_index) = crate::keymap::KeymapFlavor::ALL
            .iter()
            .position(|&f| f == active)
            && let Some(pos) = s.items.iter().position(|&i| i == active_index)
        {
            s.selected = pos;
            s.scroll_to_selected();
        }
        s
    }

    /// The switch-project card and the TWO routes its roster has: `folders`, one
    /// directory level read as leaf names, and `recent_roots`, what the MRU
    /// remembers as whole absolute paths. `super::build::recent` owns their
    /// difference, their resolution against the disk, and their enrolment here.
    pub fn new_project(
        dir_abs: String,
        folders: Vec<(String, bool)>,
        recent_roots: &[(String, bool)],
    ) -> Self {
        let mut corpus = vec![super::HERE_ACCEPT.to_string()];
        let mut git = vec![false];
        let mut is_dir = vec![false];
        for (name, is_git) in folders {
            corpus.push(name);
            git.push(is_git);
            is_dir.push(true);
        }
        let recent =
            super::build::recent::enrol(&dir_abs, &mut corpus, &mut git, &mut is_dir, recent_roots);
        let mut s = Self::new_marked(
            OverlayKind::Project,
            corpus,
            git,
            is_dir,
            Vec::new(),
            recent,
            Some(dir_abs),
        );
        s.selected = s
            .items
            .iter()
            .position(|&i| s.rows[i].accept != super::HERE_ACCEPT)
            .unwrap_or(0);
        s.scroll_to_selected();
        s
    }

    /// THE MOVE NAVIGATOR'S OWN LEVEL: a folders-only listing plus its two
    /// CONTEXTUAL action rows, `Move here` (always reachable — the picker's
    /// primary verb, see [`RowMeta::MoveHere`]) and `New folder…` (visible
    /// only while the typed query names no listed folder, see
    /// [`RowMeta::NewFolder`] / [`Self::move_dest_new_folder_target`]).
    /// `folders` is the caller-read directory listing (name, is-git), exactly
    /// like [`Self::new_project`]'s own `folders` argument — this stays a pure
    /// constructor over already-gathered data so a test can build a level
    /// without touching disk.
    pub fn new_move_dest(dir_rel: Option<String>, folders: Vec<(String, bool)>) -> Self {
        let mut corpus = Vec::with_capacity(folders.len() + 2);
        let mut git = Vec::with_capacity(folders.len() + 2);
        let mut is_dir = Vec::with_capacity(folders.len() + 2);
        corpus.push("Move here".to_string());
        git.push(false);
        is_dir.push(false);
        for (name, is_git) in folders {
            corpus.push(name);
            git.push(is_git);
            is_dir.push(true);
        }
        // `New folder…`'s real label is written by `sync_move_new_folder_row`
        // on the `refilter` below; the placeholder here is never drawn.
        corpus.push(String::new());
        git.push(false);
        is_dir.push(false);
        let mut s = Self::new_marked(
            OverlayKind::MoveDest,
            corpus,
            git,
            is_dir,
            Vec::new(),
            Vec::new(),
            dir_rel,
        );
        s.rows[0].meta = RowMeta::MoveHere;
        let last = s.rows.len() - 1;
        s.rows[last].meta = RowMeta::NewFolder;
        // `new_marked`'s own construction already ran one `refilter()` before
        // these two rows carried their real metadata (`RowMeta::Plain` sorts
        // and filters like any ordinary row), so `pin_move_here` and the
        // `New folder…` visibility gate never saw them — rerun it now that
        // the metadata is in place.
        s.refilter();
        s
    }

    /// GIVE THE FLAT SWITCH-PROJECT PICKER ITS ONE DOOR — a terminal
    /// [`RowMeta::ProjectDoor`] row that opens the folder navigator
    /// ([`OverlayKind::ProjectBrowse`]). A NO-OP for every other kind, and
    /// idempotent.
    ///
    /// It is attached at the seams that summon the FLAT picker rather than in
    /// [`Self::new_project`], because `new_project` builds both of the features
    /// that share [`OverlayKind::Project`]: the flat picker, whose reach beyond
    /// the workspace's direct children is exactly this door, and the Settings
    /// folder-VALUE picker, which already walks the whole tree with `→`/`⌫` and
    /// whose descend would silently un-park the Settings surface it is filling a
    /// key for. `actions::apply_overlay_open_action` (both switch-project doors)
    /// and `actions::overlay_nav::resume_rebuild` (coming BACK from the
    /// navigator) are the callers; the Settings descend deliberately is not.
    pub fn attach_browse_door(&mut self) {
        if self.kind != OverlayKind::Project
            || self.rows.iter().any(|r| r.meta == RowMeta::ProjectDoor)
        {
            return;
        }
        let mut row = OverlayRow::plain(OverlayKind::BROWSE_DOOR_LABEL.to_string());
        row.meta = RowMeta::ProjectDoor;
        self.rows.push(row);
        self.refilter();
        self.refresh_hug_roster();
    }

    /// Is the highlighted row the door ([`Self::attach_browse_door`])? The one
    /// question the accept path asks before reading the row as a project.
    pub fn selected_is_browse_door(&self) -> bool {
        self.selected_corpus_index()
            .and_then(|ci| self.rows.get(ci))
            .is_some_and(|r| r.meta == RowMeta::ProjectDoor)
    }

    pub fn new_command(names: Vec<String>, bindings: Vec<String>, hidden: Vec<bool>) -> Self {
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Command,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(bindings);
        for (row, h) in s.rows.iter_mut().zip(hidden) {
            if h {
                row.meta = RowMeta::CommandHidden;
            }
        }
        s.refilter();
        s
    }

    pub fn new_keybindings(names: Vec<String>, bindings: Vec<String>) -> Self {
        let n = names.len();
        let mut s = Self::new_marked(
            OverlayKind::Keybindings,
            names,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(bindings);
        s
    }

    pub fn selected_command_slug(&self) -> Option<String> {
        self.selected_corpus_index()
            .map(crate::commands::visible_slug_of)
    }

    /// The footer sentence with no [`super::Bind`] context — correct for
    /// every kind except [`OverlayKind::Project`], and correct for THAT kind
    /// too whenever the card was reached directly (not mid-descend from
    /// Settings, the only place a `Bind::Path` exists). [`super::Journey::
    /// foot_hint`] is the entry point that has the real answer; this is the
    /// convenience the rest of the suite — which builds bare cards — keeps
    /// calling without a Journey to hand.
    pub fn foot_hint(&self) -> String {
        self.foot_hint_scoped(None)
    }

    /// **THE ONE OWNER of the footer sentence.** `bind` disambiguates the two
    /// features [`OverlayKind::Project`] draws as: the flat switch-project
    /// picker (no bind, or `Bind::Value`) is one level over the workspace's
    /// direct children with no ascend affordance, so its line drops the `⌫ up`
    /// cell the Settings folder-VALUE picker (`Bind::Path`) still earns —
    /// `crate::actions::overlay_nav::overlay_intercept`'s `DeleteBackward` arm
    /// is the intercept this line has to keep telling the truth about. Every
    /// other kind's hint is bind-independent, so `bind` only ever changes
    /// this one kind's line.
    pub fn foot_hint_scoped(&self, bind: Option<&super::Bind>) -> String {
        if let Some(re) = &self.rename_edit {
            if self.save_copy_dest.is_some() {
                return format!(
                    "save a copy as: {}   Enter commit   Esc cancel",
                    re.input.text()
                );
            }
            return re.prompt();
        }
        if let Some(le) = &self.link_edit {
            return le.prompt();
        }
        if let Some(ke) = &self.keep_edit {
            return ke.prompt();
        }
        if let Some(td) = &self.table_dims {
            return td.prompt();
        }
        if let Some(cap) = &self.capture {
            return cap.prompt();
        }
        if !self.notice.is_empty() {
            return self.notice.clone();
        }
        // A summoned WORKSPACE advertises the stage that holds focus.
        // Its PRIMARY list is the navigation rail or the timeline; its DETAIL
        // stage is the rows pane or the comparison, whose keys are that stage's
        // own ([`OverlayKind::hint_actions`], including the per-row range variant
        // below).
        //
        // Both stages' hints are per-kind statements, and both live in `kind.rs`.
        // Spelling one of them inline here — under a bare `detail_focus` test —
        // only reads as correct while exactly one kind has a detail stage: a
        // second kind reaching it would silently take the first one's line.
        //
        // THE BACK CELL IS APPENDED, NEVER AUTHORED. `Esc` leaves a workspace
        // from either region, so the detail stage owes an explicit Back — but
        // WHICH key performs it depends on whether this stage's own query has
        // the erase key busy, which is a fact about the state and not about the
        // kind. So no per-kind arm spells it: `detail_back` decides, the action
        // seam reads that same answer, and the sentence cannot come to disagree
        // with the keyboard.
        if self.workspace_shape().is_some() {
            if !self.detail_focus {
                return super::format_hint(&self.kind.rail_hint_actions());
            }
            let mut actions = match self.selected_range().is_some() {
                true => self.kind.range_row_actions(),
                false => self.kind.detail_hint_actions(),
            };
            actions.extend(self.detail_back().map(super::workspace::BackKey::hint));
            return super::format_hint(&actions);
        }
        if self.selected_range().is_some() {
            return self.kind.range_row_hint();
        }
        if self.kind == OverlayKind::Project && !matches!(bind, Some(super::Bind::Path { .. })) {
            return self.kind.project_flat_hint();
        }
        if self.save_copy && self.kind == OverlayKind::ExportDest {
            return "type to filter   ↵ save a copy here   → open   ← up".to_string();
        }
        self.kind.hint()
    }

    pub fn attach_headings(&mut self, headings: Vec<(String, usize)>) {
        if headings.is_empty() {
            return;
        }
        for (display, line) in headings {
            self.rows.push(OverlayRow {
                accept: display,
                secondary: String::new(),
                is_dir: false,
                git: false,
                meta: RowMeta::GotoHeading { line },
                range: None,
            });
        }
        self.refilter();
        self.refresh_hug_roster();
    }

    /// Fold authored folder destinations into Go-to. `recent_paths` is ordered
    /// newest-first and is translated into corpus indices here, beside the rows
    /// it ranks, so Files and Folders share one Recent lens without parallel
    /// index arithmetic at callers.
    pub fn attach_folders(&mut self, folders: Vec<(String, bool)>, recent_paths: &[String]) {
        if self.kind != OverlayKind::Goto {
            return;
        }
        let start = self.rows.len();
        for (path, is_git) in folders {
            self.rows.push(OverlayRow {
                accept: path,
                secondary: String::new(),
                is_dir: true,
                git: is_git,
                meta: RowMeta::GotoFolder,
                range: None,
            });
        }
        for path in recent_paths {
            if let Some(ci) = self.rows[start..]
                .iter()
                .position(|row| &row.accept == path)
                .map(|i| start + i)
                && !self.recent.contains(&ci)
            {
                self.recent.push(ci);
            }
        }
        let mut chooser = OverlayRow::plain("Choose another folder…".to_string());
        chooser.meta = RowMeta::FolderChooser;
        self.rows.push(chooser);
        self.refilter();
        self.refresh_hug_roster();
    }

    /// Fold the destination buffer's LINE COUNT into Go-to and append Go to
    /// Line's own terminal row -- the Headings lens's numeric companion
    /// (queue: "long prose and light code", where a title-typed heading jump
    /// has nothing to search). The row is a single fixed slot: its label and
    /// `RowMeta::GotoLine { line }` are refreshed live from the typed query by
    /// `refilter`'s `sync_goto_line_row` step every keystroke, never rebuilt
    /// here. A no-op past setting `goto_line_count` when `line_count` is `0`
    /// (no buffer known -- most bare test-constructed pickers), mirroring
    /// `attach_headings`/`attach_folders`'s own opt-in shape: nothing to
    /// attach, so nothing is.
    pub fn attach_line_jump(&mut self, line_count: usize) {
        self.goto_line_count = line_count;
        if line_count == 0 {
            return;
        }
        self.rows.push(OverlayRow {
            accept: String::new(),
            secondary: String::new(),
            is_dir: false,
            git: false,
            meta: RowMeta::GotoLine { line: 0 },
            range: None,
        });
        self.refilter();
        self.refresh_hug_roster();
    }

    pub fn attach_settings_rows(
        &mut self,
        rows: Vec<&'static crate::settings::SettingRow>,
        values: Vec<String>,
    ) {
        if rows.is_empty() {
            return;
        }
        for (row, value) in rows.into_iter().zip(values) {
            self.rows.push(OverlayRow {
                accept: row.name.to_string(),
                secondary: value,
                is_dir: false,
                git: false,
                meta: RowMeta::CommandSetting { id: row.id },
                range: None,
            });
        }
        self.refilter();
        self.refresh_hug_roster();
    }

    pub fn selected_setting_row(&self) -> Option<crate::settings::SettingRow> {
        let ci = self.selected_corpus_index()?;
        let row = self.rows.get(ci)?;
        let RowMeta::CommandSetting { id } = row.meta else {
            return None;
        };
        Some(crate::settings::row_of(id))
    }

    pub fn new_spell(
        mut suggestions: Vec<String>,
        target: (usize, usize, usize),
        word: String,
    ) -> Self {
        suggestions.truncate(OverlayKind::MAX_SUGGESTIONS);
        suggestions.push(add_to_dictionary_label(&word));
        let len = suggestions.len();
        let mut s = Self::new_marked(
            OverlayKind::Spell,
            suggestions,
            vec![false; len],
            vec![false; len],
            Vec::new(),
            Vec::new(),
            None,
        );
        if let Some(last) = s.rows.last_mut() {
            last.meta = RowMeta::SpellAdd;
        }
        s.spell_target = Some(target);
        s.add_word = Some(word);
        s
    }

    pub fn new_history(
        rows: Vec<crate::history::TimelineRow>,
        now: Option<u64>,
        session_start: Option<u64>,
    ) -> Self {
        let n = rows.len();
        let mut corpus = Vec::with_capacity(n);
        let mut secondaries = Vec::with_capacity(n);
        let mut ids = Vec::with_capacity(n);
        let mut ts = Vec::with_capacity(n);
        for row in rows {
            if let Some(name) = row.name {
                corpus.push(name);
                secondaries.push(format!("{} · {}", row.when, row.counts));
            } else {
                corpus.push(if row.which.is_empty() {
                    row.when
                } else {
                    format!("{} · {}", row.when, row.which)
                });
                secondaries.push(if row.pinned {
                    format!("{PIN_TAG} · {}", row.counts)
                } else {
                    row.counts
                });
            }
            ids.push(row.id);
            ts.push(row.timestamp);
        }
        let mut s = Self::new_marked(
            OverlayKind::History,
            corpus,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(secondaries); // the faint right column shows each version's changed-count
        for (row, (id, ts)) in s.rows.iter_mut().zip(ids.into_iter().zip(ts)) {
            row.meta = RowMeta::History { id, ts };
        }
        s.facet_now = now;
        s.facet_session_start = session_start;
        s
    }

    pub fn new_assets(orphans: Vec<crate::assets::Orphan>) -> Self {
        let n = orphans.len();
        let mut corpus = Vec::with_capacity(n);
        let mut secondary = Vec::with_capacity(n);
        for o in &orphans {
            secondary.push(crate::assets::secondary_label(o));
            corpus.push(o.rel.clone());
        }
        let mut s = Self::new_marked(
            OverlayKind::Assets,
            corpus,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        s.set_secondaries(secondary);
        s
    }

    pub fn remove_asset_row(&mut self, rel: &str) -> bool {
        let Some(ci) = self.rows.iter().position(|r| r.accept == rel) else {
            return false;
        };
        self.rows.remove(ci);
        self.refilter();
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        true
    }
}

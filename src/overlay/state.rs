use super::{Capture, KeepEdit, LinkEdit, OverlayKind, PIN_TAG, RenameEdit, ValueEdit};
use crate::textbox::TextBox;

pub fn add_to_dictionary_label(word: &str) -> String {
    format!("Add '{word}' to dictionary")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayRow {
    pub accept: String,
    pub secondary: String,
    pub is_dir: bool,
    pub git: bool,
    pub meta: RowMeta,
    pub range: Option<RangeCell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeCell {
    pub id: crate::settings::SettingId,
    pub step: u16,
}

impl OverlayRow {
    fn plain(accept: String) -> Self {
        Self {
            accept,
            secondary: String::new(),
            is_dir: false,
            git: false,
            meta: RowMeta::Plain,
            range: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowMeta {
    Plain,
    GotoFile {
        time: String,
    },
    GotoHeading {
        line: usize,
    },
    CommandSetting {
        id: crate::settings::SettingId,
    },
    CommandHidden,
    SpellAdd,
    History {
        id: String,
        ts: u64,
    },
    /// THE FLAT SWITCH-PROJECT PICKER'S DOOR ROW — the one row that opens a
    /// further surface ([`OverlayKind::ProjectBrowse`]) instead of naming a
    /// project. Carried as METADATA rather than tested as a label, so the
    /// wording is free to change without the accept path following it.
    ProjectDoor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // consumed only by OverlayKind::row_meta_roster + overlay::tests today.
pub enum RowMetaTag {
    Plain,
    GotoFile,
    GotoHeading,
    CommandSetting,
    CommandHidden,
    SpellAdd,
    History,
    ProjectDoor,
}

impl RowMeta {
    #[allow(dead_code)] // consumed only by overlay::tests's exhaustiveness witness + roster sweep today.
    pub fn tag(&self) -> RowMetaTag {
        match self {
            RowMeta::Plain => RowMetaTag::Plain,
            RowMeta::GotoFile { .. } => RowMetaTag::GotoFile,
            RowMeta::GotoHeading { .. } => RowMetaTag::GotoHeading,
            RowMeta::CommandSetting { .. } => RowMetaTag::CommandSetting,
            RowMeta::CommandHidden => RowMetaTag::CommandHidden,
            RowMeta::SpellAdd => RowMetaTag::SpellAdd,
            RowMeta::History { .. } => RowMetaTag::History,
            RowMeta::ProjectDoor => RowMetaTag::ProjectDoor,
        }
    }

    /// A ROW THAT MUST STAY LAST, whatever the ranker does with it. Two rows
    /// earn it and they earn it for the same reason: they act on something other
    /// than the query, so a fuzzy match on their own label must not float them
    /// above the answers they trail. The spell picker's "Add '<word>' to
    /// dictionary" acts on the TARGETED word; the switch-project door opens a
    /// surface. [`OverlayState::refilter`] is the one consumer.
    pub fn terminal(&self) -> bool {
        match self {
            RowMeta::SpellAdd | RowMeta::ProjectDoor => true,
            RowMeta::Plain
            | RowMeta::GotoFile { .. }
            | RowMeta::GotoHeading { .. }
            | RowMeta::CommandSetting { .. }
            | RowMeta::CommandHidden
            | RowMeta::History { .. } => false,
        }
    }
}

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
    pub detail_focus: bool,
    pub diff_scroll: usize,
    pub last_hover_px: Option<(f32, f32)>,
    pub context_actions: Vec<Option<crate::keymap::Action>>,
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
}

impl OverlayState {
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
            detail_focus: false,
            diff_scroll: 0,
            last_hover_px: None,
            context_actions: Vec::new(),
            context_anchor: None,
            conflict: None,
            export_format: None,
        };
        s.refilter();
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
    }

    pub fn accepts(&self) -> Vec<&str> {
        self.rows.iter().map(|r| r.accept.as_str()).collect()
    }

    pub fn set_secondaries(&mut self, secondaries: Vec<String>) {
        for (row, s) in self.rows.iter_mut().zip(secondaries) {
            row.secondary = s;
        }
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

    pub fn new_project(
        dir_abs: String,
        folders: Vec<(String, bool)>,
        recent_roots: &[String],
    ) -> Self {
        let mut corpus = vec![super::HERE_ACCEPT.to_string()];
        let mut git = vec![false];
        let mut is_dir = vec![false];
        for (name, is_git) in folders {
            corpus.push(name);
            git.push(is_git);
            is_dir.push(true);
        }
        let base = std::path::Path::new(&dir_abs);
        let mut recent = Vec::new();
        for root in recent_roots {
            let rp = std::path::Path::new(root);
            if let Some(ci) = (1..corpus.len()).find(|&i| base.join(&corpus[i]) == rp)
                && !recent.contains(&ci)
            {
                recent.push(ci);
            }
        }
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
            return re.prompt();
        }
        if let Some(le) = &self.link_edit {
            return le.prompt();
        }
        if let Some(ke) = &self.keep_edit {
            return ke.prompt();
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

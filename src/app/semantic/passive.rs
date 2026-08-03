//! The PASSIVE / GLOBAL surfaces: the summoned cards (About, Lifetime,
//! Streaks, the stats HUD, the shortcut peek), the which-key continuation
//! panel, and the awl-rendered menu bar.
//!
//! Passive means announced without stealing focus. None of these nodes is ever
//! `focused`: the focus owner is whatever `Layer` named, and
//! `semantic_snapshot`'s exactly-one-focus assertion stays true no matter how
//! many of these are up at once. The menu bar is the one exception to
//! "informational": its titles and rows really are operable, so they advertise
//! actions — and every one of those actions is routed in `requests.rs`.
//!
//! Card CONTENT is NOT described here. It is composed by `crate::card::content`
//! from figures whose one owner is `crate::card::figures`, the same pair
//! `render/chrome/hud.rs` draws from — so an assistive technology hears the
//! card that is drawn rather than a second description of it, and a capture
//! with no render pipeline of its own can still announce one.

use super::*;

impl App {
    pub(super) fn fold_passive(&self, nodes: &mut Vec<SemanticNode>) {
        self.fold_card(nodes, self.card_content());
        self.fold_whichkey(nodes);
        self.fold_menu_bar(nodes);
    }

    pub(super) fn fold_card(
        &self,
        nodes: &mut Vec<SemanticNode>,
        content: Option<crate::card::content::CardContent>,
    ) {
        let Some(content) = content else {
            return;
        };
        let id = content.kind.id();
        let mut card = SemanticNode::new(id, SemanticRole::Status, content.kind.title());
        card.value = Some(content.lines().join(", "));
        for (index, span) in content.spans.iter().enumerate() {
            let line_id = format!("{id}.line.{index}");
            card.children.push(line_id.clone());
            nodes.push(SemanticNode::new(
                line_id,
                SemanticRole::StaticText,
                &span.text,
            ));
        }
        nodes.push(card);
        nodes[0].children.push(id.to_string());
    }

    /// Everything a summoned card can say, gathered from the `App`'s own state.
    ///
    /// The three DOCUMENT figures — word count, frontmatter language, through-
    /// doc percent — come from [`crate::card::figures`], the pure owner the
    /// renderer derives them through as well, so no second description of them
    /// exists to drift. The LIVE-only figures are read back out of the render
    /// pipeline, which is where the `App`'s own `sync_*` push put them; with no
    /// pipeline they are the all-absent default, which is exactly the
    /// placeholder set a headless capture's offscreen pipeline draws.
    pub(in crate::app) fn card_inputs(&self, text: &str) -> crate::card::content::CardInputs {
        let buffer = self.document.buffer();
        let (cursor_line, cursor_col) = buffer.cursor_line_col();
        let overlay_active = self.workspace_state.overlay_open();
        crate::card::content::CardInputs {
            hud_held: crate::card::hud_shown(overlay_active),
            peek_shown: crate::card::peek_shown(overlay_active),
            streaks_page: crate::streaks::card_view(),
            doc: crate::card::figures::DocFigures::of(
                text,
                buffer.is_markdown(),
                cursor_line,
                cursor_col,
            ),
            eol: buffer.eol(),
            live: self
                .frame
                .gpu()
                .map(|gpu| gpu.pipeline.card_live())
                .unwrap_or_default(),
        }
    }

    /// WHICH card is open, asked from the gates alone. The gate is cheap; the
    /// INPUTS behind it are not — [`crate::card::figures::DocFigures::of`]
    /// walks the whole document — so a frame with no card up must be able to
    /// find that out without walking one (item 218).
    pub(in crate::app) fn card_kind_open(&self) -> Option<crate::card::content::CardKind> {
        let overlay_active = self.workspace_state.overlay_open();
        crate::card::content::open_kind(
            crate::card::hud_shown(overlay_active),
            crate::card::peek_shown(overlay_active),
        )
    }

    /// The card this frame, as CONTENT — composed by the same
    /// [`crate::card::content::card`] the renderer draws from, and only when
    /// one is actually open.
    fn card_content(&self) -> Option<crate::card::content::CardContent> {
        let kind = self.card_kind_open()?;
        let text = self.document.buffer().text();
        Some(crate::card::content::card(kind, &self.card_inputs(&text)))
    }

    /// The which-key panel's rows when it is up, `None` when it is not — the
    /// one gate the semantic fold and the live-`App` capture's own `CaptureOpts`
    /// both read, so the panel is announced exactly when it is drawn.
    pub(in crate::app) fn whichkey_panel_rows(&self) -> Option<Vec<(String, String)>> {
        (self.whichkey_is_shown() || crate::whichkey::force_shown())
            .then(|| self.whichkey_continuation_rows())
    }

    fn fold_whichkey(&self, nodes: &mut Vec<SemanticNode>) {
        let Some(rows) = self.whichkey_panel_rows() else {
            return;
        };
        let id = WHICHKEY_ID;
        let mut panel = SemanticNode::new(
            id,
            SemanticRole::Status,
            format!("{} continuations", crate::whichkey::PREFIX_CX),
        );
        // Informational, button-free: which-key teaches the key, it does not
        // press it. So the rows advertise nothing, and nothing is advertised
        // that `requests.rs` would have to fake.
        for (index, (key, name)) in rows.iter().enumerate() {
            let row_id = format!("{id}.row.{index}");
            panel.children.push(row_id.clone());
            nodes.push(SemanticNode::new(
                row_id,
                SemanticRole::StaticText,
                format!("{key} {name}"),
            ));
        }
        panel.value = Some(
            rows.iter()
                .map(|(key, name)| format!("{key} {name}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        nodes.push(panel);
        nodes[0].children.push(id.to_string());
    }

    fn fold_menu_bar(&self, nodes: &mut Vec<SemanticNode>) {
        if !crate::menubar::menu_bar_on() {
            return;
        }
        let open = crate::menubar::open_menu();
        let mut bar = SemanticNode::new(MENUBAR_ID, SemanticRole::MenuBar, "Menu bar");
        let is_markdown = self.document.buffer().is_markdown();
        for (index, menu) in crate::menu::roster().iter().enumerate() {
            let title_id = menu_title_id(index);
            let mut title = SemanticNode::new(&title_id, SemanticRole::MenuItem, menu.title);
            title.expanded = Some(open == Some(index));
            title.focusable = true;
            title.actions = vec![
                SemanticAction::Click,
                SemanticAction::Expand,
                SemanticAction::Collapse,
            ];
            if open == Some(index) {
                for (row, item) in crate::menu::dropdown_items(menu).iter().enumerate() {
                    let Some(label) = menu_item_label(item) else {
                        continue;
                    };
                    let item_id = menu_item_id(index, row);
                    let mut node = SemanticNode::new(&item_id, SemanticRole::MenuItem, label);
                    node.focusable = true;
                    node.actions = vec![SemanticAction::Click];
                    node.value = (!crate::menu::dropdown_item_enabled(item, is_markdown))
                        .then(|| "unavailable".to_string());
                    title.children.push(item_id);
                    nodes.push(node);
                }
            }
            bar.children.push(title_id);
            nodes.push(title);
        }
        nodes.push(bar);
        nodes[0].children.push(MENUBAR_ID.to_string());
    }

    /// The menu index a `menubar.<n>` id names, or `None` — the ONE decoder,
    /// shared by the fold above and every request arm, so an id can never be
    /// spelled one way and parsed another.
    pub(super) fn menu_title_index(&self, id: &str) -> Option<usize> {
        let index: usize = id.strip_prefix("menubar.")?.parse().ok()?;
        (crate::menubar::menu_bar_on() && index < crate::menu::roster().len()).then_some(index)
    }

    pub(super) fn menu_item_indices(&self, id: &str) -> Option<(usize, usize)> {
        let (menu, row) = id.strip_prefix("menubar.")?.split_once(".item.")?;
        let menu: usize = menu.parse().ok()?;
        let row: usize = row.parse().ok()?;
        let roster = crate::menu::roster();
        let entry = roster.get(menu)?;
        (crate::menubar::menu_bar_on() && row < crate::menu::dropdown_items(entry).len())
            .then_some((menu, row))
    }
}

fn menu_title_id(index: usize) -> String {
    format!("{MENUBAR_ID}.{index}")
}

fn menu_item_id(menu: usize, row: usize) -> String {
    format!("{MENUBAR_ID}.{menu}.item.{row}")
}

/// A separator has no name and no target, so it is not a node. Everything else
/// in a dropdown is something a user can land on.
fn menu_item_label(item: &crate::menu::RosterItem) -> Option<&'static str> {
    match item {
        crate::menu::RosterItem::Routed { label, .. } => Some(label),
        crate::menu::RosterItem::Predefined(kind) => Some(crate::menu::predefined_label(*kind)),
        crate::menu::RosterItem::Submenu { label, .. } => Some(label),
        crate::menu::RosterItem::Separator => None,
    }
}

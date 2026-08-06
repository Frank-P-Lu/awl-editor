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

impl SemanticView<'_> {
    pub(super) fn fold_passive(&self, nodes: &mut Vec<SemanticNode>) {
        self.fold_card(nodes, self.card.as_ref());
        self.fold_whichkey(nodes);
        self.fold_menu_bar(nodes);
    }

    pub(super) fn fold_card(
        &self,
        nodes: &mut Vec<SemanticNode>,
        content: Option<&crate::card::content::CardContent>,
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

    fn fold_whichkey(&self, nodes: &mut Vec<SemanticNode>) {
        let Some(rows) = self.whichkey.as_deref() else {
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
        let is_markdown = self.buffer().is_markdown();
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
}

/// The menu index a `menubar.<n>` id names, or `None` — the ONE decoder,
/// shared by the fold above and every request arm, so an id can never be
/// spelled one way and parsed another. It reads the menu-bar globals and no
/// application state at all, which is why it is not a method on either side.
pub(super) fn menu_title_index(id: &str) -> Option<usize> {
    let index: usize = id.strip_prefix("menubar.")?.parse().ok()?;
    (crate::menubar::menu_bar_on() && index < crate::menu::roster().len()).then_some(index)
}

pub(super) fn menu_item_indices(id: &str) -> Option<(usize, usize)> {
    let (menu, row) = id.strip_prefix("menubar.")?.split_once(".item.")?;
    let menu: usize = menu.parse().ok()?;
    let row: usize = row.parse().ok()?;
    let roster = crate::menu::roster();
    let entry = roster.get(menu)?;
    (crate::menubar::menu_bar_on() && row < crate::menu::dropdown_items(entry).len())
        .then_some((menu, row))
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

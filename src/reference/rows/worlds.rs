//! The worlds section's rows, read from `theme::THEMES`.

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// Every theme world, its ground, and the two faces it wears.
pub(crate) fn worlds() -> Vec<Block> {
    let mut t = Table::new(&["World", "Ground", "Display face", "Mono face"]);
    for th in crate::theme::THEMES.iter() {
        t.push(vec![
            Cell::text(th.name),
            Cell::text(if th.dark { "Dark" } else { "Light" }),
            Cell::text(th.font),
            Cell::text(th.mono),
        ]);
    }
    let note = format!(
        "The default world is {}. `--list-worlds` prints this roster; \
         `--theme <World>` selects one for a single run.",
        crate::theme::THEMES[crate::theme::DEFAULT_THEME].name
    );
    vec![block(None, Some(&note), t)]
}

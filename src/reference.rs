//! src/reference.rs — THE GENERATED REFERENCE, and the drift laws that hold it
//! to the tree.
//!
//! `REFERENCE.md` (repo root) and `site/reference.html` (the marketing site's
//! copy) are the COLD reference: what every command does, what every settings
//! row is, what may appear in `config.toml`, what worlds exist, and what
//! markdown awl renders. Both documents carry their tables between
//! `<!-- GENERATED:reference-<section>:BEGIN -->` /
//! `<!-- GENERATED:reference-<section>:END -->` markers, and NEITHER table is
//! ever hand-typed: this module builds every row by reading the same one-owner
//! rosters the app itself reads — `commands::COMMANDS`, `settings::SETTINGS`,
//! `config::Config`'s own field list, `theme::THEMES`,
//! `markdown::{MdKind, ConcealKind}`.
//!
//! WHY GENERATED RATHER THAN WRITTEN: a hand-transcribed reference is correct
//! only until the next roster change, and a reference that has quietly drifted
//! is worse than none — it is a document that lies with authority. The law
//! tests in [`law`] regenerate every section against the LIVE rosters and diff
//! byte-for-byte against what is checked in, so a new command, a changed
//! default chord, a new world, a new settings row, or a new `config.toml` key
//! fails a named test until the documents are regenerated.
//!
//! THE ROSTER IS ITSELF LAW-CHECKED. [`Section::ALL`] is swept by the laws, so
//! a new section variant with no `<!-- GENERATED -->` block in either document
//! fails by name; [`Section::marker`] and every per-section label map are
//! no-wildcard matches, so a new variant fails to COMPILE until it declares
//! what it is called.
//!
//! REGENERATE with `scripts/regen-reference.sh` (from the repo root), which
//! runs [`law::print_generated_reference_blocks`] and splices its output back
//! into both documents.
//!
//! PLATFORM INDEPENDENCE IS A HARD REQUIREMENT, not a nicety: the same law runs
//! on macOS locally and on Linux in CI, against ONE checked-in document. Every
//! row must therefore be built from a platform-independent owner — the commands
//! table asks for BOTH conventions explicitly rather than reading
//! `Convention::current()`, and the one genuinely per-OS default (`menu_bar`)
//! is rendered from `menubar`'s two authored consts rather than from `cfg!`.
//!
//! Test-only by construction (`#![cfg(...)]` below): nothing here runs in a
//! shipped binary. The documents are files; the app never reads them back.

#![cfg(all(test, not(target_arch = "wasm32")))]

mod emit;
mod law;
mod rows;

pub(crate) use emit::Table;

/// One block of generated content inside a section: an optional sub-heading, an
/// optional cold lead sentence, and the table itself.
pub(crate) struct Block {
    pub(crate) caption: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) table: Table,
}

/// A top-level section of the reference. One variant per `<!-- GENERATED -->`
/// block pair in `REFERENCE.md` and in `site/reference.html`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Section {
    Commands,
    Settings,
    Config,
    Worlds,
    Markdown,
}

impl Section {
    /// The sweep roster. Every law iterates this, so a variant added below
    /// without a matching block in both documents fails by name.
    pub(crate) const ALL: [Section; 5] = [
        Section::Commands,
        Section::Settings,
        Section::Config,
        Section::Worlds,
        Section::Markdown,
    ];

    /// The marker slug used in both documents. NO WILDCARD: a new variant fails
    /// to compile here until it declares its own marker.
    pub(crate) fn marker(self) -> &'static str {
        match self {
            Section::Commands => "reference-commands",
            Section::Settings => "reference-settings",
            Section::Config => "reference-config",
            Section::Worlds => "reference-worlds",
            Section::Markdown => "reference-markdown",
        }
    }

    /// The section's generated content, read fresh from the live rosters.
    /// NO WILDCARD, same reason as [`Self::marker`].
    pub(crate) fn blocks(self) -> Vec<Block> {
        match self {
            Section::Commands => rows::commands(),
            Section::Settings => rows::settings(),
            Section::Config => rows::config(),
            Section::Worlds => rows::worlds(),
            Section::Markdown => rows::markdown(),
        }
    }

    /// The section rendered as the markdown that belongs between its markers in
    /// `REFERENCE.md`.
    pub(crate) fn markdown(self) -> String {
        let mut out = String::new();
        for b in self.blocks() {
            if let Some(c) = &b.caption {
                out.push_str(&format!("### {c}\n\n"));
            }
            if let Some(n) = &b.note {
                out.push_str(&format!("{n}\n\n"));
            }
            out.push_str(&b.table.to_markdown());
            out.push('\n');
        }
        out.trim_end_matches('\n').to_string() + "\n"
    }

    /// The section rendered as the HTML that belongs between its markers in
    /// `site/reference.html`. Same rows, same order, same owners — only the
    /// emitter differs, so the two documents can never disagree about a fact.
    pub(crate) fn html(self) -> String {
        let mut out = String::new();
        for b in self.blocks() {
            if let Some(c) = &b.caption {
                out.push_str(&format!("<h3>{}</h3>\n", emit::escape_html(c)));
            }
            if let Some(n) = &b.note {
                out.push_str(&format!("<p class=\"note\">{}</p>\n", emit::escape_html(n)));
            }
            out.push_str(&b.table.to_html());
        }
        out
    }
}

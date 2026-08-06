//! THE SIDEBAR NAV for `site/reference.html`. One list, built by walking
//! `Section::ALL` and each section's own [`super::Block::caption`]s — the same
//! roster the tables themselves come from (a caption is a task category, a
//! config sub-table's name, a markdown sub-table's name — never a string
//! invented for the nav). A new section fails to compile until it names
//! itself ([`super::Section::title`]/[`super::Section::anchor`]); a new
//! caption inside an existing section appears in the nav the moment it
//! appears in the table, because both are read from the same `blocks()` call.
//!
//! This is deliberately NOT a `<nav>` element: `law::site`'s
//! `every_site_page_offers_the_same_navigation` sweeps every `<nav>...</nav>`
//! on every site page and requires them to offer identical destinations, which
//! is the right rule for cross-page navigation (GitHub, the editor, the other
//! docs) and the wrong rule for an in-page table of contents whose targets are
//! this page's own headings. A `role="navigation"` landmark keeps the
//! accessibility semantics without entering that sweep.

use super::Section;

/// An id-safe slug: lowercase ASCII alphanumerics, every other run of
/// characters collapsed to one `-`, no leading or trailing `-`.
fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// The id a section's sub-heading (one block's caption) anchors at —
/// namespaced under the section's own anchor so two sections can never
/// collide on a shared caption text. [`super::Section::html`] gives its
/// `<h3>` this exact id, so a nav link and its target are one computation,
/// not two strings that happen to agree today.
pub(crate) fn caption_id(section: Section, caption: &str) -> String {
    format!("{}-{}", section.anchor(), slugify(caption))
}

/// The generated sidebar: one entry per [`Section`], expanded with its own
/// sub-entries when the section carries more than one captioned block.
/// Settings and Worlds render as a single table with no caption, so they get
/// no sub-list; Commands, Configuration file, and Markdown do.
pub(crate) fn nav_html() -> String {
    let mut out = String::new();
    out.push_str(
        "<div class=\"ref-nav\" role=\"navigation\" aria-label=\"Reference sections\">\n<ul>\n",
    );
    for s in Section::ALL {
        out.push_str(&format!(
            "<li><a href=\"#{}\">{}</a>",
            s.anchor(),
            super::emit::escape_html(s.title())
        ));
        let captions: Vec<String> = s.blocks().into_iter().filter_map(|b| b.caption).collect();
        if captions.len() > 1 {
            out.push_str("\n<ul>\n");
            for c in &captions {
                out.push_str(&format!(
                    "<li><a href=\"#{}\">{}</a></li>\n",
                    caption_id(s, c),
                    super::emit::escape_html(c)
                ));
            }
            out.push_str("</ul>\n");
        }
        out.push_str("</li>\n");
    }
    out.push_str("</ul>\n</div>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_collapses_non_alnum_runs_and_trims_edges() {
        for (input, want) in [
            ("Files", "files"),
            ("What hides off the caret", "what-hides-off-the-caret"),
            ("Chords with no command", "chords-with-no-command"),
            ("  leading and trailing  ", "leading-and-trailing"),
        ] {
            assert_eq!(slugify(input), want, "slugifying {input:?}");
        }
    }

    /// The axis a hand-rolled slugifier is likeliest to get wrong: two
    /// different captions collapsing to the SAME slug. Every real caption in
    /// the roster today is checked pairwise rather than trusting that no
    /// collision exists.
    #[test]
    fn every_caption_in_every_section_has_a_distinct_id_within_its_section() {
        let _g = crate::testlock::serial();
        for s in Section::ALL {
            let mut ids: Vec<String> = s
                .blocks()
                .into_iter()
                .filter_map(|b| b.caption)
                .map(|c| caption_id(s, &c))
                .collect();
            let before = ids.len();
            ids.sort();
            ids.dedup();
            assert_eq!(
                ids.len(),
                before,
                "section `{}` has two captions that slugify to the same id",
                s.marker()
            );
        }
    }

    #[test]
    fn nav_html_carries_one_top_level_link_per_section() {
        let _g = crate::testlock::serial();
        let html = nav_html();
        for s in Section::ALL {
            assert!(
                html.contains(&format!("href=\"#{}\">{}", s.anchor(), s.title())),
                "nav_html() carries no top-level link for `{}`",
                s.marker()
            );
        }
    }
}

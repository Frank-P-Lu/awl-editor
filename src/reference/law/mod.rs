//! THE DRIFT LAWS. Each one regenerates a section from the live rosters and
//! diffs it against what is checked in, so the reference cannot quietly stop
//! being true.
//!
//! A law here fails BY NAME and says how to fix it. The fix is never "edit the
//! table": it is `scripts/regen-reference.sh`, which re-runs
//! [`print_generated_reference_blocks`] and splices the result into both
//! documents.

use super::{Section, nav};

const REFERENCE_MD: &str = crate::embedded_docs::REFERENCE_MD;
const SITE_REFERENCE_HTML: &str = crate::embedded_docs::SITE_REFERENCE_HTML;

const REGEN: &str = "regenerate with `scripts/regen-reference.sh` from the repo root";

/// The site-only nav block's marker slug. Not a [`Section`] — the nav has no
/// `REFERENCE.md` counterpart (the file carries its own hand-written, settled
/// table of contents; see the module doc), so it cannot join `Section::ALL`
/// without also demanding a markdown block that scope forbids.
const NAV_MARKER: &str = "reference-nav";

fn markers(marker: &str) -> (String, String) {
    (
        format!("<!-- GENERATED:{marker}:BEGIN -->"),
        format!("<!-- GENERATED:{marker}:END -->"),
    )
}

/// The text strictly between a marker pair, trimmed of the blank lines the
/// surrounding document naturally carries. Panics by name when the marker is
/// missing — which is exactly what a NEW [`Section`] variant with no block in
/// the document does.
fn extract<'a>(doc: &'a str, doc_name: &str, marker: &str) -> &'a str {
    let (begin, end) = markers(marker);
    let start = doc.find(&begin).unwrap_or_else(|| {
        panic!(
            "{doc_name} carries no `{begin}` — `{marker}` names a generated \
             block with no marker pair in the document; add the marker pair \
             and {REGEN}"
        )
    }) + begin.len();
    let stop = doc.find(&end).unwrap_or_else(|| {
        panic!("{doc_name} carries `{begin}` but no matching `{end}`");
    });
    assert!(
        stop > start,
        "{doc_name}'s `{end}` precedes its `{begin}` — the markers are crossed"
    );
    doc[start..stop].trim_matches('\n')
}

/// THE CENTREPIECE, repo side: every generated section of `REFERENCE.md` is
/// byte-identical to what the live rosters produce right now.
#[test]
fn every_generated_section_matches_the_tree() {
    let _g = crate::testlock::serial();
    for s in Section::ALL {
        let checked_in = extract(REFERENCE_MD, "REFERENCE.md", s.marker());
        let fresh = s.markdown();
        let fresh = fresh.trim_matches('\n');
        assert_eq!(
            checked_in,
            fresh,
            "REFERENCE.md's `{}` section has drifted from the tree it \
             describes — {REGEN}",
            s.marker()
        );
    }
}

/// THE CENTREPIECE, site side. The site copy is not a hand-mirror: it is the
/// same rows through the HTML emitter, so this is the same law with a different
/// renderer, and a roster change fails both together.
#[test]
fn every_generated_section_matches_the_tree_on_the_site() {
    let _g = crate::testlock::serial();
    for s in Section::ALL {
        let checked_in = extract(SITE_REFERENCE_HTML, "site/reference.html", s.marker());
        let fresh = s.html();
        let fresh = fresh.trim_matches('\n');
        assert_eq!(
            checked_in,
            fresh,
            "site/reference.html's `{}` section has drifted from the tree it \
             describes — {REGEN}",
            s.marker()
        );
    }
}

/// No section may be empty: a marker pair carrying nothing would satisfy the
/// byte-diff above only if the roster it reads were also empty, but an author
/// mid-edit can leave one blank and see green if the generator is stubbed.
#[test]
fn no_generated_section_is_empty() {
    let _g = crate::testlock::serial();
    for s in Section::ALL {
        assert!(
            s.markdown().lines().count() > 3,
            "section `{}` generated no rows — a reference section with no \
             content is a stub, not a document",
            s.marker()
        );
        assert!(
            s.html().contains("<td>"),
            "section `{}` generated no HTML rows",
            s.marker()
        );
    }
}

/// THE NAV CENTREPIECE: `site/reference.html`'s sidebar is byte-identical to
/// what [`nav::nav_html`] builds from `Section::ALL` and each section's own
/// captions right now — so a new section, or a new caption inside one, cannot
/// land in a table without landing in the nav in the same commit.
#[test]
fn the_generated_nav_matches_the_tree() {
    let _g = crate::testlock::serial();
    let checked_in = extract(SITE_REFERENCE_HTML, "site/reference.html", NAV_MARKER);
    let fresh = nav::nav_html();
    let fresh = fresh.trim_matches('\n');
    assert_eq!(
        checked_in, fresh,
        "site/reference.html's generated nav has drifted from the tree it \
         describes — {REGEN}"
    );
}

/// THE DANGLING-ANCHOR LAW. The byte-diff above only proves the nav block
/// equals itself regenerated — it cannot see whether the fragments the nav
/// links to actually exist on the page, because it never reads past the nav's
/// own markers. This law reads the WHOLE page: every `href="#…"` a fresh
/// [`nav::nav_html`] emits must resolve to some `id="…"` actually present in
/// `site/reference.html`, catching a slug function that changes shape on the
/// linking side (`nav::nav_html`) without changing on the target side
/// ([`Section::html`]'s `<h3 id>`, or the hand-typed section id) — or the
/// reverse.
#[test]
fn every_generated_nav_href_resolves_to_an_id_in_the_page() {
    let _g = crate::testlock::serial();
    let ids: std::collections::HashSet<&str> = SITE_REFERENCE_HTML
        .split("id=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let fresh_nav = nav::nav_html();
    let mut checked = 0usize;
    for after in fresh_nav.split("href=\"#").skip(1) {
        let frag = after
            .split('"')
            .next()
            .expect("a closing quote follows href=\"#");
        assert!(
            ids.contains(frag),
            "the generated nav links to `#{frag}`, but no `id=\"{frag}\"` \
             appears anywhere in site/reference.html"
        );
        checked += 1;
    }
    assert!(
        checked >= Section::ALL.len(),
        "the generated nav carries fewer links than there are sections — \
         nav_html() is not walking Section::ALL"
    );
}

/// The section headings are hand-typed page scaffolding, deliberately outside
/// the markers (the tables regenerate; the surrounding prose does not) — but
/// that leaves exactly one place `Section::title`/`Section::anchor` could
/// silently stop matching the page: the heading itself. This law does not
/// generate the heading; it requires the literal facts those two functions
/// predict — the anchor id and the title text — to both appear on the page,
/// so a renamed section fails here until the page's own heading is edited to
/// match.
#[test]
fn every_section_heading_in_the_page_matches_its_section() {
    let _g = crate::testlock::serial();
    for s in Section::ALL {
        assert!(
            SITE_REFERENCE_HTML.contains(&format!("id=\"{}\"", s.anchor())),
            "site/reference.html carries no `id=\"{}\"` — Section::anchor() \
             for `{:?}` no longer matches the page's section wrapper",
            s.anchor(),
            s
        );
        assert!(
            SITE_REFERENCE_HTML.contains(&format!(">{}</h2>", s.title())),
            "site/reference.html carries no `>{}</h2>` — Section::title() for \
             `{:?}` no longer matches the page's own heading text",
            s.title(),
            s
        );
    }
}

mod rosters;
mod site;

/// Not a test — the REGENERATION TOOL `scripts/regen-reference.sh` drives.
/// Prints every section in both renderings, each fenced by a delimiter the
/// script splices on. Run it through the script, not by hand.
#[test]
#[ignore]
fn print_generated_reference_blocks() {
    let _g = crate::testlock::serial();
    println!("===AWL-REFERENCE-BLOCK html {NAV_MARKER}===");
    print!("{}", nav::nav_html());
    println!("===AWL-REFERENCE-BLOCK-END===");
    for s in Section::ALL {
        println!("===AWL-REFERENCE-BLOCK md {}===", s.marker());
        print!("{}", s.markdown());
        println!("===AWL-REFERENCE-BLOCK-END===");
        println!("===AWL-REFERENCE-BLOCK html {}===", s.marker());
        print!("{}", s.html());
        println!("===AWL-REFERENCE-BLOCK-END===");
    }
}

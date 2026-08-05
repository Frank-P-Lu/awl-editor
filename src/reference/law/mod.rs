//! THE DRIFT LAWS. Each one regenerates a section from the live rosters and
//! diffs it against what is checked in, so the reference cannot quietly stop
//! being true.
//!
//! A law here fails BY NAME and says how to fix it. The fix is never "edit the
//! table": it is `scripts/regen-reference.sh`, which re-runs
//! [`print_generated_reference_blocks`] and splices the result into both
//! documents.

use super::Section;

const REFERENCE_MD: &str = crate::embedded_docs::REFERENCE_MD;
const SITE_REFERENCE_HTML: &str = crate::embedded_docs::SITE_REFERENCE_HTML;

const REGEN: &str = "regenerate with `scripts/regen-reference.sh` from the repo root";

fn markers(s: Section) -> (String, String) {
    (
        format!("<!-- GENERATED:{}:BEGIN -->", s.marker()),
        format!("<!-- GENERATED:{}:END -->", s.marker()),
    )
}

/// The text strictly between a section's two markers, trimmed of the blank
/// lines the surrounding document naturally carries. Panics by name when a
/// marker is missing — which is exactly what a NEW [`Section`] variant with no
/// block in the document does.
fn extract<'a>(doc: &'a str, doc_name: &str, s: Section) -> &'a str {
    let (begin, end) = markers(s);
    let start = doc.find(&begin).unwrap_or_else(|| {
        panic!(
            "{doc_name} carries no `{begin}` — section `{}` is in \
             `reference::Section::ALL` but has no block in the document; add \
             the marker pair and {REGEN}",
            s.marker()
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
        let checked_in = extract(REFERENCE_MD, "REFERENCE.md", s);
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
        let checked_in = extract(SITE_REFERENCE_HTML, "site/reference.html", s);
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

mod rosters;
mod site;

/// Not a test — the REGENERATION TOOL `scripts/regen-reference.sh` drives.
/// Prints every section in both renderings, each fenced by a delimiter the
/// script splices on. Run it through the script, not by hand.
#[test]
#[ignore]
fn print_generated_reference_blocks() {
    let _g = crate::testlock::serial();
    for s in Section::ALL {
        println!("===AWL-REFERENCE-BLOCK md {}===", s.marker());
        print!("{}", s.markdown());
        println!("===AWL-REFERENCE-BLOCK-END===");
        println!("===AWL-REFERENCE-BLOCK html {}===", s.marker());
        print!("{}", s.html());
        println!("===AWL-REFERENCE-BLOCK-END===");
    }
}

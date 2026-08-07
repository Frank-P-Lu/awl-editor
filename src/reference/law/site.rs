//! THE SITE LAWS: the reference must be reachable from every page of the
//! marketing site, and every page's navigation must offer the same set of
//! destinations — the site has no nav partial, so each page carries its own
//! copy and only a law can keep the copies in agreement.

fn site_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("site")
}

/// The site's top-level pages. `editor/index.html` is the Trunk wasm shell, not
/// an authored page, and is deliberately out of scope (the same boundary
/// `docs_catalog_law.rs` already draws).
fn site_pages() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(site_dir())
        .expect("site/ exists")
        .flatten()
        .collect();
    entries.sort_by_key(std::fs::DirEntry::path);
    for e in entries {
        let path = e.path();
        if path.extension().and_then(|x| x.to_str()) != Some("html") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf-8 filename")
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable page");
        out.push((name, text));
    }
    assert!(out.len() >= 4, "site/ carries its authored pages");
    out
}

/// Every `href="…"` inside a `<nav …>…</nav>` element of a page.
fn nav_hrefs(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(open) = rest.find("<nav") {
        let after = &rest[open..];
        let Some(close) = after.find("</nav>") else {
            break;
        };
        let block = &after[..close];
        let mut b = block;
        while let Some(i) = b.find("href=\"") {
            let tail = &b[i + 6..];
            let Some(j) = tail.find('"') else { break };
            out.push(tail[..j].to_string());
            b = &tail[j..];
        }
        rest = &after[close + 6..];
    }
    out
}

/// THE NAV ROSTER LAW. The site has no nav partial — `index.html` carries a top
/// nav and the document pages carry a footer nav, and both lists are
/// hand-duplicated. This round kept the duplication (the site is static files
/// served by Caddy with no build step, and introducing one to own four links
/// costs more than it saves) and replaced the silence with this law: every page
/// must offer the SAME set of destinations, so "add a link to the docs" is a
/// change that fails loudly on the page it was forgotten on.
///
/// Link TEXT is deliberately not pinned — a compact top nav may say "Try" where
/// a footer says "Try the editor". Paths ARE pinned, exactly: they were
/// normalised to root-relative so the two copies can be compared as written.
#[test]
fn every_site_page_offers_the_same_navigation() {
    let pages = site_pages();
    let mut reference: Option<(String, Vec<String>)> = None;
    for (name, text) in &pages {
        let mut hrefs = nav_hrefs(text);
        hrefs.sort();
        hrefs.dedup();
        assert!(
            !hrefs.is_empty(),
            "site/{name} carries no <nav> links — every page must reach the \
             others"
        );
        match &reference {
            None => reference = Some((name.clone(), hrefs)),
            Some((first, want)) => assert_eq!(
                &hrefs, want,
                "site/{name}'s navigation offers different destinations than \
                 site/{first}'s. The site has no nav partial, so every page \
                 carries its own copy and all copies must agree — add the \
                 missing link to site/{name}."
            ),
        }
    }
}

/// A reader must be able to reach the reference from anywhere on the site —
/// the requirement the user stated in so many words.
#[test]
fn the_reference_is_reachable_from_every_site_page() {
    for (name, text) in site_pages() {
        assert!(
            nav_hrefs(&text)
                .iter()
                .any(|h| h.ends_with("reference.html")),
            "site/{name} does not link to the reference"
        );
    }
}

/// `site/llms.txt` is a third enumeration of awl's documents. It goes stale the
/// moment a document exists that it does not name.
#[test]
fn llms_txt_names_the_reference() {
    let txt = std::fs::read_to_string(site_dir().join("llms.txt")).expect("site/llms.txt");
    assert!(
        txt.contains("REFERENCE.md"),
        "site/llms.txt enumerates awl's documents and does not name \
         REFERENCE.md — a machine reader offered every doc but the reference"
    );
}

/// The analytics beacon is on EVERY authored page or the numbers are a lie about
/// which pages people read. It is one `<script>` with no partial to share, so
/// each page carries its own copy and only a law keeps the copies in agreement —
/// the same reason `every_page_offers_the_same_nav_destinations` exists.
///
/// The set is derived from `site_pages()` rather than counted in prose, because
/// `site/README.md` said the beacon "lives in three places" while it was on
/// eight, and a hand-maintained count is wrong the first time a page is added.
/// Two copies are deliberately outside this sweep and named in that README: the
/// repo-root `index.html` (the Trunk SOURCE, where the tag lives so it survives
/// `trunk build`) and its emitted `site/editor/index.html`.
#[test]
fn every_authored_site_page_carries_the_same_analytics_beacon() {
    let pages = site_pages();
    let beacon = "src=\"//gc.zgo.at/count.js\"";
    let code = "data-goatcounter=\"https://fluflu.goatcounter.com/count\"";
    let missing: Vec<&str> = pages
        .iter()
        .filter(|(_, text)| !text.contains(beacon))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "these authored site pages carry no analytics beacon, so traffic to them \
         is invisible while the rest of the site is measured: {missing:?} \
         (of {} pages swept)",
        pages.len()
    );
    let wrong_code: Vec<&str> = pages
        .iter()
        .filter(|(_, text)| !text.contains(code))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        wrong_code.is_empty(),
        "these pages carry a beacon pointing at a DIFFERENT site code, which \
         splits the numbers across two dashboards without either looking empty: \
         {wrong_code:?}"
    );
}

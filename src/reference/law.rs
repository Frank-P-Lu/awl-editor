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

// ── THE ROSTER LAWS: a new member cannot land undocumented ────────────────────

/// Every field of [`crate::config::Config`] is either a documented
/// `config.toml` key or explicitly named as not being one.
///
/// THE COMPILE-TIME HALF is the destructuring below: it carries NO `..` arm, so
/// adding a field to `Config` fails to compile HERE until the author visits this
/// list. THE RUNTIME HALF is the set comparison: the same authored list supplies
/// both the pattern and the name strings (via `stringify!`), so there is no
/// second list to keep aligned.
#[test]
fn every_config_field_is_documented() {
    let _g = crate::testlock::serial();

    macro_rules! config_fields {
        ($($f:ident),* $(,)?) => {{
            let crate::config::Config { $($f),* } = crate::config::Config::empty();
            $( let _ = &$f; )*
            vec![$(stringify!($f)),*]
        }};
    }

    let fields: Vec<&str> = config_fields!(
        default_folder,
        workspace,
        theme,
        zoom,
        scroll_sensitivity,
        page_mode,
        page_width_prose,
        page_width_code,
        caret_mode,
        dictionary,
        writing_nits,
        spellcheck,
        history,
        autosave,
        wysiwyg,
        popover,
        inline_images,
        code_ligatures,
        cjk_priority,
        session_restore,
        outline,
        menu_bar,
        typewriter_scroll,
        file_visibility,
        stats,
        reduce_motion,
        ambient_motion,
        keymap,
        date_format,
        keys,
        linux_keep_emacs,
        path,
    );

    let documented: Vec<&str> = super::rows::documented_config_keys();
    let non_keys = super::rows::CONFIG_NON_KEYS;

    for f in &fields {
        assert!(
            documented.contains(f) || non_keys.contains(f),
            "`Config::{f}` is neither documented in REFERENCE.md's \
             configuration section nor listed in \
             `reference::rows::CONFIG_NON_KEYS` — a config key a user can \
             write and no reference entry is exactly the silent drift this \
             document exists to prevent"
        );
    }
    for d in &documented {
        assert!(
            fields.contains(d),
            "the reference documents a `config.toml` key `{d}` that \
             `Config` has no field for — the reference must never invent a key"
        );
    }
    for n in non_keys {
        assert!(
            fields.contains(n),
            "`CONFIG_NON_KEYS` names `{n}`, which `Config` no longer has — \
             shrink the list rather than leave it stale"
        );
    }
}

/// THE LAW THIS ROUND'S OWN BUG WROTE. The settings table's `config.toml` key
/// column was first generated from `toggle_key`/`value_key`/`path_key` alone,
/// and printed `project_root` — a key `path_key` returns as a DISPATCH route
/// (`App::setting_path_pick` intercepts it and switches project) and that
/// `Config` has had no field for since it was retired. The table was telling a
/// reader to write a line the loader never reads.
///
/// So: every key the settings table prints must be a real field of `Config`, and
/// every key excused from that must still be produced by one of the three maps —
/// a stale excuse is as wrong as a missing one.
#[test]
fn every_settings_row_key_is_a_real_config_key() {
    let _g = crate::testlock::serial();
    let documented = super::rows::documented_config_keys();
    for row in crate::settings::SETTINGS {
        let Some(key) = super::rows::config_key_of(row.id) else {
            continue;
        };
        assert!(
            documented.contains(&key),
            "the settings row `{}` reports `{key}` as its config.toml key, but \
             `Config` has no such field — either the key is a dispatch route \
             (add it to `reference::rows::SETTINGS_DISPATCH_ONLY_KEYS` with the \
             reason) or the configuration table is missing an entry",
            row.name
        );
    }
    for excused in super::rows::SETTINGS_DISPATCH_ONLY_KEYS {
        let still_routed = crate::settings::SETTINGS.iter().any(|r| {
            crate::settings::toggle_key(r.id) == Some(excused)
                || crate::settings::value_key(r.id) == Some(excused)
                || crate::settings::path_key(r.id) == Some(excused)
        });
        assert!(
            still_routed,
            "`{excused}` is excused from the config-key check but no settings \
             row produces it any more — shrink the list rather than leave it \
             stale"
        );
        assert!(
            !documented.contains(excused),
            "`{excused}` is excused as a dispatch-only key but IS a documented \
             config.toml key — it should be documented, not excused"
        );
    }
}

/// Every default the configuration table prints comes from a real owner. The
/// generator panics by name for a key with no owner; this runs it over the whole
/// roster so that panic is reached in CI rather than by a reader.
#[test]
fn every_documented_config_key_has_a_default_owner() {
    let _g = crate::testlock::serial();
    for key in super::rows::documented_config_keys() {
        let _ = super::rows::config_default(key);
    }
}

/// Every chord the keymap matches outside the catalog is named in the
/// reference. The generator panics by name for an unnamed slug; this reaches
/// that panic in CI rather than leaving it for a reader.
#[test]
fn every_synthetic_chord_is_named() {
    let _g = crate::testlock::serial();
    for (slug, _, _) in crate::keytoken::SYNTHETIC {
        let name = super::rows::synthetic_name(slug);
        assert!(
            Section::Commands.markdown().contains(name),
            "synthetic chord `{slug}` renders as `{name}`, which does not \
             appear in the generated commands section — {REGEN}"
        );
    }
}

/// Every span tag [`crate::markdown::MdKind`] can produce is claimed by a
/// documented construct.
///
/// THE COMPILE-TIME HALF is [`assert_md_kind_roster_covers`]: a no-wildcard
/// match over every variant, so a new span kind fails to compile until it is
/// visited. THE RUNTIME HALF sweeps the payload domains (all six heading levels,
/// both code placements, every syntax role, both task states, every conceal
/// kind) and asserts each resulting tag is documented — the axis an author
/// checking only `Bold` would miss.
#[test]
fn every_markdown_span_tag_is_documented() {
    let _g = crate::testlock::serial();
    let documented = super::rows::documented_tags();
    for k in every_md_kind() {
        assert_md_kind_roster_covers(&k);
        let tag = k.tag();
        assert!(
            documented.contains(&tag),
            "markdown span tag `{tag}` (from {k:?}) is produced by the \
             renderer but claimed by no construct in REFERENCE.md's markdown \
             section — add a row for it in `reference::rows::CONSTRUCTS`"
        );
    }
    for tag in &documented {
        assert!(
            every_md_kind().iter().any(|k| k.tag() == *tag),
            "REFERENCE.md's markdown section claims a span tag `{tag}` the \
             renderer no longer produces — the reference must never invent a \
             construct"
        );
    }
}

/// Every value `MdKind::tag` can be asked about, payload domains included.
fn every_md_kind() -> Vec<crate::markdown::MdKind> {
    use crate::markdown::MdKind;
    let mut out = vec![
        MdKind::Markup,
        MdKind::Bold,
        MdKind::Italic,
        MdKind::BoldItalic,
        MdKind::Quote,
        MdKind::ListMarker,
        MdKind::LinkText,
        MdKind::TaskDone,
        MdKind::Highlight,
        MdKind::Strikethrough,
        MdKind::Rule,
        MdKind::TablePipe,
        MdKind::TableSep,
        MdKind::TableHeader,
    ];
    for level in 1..=6u8 {
        out.push(MdKind::Heading(level));
    }
    for inline in [true, false] {
        out.push(MdKind::Code { inline });
    }
    for done in [true, false] {
        out.push(MdKind::Task(done));
    }
    for ck in crate::markdown::ConcealKind::ALL {
        out.push(MdKind::ConcealMarkup(ck));
    }
    for role in crate::syntax::SynKind::ALL {
        out.push(MdKind::CodeSyntax {
            role,
            lang: crate::syntax::Lang::ALL[0],
        });
    }
    out
}

/// NO WILDCARD, on purpose: a new `MdKind` variant fails to COMPILE here until
/// its author adds it to [`every_md_kind`] and gives it a documented construct.
fn assert_md_kind_roster_covers(k: &crate::markdown::MdKind) {
    use crate::markdown::MdKind;
    match k {
        MdKind::Markup
        | MdKind::ConcealMarkup(_)
        | MdKind::Heading(_)
        | MdKind::Bold
        | MdKind::Italic
        | MdKind::BoldItalic
        | MdKind::Code { .. }
        | MdKind::CodeSyntax { .. }
        | MdKind::Quote
        | MdKind::ListMarker
        | MdKind::LinkText
        | MdKind::Task(_)
        | MdKind::TaskDone
        | MdKind::Highlight
        | MdKind::Strikethrough
        | MdKind::Rule
        | MdKind::TablePipe
        | MdKind::TableSep
        | MdKind::TableHeader => {}
    }
}

/// The conceal table lists every kind the renderer can conceal. `ConcealKind`
/// derives its own `ALL` from its variant list (`enum_with_all!`), so this
/// sweeps the real roster rather than a copy of it.
#[test]
fn every_conceal_kind_is_documented() {
    let _g = crate::testlock::serial();
    let rendered = Section::Markdown.markdown();
    for k in crate::markdown::ConcealKind::ALL {
        let (name, _, _) = super::rows::conceal_facts_for(k);
        assert!(
            rendered.contains(name),
            "conceal kind {k:?} renders as `{name}`, which does not appear in \
             the generated markdown section — {REGEN}"
        );
    }
}

/// Every world in the roster is named in `WORLDS.md`'s table, and that document
/// names no world the roster has lost. `WORLDS.md` is prose (each world's
/// flavour) and stays hand-written; only its MEMBERSHIP is law-checked, which is
/// the drift that actually strands a reader.
#[test]
fn worlds_md_names_exactly_the_theme_roster() {
    let _g = crate::testlock::serial();
    let doc = crate::embedded_docs::WORLDS_MD;
    for t in crate::theme::THEMES.iter() {
        assert!(
            doc.contains(&format!("**{}**", t.name)),
            "world `{}` is in `theme::THEMES` but is not named in WORLDS.md's \
             table — a new world must arrive with its flavour sentence",
            t.name
        );
    }
    for bolded in bolded_names(doc) {
        assert!(
            crate::theme::THEMES.iter().any(|t| t.name == bolded)
                || !bolded.chars().next().is_some_and(char::is_uppercase),
            "WORLDS.md names `{bolded}` as a world, but `theme::THEMES` has no \
             such world — a removed world must leave the document too"
        );
    }
}

/// `**Name**` row labels inside WORLDS.md's at-a-glance table, and ONLY that
/// table — the document carries later tables (background styles, ornament
/// families) whose rows are bolded the same way and are not worlds.
fn bolded_names(doc: &str) -> Vec<String> {
    let at_a_glance = doc
        .split_once("## The worlds at a glance")
        .expect("WORLDS.md carries its at-a-glance table")
        .1;
    let at_a_glance = at_a_glance.split_once("\n## ").map_or(at_a_glance, |(a, _)| a);
    at_a_glance
        .lines()
        .filter(|l| l.trim_start().starts_with("| **"))
        .filter_map(|l| {
            let rest = l.split_once("**")?.1;
            let (name, _) = rest.split_once("**")?;
            Some(name.to_string())
        })
        .collect()
}

// ── THE SITE LAWS ─────────────────────────────────────────────────────────────

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
            nav_hrefs(&text).iter().any(|h| h.ends_with("reference.html")),
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

// ── THE REGENERATION TOOL ─────────────────────────────────────────────────────

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

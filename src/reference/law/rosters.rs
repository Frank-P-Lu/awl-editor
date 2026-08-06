//! THE ROSTER LAWS: a new command, config key, world, span kind or conceal
//! kind cannot land undocumented.

use super::REGEN;
use crate::reference::{Section, rows};

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

    let documented: Vec<&str> = rows::documented_config_keys();
    let non_keys = rows::CONFIG_NON_KEYS;

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
    let documented = rows::documented_config_keys();
    for row in crate::settings::SETTINGS {
        let Some(key) = rows::config_key_of(row.id) else {
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
    for excused in rows::SETTINGS_DISPATCH_ONLY_KEYS {
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
    for key in rows::documented_config_keys() {
        let _ = rows::config_default(key);
    }
}

/// Every chord the keymap matches outside the catalog is named in the
/// reference. The generator panics by name for an unnamed slug; this reaches
/// that panic in CI rather than leaving it for a reader.
#[test]
fn every_synthetic_chord_is_named() {
    let _g = crate::testlock::serial();
    for (slug, _, _) in crate::keytoken::SYNTHETIC {
        let name = rows::synthetic_name(slug);
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
    let documented = rows::documented_tags();
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
        let (name, _, _) = rows::conceal_facts_for(k);
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
    let at_a_glance = at_a_glance_table(doc);
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

/// The at-a-glance table's own text, isolated from the tables that follow it
/// (background styles, ornament families) — shared by [`bolded_names`] and
/// [`table_row`] so the two never disagree about which lines are the table.
fn at_a_glance_table(doc: &str) -> &str {
    let at = doc
        .split_once("## The worlds at a glance")
        .expect("WORLDS.md carries its at-a-glance table")
        .1;
    at.split_once("\n## ").map_or(at, |(a, _)| a)
}

/// One data row of the at-a-glance table, as header-name -> cell-text — by
/// COLUMN NAME rather than position, so a reordered column cannot silently
/// compare the wrong cells against each other.
fn table_row(doc: &str, world: &str) -> Option<std::collections::HashMap<String, String>> {
    let mut lines = at_a_glance_table(doc)
        .lines()
        .filter(|l| l.trim_start().starts_with('|'));
    let header = lines.next()?;
    let headers: Vec<String> = header
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().split(" (").next().unwrap_or("").trim().to_string())
        .collect();
    lines.next(); // the `| --- | --- |` separator row
    for line in lines {
        let cells: Vec<String> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        if cells.len() != headers.len() {
            continue;
        }
        let name_idx = headers.iter().position(|h| h == "World")?;
        if cells[name_idx].trim_matches('*') == world {
            return Some(headers.into_iter().zip(cells).collect());
        }
    }
    None
}

/// A roster font's human headline: a trailing run of `<digits>pt` tokens
/// dropped. `family: "Newsreader 16pt 16pt"` is the literal family name
/// fontdb resolves (`render.rs` documents why — changing the roster field
/// would break Bilby's font resolution), but WORLDS.md is the flavour
/// document, not the technical one, so its Display column may show the
/// family alone: `"Newsreader 16pt 16pt"` -> `"Newsreader"`,
/// `"Fraunces 9pt"` -> `"Fraunces"`, `"Bitter"` -> `"Bitter"` (no-op).
fn family_headline(font: &str) -> String {
    let mut words: Vec<&str> = font.split(' ').collect();
    while let Some(last) = words.last() {
        let digits = last.strip_suffix("pt").filter(|d| !d.is_empty());
        if digits.is_some_and(|d| d.chars().all(|c| c.is_ascii_digit())) {
            words.pop();
        } else {
            break;
        }
    }
    words.join(" ")
}

/// THE DISPLAY/MONO/AXIS DRIFT LAW. `worlds_md_names_exactly_the_theme_roster`
/// above only checks that a world's NAME appears; nothing checked the six
/// columns a reader actually uses the table for. Measured before this law
/// existed: 11 of the 20 rows had drifted — one wrong Display face (Mopoke's
/// row still named a font no longer in the roster) and ten stale axis tags,
/// every one in the same direction (the table claiming a Time/Register/
/// Voice/Temperature the roster had since curated away to `None` — the
/// "curated maximum of four per band" pass documented above this table
/// evidently updated `theme/worlds.rs` without updating this table). This law
/// reads the values instead of trusting the prose, so that drift cannot
/// recur silently.
///
/// A mismatch is fixed by editing WORLDS.md's row to match the roster — NOT
/// by editing `theme/worlds.rs` to match the table; the tags are a curation
/// call that belongs to whoever owns the roster, and this law only asks the
/// two to agree, never which one is "right" when they first disagree.
#[test]
fn worlds_md_display_mono_and_axis_match_the_theme_roster() {
    let _g = crate::testlock::serial();
    let doc = crate::embedded_docs::WORLDS_MD;
    for t in crate::theme::THEMES.iter() {
        let row = table_row(doc, t.name).unwrap_or_else(|| {
            panic!(
                "WORLDS.md's at-a-glance table has no row for `{}` — {REGEN_WORLDS}",
                t.name
            )
        });
        let want_display = |col: &str| {
            row.get(col)
                .unwrap_or_else(|| panic!("WORLDS.md's at-a-glance table has no `{col}` column"))
        };
        let display = want_display("Display");
        assert!(
            display == t.font || *display == family_headline(t.font),
            "WORLDS.md's `{}` row shows Display `{display}`, but \
             `theme::THEMES` carries `{}` (allowing only a `<n>pt`-trimmed \
             headline of it) — {REGEN_WORLDS}",
            t.name,
            t.font
        );
        assert_eq!(
            want_display("Mono"),
            t.mono,
            "WORLDS.md's `{}` row's Mono has drifted from `theme::THEMES` — \
             {REGEN_WORLDS}",
            t.name
        );
        for (col, tag) in [
            ("Time", t.tags.time),
            ("Register", t.tags.register),
            ("Voice", t.tags.voice),
            ("Temp", t.tags.temperature),
        ] {
            let want = tag.unwrap_or("—");
            assert_eq!(
                want_display(col),
                want,
                "WORLDS.md's `{}` row's {col} column reads `{}` but \
                 `theme::THEMES` tags it `{want}` — {REGEN_WORLDS}",
                t.name,
                want_display(col)
            );
        }
    }
}

const REGEN_WORLDS: &str = "update WORLDS.md's at-a-glance row to match \
     theme::THEMES (there is no regen script for this document — the table \
     is hand-written prose except for the fact it must agree with the \
     roster on)";

/// WORLDS.md's SECOND table, `## The margin backgrounds` — a "Shipping
/// worlds" column per `Background` variant, hand-maintained the same way the
/// at-a-glance table is. Isolated the same way [`at_a_glance_table`] isolates
/// its own section.
fn background_table(doc: &str) -> &str {
    let sec = doc
        .split_once("## The margin backgrounds")
        .expect("WORLDS.md carries its margin-backgrounds table")
        .1;
    sec.split_once("\n## ").map_or(sec, |(a, _)| a)
}

/// Every `(bold label, raw "Shipping worlds" cell)` row of the
/// margin-backgrounds table, by column HEADER name — same discipline as
/// [`table_row`] above, so a reordered column cannot compare the wrong
/// cells.
fn background_rows(doc: &str) -> Vec<(String, String)> {
    let mut lines = background_table(doc)
        .lines()
        .filter(|l| l.trim_start().starts_with('|'));
    let header = lines.next().expect("the table carries a header row");
    let headers: Vec<String> = header
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect();
    lines.next(); // the `| --- | --- |` separator row
    let label_idx = headers
        .iter()
        .position(|h| h == "Background")
        .expect("a Background column");
    let ships_idx = headers
        .iter()
        .position(|h| h == "Shipping worlds")
        .expect("a Shipping worlds column");
    let mut out = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
        if cells.len() != headers.len() {
            continue;
        }
        // The label cell is `**Name**` or `**Name** (item N)`: the bold run
        // is the row's identity, the parenthetical is a citation.
        let raw = cells[label_idx].trim();
        let Some(label) = raw.split("**").nth(1) else {
            continue;
        };
        out.push((label.to_string(), cells[ships_idx].trim().to_string()));
    }
    out
}

/// The "Shipping worlds" cell's world names, stripped of a trailing
/// parenthetical qualifier (`"Paperbark (Strata)"` -> `"Paperbark"`) and of
/// the `*(none — …)*` empty-row placeholder.
fn shipping_worlds(cell: &str) -> Vec<String> {
    if cell.trim_start().starts_with("*(") {
        return Vec::new();
    }
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split([' ', '('])
                .next()
                .unwrap_or(s)
                .trim_end_matches('*')
                .to_string()
        })
        .collect()
}

/// The `Background::as_str()` discriminant a margin-backgrounds row label
/// keys to. NO WILDCARD: an unrecognised label panics by name rather than
/// being silently skipped — the failure mode the roster-derived laws above
/// exist to avoid (see CLAUDE.md's enrolment-predicate tripwire).
fn background_variant_key(label: &str) -> &'static str {
    match label {
        "Gradient" => "gradient",
        "Dots" => "dots",
        "Pinstripe" => "pinstripe",
        "Stripes" => "stripes",
        "Lava" => "lava",
        "Bands" => "bands",
        "Zigzag" => "zigzag",
        "Deckle" => "deckle",
        "Warped grid" => "warped-grid",
        "Waves" => "waves",
        other => panic!(
            "WORLDS.md's margin-backgrounds table has a row `{other}` this \
             law does not know how to key against `Background::as_str()` — \
             add it to `background_variant_key` (or explain why the row is \
             not a `Background` variant)"
        ),
    }
}

/// Background variants with a real occupant in `theme::THEMES` but no row in
/// WORLDS.md's margin-backgrounds table. Tracked here rather than left an
/// unexamined gap: `Background::Organic`'s sole occupant is Bowerbird, and
/// giving it a row is a content decision (what to say it draws), not a
/// membership fix — out of scope for a law that only checks agreement.
const BACKGROUND_VARIANTS_WITH_NO_ROW: &[&str] = &["organic"];

/// THE BACKGROUND-MEMBERSHIP DRIFT LAW — the at-a-glance law's companion over
/// WORLDS.md's SECOND table. A world's `background` field changing is two
/// edits a human has to remember to make together (drop the name from the
/// row it left, add it to the row it joined), and nothing forced the second
/// half. Found by this law on first run: Magpie carries `Background::Bands`
/// while the Pinstripe row still claimed it and the Bands row called itself
/// dormant; Galah has carried `Background::Deckle` (Fibres) since it shipped
/// while the Gradient row still claimed it, the Deckle row named only
/// Paperbark, and the Deckle row's own prose still called Fibres dormant.
#[test]
fn worlds_md_background_membership_matches_the_theme_roster() {
    let _g = crate::testlock::serial();
    let doc = crate::embedded_docs::WORLDS_MD;
    for (label, cell) in background_rows(doc) {
        let key = background_variant_key(&label);
        let mut want: Vec<String> = crate::theme::THEMES
            .iter()
            .filter(|t| t.background.as_str() == key)
            .map(|t| t.name.to_string())
            .collect();
        let mut have = shipping_worlds(&cell);
        want.sort();
        have.sort();
        assert_eq!(
            have, want,
            "WORLDS.md's `{label}` row's Shipping worlds cell lists `{have:?}` \
             but `theme::THEMES` carries `Background::{key}` on `{want:?}` — \
             {REGEN_WORLDS}"
        );
    }
}

/// The completeness half: every `Background` variant a world actually uses is
/// either keyed by a row above or explicitly excused. Without this, a new
/// ground (or Organic's existing, still-unrowed one) could sit permanently
/// invisible to [`worlds_md_background_membership_matches_the_theme_roster`],
/// which only walks rows that already exist.
#[test]
fn every_used_background_variant_has_a_worlds_md_row_or_is_excused() {
    let _g = crate::testlock::serial();
    let doc = crate::embedded_docs::WORLDS_MD;
    let keyed: std::collections::HashSet<&str> = background_rows(doc)
        .iter()
        .map(|(label, _)| background_variant_key(label))
        .collect();
    for t in crate::theme::THEMES.iter() {
        let key = t.background.as_str();
        assert!(
            keyed.contains(key) || BACKGROUND_VARIANTS_WITH_NO_ROW.contains(&key),
            "world `{}` carries `Background::{key}`, which has no row in \
             WORLDS.md's margin-backgrounds table and is not excused in \
             `BACKGROUND_VARIANTS_WITH_NO_ROW` — add a row or excuse it with \
             a reason",
            t.name
        );
    }
    for excused in BACKGROUND_VARIANTS_WITH_NO_ROW {
        assert!(
            !keyed.contains(excused),
            "`{excused}` is excused as having no WORLDS.md row, but a row \
             now keys to it — shrink the excuse list rather than leave it \
             stale"
        );
        assert!(
            crate::theme::THEMES
                .iter()
                .any(|t| t.background.as_str() == *excused),
            "`{excused}` is excused as a used-but-unrowed background \
             variant, but no world uses it any more — shrink the list"
        );
    }
}

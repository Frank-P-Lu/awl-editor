//! THE ROSTER LAWS: a new command, config key, world, span kind or conceal
//! kind cannot land undocumented.
//!
//! Split by subject: this file carries the config/settings/chord/markdown/
//! conceal laws; [`worlds`] carries everything that reads WORLDS.md (the
//! at-a-glance table's own laws, the margin-backgrounds table's, and the
//! table-parsing helpers both share).

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

/// THE DESCRIPTION COLUMN'S OWN LAW. [`crate::commands::Command::description`]
/// is `Option<&'static str>` rather than a bare `&'static str` precisely so a
/// command with nothing reliable to say can carry an honest `None` instead of
/// an invented sentence — but that same option makes `Some("")` representable,
/// which would render as a blank cell in [`rows::commands`]'s "What it does"
/// column, indistinguishable from a roster bug (a `Cell::text_or_dash` `Some`
/// draws no dash, so a reader has no way to tell "described as empty" from "the
/// generator dropped a character"). This sweeps the one axis a byte-diff
/// against the checked-in table cannot: an author can hand-edit a catalog
/// literal to `Some("")` or `Some("  ")` and `regen-reference.sh` will happily
/// print a blank cell that still passes `every_generated_section_matches_the_tree`
/// (the blank is what's checked in, because it's what was generated) — this
/// law is what fails instead, and it fails BY NAME.
#[test]
fn every_command_description_is_meaningful_when_present() {
    let _g = crate::testlock::serial();
    for c in crate::commands::COMMANDS.iter() {
        let Some(d) = c.description else { continue };
        assert!(
            !d.trim().is_empty(),
            "command `{}` carries `description: Some(\"\")` — an empty \
             description renders as a blank cell, indistinguishable from a \
             missing one; either write a real description or use `None`",
            c.name
        );
        assert_eq!(
            d.trim(),
            d,
            "command `{}`'s description has leading/trailing whitespace",
            c.name
        );
        let bare_name = c.name.trim_end_matches('…').trim();
        assert!(
            !d.trim_end_matches('.').eq_ignore_ascii_case(bare_name),
            "command `{}`'s description just restates its own name (`{d}`) — \
             the docs-voice rule forbids paraphrasing the name back at the \
             reader; describe what the command DOES",
            c.name
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

mod worlds;

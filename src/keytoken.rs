//! CHORD + COMMAND-NAME TOKENS — the CONVENTION-TRUTHFUL SURFACES round's write
//! side for the starting docs (`samples/welcome.md`, `samples/tour.md`,
//! `GUIDE.md`), extended (docs-vs-catalog law round) to cover cited command
//! NAMES the same way.
//!
//! A literal chord glyph baked into a doc (`⌘P`) is a LIE the instant it's read
//! under a different convention or platform — a Linux visitor sees a mac glyph
//! that doesn't fire; a web visitor on a browser-reserved chord (New document,
//! Switch theme…) sees a chord that's silently eaten by the browser chrome
//! before the page ever gets it. `{{key:slug}}` is the fix: a token substituted
//! at the RIGHT moment for each surface (seed time for welcome/tour — see
//! `fs::seed_write_if_absent` — open time for GUIDE.md — see `App::open_guide` /
//! `main::run`'s headless arm) through the SAME truthful label owner every other
//! chord surface reads (`commands::resolved_native_label_truthful`), so a doc
//! can never show a chord that doesn't actually fire.
//!
//! `slug` is a catalog command's own config slug (`commands::slug`) for the vast
//! majority of `{{key:slug}}` tokens; [`SYNTHETIC`] covers the two chords that
//! are real, fixed, non-rebindable presses with NO catalog row at all (the
//! command palette's own dedicated Cmd-P, the held stats HUD's Option-Cmd-I) —
//! both are the most-taught chords in the onboarding docs, so they still need a
//! token even though `commands::COMMANDS` has no entry to hang it on.
//!
//! **`{{cmd:slug}}`** is the sibling convention for a CITED COMMAND NAME — text
//! like `"Widen page"` or `"Keep version…"` that names a palette command by
//! itself, not by its chord. A renamed or retired command leaves a literal
//! mention silently stale (the same "doc taught something that no longer
//! fires" failure mode the welcome-doc incident named, one axis over: a name
//! instead of a chord). `{{cmd:slug}}` substitutes the catalog's own current
//! display NAME for `slug` (convention/platform-independent — a command's name
//! doesn't vary by platform, unlike its chord), through the SAME render seam
//! as `{{key:}}`, so a doc author writes `{{cmd:widen_page}}` once and it
//! always reads exactly what the live palette calls that command. DOCS
//! CONVENTION: any specific command-name citation in `samples/welcome.md` /
//! `samples/tour.md` / `GUIDE.md` prose (outside the generated keys-reference
//! table, which is already whole-table catalog-verified — see `guide.rs`)
//! should be a `{{cmd:slug}}` token, not literal text — the law test
//! [`tests::every_key_token_in_the_starting_docs_resolves`] is what actually
//! enforces it (an unknown slug renders a loud `[[unknown-cmd:slug]]` marker
//! rather than vanishing, and fails that test).

use crate::commands::{self, Platform};
use crate::convention::Convention;

/// The token delimiters: `{{` ... `}}`, with the token KIND named by its
/// prefix immediately inside (`key:` a chord, `cmd:` a command name).
const OPEN: &str = "{{";
const CLOSE: &str = "}}";
const KEY_PREFIX: &str = "key:";
const CMD_PREFIX: &str = "cmd:";

/// Non-catalog synthetic slugs: `(slug, mac spec, linux spec)`, each spec in
/// the same terse form [`crate::keyspec::parse_chord`] accepts. See the module
/// doc for why these two exist outside the catalog.
const SYNTHETIC: &[(&str, &str, &str)] = &[
    // The command palette: its own dedicated Cmd-P/Ctrl-P, matched directly in
    // `keymap.rs::resolve` (never a catalog row, never rebindable via `[keys]`).
    ("command_palette", "Cmd-P", "C-p"),
    // The held stats HUD: Option-Cmd-I / Ctrl-Alt-I, matched directly in
    // `keymap.rs::resolve_named`'s `native && alt` arm (deliberately NOT a
    // catalog row — see `commands.rs`'s "Held stats HUD" doc: a discrete
    // palette selection has no key-release to dismiss a hold-only panel with).
    ("stats_hud", "Cmd-M-i", "C-M-i"),
];

/// Resolve `slug_want`'s chord LABEL for `convention`+`platform`: first the
/// catalog, through [`commands::resolved_native_label_truthful`] (the ONE
/// owner every other chord surface reads — a token can therefore never show a
/// chord that doesn't actually fire, web-reserved/Linux-displaced/web-alternate
/// included), then [`SYNTHETIC`] for the two dedicated chords with no catalog
/// row. `None` for an unknown slug.
pub fn key_token_label(
    slug_want: &str,
    convention: Convention,
    platform: Platform,
) -> Option<String> {
    key_token_spec(slug_want, convention, platform).map(|spec| match convention {
        Convention::Mac => crate::keyspec::mac_glyph_chord(&spec),
        Convention::Linux => crate::keyspec::linux_glyph_chord(&spec),
    })
}

/// [`key_token_label`]'s SPEC half — the terse chord
/// [`crate::keyspec::parse_chord`] accepts, before it becomes a ⌘ glyph or a
/// `Ctrl+` word. `None` for an unknown slug; `Some("")` for a slug that
/// resolves to NO chord on this convention/platform (Insert-link on Linux,
/// where `C-k` stays kill-line; a browser-reserved chord with no alternate).
///
/// Every reader of "what does this doc teach" goes through here, so the law
/// below can drive exactly the chord a reader is shown — never a second,
/// possibly-diverging derivation of it.
pub fn key_token_spec(
    slug_want: &str,
    convention: Convention,
    platform: Platform,
) -> Option<String> {
    if let Some(c) = commands::COMMANDS
        .iter()
        .find(|c| commands::slug(c.name) == slug_want)
    {
        return Some(commands::resolved_native_truthful(c, convention, platform));
    }
    SYNTHETIC
        .iter()
        .find(|(s, _, _)| *s == slug_want)
        .map(|(_, mac, linux)| match convention {
            Convention::Mac => (*mac).to_string(),
            Convention::Linux => (*linux).to_string(),
        })
}

/// Every [`SYNTHETIC`] chord's Mac-glyph LABEL (`"⌘P"`, `"⌘⌥I"`) — the
/// non-catalog half of "every valid Mac chord label", for a consumer (the
/// docs-vs-catalog law's HTML-surface check, `docs_catalog_law.rs`) that
/// needs the WHOLE valid set without duplicating [`SYNTHETIC`]'s two
/// hardcoded specs. Test-only: its one consumer is itself `cfg(test)`.
#[cfg(test)]
pub(crate) fn synthetic_mac_glyphs() -> Vec<String> {
    SYNTHETIC
        .iter()
        .map(|(_, mac, _)| crate::keyspec::mac_glyph_chord(mac))
        .collect()
}

/// Resolve `slug_want`'s command DISPLAY NAME straight from the live catalog
/// (`commands::COMMANDS`, keyed the same way `[keys]` rebinding is — via
/// [`commands::slug`]), for a `{{cmd:slug}}` token. `None` for an unknown slug
/// (a typo, or a command renamed/retired since the doc was written). Unlike
/// [`key_token_label`] this carries no convention/platform parameter — a
/// command's NAME doesn't vary by platform, only its chord does.
pub fn cmd_token_label(slug_want: &str) -> Option<String> {
    commands::COMMANDS
        .iter()
        .find(|c| commands::slug(c.name) == slug_want)
        .map(|c| c.name.to_string())
}

/// Replace every `{{key:slug}}` / `{{cmd:slug}}` token in `text` with
/// [`key_token_label`] / [`cmd_token_label`]'s resolved text for
/// `convention`+`platform`. An UNKNOWN slug is left as a visible
/// `[[unknown-key:slug]]` / `[[unknown-cmd:slug]]` marker — never panics,
/// never silently vanishes — so a typo'd token is obvious in the rendered doc
/// even outside the build-time law test
/// (`tests::every_key_token_in_the_starting_docs_resolves`) that actually
/// guards against one ever shipping. An unterminated `{{...` (missing `}}`)
/// is likewise left verbatim rather than eating the rest of the document, and
/// a `{{...}}` span whose inner text carries neither recognized prefix is left
/// as `[[unknown-token:...]]` (there is no third kind today, but a stray
/// unrecognized brace pair should still be loud, not silent).
pub fn render_key_tokens(text: &str, convention: Convention, platform: Platform) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        out.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];
        match after.find(CLOSE) {
            Some(end) => {
                let inner = &after[..end];
                if let Some(slug_want) = inner.strip_prefix(KEY_PREFIX) {
                    match key_token_label(slug_want, convention, platform) {
                        Some(label) => out.push_str(&label),
                        None => out.push_str(&format!("[[unknown-key:{slug_want}]]")),
                    }
                } else if let Some(slug_want) = inner.strip_prefix(CMD_PREFIX) {
                    match cmd_token_label(slug_want) {
                        Some(label) => out.push_str(&label),
                        None => out.push_str(&format!("[[unknown-cmd:{slug_want}]]")),
                    }
                } else {
                    out.push_str(&format!("[[unknown-token:{inner}]]"));
                }
                rest = &after[end + CLOSE.len()..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WELCOME: &str = crate::embedded_docs::WELCOME_MD;
    const TOUR: &str = crate::embedded_docs::TOUR_MD;
    const GUIDE: &str = crate::embedded_docs::GUIDE_MD;

    /// Every `{{key:slug}}` slug used in `text`, in order.
    fn extract_token_slugs(text: &str) -> Vec<String> {
        extract_tagged_slugs(text, KEY_PREFIX)
    }

    /// Every `{{cmd:slug}}` slug used in `text`, in order.
    fn extract_cmd_slugs(text: &str) -> Vec<String> {
        extract_tagged_slugs(text, CMD_PREFIX)
    }

    /// Shared token-body scanner: every `{{<prefix><slug>}}` occurrence in
    /// `text`, `<slug>` extracted in order. A `{{...}}` span carrying a
    /// DIFFERENT prefix (or none) is simply not this scan's business — the
    /// unified render-side law tests below drive `render_key_tokens` itself,
    /// which is what actually enforces "every token kind resolves or is loud."
    fn extract_tagged_slugs(text: &str, prefix: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = text;
        while let Some(start) = rest.find(OPEN) {
            let after = &rest[start + OPEN.len()..];
            match after.find(CLOSE) {
                Some(end) => {
                    let inner = &after[..end];
                    if let Some(slug_want) = inner.strip_prefix(prefix) {
                        out.push(slug_want.to_string());
                    }
                    rest = &after[end + CLOSE.len()..];
                }
                None => break,
            }
        }
        out
    }

    fn strip_generated_table(doc: &str) -> String {
        const BEGIN: &str = "<!-- GENERATED:keys-reference:BEGIN -->";
        const END: &str = "<!-- GENERATED:keys-reference:END -->";
        match (doc.find(BEGIN), doc.find(END)) {
            (Some(s), Some(e)) => format!("{}{}", &doc[..s], &doc[e + END.len()..]),
            _ => doc.to_string(),
        }
    }

    #[test]
    fn render_key_tokens_substitutes_a_known_slug() {
        let out = render_key_tokens(
            "press {{key:save}} to save",
            Convention::Mac,
            Platform::Native,
        );
        assert_eq!(out, "press \u{2318}S to save");
    }

    #[test]
    fn render_key_tokens_flags_an_unknown_slug_rather_than_vanishing() {
        let out = render_key_tokens("{{key:nope}}", Convention::Mac, Platform::Native);
        assert_eq!(out, "[[unknown-key:nope]]");
    }

    #[test]
    fn render_key_tokens_leaves_an_unterminated_token_verbatim() {
        let out = render_key_tokens("abc {{key:save no close", Convention::Mac, Platform::Native);
        assert_eq!(out, "abc {{key:save no close");
    }

    #[test]
    fn render_key_tokens_substitutes_a_known_cmd_slug() {
        let out = render_key_tokens(
            "open {{cmd:widen_page}} from the palette",
            Convention::Mac,
            Platform::Native,
        );
        assert_eq!(out, "open Widen page from the palette");
        // Convention/platform-independent, unlike a chord token.
        let out_linux = render_key_tokens("{{cmd:widen_page}}", Convention::Linux, Platform::Web);
        assert_eq!(out_linux, "Widen page");
    }

    #[test]
    fn render_key_tokens_flags_an_unknown_cmd_slug_rather_than_vanishing() {
        let out = render_key_tokens("{{cmd:nope}}", Convention::Mac, Platform::Native);
        assert_eq!(out, "[[unknown-cmd:nope]]");
    }

    #[test]
    fn render_key_tokens_flags_an_unrecognized_token_kind() {
        let out = render_key_tokens("{{bogus:widen_page}}", Convention::Mac, Platform::Native);
        assert_eq!(out, "[[unknown-token:bogus:widen_page]]");
    }

    #[test]
    fn synthetic_tokens_resolve_on_both_conventions() {
        assert_eq!(
            key_token_label("command_palette", Convention::Mac, Platform::Native).as_deref(),
            Some("\u{2318}P")
        );
        assert_eq!(
            key_token_label("command_palette", Convention::Linux, Platform::Native).as_deref(),
            Some("Ctrl+P")
        );
        assert!(key_token_label("stats_hud", Convention::Mac, Platform::Native).is_some());
        assert!(key_token_label("stats_hud", Convention::Linux, Platform::Native).is_some());
    }

    /// THE DOCS-VS-CATALOG LAW (chord half): guards `render_key_tokens`'s
    /// unknown-slug fallback from ever actually shipping — every
    /// `{{key:slug}}` token used in the STARTING docs must resolve, on every
    /// convention x platform combination. This is the harvest-and-resolve law
    /// the "welcome-doc taught a retired chord" incident asked for: a retired
    /// or renamed chord slug fails `cargo test` here, before it ever reaches a
    /// reader.
    #[test]
    fn every_key_token_in_the_starting_docs_resolves() {
        for (name, doc) in [
            ("welcome.md", WELCOME),
            ("tour.md", TOUR),
            ("GUIDE.md", GUIDE),
        ] {
            let slugs = extract_token_slugs(doc);
            assert!(
                !slugs.is_empty(),
                "{name}: expected at least one {{{{key:..}}}} token"
            );
            for slug_want in slugs {
                for convention in [Convention::Mac, Convention::Linux] {
                    for platform in [Platform::Native, Platform::Web] {
                        assert!(
                            key_token_label(&slug_want, convention, platform).is_some(),
                            "{name}: unknown key token slug {slug_want:?} under {convention:?}/{platform:?}"
                        );
                    }
                }
            }
        }
    }

    /// THE DOCS-VS-CATALOG LAW (command-name half): every `{{cmd:slug}}`
    /// token used in the STARTING docs must resolve against the live catalog
    /// (`commands::action_for_name`'s own slug space — see `cmd_token_label`).
    /// A command renamed or retired since a doc cited it by name fails here.
    /// Not every doc need cite a bare command name (`tour.md` today doesn't —
    /// it only teaches chords), so an empty slug list per-doc is fine; the
    /// requirement is just that whichever slugs ARE used resolve.
    #[test]
    fn every_cmd_token_in_the_starting_docs_resolves() {
        let mut total = 0usize;
        for (name, doc) in [
            ("welcome.md", WELCOME),
            ("tour.md", TOUR),
            ("GUIDE.md", GUIDE),
        ] {
            for slug_want in extract_cmd_slugs(doc) {
                total += 1;
                assert!(
                    cmd_token_label(&slug_want).is_some(),
                    "{name}: unknown cmd token slug {slug_want:?} (renamed or retired command?)"
                );
            }
        }
        assert!(
            total > 0,
            "expected at least one {{{{cmd:..}}}} token across the starting docs"
        );
    }

    /// The four surfaces a starting doc is ever READ on. A `{{key:}}` token
    /// resolves independently on each, so every law below sweeps all four
    /// rather than the one the author happened to be running under.
    const SURFACES: &[(Convention, Platform)] = &[
        (Convention::Mac, Platform::Native),
        (Convention::Mac, Platform::Web),
        (Convention::Linux, Platform::Native),
        (Convention::Linux, Platform::Web),
    ];

    /// THE NON-EMPTY LAW (queue item 24). `key_token_label` returning
    /// `Some("")` is not a resolved token — it is a BLANK in the middle of a
    /// sentence, and [`every_key_token_in_the_starting_docs_resolves`] above
    /// says nothing about it, because `Some("")` is `Some`.
    ///
    /// The trap is real and sits in the catalog today: Insert-link is Cmd-K on
    /// Mac and has NO Linux binding at all (`C-k` stays kill-line in both
    /// keymap flavours — docs/config.md's tripwire), so a welcome document that
    /// wrote `press {{key:insert_link}} to add a link` would seed a Linux
    /// reader the sentence "press  to add a link". Cmd-W and Cmd-Q do the same
    /// thing on the WEB surface, where the browser eats them and no
    /// `WEB_ALTERNATE` exists to stand in.
    #[test]
    fn every_chord_the_starting_docs_teach_is_a_chord_that_exists() {
        for (name, doc) in [
            ("welcome.md", WELCOME),
            ("tour.md", TOUR),
            ("GUIDE.md", GUIDE),
        ] {
            for slug_want in extract_token_slugs(&strip_generated_table(doc)) {
                for (convention, platform) in SURFACES {
                    let label = key_token_label(&slug_want, *convention, *platform)
                        .unwrap_or_else(|| panic!("{name}: unknown key token {slug_want:?}"));
                    assert!(
                        !label.trim().is_empty(),
                        "{name}: {{{{key:{slug_want}}}}} renders EMPTY under \
                         {convention:?}/{platform:?} — the doc would show a blank where a \
                         chord should be. Cite the command by name ({{{{cmd:{slug_want}}}}}) \
                         instead, or teach a command that has a binding everywhere."
                    );
                }
            }
        }
    }

    /// THE DISPATCH LAW (queue item 24). A chord that resolves to a non-empty
    /// LABEL still proves nothing about whether pressing it does anything: the
    /// label owner and the keymap are separate code. So every chord the
    /// starting docs teach is pressed — through a real
    /// [`crate::keymap::KeymapState`] built for that convention and flavour,
    /// the same `resolve` a physical keypress takes — and must land on the
    /// command it is taught as.
    ///
    /// On the WEB surface the keymap is built with
    /// `commands::web_alternate_keys` folded in exactly as `App::new` folds it,
    /// so a browser-reserved chord's replacement is proved to dispatch rather
    /// than merely to print.
    ///
    /// SWEPT UNDER THE DEFAULT KEYMAP FLAVOUR ONLY, and that exclusion is a
    /// product fact rather than a convenience. `keymap = "emacs"` is an opt-in
    /// that asks awl to hand the whole bare-control cluster BACK to Emacs, so
    /// under it Linux `C-s` is search and Save has no native chord at all —
    /// deliberately, and documented in GUIDE's Keys section. The starting docs
    /// teach the keymap a reader has before they configure anything. (The
    /// separate, pre-existing fact that `resolved_native_label` takes no
    /// flavour parameter — so the palette's own native column still prints
    /// `Ctrl+S` for Save under that preset — is a surface this item did not
    /// touch and did not fix.)
    #[test]
    fn every_chord_the_starting_docs_teach_dispatches_through_the_real_keymap() {
        use crate::keymap::{KeymapState, linux_builtin_keep};

        let native_keep: Vec<String> = linux_builtin_keep().iter().map(|s| s.to_string()).collect();

        for (name, doc) in [
            ("welcome.md", WELCOME),
            ("tour.md", TOUR),
            ("GUIDE.md", GUIDE),
        ] {
            for slug_want in extract_token_slugs(&strip_generated_table(doc)) {
                let want = commands::COMMANDS
                    .iter()
                    .find(|c| commands::slug(c.name) == slug_want)
                    .map(|c| c.action.clone());
                for (convention, platform) in SURFACES {
                    let spec = key_token_spec(&slug_want, *convention, *platform)
                        .expect("resolved above");
                    let keeps: &[&[String]] = match convention {
                        Convention::Mac => &[&[]],
                        Convention::Linux => &[&native_keep],
                    };
                    for keep in keeps {
                        let overrides = crate::commands::web_alternate_keys(
                            &[],
                            *convention,
                            *platform,
                        );
                        let mut km =
                            KeymapState::with_overrides_and_convention(&overrides, *convention);
                        km.apply_linux_keep(keep);
                        let mut trace = Vec::new();
                        for token in spec.split_whitespace() {
                            let (key, mods) = crate::keyspec::parse_chord(token).unwrap_or_else(|e| {
                                panic!("{name}: {slug_want:?} teaches unparsable chord {spec:?}: {e}")
                            });
                            trace.push(km.resolve(&key, &mods));
                        }
                        let landed = trace.last().cloned().unwrap_or_else(|| {
                            panic!(
                                "{name}: {{{{key:{slug_want}}}}} resolves to NO chord at all \
                                 under {convention:?}/{platform:?} — there is nothing to press"
                            )
                        });
                        match &want {
                            Some(action) => assert_eq!(
                                &landed, action,
                                "{name}: {{{{key:{slug_want}}}}} is taught as {spec:?} under \
                                 {convention:?}/{platform:?} (keep={keep:?}) but that chord \
                                 resolves to {landed:?}, not the command it names"
                            ),
                            // The two SYNTHETIC chords have no catalog row to
                            // compare against; what must hold is that they are
                            // not inert.
                            None => assert_ne!(
                                landed,
                                crate::keymap::Action::Ignore,
                                "{name}: synthetic {{{{key:{slug_want}}}}} = {spec:?} resolves to \
                                 nothing under {convention:?}/{platform:?}"
                            ),
                        }
                    }
                }
            }
        }
    }

    /// THE AVAILABILITY LAW (queue item 24). `samples/welcome.md` is SEEDED —
    /// one set of bytes written per reader, on whichever platform they are on —
    /// so every command it names must actually exist for that reader. A
    /// native-only command cited in the welcome would be seeded into a browser
    /// tab where the palette does not list it, and a web-only one into a
    /// desktop where it does not either.
    ///
    /// `GUIDE.md` and `tour.md` are deliberately NOT swept: GUIDE is one
    /// document that discusses both platforms explicitly (its "Hidden on web"
    /// paragraph cites native-only commands ON PURPOSE, by name, as its
    /// subject), so a blanket ban there would forbid the page from doing its
    /// job.
    #[test]
    fn every_command_the_welcome_names_exists_on_both_platforms() {
        for slug_want in extract_cmd_slugs(WELCOME) {
            let command = commands::COMMANDS
                .iter()
                .find(|c| commands::slug(c.name) == slug_want)
                .unwrap_or_else(|| panic!("welcome.md: unknown cmd token {slug_want:?}"));
            for platform in [Platform::Native, Platform::Web] {
                assert!(
                    command.available_on(platform),
                    "welcome.md names {:?}, which is not available on {platform:?} — the \
                     seeded welcome would teach a reader a command their palette does not \
                     list",
                    command.name
                );
            }
        }
        for slug_want in extract_token_slugs(WELCOME) {
            if let Some(command) = commands::COMMANDS
                .iter()
                .find(|c| commands::slug(c.name) == slug_want)
            {
                for platform in [Platform::Native, Platform::Web] {
                    assert!(
                        command.available_on(platform),
                        "welcome.md teaches {:?}'s chord, but the command is not available \
                         on {platform:?}",
                        command.name
                    );
                }
            }
        }
    }

    /// THE GREP-LAW: no literal chord glyph survives in the STARTING docs'
    /// PROSE — every specific chord mention must be a `{{key:slug}}` token, so
    /// it can never silently drift from what actually fires. Two literal forms
    /// are banned: the mac ⌘ glyph, and the generated table's own `Ctrl+<key>`
    /// word form. Exempted: the ONE generated keys-reference fence in
    /// GUIDE.md (dual-column BY DESIGN — see `commands::generate_keys_
    /// reference_markdown`'s doc) and a short, individually curated allowlist
    /// of lines that talk ABOUT the two-convention split itself (both
    /// conventions named explicitly in the SAME breath, so there is no single
    /// per-viewer truth a token could substitute without erasing the other).
    #[test]
    fn no_literal_chord_glyphs_survive_outside_tokens_and_the_generated_table() {
        // Curated, honest allowlist: lines that name BOTH conventions in the
        // same breath (never claim a single truth a token could replace).
        const ALLOWED_MAC_GLYPH_SUBSTRINGS: &[&str] = &[
            "**slot 1 is native** (\u{2318} on",
            "**The hold-\u{2318} peek.** Hold the arming modifier alone for a beat (\u{2318} on",
        ];
        // The Omarchy/Hyprland recipe: `Ctrl+C/X/V` here names the LITERAL
        // signal the compositor forwards to every app (a hardware/OS fact,
        // not an awl chord label) — explicitly out of scope, per the round's
        // own "curate honestly" instruction.
        const ALLOWED_CTRL_WORD_SUBSTRINGS: &[&str] = &["as Ctrl+C/X/V for the system clipboard"];
        for (name, doc) in [
            ("welcome.md", WELCOME),
            ("tour.md", TOUR),
            ("GUIDE.md", GUIDE),
        ] {
            let body = strip_generated_table(doc);
            for line in body.lines() {
                if line.contains('\u{2318}')
                    && !ALLOWED_MAC_GLYPH_SUBSTRINGS
                        .iter()
                        .any(|a| line.contains(a))
                {
                    panic!("{name}: literal \u{2318} glyph outside a token/allowlist: {line:?}");
                }
                if line.contains("Ctrl+")
                    && !ALLOWED_CTRL_WORD_SUBSTRINGS
                        .iter()
                        .any(|a| line.contains(a))
                {
                    panic!(
                        "{name}: literal Ctrl+ word-form outside the generated table/allowlist: {line:?}"
                    );
                }
            }
        }
    }
}

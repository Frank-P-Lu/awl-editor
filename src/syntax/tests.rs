/// The languages that do NOT run the shared definition walk, each with the reason
/// its shape is different. This roster is the deliberate escape hatch: a new
/// `syntax/<lang>.rs` that writes its own loop without appearing here trips
/// [`no_lexer_module_writes_its_own_definition_walk`], the law that reads
/// it.
#[cfg(test)]
const BESPOKE: &[(&str, &str)] = &[
    (
        "ruby",
        "heredoc bodies pend across lines; `?c`/`%w[]` disambiguate on the previous byte",
    ),
    (
        "bash",
        "`$`-expansion, `<<`-heredocs, and single-quote-is-raw have no C-family analogue",
    ),
    ("html", "tag/attribute grammar, not a token stream"),
    (
        "css",
        "selector/property grammar; `-` is an identifier byte",
    ),
    ("json", "a closed grammar: keys vs values decide the role"),
    ("yaml", "indentation- and key-driven, with block scalars"),
    ("toml", "key/value/table grammar with date-time literals"),
    (
        "sql",
        "case-insensitive multi-word introducers with skip-words; `\"…\"` is \
         an identifier, not a string",
    ),
];

use super::*;

/// LAW: the definition walk has exactly ONE owner. Every `syntax/<lang>.rs`
/// either drives [`scanner::scan`] through a `LangSpec` or is declared in
/// [`BESPOKE`] with the reason its shape differs. The sweep is over the
/// DIRECTORY rather than a roster, so a new lexer file that copy-pastes a loop
/// goes red before it is even wired into [`Lang`]; the roster is then tied
/// back to [`lexer`]'s wildcard-free match, which a new `Lang` variant cannot
/// compile past without choosing a side.
#[test]
fn no_lexer_module_writes_its_own_definition_walk() {
    // The state variable every copy of the walk carries.
    const WALK: &str = "expect_def";
    // `mod.rs` holds the dispatch and `ident_role`; `scanner.rs` IS the walk.
    const NOT_A_LANGUAGE: &[&str] = &["mod", "scanner", "tests"];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/syntax");

    let owner = std::fs::read_to_string(dir.join("scanner.rs")).unwrap();
    assert!(
        owner.contains(WALK),
        "{WALK:?} no longer names the walk — this law would sweep nothing"
    );

    let mut modules: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .filter(|s| !NOT_A_LANGUAGE.contains(&s.as_str()))
        .collect();
    modules.sort();
    assert!(
        modules.len() >= Lang::ALL.len(),
        "{modules:?} missed lexer files"
    );

    let declared: Vec<&str> = BESPOKE.iter().map(|(m, _)| *m).collect();
    for stem in &modules {
        let src = std::fs::read_to_string(dir.join(format!("{stem}.rs"))).unwrap();
        assert!(
            !src.contains(WALK) || declared.contains(&stem.as_str()),
            "{stem}.rs carries its own {WALK:?} walk: run the shared scanner \
             through a LangSpec instead, or declare it in syntax::BESPOKE with \
             the reason its shape differs"
        );
    }
    for (m, why) in BESPOKE {
        assert!(
            modules.contains(&m.to_string()),
            "BESPOKE names {m}, which is not a lexer module"
        );
        assert!(
            !why.trim().is_empty(),
            "BESPOKE entry {m} carries no reason"
        );
    }

    for lang in Lang::ALL {
        let bespoke = declared.contains(&lang.name());
        match lexer(lang) {
            Lexer::Table(_) => assert!(
                !bespoke,
                "{} is table-driven but declared BESPOKE",
                lang.name()
            ),
            Lexer::Own(_) => assert!(
                bespoke,
                "{} dispatches to its own lexer but is not declared in BESPOKE",
                lang.name()
            ),
        }
    }

    // `lexer`'s match is exhaustive and wildcard-free, so its arm count IS the
    // variant count, matching the roster generated from the enum declaration.
    let src = std::fs::read_to_string(dir.join("mod.rs")).unwrap();
    let body = src
        .split_once("fn lexer(lang: Lang) -> Lexer {")
        .expect("the dispatch moved")
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    assert_eq!(
        body.matches("Lang::").count(),
        Lang::VARIANT_COUNT,
        "the wildcard-free dispatch must cover the generated language roster"
    );
    assert!(!Lang::ALL.is_empty(), "the language sweep is non-vacuous");
}

#[test]
fn extension_detection_covers_all_languages() {
    // A flat extension -> language table, one row per recognized extension,
    // so every case is the same assertion rather than a fresh branch per
    // language (several distinct `for e in [...]` loops interleaved with
    // bare single-extension asserts scored high on branching alone).
    const CASES: &[(&str, Lang)] = &[
        ("rs", Lang::Rust),
        ("py", Lang::Python),
        ("js", Lang::JavaScript),
        ("mjs", Lang::JavaScript),
        ("cjs", Lang::JavaScript),
        ("jsx", Lang::JavaScript),
        ("ts", Lang::TypeScript),
        ("tsx", Lang::TypeScript),
        ("go", Lang::Go),
        ("c", Lang::C),
        ("h", Lang::C),
        ("cpp", Lang::Cpp),
        ("cc", Lang::Cpp),
        ("cxx", Lang::Cpp),
        ("hpp", Lang::Cpp),
        ("hh", Lang::Cpp),
        ("java", Lang::Java),
        ("cs", Lang::CSharp),
        ("rb", Lang::Ruby),
        ("php", Lang::Php),
        ("swift", Lang::Swift),
        ("kt", Lang::Kotlin),
        ("kts", Lang::Kotlin),
        ("sh", Lang::Bash),
        ("bash", Lang::Bash),
        ("zsh", Lang::Bash),
        ("html", Lang::Html),
        ("htm", Lang::Html),
        ("css", Lang::Css),
        ("json", Lang::Json),
        ("yaml", Lang::Yaml),
        ("yml", Lang::Yaml),
        ("toml", Lang::Toml),
        ("sql", Lang::Sql),
    ];
    for (ext, lang) in CASES {
        assert_eq!(Lang::from_extension(ext), Some(*lang), "{ext}");
    }
}

#[test]
fn extension_detection_is_case_insensitive() {
    assert_eq!(Lang::from_extension("RS"), Some(Lang::Rust));
    assert_eq!(Lang::from_extension("Py"), Some(Lang::Python));
}

#[test]
fn excluded_and_unknown_extensions_are_none() {
    for e in ["env", "md", "markdown", "txt", "", "log", "bin", "lock"] {
        assert_eq!(Lang::from_extension(e), None, "{e:?} must not highlight");
    }
}

#[test]
fn from_path_uses_extension() {
    use std::path::Path;
    assert_eq!(Lang::from_path(Path::new("/a/b/main.rs")), Some(Lang::Rust));
    assert_eq!(Lang::from_path(Path::new("notes.md")), None);
    assert_eq!(Lang::from_path(Path::new("README")), None);
    assert_eq!(Lang::from_path(Path::new(".env")), None);
}

#[test]
fn tags_are_stable() {
    assert_eq!(SynKind::Comment.tag(), "comment");
    assert_eq!(SynKind::CommentCode.tag(), "comment_code");
    assert_eq!(SynKind::Str.tag(), "string");
    assert_eq!(SynKind::Constant.tag(), "constant");
    assert_eq!(SynKind::Definition.tag(), "definition");
}

#[test]
fn comment_body_strips_markers() {
    assert_eq!(comment_body("// hi there"), "hi there");
    assert_eq!(comment_body("/// doc prose"), "doc prose");
    assert_eq!(comment_body("//! inner doc"), "inner doc");
    assert_eq!(comment_body("/* block */"), "block");
    assert_eq!(comment_body("/** doc block */"), "doc block");
    assert_eq!(comment_body("# python note"), "python note");
    assert_eq!(comment_body("-- sql note"), "sql note");
    assert_eq!(comment_body("<!-- html note -->"), "html note");
    assert_eq!(comment_body(" * continuation line"), "continuation line");
    assert_eq!(comment_body("   //   padded   "), "padded");
    assert_eq!(comment_body("//"), "");
    assert_eq!(comment_body(""), "");
}

#[test]
fn looks_like_code_two_tier_table() {
    for prose in [
        "// TODO: fix the wrap",
        "// return early here",       // keyword alone, no symbol
        "// use two spaces here",     // keyword alone, no symbol
        "// If you set x, it breaks", // capitalized prose never trips rule 2
        "// A calm, quiet note.",
        "// don't check: prose punctuation!",
        "//",    // empty body
        "// ok", // short body
        "# reads the active theme",
        "-- migration notes below",
        "<!-- page header -->",
    ] {
        assert!(
            !looks_like_code(comment_body(prose)),
            "{prose:?} must classify as PROSE"
        );
    }
    for code in [
        "// let x = foo(bar);",   // trailing ;
        "// x += 1;",             // trailing ;
        "# print(x)",             // keyword + call parens
        "-- select * from users", // keyword + the * projection
        "// return None;",        // trailing ;
        "// if (a && b) {",       // trailing {
        "// }",                   // trailing }
        "// foo(a, b) == bar[i]", // symbol density
    ] {
        assert!(
            looks_like_code(comment_body(code)),
            "{code:?} must classify as CODE"
        );
    }
}

#[test]
fn multiline_block_comment_prose_wins_on_mix() {
    assert!(looks_like_code(comment_body(
        "/* let a = 1;\n * let b = 2;\n */"
    )));
    assert!(!looks_like_code(comment_body(
        "/* This sets the default.\n * let a = 1;\n */"
    )));
    assert!(!looks_like_code(comment_body(
        "/* A quiet block of prose\n * that keeps explaining. */"
    )));
}

#[test]
fn spans_post_pass_splits_comment_tiers() {
    let rs = "// TODO: fix the wrap\n// let x = foo(bar);\nfn main() {}\n";
    let s = spans(Lang::Rust, rs);
    assert!(
        s.iter()
            .any(|(r, k)| *k == SynKind::Comment && rs[r.clone()].contains("TODO")),
        "prose comment stays Comment: {s:?}"
    );
    assert!(
        s.iter()
            .any(|(r, k)| *k == SynKind::CommentCode && rs[r.clone()].contains("let x")),
        "commented-out statement becomes CommentCode: {s:?}"
    );
    let py = "# reads the config\n# print(x)\n";
    let s = spans(Lang::Python, py);
    assert!(
        s.iter()
            .any(|(r, k)| *k == SynKind::Comment && py[r.clone()].contains("config"))
    );
    assert!(
        s.iter()
            .any(|(r, k)| *k == SynKind::CommentCode && py[r.clone()].contains("print"))
    );
}

#[test]
fn dispatch_routes_to_implemented_lexers() {
    assert!(!spans(Lang::Rust, "// hi\n").is_empty());
    assert!(!spans(Lang::Python, "# hi\n").is_empty());
    assert!(!spans(Lang::Go, "// hi\n").is_empty());
    assert!(!spans(Lang::Sql, "-- hi\n").is_empty());
}

#[test]
fn shared_is_ident_is_the_ascii_common_case() {
    assert!(is_ident_start(b'_') && is_ident_start(b'a') && is_ident_start(b'Z'));
    assert!(!is_ident_start(b'0') && !is_ident_start(b'$') && !is_ident_start(b'-'));
    assert!(is_ident_continue(b'_') && is_ident_continue(b'9') && is_ident_continue(b'x'));
    assert!(!is_ident_continue(b'$') && !is_ident_continue(b' '));
}

#[test]
fn shared_scan_line_comment_runs_to_newline_or_eof() {
    let t = b"// hi\nx";
    assert_eq!(scan_line_comment(t, 0), 5);
    let e = b"-- end";
    assert_eq!(scan_line_comment(e, 0), e.len());
}

#[test]
fn shared_scan_block_comment_nesting_flag() {
    let flat = b"/* a /* b */ c */ x";
    assert_eq!(scan_block_comment(flat, 0, false), 12);
    assert_eq!(scan_block_comment(flat, 0, true), 17);
    let un = b"/* open";
    assert_eq!(scan_block_comment(un, 0, false), un.len());
    assert_eq!(scan_block_comment(un, 0, true), un.len());
}

#[test]
fn shared_scan_quoted_handles_escapes_quote_and_newline() {
    let t = br#""ab"x"#;
    assert_eq!(scan_quoted(t, 0, b'"', false), 4);
    let e = br#""a\"b""#;
    assert_eq!(scan_quoted(e, 0, b'"', false), e.len());
    let nl = b"\"ab\ncd";
    assert_eq!(scan_quoted(nl, 0, b'"', true), 3);
    assert_eq!(scan_quoted(nl, 0, b'"', false), nl.len());
    let sq = b"'q' ";
    assert_eq!(scan_quoted(sq, 0, b'\'', false), 3);
}

#[test]
fn shared_scan_number_radix_fraction_and_boundaries() {
    let o = || NumOpts {
        radix: b"xXoObB",
        radix_extra: b"",
        dot_dot_stops: true,
    };
    let hex = b"0xFF_u8;";
    assert_eq!(scan_number(hex, 0, o(), is_ident_start), 7);
    let f = b"3.14 ";
    assert_eq!(scan_number(f, 0, o(), is_ident_start), 4);
    let r = b"0..5";
    assert_eq!(scan_number(r, 0, o(), is_ident_start), 1);
    let m = b"1.foo";
    assert_eq!(scan_number(m, 0, o(), is_ident_start), 1);
    let no = NumOpts {
        radix: b"xXbB",
        radix_extra: b"",
        dot_dot_stops: false,
    };
    assert_eq!(scan_number(b"1.5", 0, no, is_ident_start), 3);
}

#[test]
fn shared_ident_role_precedence_and_arming() {
    const DEF: &[&str] = &["fn", "struct"];
    const CONST: &[&str] = &["true", "None"];
    let mut e = false;
    assert_eq!(ident_role("fn", DEF, CONST, &mut e), None);
    assert!(e, "an introducer arms expect_def");
    assert_eq!(
        ident_role("main", DEF, CONST, &mut e),
        Some(SynKind::Definition)
    );
    assert!(!e, "emitting a name clears expect_def");
    assert_eq!(
        ident_role("true", DEF, CONST, &mut e),
        Some(SynKind::Constant)
    );
    assert_eq!(ident_role("foo", DEF, CONST, &mut e), None);
    let mut e2 = true;
    assert_eq!(
        ident_role("true", DEF, CONST, &mut e2),
        Some(SynKind::Definition)
    );
    assert!(!e2);
}

#[test]
fn from_name_maps_fence_languages_and_aliases() {
    assert_eq!(Lang::from_name("rust"), Some(Lang::Rust));
    assert_eq!(Lang::from_name("python"), Some(Lang::Python));
    assert_eq!(Lang::from_name("bash"), Some(Lang::Bash));
    assert_eq!(Lang::from_name("javascript"), Some(Lang::JavaScript));
    assert_eq!(Lang::from_name("Rust"), Some(Lang::Rust));
    assert_eq!(Lang::from_name("golang"), Some(Lang::Go));
    assert_eq!(Lang::from_name("c++"), Some(Lang::Cpp));
    assert_eq!(Lang::from_name("c#"), Some(Lang::CSharp));
    assert_eq!(Lang::from_name("shell"), Some(Lang::Bash));
    assert_eq!(Lang::from_name("rs"), Some(Lang::Rust));
    assert_eq!(Lang::from_name("py"), Some(Lang::Python));
    assert_eq!(Lang::from_name("sh"), Some(Lang::Bash));
    assert_eq!(Lang::from_name("zsh"), Some(Lang::Bash));
    assert_eq!(Lang::from_name("yml"), Some(Lang::Yaml));
    assert_eq!(Lang::from_name("plaintext"), None);
    assert_eq!(Lang::from_name("text"), None);
    assert_eq!(Lang::from_name(""), None);
}

#[test]
fn from_info_takes_the_first_token() {
    assert_eq!(Lang::from_info("rust"), Some(Lang::Rust));
    assert_eq!(Lang::from_info("rust ignore"), Some(Lang::Rust));
    assert_eq!(Lang::from_info("rust,ignore"), Some(Lang::Rust));
    assert_eq!(Lang::from_info("sh title=demo"), Some(Lang::Bash));
    assert_eq!(Lang::from_info("   python  "), Some(Lang::Python));
    assert_eq!(Lang::from_info(""), None);
    assert_eq!(Lang::from_info("   "), None);
    assert_eq!(Lang::from_info("unknownlang"), None);
}

#[test]
fn lang_names_are_stable_and_lowercase() {
    assert_eq!(Lang::Rust.name(), "rust");
    assert_eq!(Lang::Cpp.name(), "cpp");
    assert_eq!(Lang::CSharp.name(), "csharp");
    for l in Lang::ALL {
        let n = l.name();
        assert!(
            !n.is_empty() && n == n.to_ascii_lowercase(),
            "{n:?} must be lowercase"
        );
    }
}

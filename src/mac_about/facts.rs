//! The About window's TEXT — every word it can say about itself, and the one
//! rule governing all of them: **a fact is shown only when it is known.**
//!
//! Nothing here is AppKit; it is plain data and one composition function, so
//! the entire content of the window is unit-testable at the purest seam. The
//! window builder in the parent module does no string work of its own — it
//! renders exactly what [`fact_lines`] returns, in order.
//!
//! **Where each fact comes from:**
//!
//! | line | source | when it is absent |
//! |---|---|---|
//! | Version | `CFBundleShortVersionString` (packaged) → `CARGO_PKG_VERSION` | never |
//! | Build | `CFBundleVersion` (packaged only) | unpackaged, or identical to the version |
//! | Commit | `AWL_GIT_COMMIT`, stamped by `build.rs` from `git rev-parse` | no git at build time |
//!
//! There is deliberately NO placeholder anywhere in this file: an "unknown" or
//! "—" sitting in a facts block reads as a fact. An unknown fact loses its
//! line, and the window's height shrinks by exactly that line
//! ([`super::layout`] takes the count).

/// The product name, capitalised. The App menu ("About Awl"), the bundle's
/// `CFBundleName` and this title all say `Awl`; only the CLI/executable is the
/// lowercase `awl`. Deliberately NOT read from the bundle: the window must
/// name awl even when running unpackaged from `cargo run`.
pub const NAME: &str = "Awl";

/// The one product line under the title. Verbatim from `README.md`'s opening
/// sentence — the project's own words, pinned by the law of the same name in
/// this file rather than re-written here as marketing copy.
pub const TAGLINE: &str = "A calm, opinionated plain-text editor for prose and light code.";

/// The credit line: who holds the copyright, and under which license. Both
/// halves are pinned to their real sources by
/// [`tests::attribution_matches_notice_and_cargo_license`] — `NOTICE` names the
/// copyright holder, `Cargo.toml` names the license. This is NOT a copyright
/// notice (no year, no "©"); the full grant lives in `LICENSE`/`NOTICE`, which
/// `scripts/package-macos.sh` copies into the bundle's `Resources/`.
pub const ATTRIBUTION: &str = "Frank Lu · GPL-3.0";

/// The "Docs" button's fixed destination — the published guide on awl's own
/// site. Opened ONLY by an explicit click (see the parent module); nothing in
/// this window fetches anything, ever.
pub const DOCS_URL: &str = "https://awl-editor.fly.dev/guide.html";

/// The "GitHub" button's fixed destination — the source repository.
pub const GITHUB_URL: &str = "https://github.com/Frank-P-Lu/awl-next";

/// What the running process could learn about its own `.app` bundle. Both
/// fields are `None` outside a bundle (a bare `cargo run`, a CLI launch from
/// `target/release/awl`), which is exactly why they are `Option`: the absence
/// is a real state, not an error to paper over.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BundleFacts {
    /// `CFBundleShortVersionString` — the human version ("0.1.0").
    pub short_version: Option<String>,
    /// `CFBundleVersion` — the build identifier. `scripts/package-macos.sh`
    /// currently writes the SAME string as the short version, which is why
    /// [`fact_lines`] suppresses a Build line that merely repeats the Version
    /// one; a future packaging change that gives it a distinct value gets its
    /// own line for free, with no code change here.
    pub build: Option<String>,
}

/// The commit this binary was built from, or `None` when `build.rs` could not
/// ask git (see its module doc). The ONE reader of the build-time stamp.
pub fn commit() -> Option<&'static str> {
    option_env!("AWL_GIT_COMMIT")
}

/// The version this binary reports: the bundle's if it is packaged, otherwise
/// the crate version it was compiled with. Both are real; neither is invented.
pub fn version(bundle: &BundleFacts) -> String {
    bundle
        .short_version
        .clone()
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

/// The provenance block, top to bottom — one line per KNOWN fact, and nothing
/// else. Order is fixed (version, build, commit): most to least human.
pub fn fact_lines(bundle: &BundleFacts, commit: Option<&str>) -> Vec<String> {
    let version = version(bundle);
    let mut lines = vec![format!("Version {version}")];
    // A Build line only when the bundle carries one AND it says something the
    // Version line did not.
    if let Some(build) = bundle.build.as_deref()
        && build != version
    {
        lines.push(format!("Build {build}"));
    }
    if let Some(commit) = commit {
        lines.push(format!("Commit {commit}"));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packaged(short: &str, build: &str) -> BundleFacts {
        BundleFacts {
            short_version: Some(short.to_string()),
            build: Some(build.to_string()),
        }
    }

    #[test]
    fn unpackaged_falls_back_to_the_compiled_crate_version() {
        let lines = fact_lines(&BundleFacts::default(), None);
        assert_eq!(
            lines,
            vec![format!("Version {}", env!("CARGO_PKG_VERSION"))],
            "outside a bundle the ONLY known facts are the crate version (and \
             a commit, when git answered) — never an invented build number"
        );
    }

    #[test]
    fn the_bundles_version_wins_over_the_crate_version() {
        // A packaged build reports what the bundle says, because that is what
        // the user's copy actually IS — the compile-time constant can lag it.
        let lines = fact_lines(&packaged("9.9.9", "9.9.9"), None);
        assert_eq!(lines[0], "Version 9.9.9");
        assert!(
            !lines
                .iter()
                .any(|l| l.contains(env!("CARGO_PKG_VERSION"))
                    && env!("CARGO_PKG_VERSION") != "9.9.9"),
            "the crate version must not leak in beside the bundle's: {lines:?}"
        );
    }

    #[test]
    fn a_build_line_appears_only_when_it_says_something_new() {
        assert_eq!(
            fact_lines(&packaged("0.1.0", "0.1.0"), None),
            vec!["Version 0.1.0".to_string()],
            "a Build line that merely repeats the Version line is noise"
        );
        assert_eq!(
            fact_lines(&packaged("0.1.0", "417"), None),
            vec!["Version 0.1.0".to_string(), "Build 417".to_string()],
            "a DISTINCT CFBundleVersion is a real fact and earns its line"
        );
    }

    #[test]
    fn the_commit_line_is_present_exactly_when_git_answered() {
        assert_eq!(
            fact_lines(&BundleFacts::default(), None).len(),
            1,
            "no git at build time ⇒ no Commit line at all"
        );
        let lines = fact_lines(&BundleFacts::default(), Some("0123456789ab"));
        assert_eq!(lines.last().unwrap(), "Commit 0123456789ab");
    }

    /// The anti-fabrication law. Sweeps the whole knowledge lattice — every
    /// combination of (bundle version?, bundle build?, commit?) — and asserts
    /// that no cell of it ever produces a line that merely GESTURES at a fact.
    /// This is the axis the author of a "just show something" fallback would
    /// not think to test.
    #[test]
    fn no_cell_of_the_knowledge_lattice_ever_prints_a_placeholder() {
        const PLACEHOLDERS: &[&str] = &[
            "unknown",
            "unavailable",
            "n/a",
            "none",
            "null",
            "?",
            "—",
            "-",
            "tbd",
            "dev",
            "local",
            "0.0.0",
        ];
        for short in [None, Some("0.1.0")] {
            for build in [None, Some("0.1.0"), Some("417")] {
                for commit in [None, Some("0123456789ab")] {
                    let bundle = BundleFacts {
                        short_version: short.map(str::to_string),
                        build: build.map(str::to_string),
                    };
                    let lines = fact_lines(&bundle, commit);
                    for line in &lines {
                        let value = line
                            .split_once(' ')
                            .map(|(_, v)| v)
                            .unwrap_or_default()
                            .trim();
                        assert!(
                            !value.is_empty(),
                            "fact line {line:?} has a label and no value \
                             ({short:?}/{build:?}/{commit:?})"
                        );
                        for placeholder in PLACEHOLDERS {
                            assert!(
                                !value.eq_ignore_ascii_case(placeholder),
                                "fact line {line:?} states a PLACEHOLDER as if it \
                                 were a fact ({short:?}/{build:?}/{commit:?}); an \
                                 unknown fact loses its line instead"
                            );
                        }
                    }
                    // Every line is "Label value" — never a bare value, never a
                    // bare label.
                    for line in &lines {
                        assert!(
                            line.split_whitespace().count() >= 2,
                            "fact line {line:?} is not a labelled fact"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn tagline_is_the_readmes_own_sentence() {
        let readme = std::fs::read_to_string("README.md").expect("README.md at the repo root");
        assert!(
            readme.contains(TAGLINE),
            "the About window's product line must be the project's OWN sentence, \
             verbatim from README.md — not marketing written for this window. \
             Missing: {TAGLINE:?}"
        );
    }

    #[test]
    fn attribution_matches_notice_and_cargo_license() {
        let notice = std::fs::read_to_string("NOTICE").expect("NOTICE at the repo root");
        let cargo = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml at the repo root");
        let (holder, license) = ATTRIBUTION
            .split_once(" · ")
            .expect("the credit line is '<holder> · <license>'");
        assert!(
            notice.contains(&format!("Copyright (C) 2026 {holder}")),
            "the About window names a copyright holder NOTICE does not: {holder:?}"
        );
        assert!(
            cargo.contains(&format!("license = \"{license}-only\"")),
            "the About window states a license Cargo.toml does not: {license:?}"
        );
        assert!(
            !ATTRIBUTION.contains('©') && !ATTRIBUTION.contains("Copyright"),
            "the credit line is a credit, not a copyright notice — the notice \
             lives in LICENSE/NOTICE, which the bundle ships in Resources/"
        );
    }

    /// Both buttons point at FIXED, plain destinations. No query string, no
    /// version parameter, no identifier — the window is not allowed to say
    /// anything about this machine, even by way of a link.
    #[test]
    fn link_destinations_are_fixed_plain_https_urls() {
        for url in [DOCS_URL, GITHUB_URL] {
            assert!(
                url.starts_with("https://"),
                "{url} must be https — a plaintext link is not a destination we ship"
            );
            assert!(
                !url.contains('?') && !url.contains('#') && !url.contains('&'),
                "{url} carries a query/fragment; a click must transmit NOTHING \
                 about this build or this machine"
            );
            assert!(
                !url.contains(env!("CARGO_PKG_VERSION")),
                "{url} embeds the running version — that is telemetry by another name"
            );
        }
    }

    /// The Docs link and the update-check URL are the same site; one origin,
    /// two paths. If either drifts (a domain move, a typo), this goes red
    /// instead of shipping a dead About button.
    #[test]
    fn docs_url_shares_the_site_origin_with_the_update_check() {
        let origin = |url: &str| {
            let rest = url.strip_prefix("https://").expect("https url");
            format!("https://{}", rest.split('/').next().unwrap_or_default())
        };
        assert_eq!(
            origin(DOCS_URL),
            origin(crate::updates::CHECK_BASE_URL),
            "the About window's Docs link and the update check must name ONE site"
        );
    }

    /// The Docs link resolves to a page this repository actually publishes —
    /// the same guarantee `scripts/site-links.sh` gives the website's own
    /// links, applied to the one link the APP ships. No network: the path
    /// suffix is checked against the working tree.
    #[test]
    fn the_docs_page_exists_in_this_repository() {
        let page = DOCS_URL
            .rsplit('/')
            .next()
            .expect("a trailing path segment");
        let path = std::path::Path::new("site").join(page);
        assert!(
            path.exists(),
            "the About window's Docs button points at {DOCS_URL}, but this repo \
             publishes no {path:?} — the button would 404 for a real reader"
        );
    }

    /// The GitHub button names THIS repository, the one `site/` already links.
    #[test]
    fn the_github_link_is_this_repository() {
        let index = std::fs::read_to_string("site/index.html").expect("site/index.html");
        assert!(
            index.contains(GITHUB_URL),
            "the About window's GitHub link ({GITHUB_URL}) is not the repository \
             the website itself links to"
        );
    }
}

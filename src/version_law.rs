//! src/version_law.rs — THE VERSION LAW: `Cargo.toml`'s `package.version` is
//! the ONE owner of awl's own version string. Every runtime surface (crash
//! reports, the About window, Check for Updates, this bench suite's report)
//! already reads the compiled `CARGO_PKG_VERSION` constant directly, never a
//! copy — so those need no law. What DOES need one is the handful of places
//! that cannot read a compile-time constant because they are read by a
//! HUMAN before any binary exists: `README.md`'s Download section and
//! `site/index.html`'s download snippet both restate the versioned archive
//! filename as literal prose, and `Cargo.lock`'s own `awl` package entry is
//! a second, hand-adjacent copy of the same field that `cargo build` keeps
//! in sync only when someone actually runs it.
//!
//! A bare `grep -r 0.1.0` cannot be that law — the repo carries several
//! LEGITIMATE `0.1.0`s that have nothing to do with awl's own version:
//! dependency crate versions in `Cargo.lock`, `scripts/test-sccache.sh`'s
//! disposable fixture crate, `scripts/test-native-gate.sh`'s mocked cargo
//! output line (testing string matching, not real version equality), and
//! the literal placeholder strings `updates.rs`/`mac_about/facts.rs` use to
//! exercise URL-encoding and layout logic. Sweeping all of those into "no
//! 0.1.0 anywhere" would make the law fail on itself. Instead this pins the
//! ACTUAL version-bearing surfaces to the real, compiled version — which is
//! a stronger check than a placeholder-string ban: it fails on ANY drift,
//! not just a reversion to exactly "0.1.0".
#![cfg(test)]

/// `Cargo.toml`'s `package.version`, parsed from the checked-in file text
/// (`include_str!`, a compile-time embed — not a runtime fs read, so this
/// stays hermetic under the harness's no-filesystem-surprises rule). Parsed
/// independently of `env!("CARGO_PKG_VERSION")` rather than trusted to agree
/// with it by construction, so a hand-edit that breaks the `version = "..."`
/// line's shape is itself a finding, not silently invisible.
fn cargo_toml_version() -> &'static str {
    let text = include_str!("../Cargo.toml");
    for line in text.lines() {
        let line = line.trim();
        if line == "[dependencies]" {
            break;
        }
        if let Some(rest) = line.strip_prefix("version = \"") {
            return rest.split('"').next().expect("closing quote");
        }
    }
    panic!("Cargo.toml has no `version = \"...\"` line before [dependencies]");
}

/// `Cargo.lock`'s own `[[package]] name = "awl"` entry's `version` field —
/// the lockfile's copy of the same fact, refreshed only when `cargo build`/
/// `cargo check` next runs against the bumped `Cargo.toml`.
fn cargo_lock_awl_version() -> &'static str {
    let text = include_str!("../Cargo.lock");
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"awl\"" {
            let version_line = lines
                .next()
                .expect("Cargo.lock: `name = \"awl\"` has no following line");
            let rest = version_line
                .trim()
                .strip_prefix("version = \"")
                .unwrap_or_else(|| {
                    panic!(
                        "Cargo.lock: the line after `name = \"awl\"` is not \
                         a version field: {version_line:?}"
                    )
                });
            return rest.split('"').next().expect("closing quote");
        }
    }
    panic!("Cargo.lock has no `[[package]] name = \"awl\"` entry");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_toml_version_matches_the_compiled_crate_version() {
        assert_eq!(
            cargo_toml_version(),
            env!("CARGO_PKG_VERSION"),
            "Cargo.toml's package.version and the compiled CARGO_PKG_VERSION disagree"
        );
    }

    #[test]
    fn the_package_version_is_not_the_stale_pre_beta_placeholder() {
        assert_ne!(
            cargo_toml_version(),
            "0.1.0",
            "Cargo.toml still carries the pre-release placeholder 0.1.0 — \
             bump it to the real tag version before a public tag"
        );
    }

    #[test]
    fn cargo_lock_awl_entry_matches_cargo_toml() {
        assert_eq!(
            cargo_lock_awl_version(),
            cargo_toml_version(),
            "Cargo.lock's own `awl` package entry has drifted from Cargo.toml's \
             package.version — run `cargo build` (or `cargo check`) to refresh it"
        );
    }

    /// `README.md`'s Download table + `tar xzf` snippet are read by a human
    /// before the binary exists, so they cannot derive the version live —
    /// they carry it as literal text. This pins that text to the ACTUAL
    /// package version, in the exact shape `scripts/package-linux.sh`
    /// produces (`awl-<version>-linux-x86_64.tar.gz`), so a bump anywhere
    /// else is caught here instead of shipping a doc that names a filename
    /// that does not exist.
    #[test]
    fn readme_download_snippet_names_the_real_artifact() {
        let readme = include_str!("../README.md");
        let expected = format!("awl-{}-linux-x86_64.tar.gz", cargo_toml_version());
        assert!(
            readme.contains(&expected),
            "README.md's Download section does not name `{expected}` — \
             it still names a stale artifact filename"
        );
    }

    /// The site's download section carries the SAME literal artifact name,
    /// for the same reason: `site/index.html` is a static page, not the
    /// running binary, and cannot read `CARGO_PKG_VERSION`.
    #[test]
    fn site_download_snippet_names_the_real_artifact() {
        let site = include_str!("../site/index.html");
        let expected = format!("awl-{}-linux-x86_64.tar.gz", cargo_toml_version());
        assert!(
            site.contains(&expected),
            "site/index.html's download snippet does not name `{expected}` — \
             it still names a stale artifact filename"
        );
    }

    /// `scripts/package-linux.sh` is the one place the Linux archive name is
    /// actually assembled at build/release time. This pins that it still
    /// DERIVES the name from `$AWL_VERSION` rather than a re-hardcoded
    /// literal — a tag, the binary and the tarball disagreeing is exactly
    /// the drift this whole law exists to catch, and it would silently
    /// reappear if a future edit reverted this line to a fixed string.
    #[test]
    fn package_linux_script_derives_the_archive_name_from_awl_version() {
        let script = include_str!("../scripts/package-linux.sh");
        assert!(
            script.contains(r#"STAGE_NAME="awl-${AWL_VERSION}-linux-x86_64""#),
            "scripts/package-linux.sh no longer derives its archive name from $AWL_VERSION"
        );
    }
}

//! src/repo_url_law.rs — the REPO-URL RENAME law (queue item 238).
//!
//! GitHub's rename to `Frank-P-Lu/awl-editor` left the OLD name spelled out
//! in shipped artifacts: the GPLv3 §6(d) source offer in
//! `scripts/package-linux.sh`, the About window's `GITHUB_URL`
//! (`src/mac_about/facts.rs` — already the one Rust owner), `README.md`, and
//! 27 links across `site/` (18 in the HTML pages, `site/check.js`'s
//! `RELEASES_URL`, and 8 in `site/llms.txt`). This law is the
//! grep-over-tracked-files half of the fix: shell, HTML, JS and txt can't
//! share a Rust constant, so this test — not a single owner — is what keeps
//! the cross-language surface honest.
//!
//! DELIBERATELY NOT a ban on the bare token `awl-next` — the local working
//! directory is still named that, and `src/render/rowlayout.rs`,
//! `src/render/framebench.rs`, `src/capture/tests/schema_chrome.rs` and
//! `src/render/tests/chrome_overlay.rs` all use `"awl-next"` as a realistic
//! sample project-name fixture PRECISELY BECAUSE it is the real directory
//! name. Banning the token outright would fail on that legitimate content
//! (queue item 238's own survey correction). The law instead bans the
//! REPOSITORY REFERENCE `Frank-P-Lu/awl-next`, with or without a
//! `github.com/` prefix — so it also catches a bare `gh api
//! repos/Frank-P-Lu/awl-next/…` call, not just an `https://` link.
//!
//! TWO DELIBERATE EXCLUSIONS, both named here rather than silently:
//!  - `.orchestrator/queue.md` — the orchestration board's append-only
//!    history. Item 238's own entry quotes the bad reference verbatim while
//!    describing the defect, and always will (as will any future item that
//!    cites this one) — banning it there would make the law fail on its own
//!    bug report. Board content is orchestrator-owned, not this law's.
//!  - `site/editor/` — the checked-in, LEGACY wasm demo bundle (e.g.
//!    `awl-347842567538f209_bg.wasm`). It is a BUILT artifact, not authored
//!    text: `deploy-web.yml` assembles a fresh build over a COPY of `site/`
//!    and never commits into `site/editor/`, and `scripts/web-smoke.sh`
//!    without `--trunk` never touches it either — RELEASING.md calls it out
//!    as refreshed only by an occasional deliberate "deploy: refresh /editor
//!    wasm bundle" commit. The old reference living inside its compiled bytes
//!    came from `README.md`, which this round already fixed at the source, so
//!    the next such refresh clears it for free. Forcing a trunk rebuild inside
//!    this law would couple a licence-text fix to an unrelated, unreviewed
//!    multi-file bundle regeneration — excluded by path instead, deliberately.
//!    (Binary files need no separate handling beyond this: `read_to_string`
//!    simply fails to decode them as UTF-8 and they're skipped, so no other
//!    checked-in binary needs naming here.)
//!
//! Not gated to wasm32: this law reads the repo's own tree off a real
//! filesystem, which the browser build has none of — same reasoning as
//! `macos_identity_law`.
#![cfg(all(test, not(target_arch = "wasm32")))]

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Directories never worth descending into: VCS internals, build output, and
/// dependency/tooling caches — never hand-authored text.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "dist",
    "dist-linux",
    "gallery",
    "tmp",
    "coverage",
    "node_modules",
    "__pycache__",
];

/// Individual files excluded for a stated, deliberate reason (see module doc).
/// `src/repo_url_law.rs` excludes ITSELF — its own doc comments and the
/// needle string literal necessarily quote the banned reference, the same
/// self-exclusion `embedded_docs_law.rs` uses for its own citations.
const SKIP_FILES: &[&str] = &[".orchestrator/queue.md", "src/repo_url_law.rs"];

/// Path prefixes (repo-relative) excluded for a stated, deliberate reason
/// (see module doc).
const SKIP_PATH_PREFIXES: &[&str] = &["site/editor/"];

fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        // `file_type()` does NOT follow symlinks (unlike `path.is_dir()`).
        // Skip symlinks entirely rather than following them: this repo
        // aliases `.orchestrator` at `.claude/orchestrator` and
        // `.codex/orchestrator`, and following those would rescan
        // `queue.md` under a path this law's exclusion list doesn't name —
        // a duplicate-scan bug, not a real second copy of the file. Every
        // real file is still reached once, via its own canonical directory
        // (`AGENTS.md` -> `CLAUDE.md` is the same shape and the same reason).
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk(&path, root, out);
        } else {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = rel.to_string_lossy();
            if SKIP_FILES.contains(&rel_str.as_ref()) {
                continue;
            }
            if SKIP_PATH_PREFIXES.iter().any(|p| rel_str.starts_with(p)) {
                continue;
            }
            out.push(path);
        }
    }
}

/// THE LAW. No tracked file — excluding the two documented carve-outs above —
/// spells the old repository reference `Frank-P-Lu/awl-next`.
#[test]
fn no_tracked_file_spells_the_old_repository_reference() {
    let root = repo_root();
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files.sort();

    let needle = "Frank-P-Lu/awl-next";
    let mut offenders: Vec<String> = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue; // not UTF-8 (binary) — not a hand-authored citation
        };
        for (i, line) in text.lines().enumerate() {
            if line.contains(needle) {
                let rel = path.strip_prefix(&root).unwrap_or(path);
                offenders.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
            }
        }
    }
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "tracked file(s) still spell the old repository reference `{needle}` \
         (the GitHub rename landed on `Frank-P-Lu/awl-editor` — queue item \
         238). Fix the URL/reference, or if this is new legitimate content \
         that only coincidentally string-matches, extend this law's \
         documented exclusions rather than silencing it:\n{}",
        offenders.join("\n")
    );
}

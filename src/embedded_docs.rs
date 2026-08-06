//! src/embedded_docs.rs — the ONE owner of every repo-doc / sample / bundled-
//! license `include_str!` path.
//!
//! WHY THIS MODULE EXISTS: an accessibility audit found that doc files were
//! pinned in place purely because scattered modules each carried their own
//! `include_str!` of a doc one directory up. A single doc move meant hunting
//! every embed site. Here every such path lives ONCE; a future move of `GUIDE`,
//! `CREDITS.md`, a `samples/*.md`, or a bundled license file is a one-line edit
//! in THIS file, and every consumer imports the const.
//!
//! LAW: `embed_owner_is_the_only_include_str_site` (in `src/embedded_docs_law.rs`)
//! greps `src/` and fails if an `include_str!` of any of these doc/sample/
//! license paths appears in a module OTHER than this one. Asset BYTES (`.ttf`,
//! `.png`, dictionaries, the keymap-defaults TOML) are a different axis and stay
//! embedded beside their loaders — this owner covers human-readable docs.
//!
//! The paths are relative to THIS file (`src/`), i.e. one level under the repo
//! root. Asset-adjacent licenses are still embedded from their asset dirs
//! (`assets/fonts/…`) — the OWNER is this module, the SOURCE stays beside the
//! asset it documents.

/// The repo's `GUIDE.md` (the in-app Guide; carries the generated keys table).
pub const GUIDE_MD: &str = include_str!("../GUIDE.md");

/// The repo's `REFERENCE.md` — the COLD reference, whose every table is
/// generated from the live rosters and diffed byte-for-byte against them by
/// `reference::law`. Test-only: the reference ships as a file, and the binary
/// never reads it back.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub const REFERENCE_MD: &str = include_str!("../REFERENCE.md");

/// `site/reference.html` — the marketing site's copy of the reference. NOT a
/// hand-mirror (the arrangement `site/guide.html` carries against `GUIDE.md`):
/// the same rows through an HTML emitter, held to the tree by the same law.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub const SITE_REFERENCE_HTML: &str = include_str!("../site/reference.html");

/// The repo's `WORLDS.md` — the hand-written flavour reference for the theme
/// worlds. Only its MEMBERSHIP is law-checked against `theme::THEMES`
/// (`reference::law::worlds_md_names_exactly_the_theme_roster`); the prose is
/// the author's.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub const WORLDS_MD: &str = include_str!("../WORLDS.md");

/// The repo's `CREDITS.md` (the in-app Credits card source).
pub const CREDITS_MD: &str = include_str!("../CREDITS.md");

/// `samples/welcome.md` — the FIRST-RUN document (queue item 24), on both
/// builds: seeded into the active folder by `crate::firstrun` on a native first
/// launch, and into the virtual fs by `fs::SEED_SAMPLES` on a browser's first
/// load. Unconditional — unlike its three siblings below it has a production
/// consumer in a plain native build.
pub const WELCOME_MD: &str = include_str!("../samples/welcome.md");

// The remaining seed samples are consumed only by `fs::SEED_SAMPLES` (wasm/test
// seeding) and the keytoken tests, so they carry that module's exact `cfg` to
// stay warning-clean in a plain native build.

/// `samples/tour.md` — the markdown-showcase seed doc.
#[cfg(any(test, target_arch = "wasm32"))]
pub const TOUR_MD: &str = include_str!("../samples/tour.md");

/// `samples/prose.md` — the prose seed doc.
#[cfg(any(test, target_arch = "wasm32"))]
pub const PROSE_MD: &str = include_str!("../samples/prose.md");

/// `samples/japanese.md` — the CJK seed doc.
#[cfg(any(test, target_arch = "wasm32"))]
pub const JAPANESE_MD: &str = include_str!("../samples/japanese.md");

/// `assets/fonts/LICENSES.md` — the bundled-font license inventory (OFL).
// The PDF-export module (the only consumer) is native-only, so these carry its
// `cfg(all(test, not(wasm32)))` gate to stay warning-clean in the wasm test build.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub const FONT_LICENSES_MD: &str = include_str!("../assets/fonts/LICENSES.md");

/// `assets/fonts/OFL.txt` — the SIL Open Font License text the inventory cites.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub const FONT_OFL_TXT: &str = include_str!("../assets/fonts/OFL.txt");

/// `site/guide.html` — the hand-mirrored marketing-site copy of `GUIDE.md`
/// (see that file's own header comment: an accepted, LOGGED drift risk against
/// the real doc). Test-only: verified against the live catalog by
/// `docs_catalog_law.rs`, never read at runtime (the site is served as a
/// static file, not by the binary).
#[cfg(test)]
pub const SITE_GUIDE_HTML: &str = include_str!("../site/guide.html");

/// The repo's `CAPTURE.md` — verified test-only against
/// `capture::SCHEMA_VERSION` by `capture::tests::schema_ledger` (item 187), so
/// the sidecar-schema reservation header can't drift from the const.
#[cfg(test)]
pub const CAPTURE_MD: &str = include_str!("../CAPTURE.md");

/// Product-boundary source used by the first-launch documentation law.
#[cfg(test)]
pub const PHILOSOPHY_MD: &str = include_str!("../PHILOSOPHY.md");

/// The repo's `README.md`. Test-only: `version_law.rs` pins its Download
/// section's artifact filename to the live package version.
#[cfg(test)]
pub const README_MD: &str = include_str!("../README.md");

# RELEASING.md — cutting a release + deploying the website

Three GitHub Actions workflows. Two are release/deploy pipelines, both
`workflow_dispatch` (deliberate, never automatic): `.github/workflows/deploy-web.yml`
(the site + `/editor/` demo, on Fly.io) and `.github/workflows/release.yml`
(macOS / Linux / web downloadable artifacts, on a `v*` tag push or a manual dry
run). The third, `.github/workflows/ci.yml`, runs automatically on every push /
pull request (linux build + test, wasm build + smoke) — the merge gate, not a
release step. This doc is the one-time setup for the two release pipelines, plus
how to actually cut a release.

**A tag publishes Linux only.** The mac and web jobs build on a dry run and are
skipped on a tag, so no unsigned `.app` can reach a public Release. §5 is the
pre-tag checklist, including the two decisions that are still open.

## 1. Apple setup (macOS signing + notarization)

Signing is **optional but gated** — without these five secrets, `release.yml`
still builds an unsigned universal `Awl.app` + `.dmg` (loudly logged as
unsigned). Set all five together or none; a partial set is treated as "not
configured."

**(a) Export your Developer ID Application certificate as a `.p12`:**

1. In Xcode or the [Apple Developer portal](https://developer.apple.com/account/resources/certificates/list),
   create/download a **"Developer ID Application"** certificate (requires a
   paid Apple Developer Program membership).
2. In Keychain Access, find the certificate + its private key, select both,
   right-click → **Export 2 items…** → save as `DeveloperIDApplication.p12`,
   set an export password.

```sh
base64 -i DeveloperIDApplication.p12 | pbcopy
gh secret set MACOS_CERT_P12 --body "$(pbpaste)"
gh secret set MACOS_CERT_PASSWORD --body "<the export password you set>"
```

**(b) Create an App Store Connect API key** (for `notarytool`, no separate
Apple ID password/2FA prompt needed in CI):

1. [App Store Connect](https://appstoreconnect.apple.com/) → Users and Access
   → Integrations → **App Store Connect API** → generate a key with the
   **Developer** role. Download the `.p8` **once** (Apple won't let you
   re-download it).

```sh
gh secret set APPLE_API_KEY_ID --body "<the Key ID shown in the portal>"
gh secret set APPLE_API_ISSUER --body "<the Issuer ID shown in the portal>"
base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy
gh secret set APPLE_API_KEY_B64 --body "$(pbpaste)"
```

That's all five secrets `release.yml`'s mac job checks for:
`MACOS_CERT_P12`, `MACOS_CERT_PASSWORD`, `APPLE_API_KEY_ID`,
`APPLE_API_ISSUER`, `APPLE_API_KEY_B64`.

## 2. Fly.io setup (website deploy)

```sh
fly tokens create deploy -a awl-editor    # scoped deploy token for the app in site/fly.toml
gh secret set FLY_API_TOKEN --body "<the token printed above>"
```

That's the one secret `deploy-web.yml` checks for. If it's missing, the
workflow fails immediately on its first step rather than burning a wasm build
for nothing.

## 3. Cutting a release

**Website (landing + `/editor/` wasm demo):**

```sh
gh workflow run deploy-web.yml
gh run watch   # or check the Actions tab
```

Builds a fresh `trunk build --release --public-url /editor/`, assembles it
over a copy of `site/`, and `flyctl deploy`s that assembled directory. Never
touches or commits `site/editor/`'s checked-in bundle (legacy — see below).

**Downloadable artifacts (Linux):**

```sh
# 1. on the final combined main, prove debug/release state parity
scripts/release-profile-gate.sh
# 2. bump Cargo.toml's package.version if this is a real version bump
# 3. work §5's checklist, then tag and push
git tag v0.9.0
git push origin v0.9.0
```

The tag push triggers `release.yml`'s `linux` and `publish` jobs: a
`cargo build --release`, the headless parity law again, `scripts/package-linux.sh`
**and** `scripts/package-appimage.sh` (item 227 — the AppImage rides
alongside the tarball, never instead of it), and a new GitHub Release
carrying `awl-<version>-linux-x86_64.tar.gz`, `awl-<version>-linux-x86_64.AppImage`
and `SHA256SUMS` covering both (e.g. `awl-0.9.0-linux-x86_64.tar.gz` /
`awl-0.9.0-linux-x86_64.AppImage` for the `v0.9.0` tag — both scripts derive
the exact name from `Cargo.toml`'s version / the tag, never a separate
hardcoded string; see item 228).
The mac and web jobs do not run on a tag (see §5's open decisions), so no
unsigned `.app` and no `dist/` zip can be attached. Rerunning the parity law
inside the release job means a missed local pre-tag run still blocks publication.

**Dry run (no tag, nothing published) — verify the pipeline is healthy:**

```sh
gh workflow run release.yml -f dry_run=true
gh run watch
```

All three build jobs run; artifacts land in the run's **Artifacts** tab instead
of a GitHub Release, `publish` is skipped, and no tag or release is created.

**Locally, without GitHub** — the packaging steps alone, against any Linux build:

```sh
scripts/package-linux.sh path/to/linux/awl dist-linux
scripts/package-appimage.sh path/to/linux/awl dist-linux
```

`package-linux.sh` stages the payload, writes the tarball and its `.sha256`,
and prints the archive listing. `package-appimage.sh` assembles the AppDir
(binary, `.desktop` launcher, the icon cut from `assets/macos/Awl.icns`,
licences) and — Linux x86_64 only, since appimagetool's own upstream binary
is a Linux x86_64 ELF — cuts it into the `.AppImage` + its `.sha256`; on any
other host (a macOS dev machine included) it still assembles and structurally
verifies the AppDir, then skips the cut with a named, loud reason rather than
a confusing exec failure. `--assemble-only` skips the cut deliberately, for
the same structural check with no host-detection ambiguity. Both scripts
treat a missing licence file as a hard failure, not a warning.

**Which Linux build is the release build.** Three exist; only one ships.

| Path | What it is |
|---|---|
| `release.yml`'s `linux` job | **the release build.** A native x86_64 `cargo build --release` on `ubuntu-latest`, so the artifact's glibc floor is that runner image's |
| `Dockerfile.linux` + `scripts/build-linux.sh` | a developer convenience — cross-builds on a Mac against Debian bookworm (glibc 2.36) for a personal laptop. Never packaged, never released |
| `run-linux.sh` | a from-source bootstrap on the target machine. Installs system packages and compiles; not a download |

### What lands where

| Artifact | Where |
|---|---|
| `awl-<version>-linux-x86_64.tar.gz` + `awl-<version>-linux-x86_64.AppImage` + `SHA256SUMS` (covering both) | GitHub Release (tag) |
| same two files + their own `.sha256`s | workflow artifact `awl-linux` (dry run — `<version>` is `0.0.0-dryrun`) |
| `Awl.app` (universal, unsigned until §1 is done) + `Awl.dmg` | workflow artifact `awl-macos` — **dry run only**, never attached to a Release |
| `awl-web-dist.zip` (the `trunk build --release` output) | workflow artifact `awl-web` — **dry run only**, never attached to a Release |
| the live website + `/editor/` demo | Fly.io (`awl-editor`, `site/fly.toml`) — via `deploy-web.yml`, separately |

### Icons (RESOLVED — the per-world app-icon round)

`assets/macos/Awl.icns` is committed, and `scripts/package-macos.sh` copies
it into `Contents/Resources/` and names it in `Info.plist`'s
`CFBundleIconFile`. A release bundle no longer falls back to the generic
application icon. (A missing file is still only a loud warning, so the script
stays runnable against an older checkout.)

| Icon | What it is | Changes when |
|---|---|---|
| `Contents/Resources/Awl.icns` | the canonical bundle icon Finder, Launchpad and the About panel draw | never at runtime — a bundle icon belongs to the bundle |
| the running app's Dock / ⌘-Tab tile | the ACTIVE world's own icon (`app_icon::adopt`) | at launch after the sticky theme is restored, and on a theme picker **commit** — never on a hover preview |

The canonical file is the DEFAULT world's icon, byte for byte (a law test
pins it to `DEFAULT_THEME`), so retargeting the default retargets Finder's
icon with it.

**Regenerating** (only when a world's palette/face changes, a world is added,
or the lockup is retuned) — offline, pinned, no network:

```sh
scripts/export-icons.sh            # manifest -> pages -> PNGs -> pixel checks -> .icns
scripts/export-icons.sh --check    # ... and re-render, comparing sha256s
```

It rewrites `assets/macos/world/<World>.icns`, `assets/macos/Awl.icns` and the
generated `src/app_icon/embedded.rs`; commit all three together. An ordinary
`cargo build` runs none of it. See `scripts/icons/README.md` and
`src/app_icon/`.

### AppImage (RESOLVED — item 227, the friendly Linux download)

The tarball has no launcher metadata or icon integration — appropriate for a
technical early adopter, not a normal desktop install. `scripts/package-appimage.sh`
assembles an AppDir and (Linux x86_64 only) cuts it into
`awl-<version>-linux-x86_64.AppImage`, published **alongside** the tarball,
never instead of it (the tarball is the documented fallback for a desktop
with no FUSE and no `--appimage-extract-and-run` support).

**What's inside, and where each piece comes from:**

| Piece | Source |
|---|---|
| `usr/bin/awl` | the same `cargo build --release` binary the tarball ships |
| `AppRun` | a plain symlink to `usr/bin/awl` — no wrapper script; awl needs no environment setup before it can run |
| `<id>.desktop` (root + `usr/share/applications/`) | written by the packaging script; `Name=Awl`, `Exec=awl %f` (lowercase — awl's CLI takes exactly one file argument, never a list), `Icon=<id>`, `Categories=Utility;TextEditor;Development;` |
| `<id>.png` (root, 256px, + `usr/share/icons/hicolor/256x256/apps/`) | **not a second hand-drawn asset** — `awl --export-linux-icon` cuts the 256px PNG straight out of the committed canonical `assets/macos/Awl.icns` via `app_icon::icns::unpack`, the same parser the macOS icon law tests use as their oracle |
| `usr/share/doc/awl/{LICENSE,NOTICE,CREDITS.md,THIRD-PARTY-LICENSES.md,licenses/,README.txt}` | the identical required set §4 names for the tarball |
| shared libraries | **none, deliberately** — every runtime dependency is either part of the base desktop stack a normal Linux install already has (fontconfig, libxkbcommon, X11/Wayland — same expectation the tarball's own `README.txt` documents) or a GPU-adapter-specific library (the Vulkan loader/ICD) that is explicitly excluded per this item's own brief: **no bundled GPU driver** |

**Cutting the AppDir into a single file** needs `appimagetool`
(`scripts/install-appimagetool.sh`, pinned to release `1.9.1`, sha256
verified against the release asset's own GitHub-computed digest — a
build-time fetch, not a runtime one; the shipped AppImage itself makes no
network call, ever) and only runs on Linux x86_64, since that tool's own
upstream binary is a Linux x86_64 ELF. Any other host — the macOS dev
machine included — still assembles and structurally verifies the AppDir
(the `.desktop` required keys, the icon present at both required paths and
decodable as a real PNG, the licence set), then skips the cut with a named,
loud reason. `--assemble-only` requests exactly that half deliberately.

**The identifier** (`dev.franklu.awl`, overridable via `AWL_BUNDLE_ID`) is
the same reverse-DNS string `scripts/package-macos.sh` already defaults to —
one identity across both packagers rather than a second name invented for
Linux.

**Owed to a human at a Linux desktop** (not reachable from this macOS
repository): actual launch on Debian/Ubuntu and Fedora-like environments,
Wayland vs. X11 behaviour, GPU-adapter smoke on real hardware, and whether
the desktop environment's launcher actually shows the name/icon this AppImage
declares.

## 4. The LICENSE gap (RESOLVED — the LICENSE + CREDITS round)

The repo ships a full GPL-3.0 `LICENSE` file (matching `Cargo.toml`'s
`license = "GPL-3.0-only"`, flippable to `-or-later` — see that file's own
header comment) and a `NOTICE` naming the copyright holder (Frank Lu, 2026)
and the bundled-asset carve-outs. The two gaps this section used to name are
both closed:

- **The Hunspell dictionaries** now have `assets/dict/LICENSES.md` — an
  honest per-variant audit (`en_GB` = confirmed LGPL 2.1 in-file; `en_US`/
  `en_AU` = SCOWL permissive grant + Ispell BSD license, resolved via the
  bundled `README_en_AU.txt`, which expressly covers BOTH variants — no longer
  an open question). Provenance verified byte-for-byte against versioned
  LibreOffice / Chromium upstream commits; all three pairs are GPL-3.0-compatible
  as bundled plain-text data.
- **Copyright on awl's own code** is stated in `NOTICE` + `Cargo.toml`'s
  header comment. A CONTRIBUTORS file remains unnecessary (`NOTICE`'s
  "CONTRIBUTIONS" section already states the project isn't soliciting
  outside patches).

Two more artifacts landed alongside: `THIRD-PARTY-LICENSES.md` (the
generated Rust-crate license inventory, `cargo about generate about.hbs -o
THIRD-PARTY-LICENSES.md` — regeneration instructions in the file's own
header) and `CREDITS.md` (the human-readable thank-you, also reachable
in-app via Cmd-P → "Credits" and on the website at `/credits.html`).

All six license-adjacent docs ride every release artifact — the AppImage
(item 227) makes it three packaging paths, not two. Until item 226 only four
of them did: the font and dictionary audits were named here as shipping and
were not copied by any packaging path, while the fonts and dictionaries
themselves are `include_bytes!`d into the binary — so a downloaded awl carried
OFL and LGPL-2.1 material with neither licence beside it.

| Doc | Tarball | `Awl.app` | AppImage (`AppDir/`) |
|---|---|---|---|
| `LICENSE` (GPL-3.0 full text) | root | `Contents/Resources/` + DMG root | `usr/share/doc/awl/` |
| `NOTICE` (copyright holder, asset carve-outs) | root | `Contents/Resources/` + DMG root | `usr/share/doc/awl/` |
| `CREDITS.md` | root | `Contents/Resources/` + DMG root | `usr/share/doc/awl/` |
| `THIRD-PARTY-LICENSES.md` (generated crate inventory) | root | `Contents/Resources/` + DMG root | `usr/share/doc/awl/` |
| `assets/fonts/LICENSES.md` (SIL OFL 1.1) | `licenses/fonts-LICENSES.md` | `Contents/Resources/licenses/` | `usr/share/doc/awl/licenses/fonts-LICENSES.md` |
| `assets/dict/LICENSES.md` (LGPL-2.1 + SCOWL/Ispell) | `licenses/dict-LICENSES.md` | `Contents/Resources/licenses/` | `usr/share/doc/awl/licenses/dict-LICENSES.md` |

The tarball's `README.txt` carries the GPLv3 §6(d) source offer — a link to the
public repository; the AppImage's own `usr/share/doc/awl/README.txt` carries
the same offer. `scripts/package-linux.sh` and `scripts/package-appimage.sh`
both exit non-zero on any missing file in that set; `scripts/package-macos.sh`
warns, because it must stay runnable against an older checkout.

## 5. Pre-tag checklist

Nothing here is automatic. Work it top to bottom on the exact commit the tag
will name. `scripts/pretag-journeys.py` is the journey-sweep instrument this
checklist's audit policy requires (CLAUDE.md, "pre-tag: a journey sweep
across worlds") — run it before step 1.

| # | Step | Done when |
|---|---|---|
| 1 | `scripts/native-gate.sh` on combined main | receipt names the tag's commit (see the note below on what it does *not* cover) |
| 2 | `scripts/audit.sh` | cargo-deny clean, or a recorded narrow ignore |
| 3 | `scripts/release-profile-gate.sh` | all 8 action families match debug↔release |
| 4 | `cargo about generate about.hbs -o THIRD-PARTY-LICENSES.md` | regenerated, diff reviewed, committed |
| 5 | `gh workflow run release.yml -f dry_run=true` | three green build jobs, `publish` skipped |
| 6 | Download `awl-linux` from the run; unpack the tarball, `chmod +x` the AppImage | listing matches §4's table; `sha256sum -c` passes on both |
| 7 | Launch both the unpacked tarball binary AND the AppImage on a real Linux desktop | each opens a window, opens a file, `--screenshot` writes a PNG; the AppImage's launcher name/icon show correctly where the desktop supports it |
| 8 | `Cargo.toml`'s `package.version` matches the tag | `v<version>` — no stale `0.1.0` |
| 9 | `git tag`, `git push origin <tag>` | **user's explicit word, every time** |
| 10 | After the tag: `gh workflow run deploy-web.yml` | **user's explicit word too.** `version.json` comes from `git describe --tags`, so until the site is redeployed at or after the tag, Check for Updates keeps reporting "no tagged release yet" |

#### What step 1's receipt does not certify

A `native-gate.sh` receipt is **hardware-bounded**: it certifies "sound on the
hardware the receipts run on, with virtualised-GPU behaviour untested by any
local gate." The gate uses the host's own adapter — real Apple Silicon Metal on
the dev machine — and a shipped binary will run on machines that have no such
thing: VMs, remote desktops, and hosted CI. Item 231's wedge was green under
that receipt for ~140 commits while the hosted-macOS CI job hung on virtualised
Metal, so **a green receipt is not evidence about this axis and never was.**

A software adapter does not close the gap (measured, item 232 — see the CI
workflow's `mac` job comments). CI's `linux` job runs this same gate against
Mesa lavapipe on every push and stayed green through the entire streak, and a
local lavapipe container never hung at either bisect boundary. **The hosted-mac
jobs are the only arm awl has on virtualised-GPU behaviour**, and since item 243
(user decision 2026-08-03, resolving item 232's parked question) that arm is
split: `mac (build + test, minus render::tests)` gates every push, and
`mac (render::tests)` is tolerated red, pinned by name to item 231. Before a
tag, check both on the tag's exact commit rather than assuming: a red gating
job already blocked `main` from reaching this point, and a red `render::tests`
job is item 231's known wedge, not a hard release gate by itself — but a user
on a VM is inside its blast radius, so know *why* it's red before tagging.

**Neither mac job prints a `native-gate-receipt`, and no replacement exists.**
`native-gate.sh` refuses a filtered invocation by construction — that refusal is
what makes its receipt mean "the full suite, both conventions, every target" —
and both mac jobs are filtered by definition of the split. Before item 243 a
reader could see a receipt in the mac job's log and take it as informal
confirmation that *that exact commit* passed the full suite on virtualised
Metal. **That confirmation no longer exists in any form, and nothing was built
to replace it.** Nothing consumes the string, so nothing is broken; this note
exists so no one goes looking for a signal that is gone.

Read the two mac jobs' own conclusions instead. They are individually
meaningful, which is the point of the split — a synthesised combined receipt
would re-bundle exactly what was deliberately unbundled, and would have to lie
about scope to call itself a receipt.

### Still open — decisions, not tasks

| Decision | State today | Owner |
|---|---|---|
| Cut a public tag at all | **settled — tags are cut.** `v0.9.0`, `v0.10.0`, `v0.11.0` and `v0.12.0` are published, each Linux-only and marked prerelease. Every tag still waits on the user's explicit word, every time | the user, explicitly (CLAUDE.md §Branches) |
| macOS artifacts | none of the five Apple secrets in §1 are set; the mac job is skipped on a tag so an unsigned `.app` cannot publish | the user — needs a paid Apple Developer Program membership |
| Version + prerelease flag | **resolved by item 228 for the GitHub Release; the "and the site" half of the original premise was false.** `Cargo.toml` is pre-1.0. `release.yml`'s `plan` job now computes `prerelease` from the tag's major version (`< 1` ⇒ true) and the `publish` step passes it to `softprops/action-gh-release`, so `v0.9.0` publishes correctly marked prerelease — verified against that action's own source (`INPUT_PRERELEASE == "true"`), not just its docs. `deploy-web.yml`'s `version.json` `prerelease` field is a DIFFERENT thing sharing a name: `site/check.js`'s `checkState()` (locked by `site/check.test.js`) reads it only as "no tag has ever shipped" — the page never renders a stable/beta claim at all, so there was nothing on the site for a beta tag to invert. That field stays `false` for any real tag, unchanged | settled |
| glibc floor | **RESOLVED 2026-08-06 — the linux job builds on `ubuntu-22.04` and the floor is `GLIBC_2.35`**, reaching Debian 12, Ubuntu 22.04 LTS and RHEL 9. Measured, not reasoned: `objdump -T` finds exactly two dynsyms that could raise the floor — `pidfd_spawnp` and `pidfd_getpid`, both weak, both from Rust std's OPTIONAL pidfd fast path for reaping a child it already spawned. ⚠️ Both are **unversioned** (`w D *UND*`, no `GLIBC_*` tag), so they never appear in the version-needs list: a re-check that greps that list for anything above 2.35 finds NOTHING and reads as "this note has gone stale." It has not — grep `objdump -T` for `pidfd` itself, or `readelf --dyn-syms`, and use GNU binutils rather than the host `objdump` on a Mac. awl references no PidFd API and every `std::process::Command` in the tree blocks on `.output()`/`.wait()`, so std's fork/exec fallback costs nothing observable. Binaries built on `debian:bookworm` and `ubuntu:22.04` both cap at 2.35 and render byte-identical PNGs. ⚠️ The cache key had to move with it: `Swatinem/rust-cache` mixes `runner.os`, which is `"Linux"` for both images, so it is keyed on `ImageOS` now | settled |
| Web download | `awl-web-dist.zip` builds on dry runs and is not attached; the site is the web distribution | settled unless a self-host story is wanted |
| Asset filename | **decided by the user 2026-08-06: the version goes in, per queue item 228.** The long-standing "these cannot both hold" conflict with `/releases/latest/download/` was false and is retired: **no tracked file has ever hardcoded that URL.** The site (`site/index.html`) and README both link to the releases *page*; the unversioned name appears only in the two producers (`release.yml`, `scripts/package-linux.sh`) and in two instructional `tar xzf` snippets. So versioning the asset costs those snippets and nothing else — there is no stable-URL dependency to break, and no per-release site edit | settled |

#### What the build base costs, for the glibc decision

Where the release binary is built decides which systems can run it. Nothing
else in the pipeline changes.

| Build base | glibc | Reaches |
|---|---|---|
| `ubuntu-latest` (Ubuntu 24.04) | 2.39 | Ubuntu 24.04+, Fedora 40+, Arch. **Not** Debian 12, **not** Ubuntu 22.04 LTS, **not** RHEL 9 |
| `ubuntu-22.04` runner — **shipped** (see the glibc-floor row above) | 2.35 | adds Ubuntu 22.04 LTS and Debian 12. Runner image is on GitHub's retirement track |
| `container: debian:bookworm` | 2.36 | adds Debian 12. Matches `Dockerfile.linux`'s existing, stated choice |
| `container: debian:bullseye` | 2.31 | adds Ubuntu 20.04, Debian 11, RHEL 9. Oldest toolchain and headers, so the highest build risk |

**Item 227 (RESOLVED, 2026-08-07): the AppImage does not change the glibc
decision.** It wraps the identical `linux` job binary — same `ubuntu-22.04`
build, same `GLIBC_2.35` floor — in desktop integration, not a different
toolchain or a bundled libc. The "friendly download vs. technical tarball"
framing this row used to carry is retired: both are the same binary now,
published together, and a system too old for one is too old for the other.

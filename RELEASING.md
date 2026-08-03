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
git tag v0.1.0
git push origin v0.1.0
```

The tag push triggers `release.yml`'s `linux` and `publish` jobs: a
`cargo build --release`, the headless parity law again, `scripts/package-linux.sh`,
and a new GitHub Release carrying `awl-linux-x86_64.tar.gz` and `SHA256SUMS`.
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

**Locally, without GitHub** — the packaging step alone, against any Linux build:

```sh
scripts/package-linux.sh path/to/linux/awl dist-linux
```

It stages the payload, writes the tarball and its `.sha256`, and prints the
archive listing. A missing licence file is a hard failure, not a warning.

**Which Linux build is the release build.** Three exist; only one ships.

| Path | What it is |
|---|---|
| `release.yml`'s `linux` job | **the release build.** A native x86_64 `cargo build --release` on `ubuntu-latest`, so the artifact's glibc floor is that runner image's |
| `Dockerfile.linux` + `scripts/build-linux.sh` | a developer convenience — cross-builds on a Mac against Debian bookworm (glibc 2.36) for a personal laptop. Never packaged, never released |
| `run-linux.sh` | a from-source bootstrap on the target machine. Installs system packages and compiles; not a download |

### What lands where

| Artifact | Where |
|---|---|
| `awl-linux-x86_64.tar.gz` + `SHA256SUMS` | GitHub Release (tag) |
| `awl-linux-x86_64.tar.gz` + `.sha256` | workflow artifact `awl-linux` (dry run) |
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

All six license-adjacent docs ride every release artifact. Until item 226 only
four of them did: the font and dictionary audits were named here as shipping
and were not copied by any packaging path, while the fonts and dictionaries
themselves are `include_bytes!`d into the binary — so a downloaded awl carried
OFL and LGPL-2.1 material with neither licence beside it.

| Doc | Tarball | `Awl.app` |
|---|---|---|
| `LICENSE` (GPL-3.0 full text) | root | `Contents/Resources/` + DMG root |
| `NOTICE` (copyright holder, asset carve-outs) | root | `Contents/Resources/` + DMG root |
| `CREDITS.md` | root | `Contents/Resources/` + DMG root |
| `THIRD-PARTY-LICENSES.md` (generated crate inventory) | root | `Contents/Resources/` + DMG root |
| `assets/fonts/LICENSES.md` (SIL OFL 1.1) | `licenses/fonts-LICENSES.md` | `Contents/Resources/licenses/` |
| `assets/dict/LICENSES.md` (LGPL-2.1 + SCOWL/Ispell) | `licenses/dict-LICENSES.md` | `Contents/Resources/licenses/` |

The tarball's `README.txt` carries the GPLv3 §6(d) source offer — a link to the
public repository. `scripts/package-linux.sh` exits non-zero on any missing
file in that set; `scripts/package-macos.sh` warns, because it must stay
runnable against an older checkout.

## 5. Pre-tag checklist

Nothing here is automatic. Work it top to bottom on the exact commit the tag
will name.

| # | Step | Done when |
|---|---|---|
| 1 | `scripts/native-gate.sh` on combined main | receipt names the tag's commit (see the note below on what it does *not* cover) |
| 2 | `scripts/audit.sh` | cargo-deny clean, or a recorded narrow ignore |
| 3 | `scripts/release-profile-gate.sh` | all 8 action families match debug↔release |
| 4 | `cargo about generate about.hbs -o THIRD-PARTY-LICENSES.md` | regenerated, diff reviewed, committed |
| 5 | `gh workflow run release.yml -f dry_run=true` | three green build jobs, `publish` skipped |
| 6 | Download `awl-linux` from the run and unpack it | listing matches §4's table; `sha256sum -c` passes |
| 7 | Launch the unpacked binary on a real Linux desktop | window opens, a file opens, `--screenshot` writes a PNG |
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
workflow's `mac` job comment). CI's `linux` job runs this same gate against
Mesa lavapipe on every push and stayed green through the entire streak, and a
local lavapipe container never hung at either bisect boundary. **The hosted-mac
`mac (build + test)` job is the only arm awl has on virtualised-GPU behaviour.**
Before a tag, check it on the tag's exact commit rather than assuming, and if it
is red, know *why* before tagging: a user on a VM is inside its blast radius.
Whether a red `mac` is a hard release blocker is a parked user decision (item
232; the recommendation is yes).

### Still open — decisions, not tasks

| Decision | State today | Owner |
|---|---|---|
| Cut a public tag at all | no tag has ever been pushed; `gh release list` is empty | the user, explicitly (CLAUDE.md §Branches) |
| macOS artifacts | none of the five Apple secrets in §1 are set; the mac job is skipped on a tag so an unsigned `.app` cannot publish | the user — needs a paid Apple Developer Program membership |
| Version + prerelease flag | `Cargo.toml` says `0.1.0`; queue item 228 proposes `v0.9.0` marked prerelease. `release.yml` does not pass `prerelease:` and `deploy-web.yml`'s `version.json` sets `prerelease: false` for any tag it finds — so a beta would currently read as stable on both surfaces | queue item 228 |
| glibc floor | **measured, not estimated.** `ubuntu-latest` is Ubuntu 24.04, so the binary's highest referenced symbol is `GLIBC_2.39`. Verified by running the produced tarball on Debian 12: `libc.so.6: version 'GLIBC_2.39' not found`. See the table below | the user — it is a support-matrix choice, and item 227's AppImage may answer it instead |
| Web download | `awl-web-dist.zip` builds on dry runs and is not attached; the site is the web distribution | settled unless a self-host story is wanted |
| Asset filename | `awl-linux-x86_64.tar.gz` carries no version, which is what keeps `/releases/latest/download/awl-linux-x86_64.tar.gz` a stable URL the site and docs can hardcode. Queue item 228 asks for `0.9.0` in artifact names — the two cannot both hold | the user, when 228 lands |

#### What the build base costs, for the glibc decision

Where the release binary is built decides which systems can run it. Nothing
else in the pipeline changes.

| Build base | glibc | Reaches |
|---|---|---|
| `ubuntu-latest` (Ubuntu 24.04) — **today** | 2.39 | Ubuntu 24.04+, Fedora 40+, Arch. **Not** Debian 12, **not** Ubuntu 22.04 LTS, **not** RHEL 9 |
| `ubuntu-22.04` runner | 2.35 | adds Ubuntu 22.04 LTS and Debian 12. Runner image is on GitHub's retirement track |
| `container: debian:bookworm` | 2.36 | adds Debian 12. Matches `Dockerfile.linux`'s existing, stated choice |
| `container: debian:bullseye` | 2.31 | adds Ubuntu 20.04, Debian 11, RHEL 9. Oldest toolchain and headers, so the highest build risk |

Item 227's AppImage may make this moot for the friendly download while the
tarball stays technical — decide the two together.

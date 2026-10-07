#!/usr/bin/env bash
#
# package-macos.sh — assemble Awl.app, either the ORDINARY flavor (from one
# already-built native `awl` binary — the release workflow packages arm64 and
# x86_64 separately) or the MAS
# (Mac App Store / App Sandbox) flavor (`--mas`, which builds the
# `--features mas` binary itself and bundles the sandbox entitlements
# alongside for a human to sign against). One script, two modes — folded
# together deliberately: a standalone MAS packaging script briefly existed
# on a concurrent worktree before this merge; its `--mas` behavior now lives
# here instead of as a second parallel script.
#
# Usage:
#   scripts/package-macos.sh <path-to-native-binary> <output-dir>
#     Ordinary flavor: assemble one arm64 or x86_64 app (does NOT build).
#
#   scripts/package-macos.sh --mas [output-dir]
#     MAS flavor: builds `cargo build --release --features mas` itself
#     (output-dir defaults to `dist`) and assembles Awl.app with
#     `packaging/mas/entitlements.plist` copied into Resources/.
#
#   scripts/package-macos.sh --dmg-only <path-to-Awl.app> <output.dmg> [arch]
#     Create a DMG from an already assembled (and possibly signed/stapled)
#     bundle. The release workflow uses this after notarization so the DMG
#     contains the exact app it validated. Optional arch is arm64 or x86_64.
#
#   scripts/package-macos.sh --print-arch <path-to-binary> [expected-arch]
#     Print one accepted native architecture; reject universal, unsupported,
#     unreadable, or mismatched binaries.
#
# Produces (ordinary):
#   <output-dir>/Awl.app/Contents/{MacOS/awl, Info.plist, Resources/}
#   <output-dir>/Awl.dmg          (only if `hdiutil` succeeds — see below)
#
# Produces (--mas):
#   <output-dir>/Awl.app/Contents/{MacOS/awl, Info.plist, Resources/}
#   <output-dir>/Awl.app/Contents/Resources/entitlements.plist
#   No DMG (Mac App Store submissions never ship a DMG).
#
# Env overrides (all optional):
#   AWL_BUNDLE_ID      reverse-DNS bundle identifier (default below)
#   AWL_VERSION        CFBundleShortVersionString (default: Cargo.toml's
#                       package.version, read via `cargo metadata` if cargo
#                       is on PATH, else "0.0.0")
#   AWL_BUILD_VERSION  CFBundleVersion (default: "1.0.0"). Apple permits up
#                       to four digits in its positive first component and up
#                       to two digits in each remaining component.
#   AWL_SKIP_DMG=1      skip DMG creation (bundle-only; --mas never makes one
#                       regardless of this flag)
#
# --reclaim / CI=true — GitHub mac-runner disk-exhaustion hardening:
#   After the workflow's two `cargo build --release --target ...` steps,
#   `target/<triple>/release/deps` (the per-arch object-file cache — several
#   GB, dead weight once both native executables exist) is deleted
#   before assembling/DMGing. Gated so a LOCAL run never loses incremental
#   build state: fires only when `--reclaim` is passed explicitly OR the
#   ambient `CI=true` (GitHub Actions sets this natively on every job) — the
#   release workflow passes `--reclaim` explicitly too, belt-and-suspenders,
#   so the behavior doesn't depend on an inherited env var alone. Never
#   touches `target/release` (the ordinary, non-cross-compiled profile a dev
#   iterates on locally) and never fires in `--mas` mode (which builds
#   in-place, no per-arch split to reclaim).
#
# DMG staging + hdiutil TMPDIR — the other half of the same hardening: DMG
# staging now happens in a directory UNDER the caller's own <output-dir>
# (which the release workflow points at a workspace-relative path,
# `dist-mac`) instead of the system `mktemp -d`/$TMPDIR default — on GitHub
# mac runners the system temp volume has been observed nearly full while the
# workspace volume has room. `TMPDIR` is exported (workspace-local) around
# the `hdiutil create` invocation specifically, since that's the documented
# lever hdiutil honors for its own scratch/temp work (there is no `-tmpdir`
# flag); the explicit `cp -R`-populated staging dir was already a `mktemp -d`
# consumer of the same default, so both move together.
#
# SIGNING IS DELIBERATELY OUT OF SCOPE in both modes: this script never calls
# `codesign` — it only PRINTS the command a human should run next (with their
# own Developer ID / Apple Development / Apple Distribution identity),
# mirroring the ordinary flavor's own release workflow (gated on secrets,
# kept separate so this script stays runnable standalone with no Apple
# developer account at all).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Pin this Mac's toolchain so cargo is findable regardless of cwd / a bare
# shell (only matters for --mas, which invokes cargo itself).
if ! command -v cargo >/dev/null 2>&1; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Release downloads are architecture-specific. This is the one parser for the
# executable's real Mach-O roster: filenames and target paths are not evidence.
# A universal input would recreate the oversized download this split replaces,
# while accepting an arbitrary thin architecture would mislabel a public DMG.
single_macos_arch() {
  local binary="$1"
  local expected="${2:-}"
  local archs

  if [ ! -f "$binary" ]; then
    echo "error: binary not found at $binary" >&2
    return 1
  fi
  if ! archs="$(lipo -archs "$binary" 2>/dev/null)"; then
    echo "error: cannot read a Mach-O architecture from $binary" >&2
    return 1
  fi
  case "$archs" in
    arm64|x86_64) ;;
    *)
      echo "error: $binary has unsupported architecture roster '$archs'; expected exactly one of arm64 or x86_64" >&2
      return 1
      ;;
  esac
  if [ -n "$expected" ] && [ "$archs" != "$expected" ]; then
    echo "error: $binary is $archs, expected $expected" >&2
    return 1
  fi
  printf '%s\n' "$archs"
}

# Apple's marketing version is exactly three dot-separated integers. Its build
# version is separate: the first component is positive and at most four digits;
# the second and third are each at most two digits. Validate before interpolating
# either value into Info.plist so signing/notarization never sees malformed data.
validate_macos_versions() {
  local short_version="$1"
  local build_version="$2"

  if [[ ! "$short_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: AWL_VERSION '$short_version' must be three dot-separated non-negative integers" >&2
    return 1
  fi
  if [[ ! "$build_version" =~ ^[1-9][0-9]{0,3}\.[0-9]{1,2}\.[0-9]{1,2}$ ]]; then
    echo "error: AWL_BUILD_VERSION '$build_version' must match 1-9999.0-99.0-99" >&2
    return 1
  fi
}

# --- THE BUNDLE-IDENTITY CONTRACT (one owner) --------------------------------
#
# macOS reads a live app's product identity out of the bundle, not out of the
# process. Everything the OS is willing to honor comes from these keys plus the
# icon file they name, so they are asserted in ONE place — called by the
# assembly path below AND by `--verify`, so the gate and the build can never
# drift into checking different things.
#
# The split this exists to protect: Finder, the menu bar, Stage Manager and the
# app switcher all display the CAPITALISED product name, while the executable —
# and therefore the CLI command a person types — stays lowercase `awl`. A future
# plist edit that merges those two contracts fails here.
verify_bundle_identity() {
  local app="$1"
  local expected_arch="${2:-}"
  local contents="$app/Contents"
  local plist="$contents/Info.plist"
  local root actual_arch fail=0
  root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

  if [ ! -f "$plist" ]; then
    echo "!! $plist missing — not an assembled bundle" >&2
    return 1
  fi
  bundle_value() { /usr/libexec/PlistBuddy -c "Print :$1" "$plist" 2>/dev/null; }
  check() { # <label> <actual> <expected>
    if [ "$2" != "$3" ]; then
      echo "!! bundle identity: $1 is '$2', expected '$3'" >&2
      fail=1
    fi
  }

  # DISPLAYED identity — capitalised, everywhere macOS shows a product name.
  check CFBundleName "$(bundle_value CFBundleName)" "Awl"
  check CFBundleDisplayName "$(bundle_value CFBundleDisplayName)" "Awl"
  # TYPED identity — lowercase, the command and the executable on disk.
  check CFBundleExecutable "$(bundle_value CFBundleExecutable)" "awl"
  if [ ! -x "$contents/MacOS/awl" ]; then
    echo "!! bundle identity: $contents/MacOS/awl is missing or not executable" >&2
    fail=1
  elif ! actual_arch="$(single_macos_arch "$contents/MacOS/awl" "$expected_arch")"; then
    fail=1
  fi

  # THE ICON macOS reads for the surfaces that ignore the running app's own
  # `setApplicationIconImage` — Finder, and Stage Manager. Named by the plist,
  # present in Resources/, and byte-identical to the committed canonical icon,
  # so a stale copy can never ship in its place.
  check CFBundleIconFile "$(bundle_value CFBundleIconFile)" "Awl.icns"
  if [ ! -f "$contents/Resources/Awl.icns" ]; then
    echo "!! bundle identity: Contents/Resources/Awl.icns is missing" >&2
    fail=1
  elif ! cmp -s "$root/assets/macos/Awl.icns" "$contents/Resources/Awl.icns"; then
    echo "!! bundle identity: bundled Awl.icns differs from assets/macos/Awl.icns" >&2
    fail=1
  fi

  if [ "$fail" -ne 0 ]; then
    echo "!! bundle identity check FAILED for $app" >&2
    return 1
  fi
  echo "==> bundle identity OK: Awl / Awl / awl / Awl.icns / $actual_arch  ($app)"
}

# One owner for DMG layout and hdiutil hardening. Both ordinary local
# packaging and the post-notarization release path call this function.
create_dmg() {
  local app="$1"
  local output="$2"
  local expected_arch="${3:-}"
  local out_dir dmg_work dmg_staging apparent_bytes dmg_size_mb
  out_dir="$(cd "$(dirname "$output")" && pwd)"
  output="$out_dir/$(basename "$output")"

  verify_bundle_identity "$app" "$expected_arch"
  echo "==> creating $output"

  # Stage BOTH the DMG source-folder copy AND hdiutil's own scratch/temp work
  # under the output directory rather than the system temporary volume. The
  # explicit size avoids hdiutil's sparse-file auto-size undershoot observed
  # on hosted macOS runners; stat's apparent size is intentional here.
  dmg_work="$out_dir/.dmg-work"
  rm -rf "$dmg_work"
  mkdir -p "$dmg_work/staging" "$dmg_work/tmp"
  dmg_staging="$dmg_work/staging"
  cp -R "$app" "$dmg_staging/"
  ln -s /Applications "$dmg_staging/Applications"

  # Keep the same licence documents visible beside the app and inside it.
  for doc in LICENSE NOTICE CREDITS.md THIRD-PARTY-LICENSES.md; do
    [ -f "$ROOT/$doc" ] && cp "$ROOT/$doc" "$dmg_staging/$doc"
  done

  echo "==> disk space before hdiutil:"
  df -h
  apparent_bytes="$(find "$dmg_staging" -type f -exec stat -f%z {} + | awk '{sum+=$1} END{print sum+0}')"
  dmg_size_mb=$(( (apparent_bytes * 2 / 1024 / 1024) + 64 ))
  echo "==> sizing DMG scratch image: ${dmg_size_mb}m (from ${apparent_bytes} apparent bytes staged)"

  if ! TMPDIR="$dmg_work/tmp" hdiutil create -volname "Awl" -srcfolder "$dmg_staging" \
    -size "${dmg_size_mb}m" -ov -format UDZO "$output"; then
    rm -rf "$dmg_work"
    echo "!! hdiutil failed while creating $output" >&2
    return 1
  fi
  rm -rf "$dmg_work"
  echo "==> $output created"
}

MAS=0
RECLAIM=0
VERIFY_ONLY=0
DMG_ONLY=0
PRINT_ARCH_ONLY=0
POSITIONAL=()
for arg in "$@"; do
  case "$arg" in
    --mas) MAS=1 ;;
    --reclaim) RECLAIM=1 ;;
    --verify) VERIFY_ONLY=1 ;;
    --dmg-only) DMG_ONLY=1 ;;
    --print-arch) PRINT_ARCH_ONLY=1 ;;
    -h|--help)
      sed -n '2,80p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) POSITIONAL+=("$arg") ;;
  esac
done
[ "${CI:-}" = "true" ] && RECLAIM=1

if [[ "$PRINT_ARCH_ONLY" -eq 1 ]]; then
  ARCH_BINARY="${POSITIONAL[0]:?usage: package-macos.sh --print-arch <path-to-binary> [expected-arch]}"
  EXPECTED_ARCH="${POSITIONAL[1]:-}"
  single_macos_arch "$ARCH_BINARY" "$EXPECTED_ARCH"
  exit $?
fi

# `--verify <path-to-Awl.app>`: assert an ALREADY-ASSEMBLED bundle's identity
# and exit. This is the native packaging gate — it builds nothing, so it is
# cheap enough to run against any bundle a dev or CI job has produced.
if [[ "$VERIFY_ONLY" -eq 1 ]]; then
  APP_TO_VERIFY="${POSITIONAL[0]:?usage: package-macos.sh --verify <path-to-Awl.app> [arch]}"
  EXPECTED_ARCH="${POSITIONAL[1]:-}"
  verify_bundle_identity "$APP_TO_VERIFY" "$EXPECTED_ARCH"
  exit $?
fi

if [[ "$DMG_ONLY" -eq 1 ]]; then
  APP_TO_PACKAGE="${POSITIONAL[0]:?usage: package-macos.sh --dmg-only <path-to-Awl.app> <output.dmg> [arch]}"
  DMG_OUTPUT="${POSITIONAL[1]:?usage: package-macos.sh --dmg-only <path-to-Awl.app> <output.dmg> [arch]}"
  EXPECTED_ARCH="${POSITIONAL[2]:-}"
  mkdir -p "$(dirname "$DMG_OUTPUT")"
  create_dmg "$APP_TO_PACKAGE" "$DMG_OUTPUT" "$EXPECTED_ARCH"
  exit 0
fi

if [[ "$MAS" -eq 1 ]]; then
  command -v cargo >/dev/null 2>&1 || { echo "!! cargo not found on PATH" >&2; exit 1; }
  echo "==> building MAS (--features mas) release binary..."
  # with-remap.sh strips the builder's $HOME/registry paths from this shipped
  # release binary (rustc bakes compile-time source locations otherwise).
  (cd "$ROOT" && "$ROOT/scripts/with-remap.sh" cargo build --release --features mas)
  BIN_PATH="$ROOT/target/release/awl"
  OUT_DIR="${POSITIONAL[0]:-$ROOT/dist}"
else
  BIN_PATH="${POSITIONAL[0]:?usage: package-macos.sh <path-to-native-binary> <output-dir> (or --mas [output-dir])}"
  OUT_DIR="${POSITIONAL[1]:?usage: package-macos.sh <path-to-native-binary> <output-dir> (or --mas [output-dir])}"
fi

BINARY_ARCH="$(single_macos_arch "$BIN_PATH")"

mkdir -p "$OUT_DIR"

# --- Reclaim headroom (CI only — see the module doc's "--reclaim" note) ----
# By the time release assembly starts, both per-arch executables already exist;
# each `target/<triple>/release/deps` directory (every dependency crate's
# compiled object files) is dead weight, and on a GitHub mac runner it is
# several GB of headroom we want back before hdiutil ever runs.
if [[ "$MAS" -eq 0 && "$RECLAIM" -eq 1 ]]; then
  for triple in aarch64-apple-darwin x86_64-apple-darwin; do
    DEPS="$ROOT/target/$triple/release/deps"
    if [ -d "$DEPS" ]; then
      SIZE_BEFORE="$(du -sh "$DEPS" 2>/dev/null | cut -f1)"
      echo "==> reclaim: removing $DEPS ($SIZE_BEFORE — dead post-build artifacts)"
      rm -rf "$DEPS"
    fi
  done
fi

# reverse-DNS identifier — a placeholder namespace, USER-CHANGEABLE. There is
# no registered organization domain for awl yet; this is a reasonable-looking
# default (dev.<author>.awl) that the user should replace with their own once
# they pick a real domain / Apple Developer Team identity. See RELEASING.md.
AWL_BUNDLE_ID="${AWL_BUNDLE_ID:-dev.franklu.awl}"

AWL_VERSION="${AWL_VERSION:-}"
if [ -z "$AWL_VERSION" ]; then
  if command -v cargo >/dev/null 2>&1; then
    AWL_VERSION="$(cd "$ROOT" && cargo metadata --no-deps --format-version 1 2>/dev/null \
      | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)"
  fi
  AWL_VERSION="${AWL_VERSION:-0.0.0}"
fi
AWL_BUILD_VERSION="${AWL_BUILD_VERSION:-1.0.0}"
validate_macos_versions "$AWL_VERSION" "$AWL_BUILD_VERSION"
AWL_SOURCE_COMMIT="${AWL_SOURCE_COMMIT:-$(git -C "$(dirname "${BASH_SOURCE[0]}")/.." rev-parse HEAD)}"
[[ "$AWL_SOURCE_COMMIT" =~ ^[0-9a-f]{40}$ ]] || { echo "error: invalid source commit" >&2; exit 1; }

APP="$OUT_DIR/Awl.app"
CONTENTS="$APP/Contents"
echo "==> assembling $APP  (version $AWL_VERSION, build $AWL_BUILD_VERSION, bundle id $AWL_BUNDLE_ID, architecture $BINARY_ARCH)$([ "$MAS" -eq 1 ] && echo '  [MAS]')"

rm -rf "$APP"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"

cp "$BIN_PATH" "$CONTENTS/MacOS/awl"
chmod +x "$CONTENTS/MacOS/awl"

# ICON: the canonical bundle icon — the DEFAULT world's pre-rendered app icon,
# cut by `awl --pack-icns` and committed (see `src/app_icon/`). This is
# what FINDER, the Dock's launch tile and the About panel show; a bundle icon is
# a property of the bundle, not of the session, so it never follows the user's
# chosen world. The RUNNING app swaps its own Dock/app-switcher image to the
# active world (`app_icon::adopt`) — Finder keeps this one.
#
# Missing file = a loud warning + the generic application icon, never a hard
# failure: this script must stay runnable against an older checkout (same rule
# as the license docs below). `CFBundleIconFile` is written further down under
# the same `-f` test, so the plist can never name an icon that is not there.
ICON_SRC="$ROOT/assets/macos/Awl.icns"
if [ -f "$ICON_SRC" ]; then
  cp "$ICON_SRC" "$CONTENTS/Resources/Awl.icns"
else
  echo "warning: $ICON_SRC not found — bundling WITHOUT an icon (macOS will draw the generic application icon). Run scripts/export-icons.sh." >&2
fi

# LICENSING: LICENSE (GPL-3.0 full text), CREDITS.md (the human-readable
# thank-you), and THIRD-PARTY-LICENSES.md (the generated crate inventory)
# ride into Contents/Resources/ — the standard macOS bundle home for
# license-adjacent docs (see also Apple's own apps' Resources/ folders).
# Missing files are a loud warning, never a hard failure (this script must
# stay runnable standalone against an older checkout too).
for doc in LICENSE NOTICE CREDITS.md THIRD-PARTY-LICENSES.md; do
  if [ -f "$ROOT/$doc" ]; then
    cp "$ROOT/$doc" "$CONTENTS/Resources/$doc"
  else
    echo "warning: $ROOT/$doc not found — skipping (bundle built without it)" >&2
  fi
done

# The fonts (SIL OFL 1.1) and Hunspell dictionaries are `include_bytes!`d into
# the binary, so their audits belong beside the binary that contains them —
# the same pair scripts/package-linux.sh puts in the tarball's licenses/.
mkdir -p "$CONTENTS/Resources/licenses"
for pair in fonts dict; do
  if [ -f "$ROOT/assets/$pair/LICENSES.md" ]; then
    cp "$ROOT/assets/$pair/LICENSES.md" "$CONTENTS/Resources/licenses/$pair-LICENSES.md"
  else
    echo "warning: $ROOT/assets/$pair/LICENSES.md not found — skipping (bundle built without it)" >&2
  fi
done

# DOCUMENT TYPES: declares Awl as an EDITOR for markdown/plain-text/
# text documents at `LSHandlerRank Alternate` — OFFERED in Finder's Open With
# menu (and "Change All…" can make it the default) without silently stealing
# the OS default the way `Default` would. `net.daringfireball.markdown` is a
# third-party-declared UTI (not an Apple built-in), so its own
# `CFBundleTypeExtensions` fallback (`md`/`markdown`, plus `txt` under
# `public.plain-text`) is what still matches on a machine where nothing else
# has ever declared that UTI — LaunchServices falls back to extension matching
# when no UTI claims one. Declaring the type is only HALF the story: the
# process must also accept the resulting Apple Event, which
# `crate::mac_open_documents` (`src/mac_open_documents.rs`) does at runtime by
# injecting `application:openURLs:` onto winit's own delegate class — see that
# file's module doc. MAS ENTITLEMENTS: a Finder-opened file arrives to a
# sandboxed process exactly like an `NSOpenPanel` selection (both are
# USER-SELECTED file access grants at the point the user picked something in
# Finder/a panel), so the existing
# `com.apple.security.files.user-selected.read-write` entitlement
# (`packaging/mas/entitlements.plist`) already covers it — no new entitlement
# needed for the `--mas` flavor.
cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Awl</string>
  <key>CFBundleDisplayName</key>
  <string>Awl</string>
  <key>CFBundleExecutable</key>
  <string>awl</string>
  <key>CFBundleIdentifier</key>
  <string>${AWL_BUNDLE_ID}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${AWL_VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${AWL_BUILD_VERSION}</string>
  <key>AwlSourceCommit</key>
  <string>${AWL_SOURCE_COMMIT}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSHumanReadableCopyright</key>
  <string>GPL-3.0-only</string>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Markdown</string>
      <key>CFBundleTypeRole</key>
      <string>Editor</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>net.daringfireball.markdown</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>md</string>
        <string>markdown</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Plain Text</string>
      <key>CFBundleTypeRole</key>
      <string>Editor</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>public.plain-text</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>txt</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Text</string>
      <key>CFBundleTypeRole</key>
      <string>Editor</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>public.text</string>
      </array>
    </dict>
  </array>
PLIST

if [ -f "$ICON_SRC" ]; then
  cat >> "$CONTENTS/Info.plist" <<PLIST
  <key>CFBundleIconFile</key>
  <string>Awl.icns</string>
PLIST
fi

cat >> "$CONTENTS/Info.plist" <<PLIST
</dict>
</plist>
PLIST

verify_bundle_identity "$APP" "$BINARY_ARCH"

echo "==> Awl.app assembled"

if [[ "$MAS" -eq 1 ]]; then
  # MAS SANDBOX ENTITLEMENTS: copied into the bundle's own Resources/ for
  # reference, and named again below so a human knows exactly what to
  # `codesign --entitlements` with. Never signed here (see the module doc).
  cp "$ROOT/packaging/mas/entitlements.plist" "$CONTENTS/Resources/entitlements.plist"
  echo "==> done: $APP (unsigned, MAS entitlements at packaging/mas/entitlements.plist)"
  echo "    next (out of scope here): codesign --entitlements packaging/mas/entitlements.plist \\"
  echo "         --sign <your Apple Distribution identity> --deep --force \"$APP\""
  # Mac App Store submissions never ship a DMG (App Store Connect / Transporter
  # takes the signed .app / .pkg directly) — exit before the DMG step below.
  exit 0
fi

if [ "${AWL_SKIP_DMG:-0}" = "1" ]; then
  exit 0
fi

create_dmg "$APP" "$OUT_DIR/Awl.dmg" "$BINARY_ARCH"

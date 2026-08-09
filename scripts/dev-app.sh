#!/usr/bin/env bash
#
# dev-app.sh — THE supported way to run awl on macOS during development.
#
# Why this exists instead of `cargo run`: macOS reads a live app's product
# identity out of its BUNDLE, not out of the process. A bare
# `target/release/awl` has no bundle, so several surfaces fall back to the
# executable's filename and to no icon at all — measured, on 2026-07-29
# against this exact binary:
#
#   surface        bare binary            this script's bundle
#   ------------   --------------------   ---------------------------
#   menu bar       "awl" (lowercase)      "Awl"
#   Stage Manager  no icon at all         the canonical Awl icon
#   ⌘-Tab name     "awl" (lowercase)      "Awl"
#   ⌘-Tab icon     the active world       the active world
#   Dock           the active world       the active world
#
# The two columns differ ONLY where macOS insists on a bundle. The Dock and
# ⌘-Tab tiles already work on the bare binary because the running app sets them
# itself (`app_icon::adopt` → `setApplicationIconImage`), and they keep
# following the active world here — that behavior is untouched.
#
# The bare binary's menu-bar name and Stage Manager icon CANNOT be fixed
# without a bundle. AppKit takes the application-menu title from the main
# bundle's `CFBundleName` and falls back to the process name when there is no
# Info.plist, and Stage Manager reads the icon LaunchServices has registered
# for the bundle — neither consults anything the process can set at runtime.
# Forcing them would mean spoofing the process title, which lies about what is
# running; awl does not do that. Use this script instead. See docs/platform.md.
#
# LAUNCHSERVICES REGISTRATION IS NOT OPTIONAL (the non-obvious half). Assembling
# the bundle is enough for the menu bar, but Stage Manager showed the GENERIC
# blueprint tile until the bundle was registered — a locally built .app in a
# build directory is not somewhere LaunchServices has looked. `lsregister -f`
# below is what makes the icon real. A released build never needs this: the user
# drags Awl.app out of the DMG into /Applications, which registers it.
#
# Usage:
#   scripts/dev-app.sh [--debug] [--no-launch] [-- <args to awl>]
#
#   --debug       build and bundle the dev profile instead of --release.
#                 Off by default: CLAUDE.md's rule is that feel is judged in
#                 --release, where frames are 10-20x faster.
#   --no-launch   assemble, register and verify, but do not open the app.
#   --            everything after it is passed through to awl (a file to open,
#                 --theme, --root, ...).
#
# Produces:
#   target/dev-app/Awl.app     (never signed; signing stays a release concern)
#
# The bundle is assembled by `package-macos.sh`, so it carries exactly the
# canonical metadata a release carries — the same Info.plist writer, the same
# committed `assets/macos/Awl.icns`. There is no second source of product
# identity to drift.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "dev-app.sh is macOS-only (it assembles an .app bundle). On Linux run the binary directly." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
fi

PROFILE_FLAG="--release"
PROFILE_DIR="release"
LAUNCH=1
APP_ARGS=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --debug) PROFILE_FLAG=""; PROFILE_DIR="debug" ;;
    --no-launch) LAUNCH=0 ;;
    -h|--help) sed -n '2,55p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    --) shift; APP_ARGS=("$@"); break ;;
    *) APP_ARGS+=("$1") ;;
  esac
  shift || true
done

echo "==> building ($PROFILE_DIR)"
# shellcheck disable=SC2086 -- PROFILE_FLAG is deliberately word-split (empty = dev profile).
(cd "$ROOT" && cargo build $PROFILE_FLAG)

OUT_DIR="$ROOT/target/dev-app"
APP="$OUT_DIR/Awl.app"
AWL_SKIP_DMG=1 "$SCRIPT_DIR/package-macos.sh" "$ROOT/target/$PROFILE_DIR/awl" "$OUT_DIR"

# Tell LaunchServices this bundle exists, so Stage Manager (and anything else
# that resolves an icon through LS rather than through the running process) can
# find `Awl.icns`. Without this the tile is the generic blueprint placeholder.
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
if [ -x "$LSREGISTER" ]; then
  "$LSREGISTER" -f "$APP"
  echo "==> registered with LaunchServices"
else
  echo "warning: lsregister not found at the expected path — Stage Manager may show the generic icon" >&2
fi

# The identity gate, against the bundle that is about to run. Same function the
# assembly step above already called; run again here so `--no-launch` is a
# complete check on its own.
"$SCRIPT_DIR/package-macos.sh" --verify "$APP"

if [ "$LAUNCH" -eq 0 ]; then
  echo "==> $APP ready (not launched)"
  exit 0
fi

echo "==> launching $APP"
# `-n` forces a NEW instance, so this never hands off to a release copy that
# happens to be running; awl's own single-instance daemon still applies within
# a data root.
if [ "${#APP_ARGS[@]}" -gt 0 ]; then
  open -n -a "$APP" --args "${APP_ARGS[@]}"
else
  open -n -a "$APP"
fi

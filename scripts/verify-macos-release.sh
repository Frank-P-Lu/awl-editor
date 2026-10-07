#!/usr/bin/env bash
# Validate the actual native downloads; hosted and local signing share this owner.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${1:?usage: verify-macos-release.sh <dist> <version> <build-version> <signed|unsigned> <ci|local>}"
VERSION="${2:?missing version}"
BUILD_VERSION="${3:?missing build version}"
SIGNING="${4:?missing signing policy}"
LAUNCH="${5:?missing launch context}"
case "$SIGNING" in signed|unsigned) ;; *) echo 'error: invalid signing policy' >&2; exit 1 ;; esac
case "$LAUNCH" in ci|local) ;; *) echo 'error: invalid launch context' >&2; exit 1 ;; esac
SOURCE_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
TEAM_ID="$(cat "$ROOT/assets/macos/release-team-id.txt")"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/awl-mac-release-check.XXXXXX")"
MOUNT=""
cleanup() {
  if [ -n "$MOUNT" ]; then hdiutil detach "$MOUNT" >/dev/null 2>&1 || true; fi
  rm -rf "$SCRATCH"
}
trap cleanup EXIT
for ARCH in arm64 x86_64; do
  DMG="awl-$VERSION-macos-$ARCH.dmg"
  APP_ZIP="awl-$VERSION-macos-$ARCH.app.zip"
  "$ROOT/scripts/check-macos-release-size.sh" "$DIST/$DMG"
  hdiutil verify "$DIST/$DMG"
  MOUNT="$SCRATCH/$ARCH"
  mkdir "$MOUNT"
  hdiutil attach -readonly -nobrowse -mountpoint "$MOUNT" "$DIST/$DMG"
  "$ROOT/scripts/package-macos.sh" --verify "$MOUNT/Awl.app" "$ARCH"
  SHORT_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$MOUNT/Awl.app/Contents/Info.plist")"
  ACTUAL_BUILD_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$MOUNT/Awl.app/Contents/Info.plist")"
  [ "$SHORT_VERSION" = "$VERSION" ] || { echo "error: $ARCH marketing version mismatch" >&2; exit 1; }
  [ "$ACTUAL_BUILD_VERSION" = "$BUILD_VERSION" ] || { echo "error: $ARCH build version mismatch" >&2; exit 1; }
  BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$MOUNT/Awl.app/Contents/Info.plist")"
  ACTUAL_SOURCE="$(/usr/libexec/PlistBuddy -c 'Print :AwlSourceCommit' "$MOUNT/Awl.app/Contents/Info.plist")"
  [ "$BUNDLE_ID" = dev.franklu.awl ] || { echo "error: bundle identifier mismatch" >&2; exit 1; }
  [ "$ACTUAL_SOURCE" = "$SOURCE_COMMIT" ] || { echo "error: signed source commit mismatch" >&2; exit 1; }
  if [ "$SIGNING" = signed ]; then
    python3 "$ROOT/scripts/verify-macos-signature.py" "$MOUNT/Awl.app" "$TEAM_ID"
    codesign --verify --deep --strict --verbose=2 "$MOUNT/Awl.app"
    xcrun stapler validate "$MOUNT/Awl.app"
    spctl --assess --type execute --verbose=4 "$MOUNT/Awl.app"
  fi
  launch_args=()
  if [ "$LAUNCH" = local ]; then launch_args+=(--local); fi
  python3 "$ROOT/scripts/release-launch-smoke.py" --binary "$MOUNT/Awl.app/Contents/MacOS/awl" "${launch_args[@]}"
  ditto -c -k --keepParent "$MOUNT/Awl.app" "$DIST/$APP_ZIP"
  "$ROOT/scripts/check-macos-release-size.sh" "$DIST/$DMG" "$DIST/$APP_ZIP"
  hdiutil detach "$MOUNT"
  MOUNT=""
  (cd "$DIST" && shasum -a 256 "$DMG" > "$DMG.sha256")
  (cd "$DIST" && shasum -a 256 "$APP_ZIP" > "$APP_ZIP.sha256")
done

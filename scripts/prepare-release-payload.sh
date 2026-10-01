#!/usr/bin/env bash
# Verify producer checksums and select the exact public release payload. This
# runs for manual dry runs and tags; creating the GitHub Release remains a
# separate tag-only step.
set -euo pipefail

LINUX_DIR="${1:?usage: prepare-release-payload.sh <linux-dir> <mac-dir> <output-dir> <version>}"
MAC_DIR="${2:?usage: prepare-release-payload.sh <linux-dir> <mac-dir> <output-dir> <version>}"
OUT_DIR="${3:?usage: prepare-release-payload.sh <linux-dir> <mac-dir> <output-dir> <version>}"
VERSION="${4:?usage: prepare-release-payload.sh <linux-dir> <mac-dir> <output-dir> <version>}"

TARBALL="awl-${VERSION}-linux-x86_64.tar.gz"
APPIMAGE="awl-${VERSION}-linux-x86_64.AppImage"
DMG="awl-${VERSION}-macos-universal.dmg"

if command -v sha256sum >/dev/null 2>&1; then
  SHA256=(sha256sum)
else
  SHA256=(shasum -a 256)
fi

for pair in \
  "$LINUX_DIR/$TARBALL" \
  "$LINUX_DIR/$TARBALL.sha256" \
  "$LINUX_DIR/$APPIMAGE" \
  "$LINUX_DIR/$APPIMAGE.sha256" \
  "$MAC_DIR/$DMG" \
  "$MAC_DIR/$DMG.sha256"; do
  if [ ! -f "$pair" ]; then
    echo "error: expected $pair; downloaded files were:" >&2
    find "$LINUX_DIR" "$MAC_DIR" -type f -print 2>/dev/null | sed 's/^/  /' >&2
    exit 1
  fi
done

# The strict limit applies to every public download, on rehearsals and tags.
# Check before hashing/copying so an oversized input cannot enter the payload.
MAX_BYTES=50000000
file_size() {
  if stat -f%z "$1" >/dev/null 2>&1; then
    stat -f%z "$1"
  else
    stat -c%s "$1"
  fi
}
for public_file in "$LINUX_DIR/$TARBALL" "$LINUX_DIR/$APPIMAGE" "$MAC_DIR/$DMG"; do
  public_bytes="$(file_size "$public_file")"
  echo "public download size: $public_file = $public_bytes bytes (must be under $MAX_BYTES)"
  if [ "$public_bytes" -ge "$MAX_BYTES" ]; then
    echo "error: $public_file is $public_bytes bytes; every public download must be under $MAX_BYTES bytes" >&2
    exit 1
  fi
done

(cd "$LINUX_DIR" && "${SHA256[@]}" -c "$TARBALL.sha256" "$APPIMAGE.sha256")
(cd "$MAC_DIR" && "${SHA256[@]}" -c "$DMG.sha256")

mkdir -p "$OUT_DIR"
if find "$OUT_DIR" -mindepth 1 -print -quit | grep -q .; then
  echo "error: output directory must be empty: $OUT_DIR" >&2
  exit 1
fi
cp "$LINUX_DIR/$TARBALL" "$LINUX_DIR/$APPIMAGE" "$MAC_DIR/$DMG" "$OUT_DIR/"
(cd "$OUT_DIR" && "${SHA256[@]}" "$TARBALL" "$APPIMAGE" "$DMG" > SHA256SUMS)
(cd "$OUT_DIR" && "${SHA256[@]}" -c SHA256SUMS)

echo "release payload ready:"
find "$OUT_DIR" -maxdepth 1 -type f -print | sort | sed 's/^/  /'

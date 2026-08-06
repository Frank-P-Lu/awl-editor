#!/usr/bin/env bash
#
# install-appimagetool.sh — fetch the pinned appimagetool AppImage, verify
# its checksum, and extract it (GitHub Actions runners have no working
# /dev/fuse, so an AppImage cannot mount itself there — extracting once with
# `--appimage-extract` and running the resulting `squashfs-root/AppRun`
# directly needs no FUSE at all, and is the tool's own documented escape
# hatch for exactly this environment).
#
# This is a BUILD-TIME fetch, not a runtime one: zero-network is awl's own
# invariant (docs/licensing.md), not scripts/package-appimage.sh's — the
# shipped AppImage itself makes no network call, ever. This script — like
# install-sccache.sh's `cargo install` — pulls a pinned build tool over the
# network only while assembling a release artifact.
#
# Usage:
#   scripts/install-appimagetool.sh [--version]
#
# Prints the path to the extracted AppRun on success (stdout's last line);
# every other line of output goes to stderr so `$(...)` capture is exact,
# matching install-sccache.sh's own contract.
set -euo pipefail

APPIMAGETOOL_VERSION="1.9.1"
# Pinned against the release's own asset digest (`gh api
# repos/AppImage/appimagetool/releases/tags/1.9.1`), not transcribed from a
# webpage — the digest GitHub itself computed over the uploaded bytes.
APPIMAGETOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"

if [[ "${1:-}" == "--version" ]]; then
    printf '%s\n' "$APPIMAGETOOL_VERSION"
    exit 0
fi
if [[ "$#" -ne 0 ]]; then
    echo "usage: $0 [--version]" >&2
    exit 2
fi

# appimagetool's own upstream binary is a Linux x86_64 ELF (itself shipped as
# an AppImage) — it cannot execute on any other kernel/arch, extraction
# included. Fail fast and by name rather than an opaque "Exec format error"
# from the kernel three steps down. `scripts/package-appimage.sh --assemble-only`
# is the portable half of this pipeline and needs none of this.
if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
    echo "!! appimagetool-x86_64.AppImage only runs on Linux x86_64 (got $(uname -s) $(uname -m))." >&2
    echo "   Use scripts/package-appimage.sh --assemble-only to build+validate the AppDir" >&2
    echo "   on this host; the .AppImage cut itself is a release.yml linux-job / Linux-dev-box step." >&2
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/appimagetool-$APPIMAGETOOL_VERSION"
APPIMAGE="$CACHE/appimagetool-x86_64.AppImage"
EXTRACTED="$CACHE/squashfs-root"
APPRUN="$EXTRACTED/AppRun"

if [ -x "$APPRUN" ]; then
    echo "appimagetool $APPIMAGETOOL_VERSION already extracted" >&2
    echo "$APPRUN"
    exit 0
fi

mkdir -p "$CACHE"

if [ ! -f "$APPIMAGE" ]; then
    echo "==> downloading appimagetool $APPIMAGETOOL_VERSION" >&2
    URL="https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-x86_64.AppImage"
    curl -sL --fail -o "$APPIMAGE.part" "$URL"
    mv "$APPIMAGE.part" "$APPIMAGE"
fi

echo "==> verifying sha256" >&2
if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "$APPIMAGE" | cut -d' ' -f1)"
else
    ACTUAL="$(shasum -a 256 "$APPIMAGE" | cut -d' ' -f1)"
fi
if [ "$ACTUAL" != "$APPIMAGETOOL_SHA256" ]; then
    echo "!! appimagetool checksum mismatch: expected $APPIMAGETOOL_SHA256, got $ACTUAL" >&2
    rm -f "$APPIMAGE"
    exit 1
fi

chmod +x "$APPIMAGE"

echo "==> extracting (no FUSE required this way)" >&2
(
    cd "$CACHE"
    rm -rf squashfs-root
    ./appimagetool-x86_64.AppImage --appimage-extract >/dev/null
)

if [ ! -x "$APPRUN" ]; then
    echo "!! extraction did not produce an executable $APPRUN" >&2
    exit 1
fi

echo "==> appimagetool $APPIMAGETOOL_VERSION ready" >&2
echo "$APPRUN"

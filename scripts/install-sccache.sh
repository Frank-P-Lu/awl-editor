#!/usr/bin/env bash
# Install the compiler-result cache used by .cargo/config.toml. This is an
# explicit bootstrap step: a Cargo wrapper cannot install itself because Cargo
# would try to invoke the missing wrapper while compiling it.
#
# Fetches the pinned mozilla/sccache release's prebuilt binary for this
# host's (OS, arch) and verifies it against a SHA256 hardcoded below — from
# GitHub's own computed `digest` field for the asset
# (`gh api repos/mozilla/sccache/releases/tags/v$SCCACHE_VERSION`), not
# transcribed from a webpage or from the release's own uploaded `.sha256`
# sidecar file (a compromised release process could forge that sidecar the
# same way it forged the binary), matching install-appimagetool.sh's own
# rationale. Linux assets are the musl build: statically linked, so there is
# no dynamic libc to mismatch against the runner image — the same hazard
# class CLAUDE.md's rust-cache tripwire names for a *different* artifact in
# this workflow (a glibc-linked proc-macro cached across incompatible
# `runner.os` values). This script introduces no new cache of its own: the
# download+verify+extract below runs fresh every job, so there is no key to
# collide across runner images in the first place.
#
# A host outside the pinned (OS, arch) table, or a download that fails after
# retries, falls back to the original from-source `cargo install`. A
# checksum MISMATCH does not fall back — that is a tamper signal, not a
# availability one, and silently rebuilding from source would paper over it.
set -euo pipefail

SCCACHE_VERSION="0.17.0"

if [[ "${1:-}" == "--version" ]]; then
    printf '%s\n' "$SCCACHE_VERSION"
    exit 0
fi
if [[ "$#" -ne 0 ]]; then
    echo "usage: $0 [--version]" >&2
    exit 2
fi

if command -v sccache >/dev/null 2>&1 \
    && sccache --version 2>/dev/null | grep -Fxq "sccache $SCCACHE_VERSION"; then
    echo "sccache $SCCACHE_VERSION already installed"
    exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found; install Rust before sccache" >&2
    exit 1
fi

install_from_source() {
    echo "==> installing sccache $SCCACHE_VERSION from source (cargo install)" >&2
    # Run outside the checkout so its rustc-wrapper config cannot apply before
    # the wrapper exists. cargo install puts the result in Cargo's normal bin
    # directory.
    (
        cd "${TMPDIR:-/tmp}"
        cargo install sccache --version "$SCCACHE_VERSION" --locked
    )
}

# One line per supported (uname -s)/(uname -m): "<archive-name> <sha256>".
# The tarball's top-level directory always matches the archive's basename
# (verified by hand for every entry below), so it is not stored separately.
sccache_release_asset() {
    case "$1/$2" in
        Darwin/arm64)
            echo "sccache-v${SCCACHE_VERSION}-aarch64-apple-darwin.tar.gz 0c560bfba31aef5bdfb4fb3d2677f6e61d71c5c00952f2a83344f47aa31f00f1"
            ;;
        Darwin/x86_64)
            echo "sccache-v${SCCACHE_VERSION}-x86_64-apple-darwin.tar.gz c2144cafbfe3d22e34ae637f9974ce53613543ac19477fdb287df22ea3668261"
            ;;
        Linux/x86_64)
            echo "sccache-v${SCCACHE_VERSION}-x86_64-unknown-linux-musl.tar.gz 67c4a96dd237c1f518f6b36083f270f9976d516f1e57fce891755ea782e50006"
            ;;
        Linux/aarch64)
            echo "sccache-v${SCCACHE_VERSION}-aarch64-unknown-linux-musl.tar.gz 821a86343191aa1cbab74bd42f9e93c9a63bf85e4742945f40d3ae84193c1c77"
            ;;
        *)
            return 1
            ;;
    esac
}

HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"

if ! ASSET_LINE="$(sccache_release_asset "$HOST_OS" "$HOST_ARCH")"; then
    echo "==> no pinned prebuilt sccache $SCCACHE_VERSION for $HOST_OS/$HOST_ARCH; falling back to source build" >&2
    install_from_source
    exit 0
fi
read -r ARCHIVE EXPECTED_SHA256 <<<"$ASSET_LINE"
DIRNAME="${ARCHIVE%.tar.gz}"

# The directory cargo's own bin already lives in and that CI's PATH already
# carries — no assumption about $CARGO_HOME layout beyond "next to cargo
# itself", and it is exactly what `cargo install` above would have used.
BIN_DIR="$(dirname "$(command -v cargo)")"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

URL="https://github.com/mozilla/sccache/releases/download/v${SCCACHE_VERSION}/${ARCHIVE}"
echo "==> downloading sccache $SCCACHE_VERSION ($HOST_OS/$HOST_ARCH prebuilt)" >&2
if ! curl -fsSL --retry 3 --retry-delay 2 -o "$WORKDIR/$ARCHIVE" "$URL"; then
    echo "==> download failed for $URL; falling back to source build" >&2
    install_from_source
    exit 0
fi

echo "==> verifying sha256" >&2
if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL_SHA256="$(sha256sum "$WORKDIR/$ARCHIVE" | cut -d' ' -f1)"
else
    ACTUAL_SHA256="$(shasum -a 256 "$WORKDIR/$ARCHIVE" | cut -d' ' -f1)"
fi
if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
    echo "!! sccache checksum mismatch: expected $EXPECTED_SHA256, got $ACTUAL_SHA256" >&2
    echo "!! not falling back to source build: this is a tamper signal, not a network failure" >&2
    exit 1
fi

tar xzf "$WORKDIR/$ARCHIVE" -C "$WORKDIR"
if [ ! -x "$WORKDIR/$DIRNAME/sccache" ]; then
    echo "!! extraction did not produce an executable $WORKDIR/$DIRNAME/sccache" >&2
    exit 1
fi

mkdir -p "$BIN_DIR"
install -m 755 "$WORKDIR/$DIRNAME/sccache" "$BIN_DIR/sccache"

if ! sccache --version 2>/dev/null | grep -Fxq "sccache $SCCACHE_VERSION"; then
    echo "!! installed sccache did not report version $SCCACHE_VERSION" >&2
    exit 1
fi

echo "==> sccache $SCCACHE_VERSION installed to $BIN_DIR/sccache" >&2

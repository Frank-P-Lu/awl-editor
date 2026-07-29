#!/usr/bin/env bash
# Install the compiler-result cache used by .cargo/config.toml. This is an
# explicit bootstrap step: a Cargo wrapper cannot install itself because Cargo
# would try to invoke the missing wrapper while compiling it.
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

echo "==> installing sccache $SCCACHE_VERSION"
# Run outside the checkout so its rustc-wrapper config cannot apply before the
# wrapper exists. cargo install puts the result in Cargo's normal bin directory.
(
    cd "${TMPDIR:-/tmp}"
    cargo install sccache --version "$SCCACHE_VERSION" --locked
)

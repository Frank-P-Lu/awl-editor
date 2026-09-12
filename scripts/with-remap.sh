#!/usr/bin/env bash
#
# with-remap.sh — run a RELEASE/WASM build with the builder's $HOME stripped
# from every path baked into the compiled binary.
#
# WHY. rustc bakes compile-time source locations (panic sites, `file!()`,
# `#[track_caller]` `Location`s) and dependency registry paths into optimized
# binaries. The release builds are the ONLY ones we distribute — native
# `--release` and the wasm bundle (trunk builds it `--release`) — and left alone
# they embed the BUILDER's home directory. The repo is public (CLAUDE.md:
# "tracked files carry no personal-machine paths"), so a builder's $HOME must
# not ship in a downloaded release or the freshly assembled /editor/ demo.
#
# HOW. `--remap-path-prefix` rewrites `$CARGO_HOME`→`/cargo` (more specific, so
# it goes first — rustc applies the first matching prefix) and `$HOME`→`~`, both
# read from the environment AT BUILD TIME. Nothing personal is ever written into
# a tracked file — that is the whole reason this is a wrapper reading `$HOME`
# rather than a literal prefix in `.cargo/config.toml`, which would re-leak the
# name into a public file. Any pre-existing RUSTFLAGS is preserved (appended).
#
# USE. Prepend it to any release/wasm build invocation:
#   scripts/with-remap.sh trunk build --release --public-url /editor/
#   scripts/with-remap.sh cargo build --release
#
# SCOPE. Shipped builds only — dev `cargo build` / `cargo test` keep their real
# paths for debuggability. Any code included in a shipped build must avoid
# compile-time checkout-path string literals: `--remap-path-prefix` rewrites
# rustc's emitted paths, not the contents of an `env!` string literal.
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: scripts/with-remap.sh <build command…>   (e.g. trunk build --release)" >&2
  exit 2
fi

remap="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=${HOME}=~"
export RUSTFLAGS="${remap}${RUSTFLAGS:+ ${RUSTFLAGS}}"
exec "$@"

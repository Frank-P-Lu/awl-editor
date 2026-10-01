#!/usr/bin/env bash
# Run with the committed project toolchain and rustup's linker-library setup.
# Usage: scripts/project-rust.sh cargo build. Global defaults are unchanged.
set -euo pipefail
awl_project_rust_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if (( $# == 0 )); then
  echo "usage: scripts/project-rust.sh COMMAND [ARG...]" >&2
  exit 2
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "project-rust: install official rustup before building" >&2
  exit 1
fi
awl_project_rust_active="$(cd "$awl_project_rust_root" && rustup show active-toolchain)"
export AWL_PROJECT_RUST_ACTIVE="$awl_project_rust_root"
exec rustup run "${awl_project_rust_active%% *}" "$@"

#!/usr/bin/env bash
# Activate the committed project toolchain without changing rustup's default.
# Source from verification, or execute as: scripts/project-rust.sh cargo build.
# Resolving the binary also handles local cargo links that bypass rustup proxies.
awl_project_rust_activate() {
  local awl_project_rust_root awl_project_rust_cargo
  awl_project_rust_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || return 1
  if ! command -v rustup >/dev/null 2>&1; then
    echo "project-rust: install official rustup before building" >&2
    return 1
  fi
  (cd "$awl_project_rust_root" && rustup show active-toolchain >/dev/null) || return 1
  awl_project_rust_cargo="$(cd "$awl_project_rust_root" && rustup which cargo)" || return 1
  PATH="${awl_project_rust_cargo%/*}:$PATH"
  export PATH
}
if ! awl_project_rust_activate; then
  if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then exit 1; else return 1; fi
fi
unset -f awl_project_rust_activate
if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  if (( $# == 0 )); then
    echo "usage: scripts/project-rust.sh COMMAND [ARG...]" >&2
    exit 2
  fi
  exec "$@"
fi

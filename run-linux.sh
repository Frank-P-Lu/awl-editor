#!/usr/bin/env bash
# run-linux.sh — one-shot bootstrap to build & run awl-editor on Linux.
# Installs the system libs winit/wgpu/cosmic-text need, ensures Rust, then runs.
#
#   ./run-linux.sh                 # open samples/welcome.md
#   ./run-linux.sh path/to/file.md # open a specific file
#   ./run-linux.sh --release [f]   # optimized build (slower first compile, smoother)
#   SKIP_DEPS=1 ./run-linux.sh     # skip the system-package install step
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/linux-deps.sh
. scripts/linux-deps.sh

PROFILE=()
if [[ "${1:-}" == "--release" ]]; then PROFILE=(--release); shift; fi
FILE="${1:-samples/welcome.md}"

# The package names are NOT here — they live in scripts/linux-deps.sh, the one
# owner shared with both CI workflows, Dockerfile.linux and the README.txt that
# ships inside the download. This function only maps a package manager to a
# distro key and an install command.
#
# A from-source run needs every group: TOOLCHAIN to compile, BUILD for the
# headers, GPU + DIAG to actually run and to diagnose a black window, and
# RUNTIME where the distro declares it — that last one is what carries
# libxkbcommon-x11-0, the X11 keymap module winit dlopens at startup and which
# no `-dev` package pulls in.
install_deps() {
  [[ "${SKIP_DEPS:-0}" == "1" ]] && { echo "SKIP_DEPS=1 -> skipping system packages"; return; }
  echo "==> installing system dependencies (uses sudo)..."

  local distro
  local -a install
  if command -v apt-get >/dev/null 2>&1; then
    distro=DEB; install=(sudo apt-get install -y --no-install-recommends)
    sudo apt-get update -qq
  elif command -v dnf >/dev/null 2>&1; then
    distro=FEDORA; install=(sudo dnf install -y)
  elif command -v pacman >/dev/null 2>&1; then
    distro=ARCH; install=(sudo pacman -S --needed --noconfirm)
  elif command -v zypper >/dev/null 2>&1; then
    distro=SUSE; install=(sudo zypper install -y)
  else
    echo "!! Unknown package manager. Install manually: a C toolchain, pkg-config," >&2
    echo "   fontconfig, libxkbcommon, wayland, X11/xcb dev libs, the Vulkan loader," >&2
    echo "   and a Mesa Vulkan driver, then re-run with SKIP_DEPS=1." >&2
    return
  fi

  local -a groups=(TOOLCHAIN BUILD GPU DIAG)
  awl_deps_has_group "$distro" RUNTIME && groups+=(RUNTIME)

  local pkgs
  pkgs="$(awl_deps "$distro" "${groups[@]}")"
  # shellcheck disable=SC2086  # deliberate word-splitting: one package per word.
  "${install[@]}" $pkgs
}

ensure_rust() {
  command -v cargo >/dev/null 2>&1 || { [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"; }
  if ! command -v cargo >/dev/null 2>&1; then
    echo "==> installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  fi
  echo "==> $(cargo --version)"
}

install_deps
ensure_rust

echo "==> building + launching awl (${PROFILE[*]:-debug}) on: $FILE"
echo "    first build compiles wgpu/glyphon (a few minutes); later runs are instant."
exec cargo run "${PROFILE[@]}" -- "$FILE"

#!/usr/bin/env bash
#
# linux-build-deps.sh — install the Debian/Ubuntu packages awl needs, from the
# one owner in `scripts/linux-deps.sh`. The single entry point for CI's linux
# and AT-SPI jobs, release.yml's linux job, and Dockerfile.linux.
#
# Usage:
#   scripts/linux-build-deps.sh [--timeout DUR] <profile> [extra-package...]
#
#   profile  compile   pkg-config + the `-dev` headers. Enough to `cargo build`
#                      and nothing more — Dockerfile.linux's profile, which
#                      never runs the binary it produces.
#            test      compile + the Vulkan driver. Enough to RUN the suite
#                      headless against lavapipe — the linux/release profile.
#            run       test + the RUNTIME group. For an arm that puts a built
#                      awl in front of a REAL X server, where the dlopen of
#                      libxkbcommon-x11.so is actually reached. The extra
#                      non-dev libraries are already pulled in transitively by
#                      the `-dev` packages; naming them costs nothing and is
#                      what lets the AT-SPI job stop spelling a package name
#                      of its own (see linux-deps-law.sh).
#
#   --timeout DUR      wrap each apt-get in `timeout DUR`.
#
# ⚠️ THE SUDO/TIMEOUT ORDER IS THE WHOLE MECHANISM, and it is why --timeout is
# an argument here rather than something a caller wraps around this script.
# `timeout` kills by signalling the child it FORKED. Called as `sudo timeout 5m
# apt-get …`, that child is apt-get itself and the timer works. Called the
# other way (`timeout 5m sudo …`) the child is a setuid binary running as root,
# an unprivileged `timeout` signalling it gets EPERM, and the wrapper waits
# exactly as long as the stall it was added to bound. A wrapper that cannot
# fire is worse than none on a path this rare: it reads as protected for months
# and then does not work on the one run that needed it.
#
# Wrapping THIS SCRIPT in `sudo timeout` would reintroduce the same defect one
# level up — timeout's child would be bash, which does not forward the signal
# to the apt-get it is waiting on. So callers invoke this WITHOUT sudo and pass
# --timeout, and the exact `sudo timeout DUR apt-get …` command line is
# reassembled below.
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/linux-deps.sh
. scripts/linux-deps.sh

TIMEOUT=""
if [ "${1:-}" = "--timeout" ]; then
  TIMEOUT="${2:?--timeout needs a duration}"
  shift 2
fi

PROFILE="${1:-}"; shift || true
case "$PROFILE" in
  compile) groups=(BUILD) ;;
  test)    groups=(BUILD GPU) ;;
  run)     groups=(BUILD GPU RUNTIME) ;;
  *)
    echo "linux-build-deps: profile must be 'compile', 'test' or 'run' (got '${PROFILE:-}')" >&2
    exit 2
    ;;
esac

# Already root (Docker) → no sudo binary needed and none assumed present.
SUDO=()
[ "$(id -u)" -eq 0 ] || SUDO=(sudo)
TIMER=()
[ -z "$TIMEOUT" ] || TIMER=(timeout "$TIMEOUT")

# `awl_deps` fails loudly on an unknown group rather than echoing nothing, so a
# typo here cannot degrade into an apt-get that installs no packages and exits 0.
packages="$(awl_deps DEB "${groups[@]}")"

echo "==> apt-get: profile=$PROFILE${TIMEOUT:+ timeout=$TIMEOUT}"
echo "    $packages $*"

"${SUDO[@]}" "${TIMER[@]}" apt-get update
# shellcheck disable=SC2086  # deliberate word-splitting: one package per word.
"${SUDO[@]}" "${TIMER[@]}" apt-get install -y --no-install-recommends $packages "$@"

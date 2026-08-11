#!/usr/bin/env bash
#
# linux-deps-law.sh — THE LAW that keeps scripts/linux-deps.sh the only place
# awl's Linux package names are written down.
#
# WHY THIS IS A SHELL SCRIPT AND NOT A RUST TEST. Every other law in this tree
# is a `#[test]`, because every other law's subject is Rust. This one's subject
# is two YAML workflows, two Dockerfiles and three shell scripts — none of which
# a Rust test can see. code-health.sh is the only blocking gate that can, so
# that is where it runs.
#
# WHAT IT ASSERTS. No consumer file spells a package name that linux-deps.sh
# declares. The install sites compose GROUPS; only the data file names packages.
# That is the "make the bypass module-private" shape: a new dependency has
# exactly one place it can be added, and a copy pasted back into a workflow
# fails the gate by name.
#
# ENROLMENT IS DERIVED FROM THE ROSTER, not from a hand-kept list of sentinel
# packages — a law pinned to one named member stops sweeping the moment the
# roster changes around it. Every name in every group of every distro is read
# out of linux-deps.sh at run time.
#
# ⚠️ WHAT IT DOES NOT COVER, stated out loud rather than left for a reader to
# discover. Some roster entries are ordinary English words that legitimately
# appear in prose inside the very files being scanned — run-linux.sh's
# "install manually: … fontconfig, libxkbcommon, wayland …" fallback message is
# the live example, and Arch's roster is almost entirely such names because
# Arch ships headers in the library package. Grepping for those would fire on
# prose forever, so enrolment is narrowed to PACKAGE-SHAPED names: ones bearing
# a digit or a packaging suffix (-dev, -devel, -drivers, -driver, -loader,
# -tools). The run prints both sets, so what went unswept is visible in the log
# rather than implied by a green tick.
#
# Run standalone to see the coverage report:  scripts/linux-deps-law.sh
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/linux-deps.sh
. scripts/linux-deps.sh

# The install sites. Prose docs (README.md, RELEASING.md) are deliberately NOT
# here: a sentence naming a library is not a package list, and holding prose to
# this law would make the law's own failure message a lie.
CONSUMERS=(
  .github/workflows/ci.yml
  .github/workflows/release.yml
  Dockerfile.linux
  run-linux.sh
  scripts/linux-build-deps.sh
  scripts/oom-budget-container.sh
  scripts/package-linux.sh
)

# NOT `GROUPS`: that is a bash built-in array holding the invoking user's group
# IDs, and assigning to it is silently ignored — which turns the sweep below
# into a loop over numeric gids that matches no group name and enrols nothing.
# The law caught its own version of that (it aborted rather than passing), but
# the failure read as a bash error, not as "this law swept zero packages".
DEP_GROUPS=(TOOLCHAIN BUILD GPU DIAG RUNTIME)

# Collect every declared package name across the whole roster.
all_names() {
  local d g
  for d in "${AWL_DEPS_DISTROS[@]}"; do
    for g in "${DEP_GROUPS[@]}"; do
      awl_deps_has_group "$d" "$g" || continue
      awl_deps "$d" "$g"
    done
  done | tr ' ' '\n' | sort -u | sed '/^$/d'
}

# Package-shaped: carries a digit or a packaging suffix. Everything else is a
# bare word that prose can legitimately contain.
is_package_shaped() {
  case "$1" in
    *[0-9]*) return 0 ;;
    *-dev|*-devel|*-drivers|*-driver|*-loader|*-tools) return 0 ;;
    *) return 1 ;;
  esac
}

swept=() unswept=()
while read -r name; do
  [ -n "$name" ] || continue
  if is_package_shaped "$name"; then swept+=("$name"); else unswept+=("$name"); fi
done <<EOF
$(all_names)
EOF

# A sweep of nothing is the failure mode this whole file exists to prevent —
# and it is silent by nature, because grepping for zero names finds zero
# violations and prints a tick. bash 3.2 also errors on "${empty[@]}" under
# `set -u`, so this guard is load-bearing twice over.
if [ "${#swept[@]}" -eq 0 ]; then
  echo "linux-deps-law: FAILED — the roster yielded no package-shaped names." >&2
  echo "  Either scripts/linux-deps.sh stopped declaring groups, or this script's" >&2
  echo "  group list drifted from it. A zero-name sweep passes vacuously; refusing." >&2
  exit 1
fi

# THE SUBJECT IS AN INSTALL INVOCATION, NOT A MENTION.
#
# The first draft of this law grepped whole files, and every hit it produced was
# a false one: a comment explaining WHY libxkbcommon-x11-0 is separate, and the
# shipped README.txt sentence telling a user which package carries it. Both name
# the package legitimately — documentation is not a second install list, and a
# law that forces prose to stop naming the thing it is explaining would make the
# tree worse to read. The defect this law exists to catch is narrower and
# sharper: a SECOND PLACE THAT INSTALLS. So the sweep is restricted to install
# invocations and their backslash continuations, which is exactly where a
# duplicated list can do harm.
#
# install_lines <file> — emit "line:text" for every install-invocation line.
install_lines() {
  awk '
    # A comment can quote an install command while installing nothing.
    /^[ \t]*#/ { next }
    cont {
      print FNR ": " $0
      if ($0 !~ /\\[ \t]*$/) cont = 0
      next
    }
    /(apt-get|apt|dnf|zypper)[ \t]+install|pacman[ \t]+-S/ {
      print FNR ": " $0
      if ($0 ~ /\\[ \t]*$/) cont = 1
    }
  ' "$1"
}

# scan_files <file...> — echo every "file:line:text" violation found.
scan_files() {
  local f name lines
  for f in "$@"; do
    [ -f "$f" ] || { echo "linux-deps-law: consumer not found: $f" >&2; return 2; }
    lines="$(install_lines "$f")"
    [ -n "$lines" ] || continue
    for name in "${swept[@]}"; do
      # -w so libxkbcommon0 does not match inside libxkbcommon-x11-0, and -F so
      # a name containing '+' or '.' is matched literally.
      #
      # `|| true` is REQUIRED, not defensive noise: `set -o pipefail` is on, and
      # a grep that finds nothing exits 1, which would abort the whole law on
      # the first clean file — i.e. the law would fail precisely when the tree
      # is correct, and could only ever "pass" by aborting early.
      { printf '%s\n' "$lines" | grep -nwF -- "$name" || true; } \
        | while IFS= read -r hit; do
            printf '%s: %s\n' "$f" "${hit#*:}"
          done
    done
  done
  return 0
}

# sort -u: one offending LINE is reported once, not once per package name on
# it — a pasted-back list would otherwise repeat the same line five times and
# bury the other files under it.
violations="$(scan_files "${CONSUMERS[@]}" | sort -u)"

echo "linux-deps-law: swept ${#swept[@]} package-shaped names across ${#CONSUMERS[@]} consumers."
echo "linux-deps-law: NOT swept (prose-ambiguous, bare words): ${unswept[*]:-none}"

if [ -n "$violations" ]; then
  echo >&2
  echo "linux-deps-law: FAILED — a package name owned by scripts/linux-deps.sh is" >&2
  echo "  written out in a consumer. Compose a GROUP instead of naming the package:" >&2
  echo >&2
  printf '%s\n' "$violations" | sed 's/^/    /' >&2
  echo >&2
  echo "  If the package genuinely belongs to one job alone and to no distro roster" >&2
  echo "  (the AT-SPI test rig is the precedent), it may be passed as an extra arg —" >&2
  echo "  but then it must not be declared in linux-deps.sh at all." >&2
  exit 1
fi

# NON-VACUITY. A law that cannot fail is not a law, and this one's assertion is
# a grep over files it does not control — exactly the shape that goes quietly
# vacuous when a path changes or a roster empties. So break the product on
# purpose, in a scratch copy, and require the scan to catch it. Without this,
# an empty roster or a typo'd CONSUMERS entry would report a serene pass.
probe="$(mktemp -d)"
trap 'rm -rf "$probe"' EXIT
planted="$(awl_deps DEB BUILD | tr ' ' '\n' | grep -m1 -- '-dev' || true)"
if [ -z "$planted" ]; then
  echo "linux-deps-law: FAILED — the roster yielded no -dev package to plant." >&2
  echo "  The sweep above therefore proved nothing about a roster this small." >&2
  exit 1
fi
printf 'RUN apt-get install -y %s\n' "$planted" > "$probe/Dockerfile.planted"
if scan_files "$probe/Dockerfile.planted" | grep -q "$planted"; then
  echo "linux-deps-law: non-vacuity OK (a planted '$planted' is caught)."
else
  echo "linux-deps-law: FAILED — a deliberately planted '$planted' was NOT caught." >&2
  echo "  The sweep is not discriminating; a green run above means nothing." >&2
  exit 1
fi

echo "linux-deps-law: OK — linux-deps.sh is the only place these names are written."

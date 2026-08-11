#!/usr/bin/env bash
#
# package-linux.sh — assemble the Linux download: `awl-<version>-linux-x86_64.tar.gz`
# plus its checksum, from an already-built Linux `awl` binary.
#
# Usage:
#   scripts/package-linux.sh <path-to-linux-binary> <output-dir>
#
# Produces (VERSION resolved as described below):
#   <output-dir>/awl-<version>-linux-x86_64/            the staged payload
#   <output-dir>/awl-<version>-linux-x86_64.tar.gz      the download
#   <output-dir>/awl-<version>-linux-x86_64.tar.gz.sha256   its checksum, a build product
#
# WHY A SCRIPT. The layout used to be eight inline `cp` lines in
# release.yml's linux job, which meant it could only ever be exercised by
# pushing to CI, and its licence copies were `cp … 2>/dev/null || true` —
# a tarball missing LICENSE would have published silently. One owner, run
# identically in CI and locally, mirroring scripts/package-macos.sh.
#
# LICENSING IS A HARD FAILURE HERE, not a warning. awl is GPL-3.0-only and
# the binary statically embeds third-party assets: the fonts (SIL OFL 1.1)
# and the Hunspell dictionaries (LGPL-2.1 for en_GB; SCOWL permissive +
# Ispell BSD for en_US/en_AU) are `include_bytes!`d at compile time, so
# their licence texts must travel with the binary that contains them. The
# macOS bundle's Resources/ copies are the same set. A missing file exits
# non-zero rather than shipping an under-licensed archive.
#
# ARCHIVE SHAPE. One top-level directory (`awl-<version>-linux-x86_64/`) —
# never a tarbomb, never a leading `./`, never an absolute path. Entries are
# owned by uid/gid 0 with numeric ownership so a `tar xzf` as any user lands
# 0755 on the binary and 0644 on the docs, and are sorted by name with a
# fixed mtime under GNU tar so two builds of the same tree produce the same
# archive bytes.
#
# VERSION. Read from `$AWL_VERSION` if set (release.yml's linux job passes
# the tag-derived version, exactly like package-macos.sh's own AWL_VERSION);
# otherwise from Cargo.toml's package.version via `cargo metadata`, the same
# fallback package-macos.sh uses, so a bare local invocation still produces a
# correctly-named archive. This is the ONE place the Linux archive name is
# assembled — nothing else hardcodes it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/linux-deps.sh
. "$ROOT/scripts/linux-deps.sh"

if [ "$#" -ne 2 ]; then
  echo "usage: scripts/package-linux.sh <path-to-linux-binary> <output-dir>" >&2
  exit 2
fi

BINARY="$1"
OUTDIR="$2"

AWL_VERSION="${AWL_VERSION:-}"
if [ -z "$AWL_VERSION" ]; then
  if command -v cargo >/dev/null 2>&1; then
    AWL_VERSION="$(cd "$ROOT" && cargo metadata --no-deps --format-version 1 2>/dev/null \
      | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)"
  fi
  AWL_VERSION="${AWL_VERSION:-0.0.0}"
fi

STAGE_NAME="awl-${AWL_VERSION}-linux-x86_64"
TARBALL="$STAGE_NAME.tar.gz"

if [ ! -f "$BINARY" ]; then
  echo "package-linux: no such binary: $BINARY" >&2
  exit 1
fi

mkdir -p "$OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"
STAGE="$OUTDIR/$STAGE_NAME"
rm -rf "${STAGE:?}" "${OUTDIR:?}/$TARBALL" "${OUTDIR:?}/$TARBALL.sha256"
mkdir -p "$STAGE/licenses"

install -m 0755 "$BINARY" "$STAGE/awl"

# The four repo-root licence docs, then the two bundled-asset audits. Both
# groups are required; see this file's header for why the asset pair is not
# optional when the assets are compiled into the binary.
for doc in LICENSE NOTICE CREDITS.md THIRD-PARTY-LICENSES.md; do
  if [ ! -f "$ROOT/$doc" ]; then
    echo "package-linux: required licence doc missing: $doc" >&2
    exit 1
  fi
  install -m 0644 "$ROOT/$doc" "$STAGE/$doc"
done

for pair in fonts dict; do
  src="$ROOT/assets/$pair/LICENSES.md"
  if [ ! -f "$src" ]; then
    echo "package-linux: required bundled-asset licence missing: assets/$pair/LICENSES.md" >&2
    exit 1
  fi
  install -m 0644 "$src" "$STAGE/licenses/$pair-LICENSES.md"
done

cat > "$STAGE/README.txt" <<'TXT'
awl — a calm, opinionated plain-text editor for prose and light code.

UNPACK AND RUN

    tar xzf awl-linux-x86_64.tar.gz
    cd awl-linux-x86_64
    ./awl                 # a scratch buffer
    ./awl notes.md        # open a file

The archive unpacks into one directory. Nothing is installed and nothing is
written outside your own config and data directories. To put awl on your
PATH, move or symlink the binary:

    install -Dm755 awl ~/.local/bin/awl

WHAT IT NEEDS

    x86_64                a 64-bit Intel/AMD CPU
    glibc                 GLIBC.txt names the exact version this build needs.
                          Check yours with `ldd --version`. An older system
                          cannot run this binary and says so by name.
    Vulkan                a working Vulkan 1.x driver (Mesa covers most GPUs)
    fontconfig, libxkbcommon, and the Wayland or X11 client libraries

Fonts and dictionaries are compiled into the binary; nothing is downloaded
at runtime, ever. Install the runtime libraries with your package manager:

TXT

# THE RUNTIME TABLE IS GENERATED, from the same scripts/linux-deps.sh that
# drives CI, Dockerfile.linux and the from-source bootstrap — so the names a
# user is told to install cannot drift from the ones awl is actually built and
# tested against. That drift is not hypothetical: libxkbcommon-x11-0 had to be
# hand-applied to this heredoc, run-linux.sh and ci.yml in one round.
#
# ENROLMENT IS DERIVED, not listed here: a distro appears iff it declares a
# RUNTIME group. openSUSE deliberately does not (its non-dev names have never
# been verified on a real box), so it stays out of this table without anything
# needing to remember to exclude it — and adding a verified array is the whole
# of what enrolling it takes. Generating a document moves the error from
# transcription to sourcing, so unverified names are ABSENT rather than guessed.
for distro in $(awl_deps_runtime_distros); do
  label="AWL_${distro}_LABEL"
  install_cmd="AWL_${distro}_INSTALL"
  printf '    %-16s%s %s\n' "${!label}" "${!install_cmd}" "$(awl_deps "$distro" RUNTIME)"
done >> "$STAGE/README.txt"

cat >> "$STAGE/README.txt" <<'TXT'

    On X11 sessions specifically, winit dlopens libxkbcommon-x11.so at
    startup; Debian/Ubuntu ship it in the separate libxkbcommon-x11-0
    package above (confirmed: awl panics naming that exact library without
    it). Fedora/Arch package it differently and are unconfirmed here — if
    awl reports a missing libxkbcommon-x11.so on either, install that
    distro's equivalent package.

CHECK IT WORKS WITHOUT A WINDOW

    ./awl --screenshot /tmp/awl.png

That renders one frame headlessly and writes /tmp/awl.png plus a
/tmp/awl.json state sidecar. If that succeeds, the graphics stack is fine.

LICENCE

awl is free software under the GNU General Public License, version 3 only.
The full text is in LICENSE; NOTICE names the copyright holder. The complete
corresponding source for this binary is the public repository:

    https://github.com/Frank-P-Lu/awl-editor

THIRD-PARTY-LICENSES.md lists every Rust crate compiled in. licenses/ holds
the audits for the assets embedded in the binary: fonts-LICENSES.md (SIL Open
Font License 1.1) and dict-LICENSES.md (the Hunspell dictionaries).
CREDITS.md is the human-readable thank-you.
TXT
chmod 0644 "$STAGE/README.txt"

# The heredoc above is single-quote-delimited (a literal `ldd --version` in
# its "WHAT IT NEEDS" section must never be shell-expanded as a backtick
# command substitution), so the unpack instructions are written with the
# unversioned placeholder and patched to the real, versioned name here.
sed -i.bak "s/awl-linux-x86_64/$STAGE_NAME/g" "$STAGE/README.txt"
rm -f "$STAGE/README.txt.bak"

# The glibc floor is a property of the machine that built this binary, not a
# constant, so it is recorded rather than asserted. `objdump -T` lists the
# versioned symbol references the dynamic linker must satisfy.
if command -v objdump >/dev/null 2>&1 && objdump -T "$STAGE/awl" >/dev/null 2>&1; then
  versions="$(objdump -T "$STAGE/awl" 2>/dev/null \
    | sed -n 's/.*GLIBC_\([0-9][0-9.]*\).*/\1/p' \
    | sort -u -t. -k1,1n -k2,2n -k3,3n)"
  floor="$(printf '%s\n' "$versions" | tail -1)"
  {
    echo "This build requires glibc $floor or newer."
    echo
    echo "That is the highest versioned symbol it references, so a system with an"
    echo "older glibc refuses to start it — the error names the version:"
    echo "  libc.so.6: version \`GLIBC_$floor' not found"
    echo "Check yours with \`ldd --version\`. A too-old system needs a build from"
    echo "source; see the repository."
    echo
    echo "Every glibc version referenced:"
    printf '%s\n' "$versions" | sed 's/^/  GLIBC_/'
  } > "$STAGE/GLIBC.txt"
  chmod 0644 "$STAGE/GLIBC.txt"
  echo "==> glibc floor: $floor"
else
  echo "package-linux: objdump cannot read $STAGE/awl — no GLIBC.txt in this archive" >&2
fi

# Reproducible ordering/ownership where tar supports it (GNU tar on Linux,
# which is where a release is built); bsdtar on a developer's mac still
# produces a correct archive, just not a byte-reproducible one.
TAR_FLAGS=(--owner=0 --group=0 --numeric-owner)
if tar --version 2>/dev/null | grep -q "GNU tar"; then
  TAR_FLAGS+=(--sort=name --mtime=@0 --format=gnu)
else
  echo "package-linux: not GNU tar — archive will not be byte-reproducible" >&2
fi

tar -czf "$OUTDIR/$TARBALL" "${TAR_FLAGS[@]}" -C "$OUTDIR" "$STAGE_NAME"

# Checksum as a BUILD PRODUCT: it rides beside the artifact from the moment
# the artifact exists, so the value a user verifies was never typed by hand.
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$OUTDIR" && sha256sum "$TARBALL" > "$TARBALL.sha256")
else
  (cd "$OUTDIR" && shasum -a 256 "$TARBALL" > "$TARBALL.sha256")
fi

echo "==> $OUTDIR/$TARBALL"
cat "$OUTDIR/$TARBALL.sha256"
tar -tzvf "$OUTDIR/$TARBALL"

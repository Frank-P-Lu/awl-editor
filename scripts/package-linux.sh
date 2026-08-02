#!/usr/bin/env bash
#
# package-linux.sh — assemble the Linux download: `awl-linux-x86_64.tar.gz`
# plus its checksum, from an already-built Linux `awl` binary.
#
# Usage:
#   scripts/package-linux.sh <path-to-linux-binary> <output-dir>
#
# Produces:
#   <output-dir>/awl-linux-x86_64/            the staged payload
#   <output-dir>/awl-linux-x86_64.tar.gz      the download
#   <output-dir>/awl-linux-x86_64.tar.gz.sha256   its checksum, a build product
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
# ARCHIVE SHAPE. One top-level directory (`awl-linux-x86_64/`) — never a
# tarbomb, never a leading `./`, never an absolute path. Entries are owned
# by uid/gid 0 with numeric ownership so a `tar xzf` as any user lands
# 0755 on the binary and 0644 on the docs, and are sorted by name with a
# fixed mtime under GNU tar so two builds of the same tree produce the same
# archive bytes.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$#" -ne 2 ]; then
  echo "usage: scripts/package-linux.sh <path-to-linux-binary> <output-dir>" >&2
  exit 2
fi

BINARY="$1"
OUTDIR="$2"
STAGE_NAME="awl-linux-x86_64"
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
    glibc                 see GLIBC.txt for the exact floor this build needs
    Vulkan                a working Vulkan 1.x driver (Mesa covers most GPUs)
    fontconfig, libxkbcommon, and the Wayland or X11 client libraries

Fonts and dictionaries are compiled into the binary; nothing is downloaded
at runtime, ever. Install the runtime libraries with your package manager:

    Debian/Ubuntu   sudo apt install libfontconfig1 libxkbcommon0 libvulkan1 mesa-vulkan-drivers
    Fedora          sudo dnf install fontconfig libxkbcommon vulkan-loader mesa-vulkan-drivers
    Arch            sudo pacman -S fontconfig libxkbcommon vulkan-icd-loader mesa

CHECK IT WORKS WITHOUT A WINDOW

    ./awl --screenshot /tmp/awl.png

That renders one frame headlessly and writes /tmp/awl.png plus a
/tmp/awl.json state sidecar. If that succeeds, the graphics stack is fine.

LICENCE

awl is free software under the GNU General Public License, version 3 only.
The full text is in LICENSE; NOTICE names the copyright holder. The complete
corresponding source for this binary is the public repository:

    https://github.com/Frank-P-Lu/awl-next

THIRD-PARTY-LICENSES.md lists every Rust crate compiled in. licenses/ holds
the audits for the assets embedded in the binary: fonts-LICENSES.md (SIL Open
Font License 1.1) and dict-LICENSES.md (the Hunspell dictionaries).
CREDITS.md is the human-readable thank-you.
TXT
chmod 0644 "$STAGE/README.txt"

# The glibc floor is a property of the machine that built this binary, not a
# constant, so it is recorded rather than asserted. `objdump -T` lists the
# versioned symbol references the dynamic linker must satisfy.
if command -v objdump >/dev/null 2>&1 && objdump -T "$STAGE/awl" >/dev/null 2>&1; then
  {
    echo "Minimum glibc for this build, from the versioned symbols it references."
    echo
    objdump -T "$STAGE/awl" 2>/dev/null \
      | sed -n 's/.*GLIBC_\([0-9][0-9.]*\).*/\1/p' \
      | sort -u -t. -k1,1n -k2,2n -k3,3n \
      | sed 's/^/  GLIBC_/'
  } > "$STAGE/GLIBC.txt"
  chmod 0644 "$STAGE/GLIBC.txt"
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

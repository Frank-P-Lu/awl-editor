#!/usr/bin/env bash
#
# package-appimage.sh — assemble awl's AppImage: the binary, a `.desktop`
# launcher, the canonical Linux icon, and the license set, into an AppDir;
# then (Linux x86_64 only) cut that AppDir into one self-contained
# `awl-<version>-linux-x86_64.AppImage` plus its checksum.
#
# Usage:
#   scripts/package-appimage.sh <path-to-linux-x86_64-binary> <output-dir> [--assemble-only]
#
# Produces (VERSION resolved exactly as scripts/package-linux.sh resolves it):
#   <output-dir>/AppDir/                                       the staged AppImage payload
#   <output-dir>/awl-<version>-linux-x86_64.AppImage           the download (skipped by --assemble-only,
#   <output-dir>/awl-<version>-linux-x86_64.AppImage.sha256     and its checksum   or on a non-Linux/non-x86_64 host)
#
# TWO PHASES, DELIBERATELY SPLIT — the first is pure bash + a `cargo run` of
# THIS repo's own binary, so it runs anywhere (including a macOS dev
# machine) and is where the structural law below lives:
#
#   1. ASSEMBLE  the AppDir.                              always runs.
#   2. CUT       the AppDir into a single-file .AppImage.  Linux x86_64 only —
#                appimagetool's own upstream binary is a Linux x86_64 ELF and
#                cannot execute anywhere else. `--assemble-only`, or a host
#                that fails that check, skips this phase with a clear,named
#                reason rather than a confusing exec failure three steps down.
#
# THE ICON is not a second hand-drawn asset: `awl --export-linux-icon` (added
# alongside this script, item 227) cuts the 256px PNG straight out of the
# committed canonical `assets/macos/Awl.icns` via the SAME `app_icon::icns`
# parser the macOS icon law tests use as their oracle — the artwork Finder and
# the Dock already show. Run through `cargo run --release` on whatever host
# runs this script (never by executing the target BINARY, which may be a
# foreign-arch ELF this host cannot run) — the icns bytes on disk carry no
# target-platform dependency, so the extraction is host-native and the result
# is byte-identical regardless of which host produced it.
#
# ONLY THE BINARY TRAVELS — NO BUNDLED SHARED LIBRARIES, DELIBERATELY. Every
# runtime dependency awl links against is either (a) part of the base desktop
# stack a normal Linux install already has — glibc, fontconfig, libxkbcommon,
# the X11/Wayland client libraries (scripts/package-linux.sh's tarball
# documents the exact package names per distro, and this AppImage inherits
# the same expectation) — or (b) a GPU-adapter-specific library: the Vulkan
# loader and ICD/driver. Bundling (b) is explicitly out of scope (item 227's
# own brief: "do not bundle GPU drivers") because a bundled driver would be
# wrong for whatever GPU the AppImage actually runs against; bundling (a)
# would just duplicate what's already conventionally present and, unlike a
# driver, is never hardware-specific. AppImage's own traction is against
# exotic APP-specific libraries, and awl carries none — its only embedded
# dependencies (fonts, dictionaries) are already `include_bytes!`d into the
# binary itself, not separate shared objects.
#
# LICENSING IS A HARD FAILURE HERE, same rule and same set as
# scripts/package-linux.sh (see that script's header for why): the fonts
# (SIL OFL 1.1) and Hunspell dictionaries (LGPL-2.1 / SCOWL+Ispell) are
# `include_bytes!`d into the binary this AppDir carries, so their licence
# texts travel with it. A missing file exits non-zero rather than shipping an
# under-licensed archive.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

ASSEMBLE_ONLY=0
POSITIONAL=()
for arg in "$@"; do
  case "$arg" in
    --assemble-only) ASSEMBLE_ONLY=1 ;;
    -h|--help)
      sed -n '2,45p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) POSITIONAL+=("$arg") ;;
  esac
done

if [ "${#POSITIONAL[@]}" -ne 2 ]; then
  echo "usage: scripts/package-appimage.sh <path-to-linux-x86_64-binary> <output-dir> [--assemble-only]" >&2
  exit 2
fi
BINARY="${POSITIONAL[0]}"
OUTDIR="${POSITIONAL[1]}"

if [ ! -f "$BINARY" ]; then
  echo "package-appimage: no such binary: $BINARY" >&2
  exit 1
fi

AWL_VERSION="${AWL_VERSION:-}"
if [ -z "$AWL_VERSION" ]; then
  if command -v cargo >/dev/null 2>&1; then
    AWL_VERSION="$(cd "$ROOT" && cargo metadata --no-deps --format-version 1 2>/dev/null \
      | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)"
  fi
  AWL_VERSION="${AWL_VERSION:-0.0.0}"
fi

# Same reverse-DNS identifier package-macos.sh defaults `AWL_BUNDLE_ID` to —
# one identity across both packagers rather than a second, Linux-only name
# invented here. freedesktop's desktop-entry spec recommends a reverse-DNS
# basename for exactly the collision reason a Java-style bundle id exists on
# macOS: two different "Awl"s on one system must not shadow each other.
APP_ID="${AWL_BUNDLE_ID:-dev.franklu.awl}"

mkdir -p "$OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"
APPDIR="$OUTDIR/AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" \
         "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/icons/hicolor/256x256/apps" \
         "$APPDIR/usr/share/doc/awl/licenses"

echo "==> assembling AppDir (version $AWL_VERSION, id $APP_ID)"

install -m 0755 "$BINARY" "$APPDIR/usr/bin/awl"

# AppRun: a plain symlink to the binary. awl needs no environment setup, no
# data-file location fixup and no wrapper logic before it can run — the
# fonts/dictionaries it needs are compiled in, and it takes its config/data
# roots from the ordinary XDG lookups a symlinked invocation still performs
# correctly (argv[0] is irrelevant to that lookup). The official AppImage
# spec names a symlink to the main binary as a valid AppRun; a wrapper script
# would be unexplained complexity with nothing for it to do.
ln -s usr/bin/awl "$APPDIR/AppRun"

# --- ICON: cut from the existing pipeline, not hand-drawn -------------------
# `cargo run --release` on THIS host (never the target $BINARY, which may be
# a foreign-arch ELF this host cannot execute) — see the module doc.
if ! command -v cargo >/dev/null 2>&1; then
  echo "package-appimage: cargo not found on PATH (needed for --export-linux-icon)" >&2
  exit 1
fi
ICON_PNG="$APPDIR/$APP_ID.png"
(cd "$ROOT" && cargo run --quiet --release -- --export-linux-icon "$ICON_PNG")
cp "$ICON_PNG" "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"

# --- .desktop launcher --------------------------------------------------
DESKTOP="$APPDIR/$APP_ID.desktop"
cat > "$DESKTOP" <<DESKTOP_EOF
[Desktop Entry]
Type=Application
Name=Awl
GenericName=Text Editor
Comment=A calm, opinionated plain-text editor for prose and light code
Exec=awl %F
Icon=$APP_ID
Categories=Utility;TextEditor;Development;
Terminal=false
StartupWMClass=awl
MimeType=text/plain;text/markdown;
DESKTOP_EOF
cp "$DESKTOP" "$APPDIR/usr/share/applications/$APP_ID.desktop"

# --- Licensing: same required set as scripts/package-linux.sh --------------
for doc in LICENSE NOTICE CREDITS.md THIRD-PARTY-LICENSES.md; do
  if [ ! -f "$ROOT/$doc" ]; then
    echo "package-appimage: required licence doc missing: $doc" >&2
    exit 1
  fi
  install -m 0644 "$ROOT/$doc" "$APPDIR/usr/share/doc/awl/$doc"
done
for pair in fonts dict; do
  src="$ROOT/assets/$pair/LICENSES.md"
  if [ ! -f "$src" ]; then
    echo "package-appimage: required bundled-asset licence missing: assets/$pair/LICENSES.md" >&2
    exit 1
  fi
  install -m 0644 "$src" "$APPDIR/usr/share/doc/awl/licenses/$pair-LICENSES.md"
done

cat > "$APPDIR/usr/share/doc/awl/README.txt" <<'TXT'
awl — a calm, opinionated plain-text editor for prose and light code.

This is the AppImage build: one self-contained file. Mark it executable and
run it; nothing is installed.

    chmod +x awl-*-linux-x86_64.AppImage
    ./awl-*-linux-x86_64.AppImage notes.md

Double-clicking it in most file managers works too, once it is executable.

If it refuses to start with a FUSE-related error (some newer distros, e.g.
Ubuntu 22.04+, don't ship libfuse2 by default), run it extracted instead —
no FUSE needed:

    ./awl-*-linux-x86_64.AppImage --appimage-extract-and-run notes.md

The tar.gz release asset remains available as a fallback too.

awl is free software under the GNU General Public License, version 3 only.
The full text is in LICENSE; NOTICE names the copyright holder. The complete
corresponding source for this binary is the public repository:

    https://github.com/Frank-P-Lu/awl-editor

THIRD-PARTY-LICENSES.md lists every Rust crate compiled in. licenses/ holds
the audits for the assets embedded in the binary: fonts-LICENSES.md (SIL Open
Font License 1.1) and dict-LICENSES.md (the Hunspell dictionaries).
CREDITS.md is the human-readable thank-you.
TXT
chmod 0644 "$APPDIR/usr/share/doc/awl/README.txt"

# --- THE STRUCTURAL LAW: launcher/icon packaging, verified by name ---------
# Every check below names exactly what it checks. A missing or malformed
# piece fails HERE, loudly, rather than shipping an AppImage some desktop
# environments silently refuse to integrate (no icon, no name, no launcher).
verify_appdir() {
  local fail=0
  if [ ! -L "$APPDIR/AppRun" ] && [ ! -x "$APPDIR/AppRun" ]; then
    echo "!! appdir: AppRun is missing or not executable/symlinked" >&2
    fail=1
  fi
  local desktops
  shopt -s nullglob
  desktops=("$APPDIR"/*.desktop)
  shopt -u nullglob
  if [ "${#desktops[@]}" -ne 1 ]; then
    echo "!! appdir: expected exactly one *.desktop at the AppDir root, found ${#desktops[@]}" >&2
    fail=1
  else
    for key in "Type=Application" "Name=" "Exec=" "Icon="; do
      if ! grep -q "^${key}" "${desktops[0]}"; then
        echo "!! appdir: ${desktops[0]} missing required key '$key'" >&2
        fail=1
      fi
    done
  fi
  if [ ! -f "$ICON_PNG" ]; then
    echo "!! appdir: missing root icon $ICON_PNG" >&2
    fail=1
  else
    local magic
    magic="$(od -An -tx1 -N8 "$ICON_PNG" | tr -d ' \n')"
    if [ "$magic" != "89504e470d0a1a0a" ]; then
      echo "!! appdir: $ICON_PNG is not a valid PNG (bad magic bytes)" >&2
      fail=1
    fi
  fi
  if [ ! -f "$APPDIR/usr/share/icons/hicolor/256x256/apps/$APP_ID.png" ]; then
    echo "!! appdir: missing hicolor theme copy of the icon" >&2
    fail=1
  fi
  if [ ! -f "$APPDIR/usr/share/applications/$APP_ID.desktop" ]; then
    echo "!! appdir: missing usr/share/applications copy of the desktop entry" >&2
    fail=1
  fi
  for doc in LICENSE NOTICE CREDITS.md THIRD-PARTY-LICENSES.md; do
    [ -f "$APPDIR/usr/share/doc/awl/$doc" ] || { echo "!! appdir: missing $doc" >&2; fail=1; }
  done
  if [ "$fail" -ne 0 ]; then
    echo "!! appdir verification FAILED for $APPDIR" >&2
    return 1
  fi
  echo "==> appdir OK: AppRun, $APP_ID.desktop (+ Name/Exec/Icon/Type), $APP_ID.png (root + hicolor), licences"
}
verify_appdir

echo "==> AppDir assembled: $APPDIR"

if [ "$ASSEMBLE_ONLY" -eq 1 ]; then
  echo "==> --assemble-only: skipping the .AppImage cut"
  exit 0
fi

# --- CUT: AppDir -> single-file .AppImage (Linux x86_64 only) --------------
if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
  echo "==> skipping the .AppImage cut: appimagetool is a Linux x86_64 ELF (host is $(uname -s) $(uname -m))" >&2
  echo "    the AppDir above is still fully assembled and structurally verified." >&2
  exit 0
fi

APPRUN_TOOL="${APPIMAGETOOL_APPRUN:-}"
if [ -z "$APPRUN_TOOL" ]; then
  APPRUN_TOOL="$("$ROOT/scripts/install-appimagetool.sh" | tail -1)"
fi
if [ ! -x "$APPRUN_TOOL" ]; then
  echo "package-appimage: appimagetool AppRun not found/executable at $APPRUN_TOOL" >&2
  exit 1
fi

APPIMAGE_NAME="awl-${AWL_VERSION}-linux-x86_64.AppImage"
rm -f "$OUTDIR/$APPIMAGE_NAME" "$OUTDIR/$APPIMAGE_NAME.sha256"

echo "==> cutting $APPIMAGE_NAME"
ARCH=x86_64 "$APPRUN_TOOL" --no-appstream "$APPDIR" "$OUTDIR/$APPIMAGE_NAME"
chmod +x "$OUTDIR/$APPIMAGE_NAME"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$OUTDIR" && sha256sum "$APPIMAGE_NAME" > "$APPIMAGE_NAME.sha256")
else
  (cd "$OUTDIR" && shasum -a 256 "$APPIMAGE_NAME" > "$APPIMAGE_NAME.sha256")
fi

echo "==> $OUTDIR/$APPIMAGE_NAME"
cat "$OUTDIR/$APPIMAGE_NAME.sha256"

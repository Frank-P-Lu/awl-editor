#!/usr/bin/env bash
#
# linux-deps.sh — THE one owner of awl's Linux system-package names.
#
# SOURCED, NEVER EXECUTED. It defines data and two helpers; it installs
# nothing and prints nothing on its own.
#
# WHY THIS FILE EXISTS. These names used to live in eight places: two
# workflow files, two Dockerfiles (one of them a heredoc), the from-source
# bootstrap, and the shipped README.txt — several of them carrying a comment
# CLAIMING identity with another copy ("Same apt-get list as ci.yml's linux
# job", "on Dockerfile.linux's apt list plus …") that nothing enforced. The
# claim was already load-bearing and already fragile: `libxkbcommon-x11-0` had
# to be hand-applied to three of them in one round, and the release build's
# list was byte-identical to the tested one only by luck, with no mechanism
# tying them together. `linux-deps-law.sh` is now that mechanism.
#
# THE GROUPS, and why the split is where it is. A consumer composes the groups
# it needs; the union is order-preserving and de-duplicated, so groups may
# legitimately overlap (`pkg-config` is both a toolchain and a build dep, and
# `mesa-vulkan-drivers` is both a GPU driver and part of what a user must have
# installed to run the download).
#
#   TOOLCHAIN  a C compiler, pkg-config and a fetcher — needed to COMPILE from
#              source, never by someone running the prebuilt binary.
#   BUILD      the `-dev` headers winit/wgpu/cosmic-text link against. CI and
#              Dockerfile.linux need exactly this and nothing else to build.
#   GPU        the driver (and, where the distro packages it separately, the
#              Vulkan loader) needed to actually RUN awl or its suite. CI's
#              linux job needs this on top of BUILD because it runs the test
#              suite against lavapipe; Dockerfile.linux does NOT, because it
#              only ever compiles and copies the binary out.
#   DIAG       `vulkan-tools` and friends: not required, but what the
#              from-source bootstrap installs so a user who hits a black
#              window can run `vulkaninfo`. Never installed in CI.
#   RUNTIME    the NON-dev library names a user of the prebuilt tarball must
#              install. This is the only group that reaches a user-facing
#              document, and a distro that declares it is thereby enrolled in
#              the shipped README.txt table (see `awl_deps_runtime_distros`).
#
# ⚠️ RUNTIME IS DECLARED ONLY WHERE THE NAMES HAVE BEEN VERIFIED ON THAT
# DISTRO. openSUSE deliberately has no RUNTIME array: the bootstrap's zypper
# arm gives its `-devel` names, and the non-dev equivalents have never been
# checked on a real openSUSE box. Generating a table row for it would be the
# exact failure mode a generated document invites — moving the error from
# transcription to SOURCING, and stating an unverified package name with a
# script's authority behind it. The table's enrolment is derived from which
# distros declare RUNTIME, so adding a verified array is all it takes to
# enrol one, and nothing here has to be remembered.

# Every name below is read through `${!ref}` indirect expansion, which no static
# analyser can follow, so each array reads as write-only to shellcheck. The
# helpers at the foot of the file are the only consumers.
# shellcheck disable=SC2034

# ---------------------------------------------------------------- Debian/Ubuntu
AWL_DEB_LABEL="Debian/Ubuntu"
AWL_DEB_INSTALL="sudo apt install"
AWL_DEB_TOOLCHAIN=(build-essential pkg-config curl ca-certificates)
AWL_DEB_BUILD=(
  pkg-config
  libfontconfig1-dev libxkbcommon-dev libwayland-dev
  libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
)
AWL_DEB_GPU=(mesa-vulkan-drivers)
AWL_DEB_DIAG=(libvulkan1 vulkan-tools)
# libxkbcommon-x11-0 is NOT pulled in by libxkbcommon-dev: it is a separate
# runtime package for the X11 keymap module, which winit's X11 backend dlopens
# at startup (the xkbcommon-dl crate). Without it awl panics immediately on any
# X11 session, naming that exact library. It belongs to RUNTIME rather than
# BUILD because a headless build never reaches the dlopen — which is precisely
# why CI's `linux` job omits it and the AT-SPI job, the only arm that puts a
# built awl in front of a real X server, asks for it explicitly.
AWL_DEB_RUNTIME=(libfontconfig1 libxkbcommon0 libxkbcommon-x11-0 libvulkan1 mesa-vulkan-drivers)

# ---------------------------------------------------------------------- Fedora
AWL_FEDORA_LABEL="Fedora"
AWL_FEDORA_INSTALL="sudo dnf install"
AWL_FEDORA_TOOLCHAIN=(gcc gcc-c++ make pkgconf-pkg-config curl)
AWL_FEDORA_BUILD=(fontconfig-devel libxkbcommon-devel wayland-devel libX11-devel libxcb-devel)
AWL_FEDORA_GPU=(vulkan-loader mesa-vulkan-drivers)
AWL_FEDORA_DIAG=(vulkan-tools)
AWL_FEDORA_RUNTIME=(fontconfig libxkbcommon vulkan-loader mesa-vulkan-drivers)

# ------------------------------------------------------------------------ Arch
AWL_ARCH_LABEL="Arch"
AWL_ARCH_INSTALL="sudo pacman -S"
AWL_ARCH_TOOLCHAIN=(base-devel pkgconf curl)
# Arch ships headers in the same package as the library, so BUILD and RUNTIME
# name the same things here. That is a property of the distro, not a mistake.
AWL_ARCH_BUILD=(fontconfig libxkbcommon wayland libx11 libxcb)
AWL_ARCH_GPU=(vulkan-icd-loader mesa)
AWL_ARCH_DIAG=(vulkan-tools)
AWL_ARCH_RUNTIME=(fontconfig libxkbcommon vulkan-icd-loader mesa)

# --------------------------------------------------------------------- openSUSE
AWL_SUSE_LABEL="openSUSE"
AWL_SUSE_INSTALL="sudo zypper install"
AWL_SUSE_TOOLCHAIN=(gcc gcc-c++ make pkg-config curl)
AWL_SUSE_BUILD=(fontconfig-devel libxkbcommon-devel wayland-devel libX11-devel libxcb-devel)
AWL_SUSE_GPU=(vulkan-loader Mesa-vulkan-device-driver)
AWL_SUSE_DIAG=(vulkan-tools)
# No AWL_SUSE_RUNTIME — see the warning above. Unverified, so unshipped.

# Every distro this file knows about, in the order a document should list them.
AWL_DEPS_DISTROS=(DEB FEDORA ARCH SUSE)

# awl_deps <DISTRO> <GROUP> [GROUP...]
#
# Echo the space-separated union of the named groups for one distro, in
# declaration order with duplicates removed. An unknown distro or a group that
# distro does not declare is an ERROR rather than an empty string: a silently
# empty package list installs nothing and reads as success, which is how a
# missing dependency reaches a user instead of a build log.
#
# `${!ref}` indirect expansion rather than `declare -n`: namerefs are bash 4.3+
# and the dev host's /bin/bash is 3.2.
# ⚠️ EVERY LOCAL HERE IS `_awl_`-PREFIXED, and that is not decoration. This file
# is SOURCED into scripts that have their own variables, and a `# shellcheck
# source=` directive makes shellcheck analyse it inline with each consumer — so
# a plainly-named local like `out` both risks a real collision and makes the
# analyser mis-type the consumer's own `out=` as an array reuse. That false
# positive appeared the moment this file was first sourced into
# oom-budget-container.sh.
awl_deps() {
  local _awl_distro="$1"; shift
  local _awl_seen=" " _awl_out=() _awl_group _awl_ref _awl_pkg

  case " ${AWL_DEPS_DISTROS[*]} " in
    *" $_awl_distro "*) ;;
    *) echo "linux-deps: unknown distro '$_awl_distro'" >&2; return 1 ;;
  esac

  for _awl_group in "$@"; do
    _awl_ref="AWL_${_awl_distro}_${_awl_group}[@]"
    # An undeclared array expands to nothing under `set -u`, so probe the name
    # before expanding it and fail loudly instead of contributing silence.
    if ! awl_deps_has_group "$_awl_distro" "$_awl_group"; then
      echo "linux-deps: $_awl_distro declares no $_awl_group group" >&2
      return 1
    fi
    for _awl_pkg in ${!_awl_ref}; do
      case "$_awl_seen" in
        *" $_awl_pkg "*) continue ;;
      esac
      _awl_seen="$_awl_seen$_awl_pkg "
      _awl_out+=("$_awl_pkg")
    done
  done

  printf '%s\n' "${_awl_out[*]}"
}

# awl_deps_has_group <DISTRO> <GROUP> — is the group declared (and non-empty)?
awl_deps_has_group() {
  local _awl_ref="AWL_${1}_${2}[@]"
  local _awl_expanded
  _awl_expanded="$(eval "printf '%s' \"\${$_awl_ref+set}\"" 2>/dev/null || true)"
  [ "$_awl_expanded" = "set" ]
}

# awl_deps_runtime_distros — the distros enrolled in the shipped README.txt
# table, DERIVED from which ones declare a verified RUNTIME array rather than
# pinned to a hand-kept list that can drift from the data beside it.
awl_deps_runtime_distros() {
  local _awl_d _awl_out=()
  for _awl_d in "${AWL_DEPS_DISTROS[@]}"; do
    awl_deps_has_group "$_awl_d" RUNTIME && _awl_out+=("$_awl_d")
  done
  printf '%s\n' "${_awl_out[*]}"
}

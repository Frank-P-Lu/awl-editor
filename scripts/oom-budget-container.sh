#!/usr/bin/env bash
# scripts/oom-budget-container.sh — item 231's fast local oracle, as a script.
#
# WHAT THIS IS. Under a FIXED 4 GiB container ceiling at --test-threads=1,
# `render::tests::` walks RSS monotonically to an OOM kill, and how far it gets
# is commit-correlated. Item 232 measured that (36707d06 reaches test 199,
# 8207e519 reaches test 160) and items 231/239 have called it "the fast local
# oracle that already exists" ever since — ~4 local minutes against a ~50-minute
# CI cycle. It only ever existed as prose in 96106575's commit message, and
# rebuilding it from that prose cost a rediscovery of the sccache trap below.
#
# WHAT THIS IS NOT. It is not a gate and nothing calls it. Item 243 settled that
# no local software-adapter arm belongs in anyone's gate: a CPU rasteriser has
# no system-wide GPU resource for item 231's cross-process wedge to exhaust, so
# this rig has never once reproduced the hosted-macOS HANG and cannot. It
# measures a DIFFERENT failure mode — a prompt SIGKILL with OOMKilled=true,
# never the runner's park-forever-with-memory-flat. Bounding what it sees is
# not proven to prevent the hang.
#
#   scripts/oom-budget-container.sh <git-ref> [<git-ref> ...]
#
# Alternate the refs (a b a b) rather than running each once: item 232 did, to
# control for host drift, and it is why 160 is known to be 160 and not noise.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
work="${AWL_OOM_WORKDIR:-$HOME/.awl-oom-budget}"
image=awl-oom-rig:bookworm
mkdir -p "$work/out"

# lavapipe on Debian bookworm: the same arm64 Mesa 22.3.6 stack item 232 used,
# on Dockerfile.linux's apt list plus the Vulkan loader and driver.
if ! docker image inspect "$image" >/dev/null 2>&1; then
  docker build -t "$image" - <<'DOCKERFILE'
FROM rust:1-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config procps coreutils \
      libfontconfig1-dev libxkbcommon-dev libwayland-dev \
      libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
      libvulkan1 mesa-vulkan-drivers vulkan-tools \
 && rm -rf /var/lib/apt/lists/*
DOCKERFILE
fi

for ref in "$@"; do
  sha=$(git rev-parse --short "$ref")
  tree="$work/tree-$sha"
  # A TARGET DIR PER TREE, and it is not optional. Item 232's first pass
  # silently scored the SAME BINARY TWICE: both trees came out of `git archive`
  # within one second, so Cargo's mtime fingerprint called one up to date and
  # reused the other's artifacts. The binary_sha256 printed below is the
  # assertion that catches it — two arms that print the same one measured one
  # tree, whatever their labels say.
  [ -d "$tree" ] || { mkdir -p "$tree"; git archive "$ref" | tar -x -C "$tree"; }
  docker volume create "awl-oom-$sha" >/dev/null

  # RUSTC_WRAPPER="" because .cargo/config.toml sets rustc-wrapper = "sccache"
  # for every build in this checkout, the container's included, and a wrapper
  # cannot install itself — Cargo runs it to answer `rustc -vV` before it
  # compiles anything, so its absence is an immediate `could not execute
  # process sccache (never executed)`. Same override code-health.sh makes.
  # The BUILD runs uncapped; only the measured run gets the ceiling.
  docker run --rm -v "$tree":/src -v "awl-oom-$sha":/target \
    -v awl-oom-registry:/usr/local/cargo/registry \
    -e CARGO_TARGET_DIR=/target -e RUSTC_WRAPPER= -e CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" \
    -w /src "$image" cargo test --bin awl --no-run

  label="$(date +%s)-$sha"
  docker run --rm --memory 4g --memory-swap 4g \
    -v "$tree":/src -v "awl-oom-$sha":/target -v "$work/out":/out \
    -e LABEL="$label" -w /src "$image" bash -c '
      set -uo pipefail
      out=/out/$LABEL; mkdir -p "$out"
      bin=$(ls -t /target/debug/deps/awl-* | grep -v "\.d$" | head -1)
      {
        echo "RIG label=$LABEL binary_sha256=$(sha256sum "$bin" | cut -d" " -f1)"
        echo "RIG mem_limit=$(cat /sys/fs/cgroup/memory.max)"
        echo "RIG background_wgsl_bytes=$(stat -c %s /src/shaders/background.wgsl)"
      } | tee "$out/provenance.txt"
      "$bin" render::tests:: --list > "$out/list.txt" 2>&1
      echo "RIG list_tests=$(grep -c ": test$" "$out/list.txt")" | tee -a "$out/provenance.txt"
      # The binary writes STRAIGHT TO A FILE, never through a pipe: a pipe loses
      # the tail of the trace when the kernel SIGKILLs the process, and the tail
      # is the whole measurement.
      "$bin" render::tests:: --test-threads=1 > "$out/run.log" 2>&1 &
      pid=$!
      while kill -0 $pid 2>/dev/null; do
        rss=$(awk "/^VmRSS:/{print \$2}" /proc/$pid/status 2>/dev/null) || true
        # Column 3 is how many tests had COMPLETED at this sample. It turns RSS
        # over TIME into RSS over TEST ORDINAL, which is the only form in which
        # two trees with different test SETS compare at a matched point.
        [ -n "${rss:-}" ] && echo "$(date +%s.%N) $rss $(grep -c "^test .* ok$" "$out/run.log")" >> "$out/rss.txt"
        sleep 0.2
      done
      wait $pid; echo "RIG exit_status=$? (137 = SIGKILL, i.e. the OOM)"
      echo "RIG cgroup_oom_kills=$(awk "/^oom_kill /{print \$2}" /sys/fs/cgroup/memory.events)"
    '

  out="$work/out/$label"
  awk -v l="$label" '{if($2>m)m=$2; if($3>n)n=$3}
    END{printf "RESULT %s tests_completed=%d peak_rss_mb=%d mb_per_test=%.1f\n", l, n, m/1024, m/1024/n}' \
    "$out/rss.txt"
  echo "RESULT $label died_on=$(sed -n '$s/^test //;$s/ \.\.\..*//p' "$out/run.log")"
done

echo
echo "Clean up: docker volume rm \$(docker volume ls -q | grep ^awl-oom-);"
echo "          docker rmi $image; rm -rf $work"

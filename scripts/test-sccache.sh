#!/usr/bin/env bash
# External shell law for the checked-in sccache provision/configuration seam.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-sccache-test.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

fail() {
    echo "test-sccache: $*" >&2
    exit 1
}

grep -Fq 'rustc-wrapper = "sccache"' "$ROOT/.cargo/config.toml" \
    || fail "Cargo does not set sccache as its rustc wrapper"
grep -Fq 'SCCACHE_CACHE_SIZE = { value = "5G", force = true }' "$ROOT/.cargo/config.toml" \
    || fail "sccache cache limit is not pinned"
grep -Fq 'cargo install sccache --version "$SCCACHE_VERSION" --locked' \
    "$ROOT/scripts/install-sccache.sh" || fail "installer is not exact and locked"
[[ "$(rg -F -c 'scripts/install-sccache.sh' "$ROOT/.github/workflows/ci.yml")" == 4 ]] \
    || fail "every CI Rust job must provision sccache before Cargo"

mkdir -p "$WORK/bin" "$WORK/crate/src" "$WORK/home"
cat >"$WORK/crate/Cargo.toml" <<'EOF'
[package]
name = "sccache-config-fixture"
version = "0.1.0"
edition = "2024"
EOF
printf 'fn main() {}\n' >"$WORK/crate/src/main.rs"
cat >"$WORK/bin/sccache" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "${SCCACHE_CACHE_SIZE:-}" == "5G" ]] || exit 91
printf '%s\n' "$SCCACHE_CACHE_SIZE" >>"$SCCACHE_TEST_LOG"
exec "$@"
EOF
chmod +x "$WORK/bin/sccache"

SCCACHE_TEST_LOG="$WORK/wrapper.log" HOME="$WORK/home" PATH="$WORK/bin:$PATH" \
    cargo --config "$ROOT/.cargo/config.toml" build --manifest-path "$WORK/crate/Cargo.toml" >/dev/null
[[ -s "$WORK/wrapper.log" ]] || fail "Cargo did not invoke configured wrapper"
grep -Fxq '5G' "$WORK/wrapper.log" || fail "wrapper did not inherit configured cache bound"

echo "test-sccache: wrapper and 5G bound reach a real rustc invocation; all CI Rust jobs provision it"

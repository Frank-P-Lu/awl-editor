#!/usr/bin/env bash
# Differential release-profile law: the same deterministic replay must produce
# the same state sidecar in debug and release. Run once on combined main before
# a push train, and before a tag; it is intentionally not a per-landing gate.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "${1:-}" == "--use-built-binaries" ]]; then
    if [[ "$#" -ne 1 ]]; then
        echo "usage: $0 [--use-built-binaries]" >&2
        exit 2
    fi
elif [[ "$#" -eq 0 ]]; then
    cargo build --bin awl
    cargo build --release --bin awl
else
    echo "usage: $0 [--use-built-binaries]" >&2
    exit 2
fi

for binary in target/debug/awl target/release/awl; do
    if [[ ! -x "$binary" ]]; then
        echo "release-profile gate: missing executable $binary" >&2
        exit 2
    fi
done

WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-release-profile.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/project" "$WORK/artifacts"

cat >"$WORK/project/fixture.md" <<'EOF'
| name | score |
|---|---:|
| Ada|10 |

plain text
EOF

cat >"$WORK/config.toml" <<'EOF'
history = false
autosave = false
session_restore = false

[keys]
zoom_in = "C-M-z"
bold = "C-M-b"
go_to_file = "C-M-o"
search_forward = "C-M-s"
toggle_outline = "C-M-v"
align_table = "C-M-a"
insert_link = "C-M-l"
EOF

families=(Buffer Viewport Format Overlay Session View Align Export)
specs=("x" "C-M-z" "C-M-b" "C-M-o" "C-M-s" "C-M-v" "C-M-a" "C-M-l")

for index in "${!families[@]}"; do
    family="${families[$index]}"
    spec="${specs[$index]}"
    slug="$(printf '%s' "$family" | tr '[:upper:]' '[:lower:]')"
    debug_png="$WORK/artifacts/${slug}-debug.png"
    release_png="$WORK/artifacts/${slug}-release.png"
    debug_json="${debug_png%.png}.json"
    release_json="${release_png%.png}.json"

    target/debug/awl \
        --screenshot "$debug_png" \
        --keys "$spec" \
        --strict-replay \
        --config "$WORK/config.toml" \
        --root "$WORK/project" \
        "$WORK/project/fixture.md"
    target/release/awl \
        --screenshot "$release_png" \
        --keys "$spec" \
        --strict-replay \
        --config "$WORK/config.toml" \
        --root "$WORK/project" \
        "$WORK/project/fixture.md"

    if ! cmp -s "$debug_json" "$release_json"; then
        echo "release-profile gate: ${family} sidecars differ between debug and release" >&2
        diff -u "$debug_json" "$release_json" >&2 || true
        exit 1
    fi
done

echo "release-profile gate: debug and release sidecars match for all 8 action families"

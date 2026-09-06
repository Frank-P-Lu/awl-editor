#!/usr/bin/env bash
# Law: no script in scripts/ leaves a scripts/__pycache__ behind.
#
# CPython writes no bytecode for the script it runs as `__main__`, so a plain
# `python3 scripts/foo.py` can never create that directory however often the
# recurring one gets blamed on it. The creators are the by-path loaders — every
# `importlib.util.spec_from_file_location` writes a .pyc NEXT TO THE FILE IT
# LOADS — and they are reached from Rust tests and hand-run capture pipelines
# rather than through one wrapper that could carry PYTHONDONTWRITEBYTECODE for
# all of them. Hence the guard belongs at each loader's own call site, and this
# law derives its roster from the loader call itself so a new one enrols
# automatically.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/awl-pycache-test.XXXXXX")"
WORK="$(cd "$WORK" && pwd -P)"
trap 'rm -rf "$WORK"' EXIT

fail() {
    echo "test-pycache-guards: FAIL — $*" >&2
    exit 1
}

# ---------------------------------------------------------------------------
# The roster: every tracked scripts/*.py that loads another file as a module.
# ---------------------------------------------------------------------------
loaders=""
while IFS= read -r tracked; do
    case "$tracked" in
        scripts/*.py) ;;
        *) continue ;;
    esac
    if grep -q 'spec_from_file_location' "$ROOT/$tracked"; then
        loaders="$loaders $tracked"
    fi
done < <(git -C "$ROOT" ls-files)

if [[ -z "$loaders" ]]; then
    fail "no by-path loader found in scripts/ — the roster is empty, so this law would prove nothing"
fi

# ---------------------------------------------------------------------------
# BEHAVIOURAL ARM: actually run each enrolled script and look for the
# directory. The static arm below is a claim about the source; this is the
# claim the item is actually about, so it runs FIRST — a removed guard must go
# red here, not only in a source scan. It runs against a COPY with the ambient
# PYTHONDONTWRITEBYTECODE cleared, so the result is a property of the script
# rather than of whoever invoked this gate.
# ---------------------------------------------------------------------------
cp -R "$ROOT/scripts" "$WORK/scripts"
rm -rf "$WORK/scripts/__pycache__"

for loader in $loaders; do
    name="${loader#scripts/}"
    # Exit status is deliberately ignored: these instruments want captures that
    # a hermetic copy does not have, and the by-path load — the only thing
    # under test — happens at import time, before any of that.
    (cd "$WORK" && env -u PYTHONDONTWRITEBYTECODE python3 "scripts/$name" \
        >/dev/null 2>&1) || true
    if [[ -d "$WORK/scripts/__pycache__" ]]; then
        fail "running $loader created scripts/__pycache__ ($(ls "$WORK/scripts/__pycache__" | tr '\n' ' '))"
    fi
done

# ---------------------------------------------------------------------------
# STATIC ARM: the guard precedes the first load in every enrolled script.
# A second net over the same rule, and the one that names where to put it back.
# ---------------------------------------------------------------------------
for loader in $loaders; do
    # `|| true` is load-bearing under `set -euo pipefail`: a grep that finds
    # nothing would otherwise kill this script with no message at all, which is
    # the exact failure mode a law must never have.
    guard_line="$(grep -n '^sys\.dont_write_bytecode = True' "$ROOT/$loader" | head -n1 | cut -d: -f1 || true)"
    load_line="$(grep -n 'spec_from_file_location' "$ROOT/$loader" | head -n1 | cut -d: -f1 || true)"
    if [[ -z "$guard_line" ]]; then
        fail "$loader loads a module by path but never sets sys.dont_write_bytecode"
    fi
    if (( guard_line > load_line )); then
        fail "$loader sets sys.dont_write_bytecode at line $guard_line, after its first load at line $load_line"
    fi
done

echo "test-pycache-guards: $(set -- $loaders; echo $#) by-path loaders leave no scripts/__pycache__"

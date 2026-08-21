#!/usr/bin/env bash
# Regenerate the compact Reference and Supported Markdown guide from awl's own
# Markdown roster.
#
# The reference's tables are never edited by hand: `src/reference/` builds each
# one by reading the roster the running app reads, and `src/reference/law.rs`
# fails when the checked-in text differs from a fresh generation by a byte. This
# script is the fix for that failure — run it, then re-run the tests.
#
# It drives the `#[ignore]`d generator test (the repo's regeneration convention:
# a test prints, a human-run tool splices — no test ever writes a repo file) and
# replaces the text between each generated marker pair in every target document.
#
# Run from the repo root. Takes no arguments.
set -euo pipefail

cd "$(dirname "$0")/.."

blocks=$(mktemp)
trap 'rm -f "$blocks"' EXIT

echo "regen-reference: generating from the live rosters…" >&2
cargo test --bin awl reference::law::print_generated_reference_blocks \
  -- --ignored --nocapture --exact >"$blocks"

python3 - "$blocks" <<'PY'
import pathlib, re, sys

raw = pathlib.Path(sys.argv[1]).read_text()

# Harvest every fenced block the generator printed. Anything cargo interleaves
# (compile lines, the test-result summary) sits outside the fences and is
# ignored by construction.
blocks = {}
pattern = re.compile(
    r"===AWL-REFERENCE-BLOCK (md|html) ([a-z-]+)===\n(.*?)===AWL-REFERENCE-BLOCK-END===\n",
    re.S,
)
for kind, marker, body in pattern.findall(raw):
    blocks[(kind, marker)] = body

if not blocks:
    sys.exit(
        "regen-reference: the generator printed no blocks. Run\n"
        "  cargo test --bin awl reference::law::print_generated_reference_blocks "
        "-- --ignored --nocapture --exact\n"
        "and read the failure."
    )

targets = {"md": pathlib.Path("REFERENCE.md"), "html": pathlib.Path("site/reference.html")}
targets.update({
    "supported-md": pathlib.Path("SUPPORTED-MARKDOWN.md"),
    "supported-html": pathlib.Path("site/supported-markdown.html"),
})
touched = 0
for kind, path in targets.items():
    text = path.read_text()
    for (k, marker), body in blocks.items():
        expected_kind = "md" if kind == "supported-md" else "html" if kind == "supported-html" else kind
        if k != expected_kind:
            continue
        if kind.startswith("supported-") and marker != "supported-markdown":
            continue
        if not kind.startswith("supported-") and marker == "supported-markdown":
            continue
        begin = f"<!-- GENERATED:{marker}:BEGIN -->"
        end = f"<!-- GENERATED:{marker}:END -->"
        if begin not in text or end not in text:
            sys.exit(f"regen-reference: {path} carries no {begin}/{end} marker pair")
        head, rest = text.split(begin, 1)
        _, tail = rest.split(end, 1)
        text = f"{head}{begin}\n{body}{end}{tail}"
        touched += 1
    path.write_text(text)
    print(f"regen-reference: wrote {path}", file=sys.stderr)

print(f"regen-reference: spliced {touched} generated blocks", file=sys.stderr)
PY

echo "regen-reference: done. Re-run the reference laws to confirm:" >&2
echo "  cargo test --bin awl reference::" >&2

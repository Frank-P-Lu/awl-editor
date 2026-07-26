#!/usr/bin/env bash
# One owner for every blocking Rust code-health policy. Keep CI deliberately
# boring: it invokes this command and does not restate any of these checks.
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
python3 scripts/code-health.py

#!/usr/bin/env bash
# Enforce the public macOS download limit. The app ZIP is a workflow-only
# diagnostic artifact, so its size is reported but never blocks publication.
set -euo pipefail

DMG="${1:?usage: check-macos-release-size.sh <public.dmg> [workflow-only.app.zip]}"
APP_ZIP="${2:-}"
MAX_BYTES=50000000

file_size() {
  if stat -f%z "$1" >/dev/null 2>&1; then
    stat -f%z "$1"
  else
    stat -c%s "$1"
  fi
}

DMG_BYTES="$(file_size "$DMG")"
echo "public macOS DMG size: $DMG = $DMG_BYTES bytes (must be under $MAX_BYTES)"
if [ "$DMG_BYTES" -ge "$MAX_BYTES" ]; then
  echo "error: $DMG is $DMG_BYTES bytes; the public macOS DMG must be under $MAX_BYTES bytes" >&2
  exit 1
fi

if [ -n "$APP_ZIP" ]; then
  APP_ZIP_BYTES="$(file_size "$APP_ZIP")"
  echo "workflow-only macOS app ZIP size: $APP_ZIP = $APP_ZIP_BYTES bytes (informational)"
fi

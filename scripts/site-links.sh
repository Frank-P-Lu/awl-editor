#!/usr/bin/env bash
#
# site-links.sh — prove every REPO-RELATIVE link the site publishes points at a
# file that actually exists in this repo. The site's source-links are GitHub
# blob URLs (`…/blob/main/<repo-path>`) to the contract docs + license files
# (PHILOSOPHY.md, DESIGN.md, LICENSE, NOTICE, assets/*/LICENSES.md,
# THIRD-PARTY-LICENSES.md, site/check.js …). A doc rename that leaves one of
# these dangling would 404 for a real reader; this catches it BEFORE deploy.
#
# NO NETWORK: only the `blob/main/<path>` SUFFIX is checked, against the local
# working tree (the file-path target, never the http origin). CI-runnable.
#
# Usage:  scripts/site-links.sh
# Exit:   0 = every target exists; 1 = one or more dangling (listed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

# The distinct repo-relative paths embedded as GitHub blob links across every
# site source file. Portable (no `mapfile`, macOS bash 3.2 friendly).
TARGETS="$(grep -rhoIE 'blob/main/[^"'"'"' )]+' \
  --include='*.html' --include='*.js' --include='*.css' --include='*.txt' \
  site/ 2>/dev/null | sed 's#.*blob/main/##' | sort -u)"

if [ -z "$TARGETS" ]; then
  echo "site-links: no blob/main/<path> links found under site/ — nothing to check."
  exit 0
fi

missing=0
total=0
while IFS= read -r t; do
  [ -z "$t" ] && continue
  total=$((total + 1))
  if [ -e "$ROOT/$t" ]; then
    echo "  ok   $t"
  else
    echo "  MISS $t   (site links to a repo path that does not exist)" >&2
    missing=$((missing + 1))
  fi
done <<EOF
$TARGETS
EOF

if [ "$missing" -ne 0 ]; then
  echo "site-links: $missing dangling repo-relative site link(s)." >&2
  exit 1
fi
echo "site-links: all $total repo-relative site links resolve."

# --- Social-card metadata -----------------------------------------------------
#
# A URL-valued Open Graph / Twitter tag must be ABSOLUTE. Facebook, X/Twitter,
# LinkedIn and Slack drop the card outright rather than resolving a relative
# path against the page, so `content="img/social.png"` yields no image anywhere
# while still pointing at a real file on disk — a green file-existence check
# over a card that does not work. Both halves are asserted here: absolute
# origin, AND the path resolves under site/.
#
# `site/fly.toml`'s app name is the ONE owner of the canonical origin (a Fly
# app serves at <app>.fly.dev, and the site sets force_https). The expected
# origin is derived from it rather than restated here, so a custom domain is a
# one-line change in fly.toml + the pages, and this law fails the moment the
# HTML and the deploy config disagree.

FLY_APP="$(sed -n 's/^app *= *"\([^"]*\)".*/\1/p' "$ROOT/site/fly.toml" | head -1)"
if [ -z "$FLY_APP" ]; then
  echo "site-links: could not read app name from site/fly.toml — the canonical origin has no owner." >&2
  exit 1
fi
ORIGIN="https://${FLY_APP}.fly.dev"

# The URL-valued tags, enumerated rather than pattern-matched. The sweep below
# also REJECTS any og:/twitter: tag whose name ends in `image`/`url` but is
# absent from this list, so a newly added URL-valued tag cannot opt out of the
# check silently — it fails until it is enrolled here.
URL_VALUED_TAGS="og:url og:image og:image:secure_url og:video og:audio twitter:image twitter:player"

social_bad=0
social_total=0
for page in "$ROOT"/site/*.html; do
  [ -e "$page" ] || continue
  rel="site/$(basename "$page")"

  # Every og:/twitter: tag on the page, as name<TAB>content.
  TAGS="$(grep -oE '<meta[^>]*(property|name)="(og|twitter):[^"]*"[^>]*>' "$page" 2>/dev/null \
    | while IFS= read -r tag; do
        # -E, not BRE: BSD sed has no \| alternation, and a silently empty
        # name here made this whole sweep pass over zero tags on its first run.
        n="$(printf '%s' "$tag" | sed -nE 's/.*(property|name)="([^"]*)".*/\2/p')"
        c="$(printf '%s' "$tag" | sed -nE 's/.*content="([^"]*)".*/\1/p')"
        printf '%s\t%s\n' "$n" "$c"
      done)"

  while IFS="$(printf '\t')" read -r name content; do
    [ -z "$name" ] && continue

    listed=0
    for t in $URL_VALUED_TAGS; do
      [ "$name" = "$t" ] && listed=1 && break
    done

    # Anti-opt-out: a URL-shaped tag name that is not enrolled above.
    if [ "$listed" -eq 0 ]; then
      case "$name" in
        *:url|*:image|*image:secure_url|*:video|*:audio|*:player)
          echo "  BAD  $rel  $name is URL-valued but not enrolled in URL_VALUED_TAGS" >&2
          social_bad=$((social_bad + 1))
          ;;
      esac
      continue
    fi

    social_total=$((social_total + 1))

    case "$content" in
      "$ORIGIN"/*) ;;
      https://*|http://*)
        # Absolute, but not the origin site/fly.toml deploys to — the pages and
        # the deploy config have drifted apart.
        echo "  BAD  $rel  $name=\"$content\" is absolute but not under $ORIGIN, the origin site/fly.toml deploys to" >&2
        social_bad=$((social_bad + 1))
        continue
        ;;
      *)
        echo "  BAD  $rel  $name=\"$content\" is not absolute (scrapers drop a relative card outright)" >&2
        social_bad=$((social_bad + 1))
        continue
        ;;
    esac

    # The path after the origin must be a real file under site/. A bare "/"
    # is the landing page.
    path="${content#"$ORIGIN"/}"
    [ -z "$path" ] && path="index.html"
    if [ -e "$ROOT/site/$path" ]; then
      echo "  ok   $rel  $name -> site/$path"
    else
      echo "  MISS $rel  $name=\"$content\" resolves to site/$path, which does not exist" >&2
      social_bad=$((social_bad + 1))
    fi
  done <<EOF
$TAGS
EOF
done

if [ "$social_bad" -ne 0 ]; then
  echo "site-links: $social_bad bad social-metadata reference(s)." >&2
  exit 1
fi
echo "site-links: all $social_total URL-valued social tags are absolute under $ORIGIN and resolve."

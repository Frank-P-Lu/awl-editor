#!/usr/bin/env python3
"""Resolve explicit release intent and refuse publication over an existing release."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tomllib
import urllib.error
import urllib.request


def plan(event: str, ref: str, version: str, inputs: dict) -> dict[str, str]:
    legacy_tag = event == 'push' and ref.startswith('refs/tags/v')
    if event not in ('workflow_dispatch', 'push') or (event == 'push' and not legacy_tag):
        raise ValueError('release requires manual dispatch or a legacy v tag')
    if legacy_tag and ref.removeprefix('refs/tags/v') != version:
        raise ValueError('tag version must equal Cargo.toml package.version')
    # Apple marketing versions must remain numeric in both rehearsals and releases.
    if not re.fullmatch(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', version):
        raise ValueError('package.version must be a numeric major.minor.patch')
    boolean = lambda key: inputs.get(key, False) in (True, 'true')
    publish = legacy_tag or inputs.get('dry_run', True) in (False, 'false')
    if publish and not legacy_tag and ref != 'refs/heads/main':
        raise ValueError('manual publication must run from main')
    channel = inputs.get('channel', 'preview')
    if legacy_tag:
        channel = 'preview' if version.startswith('0.') else 'stable'
    if channel not in ('preview', 'stable'):
        raise ValueError('select preview or stable')
    local = legacy_tag or boolean('local_macos_rehearsal')
    credentialed = boolean('credentialed_macos_rehearsal')
    if local and credentialed:
        raise ValueError('choose hosted signing or staged fallback, not both')
    if publish and not legacy_tag and local:
        raise ValueError('manual publication uses hosted signing; staged fallback remains tag-only')
    return {key: str(value).lower() if isinstance(value, bool) else value for key, value in {
        'is_release': publish, 'version': version, 'channel': channel,
        'sign_macos': publish or credentialed, 'local_macos': local,
        'is_prerelease': channel == 'preview',
    }.items()}


def api(path: str):
    request = urllib.request.Request('https://api.github.com/' + path, headers={
        'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
        'Accept': 'application/vnd.github+json',
        'X-GitHub-Api-Version': '2022-11-28',
    })
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return None
        raise ValueError(f'GitHub publication preflight failed (HTTP {error.code})') from None


def check_publication(repo: str, version: str, source: str) -> None:
    # Refuse updates to either drafts or published releases. The action must never
    # replace previously verified public bytes on a rerun.
    if api(f'repos/{repo}/releases/tags/v{version}') is not None:
        raise ValueError('a release already exists for this version; choose a new version')
    tag = api(f'repos/{repo}/git/ref/tags/v{version}')
    if tag is None:
        return
    obj = tag['object']
    for _ in range(8):
        if obj['type'] == 'commit':
            if obj['sha'] != source:
                raise ValueError('existing release tag does not point to this exact source commit')
            return
        if obj['type'] != 'tag':
            break
        annotated = api(f"repos/{repo}/git/tags/{obj['sha']}")
        if annotated is None:
            break
        obj = annotated['object']
    raise ValueError('cannot resolve existing tag to an exact source commit')


def check_uploaded(repo: str, version: str, directory: Path) -> None:
    uploaded = api(f'repos/{repo}/releases/tags/v{version}')
    if not uploaded or uploaded.get('draft') is not True:
        raise ValueError('uploaded payload must remain a draft until verified')
    names = {f'awl-{version}-{suffix}' for suffix in (
        'linux-x86_64.tar.gz', 'linux-x86_64.AppImage',
        'macos-arm64.dmg', 'macos-x86_64.dmg')} | {'SHA256SUMS'}
    assets = uploaded.get('assets', [])
    if len(assets) != len(names) or {asset['name'] for asset in assets} != names:
        raise ValueError('uploaded release asset roster differs from the verified payload')
    for asset in assets:
        payload = directory / asset['name']
        digest = 'sha256:' + hashlib.sha256(payload.read_bytes()).hexdigest()
        if asset.get('digest') != digest or asset.get('size') != payload.stat().st_size:
            raise ValueError(f"uploaded asset hash/size mismatch: {asset['name']}")
    print('Uploaded draft: all five asset digests and sizes match the verified payload')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['plan', 'check-publication', 'check-uploaded'])
    args = parser.parse_args()
    version = tomllib.loads(Path('Cargo.toml').read_text())['package']['version']
    if args.command == 'check-uploaded':
        check_uploaded(os.environ['GITHUB_REPOSITORY'], version, Path('release'))
        return
    if args.command == 'check-publication':
        check_publication(os.environ['GITHUB_REPOSITORY'], version, os.environ['GITHUB_SHA'])
        return
    event = json.loads(Path(os.environ['GITHUB_EVENT_PATH']).read_text())
    values = plan(os.environ['GITHUB_EVENT_NAME'], os.environ['GITHUB_REF'], version,
                  event.get('inputs', {}))
    with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
        for key, value in values.items():
            print(f'{key}={value}', file=output)
    print(f"Release plan: {values['channel']}, publish={values['is_release']}, "
          f"staged fallback={values['local_macos']}, signing={values['sign_macos']}")


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, KeyError) as error:
        raise SystemExit(f'release-plan: {error}')

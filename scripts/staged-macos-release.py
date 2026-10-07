#!/usr/bin/env python3
"""Stage only verified DMGs privately, or fetch the exact frozen candidate for CI."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
ARCHES = ('arm64', 'x86_64')
BUILD = r'[1-9][0-9]{0,3}\.[0-9]{1,2}\.[0-9]{1,2}'


def gh(*args, binary=False):
    return subprocess.check_output(['gh', *map(str, args)], cwd=ROOT, text=not binary)


def api(repo, endpoint):
    return json.loads(gh('api', f'repos/{repo}/{endpoint}'))


def digest(path):
    return 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest()


def filenames(version):
    return {f'awl-{version}-macos-{arch}.dmg' for arch in ARCHES}


def validate_metadata(release, version, commit, ref):
    pattern = rf'macos-stage-v{re.escape(version)}-{commit}-b({BUILD})'
    match = re.fullmatch(pattern, release['tag_name'])
    if not match or not release['draft'] or not release['prerelease']:
        raise ValueError('staging release must be a private draft for this version and full source SHA')
    if ref['object']['type'] != 'commit' or ref['object']['sha'] != commit:
        raise ValueError('staging tag does not directly name the frozen source commit')
    assets = release['assets']
    if len(assets) != 2 or {a['name'] for a in assets} != filenames(version):
        raise ValueError('staging asset roster must be exactly the two native DMGs')
    for asset in assets:
        if (asset['state'] != 'uploaded' or not 0 < asset['size'] < 50_000_000
                or not re.fullmatch(r'sha256:[0-9a-f]{64}', asset.get('digest') or '')):
            raise ValueError('staging asset state, strict size limit or server digest is invalid')
    return match.group(1)


def candidate(repo, version, commit):
    pages = json.loads(gh('api', '--paginate', '--slurp', f'repos/{repo}/releases?per_page=100'))
    prefix = f'macos-stage-v{version}-{commit}-b'
    matches = [release for page in pages for release in page if release['tag_name'].startswith(prefix)]
    if len(matches) != 1:
        raise ValueError('expected exactly one staging release for this frozen candidate')
    release = matches[0]
    ref = api(repo, f'git/ref/tags/{release["tag_name"]}')
    build = validate_metadata(release, version, commit, ref)
    return release, build


def verify_files(release, directory):
    for asset in release['assets']:
        path = directory / asset['name']
        if path.is_symlink() or path.stat().st_size != asset['size'] or digest(path) != asset['digest']:
            raise ValueError('download does not match GitHub server size and SHA-256 digest')


def stage(repo, directory, version, commit):
    manifest = json.loads((directory / 'local-macos-manifest.json').read_text())
    if (manifest['source_commit'] != commit or manifest['version'] != version
            or not all(manifest.get(k) is True for k in ['signed', 'notarized', 'verified'])
            or not re.fullmatch(BUILD, manifest['build_version'])):
        raise ValueError('local signing manifest does not certify this frozen candidate')
    # Independently mount and assess the exact assets again before any upload.
    subprocess.run([ROOT / 'scripts/verify-macos-release.sh', directory, version,
                    manifest['build_version'], 'signed', 'local'], cwd=ROOT, check=True)
    tag = f'macos-stage-v{version}-{commit}-b{manifest["build_version"]}'
    # Refuse collisions and partial overwrites; a non-v draft never triggers publishing.
    refs = api(repo, f'git/matching-refs/tags/{tag}')
    pages = json.loads(gh('api', '--paginate', '--slurp', f'repos/{repo}/releases?per_page=100'))
    if refs or any(r['tag_name'].startswith(f'macos-stage-v{version}-{commit}-b') for page in pages for r in page):
        raise ValueError('candidate staging already exists; refuse overwrite or ambiguous builds')
    paths = [directory / name for name in sorted(filenames(version))]
    for path in paths:
        record = next(a for a in manifest['artifacts'] if a['file'] == path.name)
        if path.is_symlink() or not 0 < path.stat().st_size < 50_000_000 or digest(path) != 'sha256:' + record['sha256']:
            raise ValueError('local DMG changed since signing verification')
    gh('release', 'create', tag, '--repo', repo, '--target', commit, '--draft', '--prerelease',
       '--title', f'Private macOS staging for {version}', '--notes',
       'Verified native DMGs staged for nonpublishing rehearsal. Signing credentials remain on the owner Mac.', *paths)
    release, _ = candidate(repo, version, commit)
    verify_files(release, directory)
    print(json.dumps({'tag': tag, 'source_commit': commit, 'verified': True}))


def fetch(repo, directory, version, commit):
    release, build = candidate(repo, version, commit)
    if directory.exists() or directory.is_symlink():
        raise ValueError('staged download output must be a new directory')
    directory.mkdir(mode=0o700)
    for asset in release['assets']:
        data = gh('api', '-H', 'Accept: application/octet-stream',
                  f'repos/{repo}/releases/assets/{asset["id"]}', binary=True)
        (directory / asset['name']).write_bytes(data)
    verify_files(release, directory)
    # Only validated public metadata enters workflow outputs.
    if os.environ.get('GITHUB_OUTPUT'):
        with open(os.environ['GITHUB_OUTPUT'], 'a') as stream:
            stream.write(f'build_version={build}\n')
    print(json.dumps({'source_commit': commit, 'build_version': build, 'verified': True}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['stage', 'fetch'])
    parser.add_argument('--repo', required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--version', required=True)
    args = parser.parse_args()
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', args.repo) or not re.fullmatch(r'\d+\.\d+\.\d+', args.version):
        parser.error('invalid repository or numeric marketing version')
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if args.mode == 'stage' and subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT).strip():
        parser.error('staging requires a clean frozen checkout')
    try:
        globals()[args.mode](args.repo, args.directory.absolute(), args.version, commit)
    except (OSError, ValueError, KeyError, StopIteration, subprocess.CalledProcessError) as error:
        print(f'staged-macos-release: {type(error).__name__}: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

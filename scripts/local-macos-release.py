#!/usr/bin/env python3
"""Sign and notarize native Mac downloads locally; never export or upload credentials."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tomllib
import uuid

ROOT = Path(__file__).resolve().parent.parent
ARCHES = {'arm64': 'aarch64-apple-darwin', 'x86_64': 'x86_64-apple-darwin'}


def command(args, *, env=None, capture=False, timeout=None):
    return subprocess.run([str(x) for x in args], cwd=ROOT, env=env, check=True,
                          stdout=subprocess.PIPE if capture else None,
                          stderr=subprocess.PIPE if capture else None, text=True, timeout=timeout)


def frozen_commit():
    if command(['git', 'status', '--porcelain'], capture=True).stdout.strip():
        raise ValueError('the local signing checkout must be clean')
    return command(['git', 'rev-parse', 'HEAD'], capture=True).stdout.strip()


def credentials(key: Path | None, issuer_file: Path):
    directory = Path.home() / '.appstoreconnect/private_keys'
    candidates = [key] if key else list(directory.glob('AuthKey_*.p8'))
    if len(candidates) != 1:
        raise ValueError('select exactly one existing local App Store Connect API key')
    key = candidates[0]
    if key.is_symlink() or not key.is_file() or key.parent.resolve() != key.parent:
        raise ValueError('the existing API key must be a regular file in its real directory')
    if stat.S_IMODE(key.stat().st_mode) & 0o077:
        raise ValueError('the existing API key permissions permit other users; ask its owner to secure it')
    matched = re.fullmatch(r'AuthKey_([A-Za-z0-9]{10,})\.p8', key.name)
    if not matched or issuer_file.is_symlink() or issuer_file.stat().st_size > 128:
        raise ValueError('existing key/issuer metadata has an unsupported format')
    issuer = str(uuid.UUID(issuer_file.read_text().strip()))
    # Private-key bytes are consumed only by Apple's authentication tool.
    return ['--key', str(key), '--key-id', matched.group(1), '--issuer', issuer]


def signing_identity(requested: str | None):
    output = command(['security', 'find-identity', '-v', '-p', 'codesigning'], capture=True).stdout
    team = (ROOT / 'assets/macos/release-team-id.txt').read_text().strip()
    identities = re.findall(r'\b([A-F0-9]{40}) "Developer ID Application: [^"]+ \(' + re.escape(team) + r'\)"', output)
    if requested:
        if requested not in identities:
            raise ValueError('the requested existing Developer ID Application identity is not valid')
        return requested
    if len(identities) != 1:
        raise ValueError('select exactly one existing valid Developer ID Application identity')
    return identities[0]


def build_version(value: str):
    if not re.fullmatch(r'[1-9][0-9]{0,3}\.[0-9]{1,2}\.[0-9]{1,2}', value):
        raise ValueError('build version must be a positive Apple numeric version, for example 25.0.0')
    return value


def progress(output: Path, phase: str, arch: str | None = None):
    record = {'phase': phase, 'architecture': arch}
    (output / 'local-progress.json').write_text(json.dumps(record) + '\n')
    print(json.dumps(record), flush=True)


def notarize(app: Path, archive: Path, auth):
    command(['ditto', '-c', '-k', '--keepParent', app, archive])
    try:
        response = command(['xcrun', 'notarytool', 'submit', archive, *auth,
                            '--wait', '--output-format', 'json'], capture=True)
        result = json.loads(response.stdout)
        submission = str(uuid.UUID(result['id']))
        if result.get('status') != 'Accepted':
            raise ValueError('Apple did not accept notarization; inspect the submission in Apple tools')
    except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as error:
        # Authentication tools' arbitrary stdout/stderr are not copied into evidence.
        raise ValueError('Apple notarization failed; credential diagnostics stay with the owner') from error
    command(['xcrun', 'stapler', 'staple', app])
    command(['xcrun', 'stapler', 'validate', app])
    command(['spctl', '--assess', '--type', 'execute', '--verbose=4', app])
    return {'id': submission, 'status': 'Accepted'}


def prepare(output: Path, version: str, apple_build: str, identity: str, auth):
    commit = frozen_commit()
    if output.is_symlink() or output.exists() or output.parent.resolve() != output.parent:
        raise ValueError('choose a new output directory under its real parent')
    output.mkdir(mode=0o700)
    env = dict(os.environ, AWL_SKIP_DMG='1', AWL_VERSION=version, AWL_BUILD_VERSION=apple_build,
               CARGO_INCREMENTAL='0', AWL_SOURCE_COMMIT=commit)
    submissions = {}
    for arch, target in ARCHES.items():
        progress(output, 'build', arch)
        command(['.orchestrator/worker-build.sh', 'scripts/with-remap.sh', 'scripts/project-rust.sh',
                 'cargo', 'build', '--release', '--target', target], env=env)
        binary = ROOT / f'target/{target}/release/awl'
        if os.fsencode(Path.home()) in binary.read_bytes():
            raise ValueError('the release binary retains its builder home path')
        progress(output, 'assemble', arch)
        command(['scripts/package-macos.sh', binary, output / arch], env=env)
        app = output / arch / 'Awl.app'
        progress(output, 'codesign_user_keychain', arch)
        try:
            command(['codesign', '--force', '--options', 'runtime', '--timestamp', '--sign', identity, app],
                    capture=True, timeout=60)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
            raise ValueError('existing keychain signing did not complete; owner approval stays in the OS dialog, with no ACL changes') from error
        command(['codesign', '--verify', '--deep', '--strict', '--verbose=2', app])
        progress(output, 'notarize_apple', arch)
        submissions[arch] = notarize(app, output / f'Awl-notarize-{arch}.zip', auth)
        progress(output, 'package', arch)
        command(['scripts/package-macos.sh', '--dmg-only', app,
                 output / f'awl-{version}-macos-{arch}.dmg', arch])
    progress(output, 'validate_actual_downloads')
    command(['scripts/verify-macos-release.sh', output, version, apple_build, 'signed', 'local'])
    if frozen_commit() != commit:
        raise ValueError('the checkout changed during local release preparation')
    artifacts = []
    for arch in ARCHES:
        dmg = output / f'awl-{version}-macos-{arch}.dmg'
        artifacts.append({'architecture': arch, 'file': dmg.name, 'bytes': dmg.stat().st_size,
                          'sha256': hashlib.sha256(dmg.read_bytes()).hexdigest(), 'notarization': submissions[arch]})
    manifest = {'format': 1, 'source_commit': commit, 'version': version, 'build_version': apple_build,
                'artifacts': artifacts, 'signed': True, 'notarized': True, 'verified': True}
    (output / 'local-macos-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    progress(output, 'verified')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--build-version', type=build_version, required=True)
    parser.add_argument('--identity', help='existing valid certificate fingerprint; no keychain permissions are changed')
    parser.add_argument('--key', type=Path, help='existing AuthKey_*.p8; bytes stay local')
    parser.add_argument('--issuer-file', type=Path, default=Path.home() / '.appstoreconnect/issuer_id')
    args = parser.parse_args()
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
    try:
        auth = credentials(args.key, args.issuer_file)
        identity = signing_identity(args.identity)
        manifest = prepare(args.output.absolute(), version, args.build_version, identity, auth)
        print(json.dumps({'source_commit': manifest['source_commit'], 'status': 'verified',
                          'artifacts': [{k: item[k] for k in ['architecture', 'bytes', 'sha256']} for item in manifest['artifacts']]}))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'local-macos-release: {type(error).__name__}; {error if isinstance(error, ValueError) else "operation failed; no credential values logged"}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

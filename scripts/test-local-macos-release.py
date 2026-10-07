#!/usr/bin/env python3
"""Pure signing metadata laws; no credentials, network, keychain or real app launches."""
import copy
import importlib.util
from pathlib import Path
import tempfile
import unittest
import sys

sys.dont_write_bytecode = True
from unittest import mock

ROOT = Path(__file__).resolve().parent.parent


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


stage = load('staged-macos-release')
local = load('local-macos-release')
signing = load('verify-macos-signature')


class StagedMetadata(unittest.TestCase):
    def setUp(self):
        self.commit = 'a' * 40
        self.version = '0.13.0'
        self.release = {
            'tag_name': f'macos-stage-v{self.version}-{self.commit}-b26.0.0',
            'draft': True, 'prerelease': True,
            'assets': [{'name': name, 'state': 'uploaded', 'size': 49_999_999,
                        'digest': 'sha256:' + 'b' * 64} for name in sorted(stage.filenames(self.version))],
        }
        self.ref = {'object': {'type': 'commit', 'sha': self.commit}}

    def validate(self, release=None, ref=None):
        return stage.validate_metadata(release or self.release, self.version, self.commit, ref or self.ref)

    def test_exact_candidate(self):
        self.assertEqual(self.validate(), '26.0.0')

    def test_every_metadata_axis_fails_closed(self):
        for key, value in [('draft', False), ('prerelease', False),
                           ('tag_name', 'v0.13.0'), ('tag_name', self.release['tag_name'].replace('26.0.0', 'dryrun')),
                           ('tag_name', self.release['tag_name'].replace(self.commit, 'c' * 40))]:
            changed = copy.deepcopy(self.release)
            changed[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.validate(changed)
        for key, value in [('type', 'tag'), ('sha', 'c' * 40)]:
            ref = copy.deepcopy(self.ref)
            ref['object'][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(ref=ref)
        for index in range(2):
            for key, value in [('name', '../other.dmg'), ('state', 'starter'), ('size', 0),
                               ('size', 50_000_000), ('size', 50_000_001), ('digest', None),
                               ('digest', 'sha256:' + 'z' * 64)]:
                changed = copy.deepcopy(self.release)
                changed['assets'][index][key] = value
                with self.subTest(index=index, key=key), self.assertRaises(ValueError):
                    self.validate(changed)
        for assets in [[], self.release['assets'][:1], self.release['assets'] * 2]:
            changed = dict(self.release, assets=assets)
            with self.assertRaises(ValueError):
                self.validate(changed)

    def test_missing_or_ambiguous_candidate_fails(self):
        for releases in [[], [self.release, self.release]]:
            with mock.patch.object(stage, 'gh', return_value=__import__('json').dumps([releases])):
                with self.assertRaises(ValueError):
                    stage.candidate('owner/repo', self.version, self.commit)

    def test_actual_bytes_size_digest_and_symlink(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            release = copy.deepcopy(self.release)
            for asset in release['assets']:
                path = directory / asset['name']
                path.write_bytes(b'synthetic dmg')
                asset.update(size=path.stat().st_size, digest=stage.digest(path))
            stage.verify_files(release, directory)
            path.write_bytes(b'changed')
            with self.assertRaises(ValueError):
                stage.verify_files(release, directory)
            asset.update(size=path.stat().st_size, digest=stage.digest(path))
            moved = path.with_suffix('.fixture')
            path.rename(moved)
            path.symlink_to(moved)
            with self.assertRaises(ValueError):
                stage.verify_files(release, directory)


class SignatureMetadata(unittest.TestCase):
    def test_codesign_actual_field_shape_and_each_security_axis(self):
        team = '2UPFLUAXMH'
        signature = "\n".join([
            'CodeDirectory v=20500 size=603284 flags=0x10000(runtime) hashes=18844+3 location=embedded',
            'Authority=Developer ID Application: Synthetic Owner (2UPFLUAXMH)',
            'TeamIdentifier=2UPFLUAXMH', 'Timestamp=7 Oct 2026 at 09:00:00',
        ])
        signing.verify_metadata(signature, team)
        for old, new in [('flags=0x10000', 'flags=0x0'),
                         ('CodeDirectory ', 'flags='),
                         ('Authority=Developer ID Application:', 'Authority=Apple Development:'),
                         ('(2UPFLUAXMH)', '(OTHERTEAM0)'),
                         ('TeamIdentifier=2UPFLUAXMH', 'TeamIdentifier=OTHERTEAM0'),
                         ('Timestamp=7 Oct 2026 at 09:00:00', 'Timestamp=none'),
                         ('Timestamp=', 'NoTimestamp=')]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                signing.verify_metadata(signature.replace(old, new), team)
        for line in signature.splitlines():
            with self.subTest(line=line), self.assertRaises(ValueError):
                signing.verify_metadata(signature.replace(line, ''), team)


class LocalMetadata(unittest.TestCase):
    def test_apple_build_version_boundary(self):
        for value in ['1.0.0', '9999.99.99', '26.0.0']:
            self.assertEqual(local.build_version(value), value)
        for value in ['0.0.0', '10000.0.0', '1.100.0', '1.0.100', 'dryrun', '01.0.0', '1.0.0\n']:
            with self.assertRaises(ValueError):
                local.build_version(value)

    def test_dirty_checkout_rejected_before_prepare(self):
        with mock.patch.object(local, 'command', return_value=mock.Mock(stdout=' M file')):
            with self.assertRaises(ValueError):
                local.frozen_commit()

    def test_notarization_failure_never_logs_auth_diagnostics(self):
        error = local.subprocess.CalledProcessError(1, ['notarytool'], output='private stdout', stderr='private stderr')
        with mock.patch.object(local, 'command', side_effect=[None, error]):
            with self.assertRaises(ValueError) as caught:
                local.notarize(Path('/synthetic/App'), Path('/synthetic/archive'), [])
            self.assertNotIn('private', str(caught.exception))


if __name__ == '__main__':
    unittest.main()

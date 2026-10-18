#!/usr/bin/env python3
"""Publication intent and immutable payload regression laws; no credentials used."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('release_plan', ROOT / 'scripts/release-plan.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleasePlan(unittest.TestCase):
    def plan(self, inputs=None, ref='refs/heads/main', event='workflow_dispatch', version='0.13.0'):
        return release.plan(event, ref, version, inputs or {})

    def test_default_and_credentialed_rehearsals_cannot_publish(self):
        for channel in ('preview', 'stable'):
            for credentialed in (True, False, 'true', 'false'):
                p = self.plan({'channel': channel, 'credentialed_macos_rehearsal': credentialed})
                self.assertEqual(p['is_release'], 'false')
                self.assertEqual(p['local_macos'], 'false')
                self.assertEqual(p['sign_macos'], str(credentialed).lower())
        self.assertEqual(self.plan({'local_macos_rehearsal': True})['local_macos'], 'true')

    def test_explicit_preview_and_stable_use_hosted_signed_path(self):
        for channel in ('preview', 'stable'):
            for dry in (False, 'false'):
                p = self.plan({'channel': channel, 'dry_run': dry})
                self.assertEqual(p['is_release'], 'true')
                self.assertEqual(p['sign_macos'], 'true')
                self.assertEqual(p['local_macos'], 'false')
                self.assertEqual(p['is_prerelease'], str(channel == 'preview').lower())

    def test_legacy_tag_fallback_and_version_agreement(self):
        p = self.plan(event='push', ref='refs/tags/v0.13.0')
        self.assertEqual(p['local_macos'], 'true')
        self.assertEqual(p['is_release'], 'true')
        self.assertEqual(p['sign_macos'], 'true')
        for args in (dict(event='push', ref='refs/tags/v0.12.0'),
                     dict(version='0.13.0-beta.1'), dict(event='pull_request'),
                     dict(inputs={'dry_run': False}, ref='refs/heads/feature'),
                     dict(inputs={'dry_run': False, 'local_macos_rehearsal': True}),
                     dict(inputs={'local_macos_rehearsal': True, 'credentialed_macos_rehearsal': True}),
                     dict(inputs={'channel': 'surprise'})):
            with self.subTest(args=args), self.assertRaises(ValueError):
                self.plan(**args)

    def test_publication_refuses_existing_release_or_wrong_source(self):
        source = 'a' * 40
        commit = {'object': {'type': 'commit', 'sha': source}}
        for responses in ([{'draft': True}], [{'draft': False}],
                          [None, {'object': {'type': 'commit', 'sha': 'b' * 40}}],
                          [None, {'object': {'type': 'blob', 'sha': source}}]):
            with patch.object(release, 'api', side_effect=responses), self.assertRaises(ValueError):
                release.check_publication('owner/repo', '0.13.0', source)
        for responses in ([None, None], [None, commit],
                          [None, {'object': {'type': 'tag', 'sha': 'c' * 40}}, commit]):
            with patch.object(release, 'api', side_effect=responses):
                release.check_publication('owner/repo', '0.13.0', source)

    def test_only_complete_identical_uploaded_draft_can_publish(self):
        import copy
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            names = ['SHA256SUMS'] + ['awl-0.13.0-' + suffix for suffix in (
                'linux-x86_64.tar.gz', 'linux-x86_64.AppImage',
                'macos-arm64.dmg', 'macos-x86_64.dmg')]
            good = {'draft': True, 'assets': []}
            for name in names:
                data = name.encode()
                (directory / name).write_bytes(data)
                good['assets'].append({'name': name, 'size': len(data),
                                       'digest': 'sha256:' + hashlib.sha256(data).hexdigest()})
            with patch.object(release, 'api', return_value=good):
                release.check_uploaded('owner/repo', '0.13.0', directory)
            bads = [dict(good, draft=False), dict(good, assets=good['assets'][:-1])]
            for key, value in [('digest', 'sha256:wrong'), ('size', 999), ('name', 'unexpected')]:
                bad = copy.deepcopy(good)
                bad['assets'][0][key] = value
                bads.append(bad)
            for bad in bads:
                with patch.object(release, 'api', return_value=bad), self.assertRaises(ValueError):
                    release.check_uploaded('owner/repo', '0.13.0', directory)



class Payload(unittest.TestCase):
    def test_exact_four_bytes_and_tamper_failures(self):
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            linux, mac = root / 'linux', root / 'mac'
            linux.mkdir(); mac.mkdir()
            names = ['awl-0.13.0-linux-x86_64.tar.gz', 'awl-0.13.0-linux-x86_64.AppImage',
                     'awl-0.13.0-macos-arm64.dmg', 'awl-0.13.0-macos-x86_64.dmg']
            for i, name in enumerate(names):
                path = (linux if i < 2 else mac) / name
                path.write_bytes(('synthetic final bytes ' + name).encode())
                path.with_name(name + '.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + name + '\n')
            def prepare(out):
                return subprocess.run(['bash', str(ROOT / 'scripts/prepare-release-payload.sh'),
                                       str(linux), str(mac), str(out), '0.13.0'], capture_output=True)
            out = root / 'payload'
            result = prepare(out)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual({p.name for p in out.iterdir()}, set(names) | {'SHA256SUMS'})
            for i, name in enumerate(names):
                self.assertEqual((out / name).read_bytes(), ((linux if i < 2 else mac) / name).read_bytes())
            (mac / names[-1]).write_bytes(b'tampered after verification')
            self.assertNotEqual(prepare(root / 'rejected').returncode, 0)
            (out / names[0]).write_bytes(b'tampered after upload')
            self.assertNotEqual(subprocess.run(['sha256sum', '-c', 'SHA256SUMS'], cwd=out,
                                              capture_output=True).returncode, 0)


if __name__ == '__main__':
    unittest.main()

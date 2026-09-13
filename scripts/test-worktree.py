#!/usr/bin/env python3
"""Exercise lifecycle deletion against disposable real Git checkouts."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

HELPER = Path(__file__).with_name('worktree.py').resolve()


class Lifecycle(unittest.TestCase):
    def test_retirement_preserves_unreviewed_and_unmerged_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fake_bin = root / '.git-test-tools'
            fake_bin.mkdir()
            (fake_bin / 'ps').write_text('#!/bin/sh\nexit 0\n')
            (fake_bin / 'ps').chmod(0o755)
            environment = dict(os.environ, PATH=str(fake_bin) + os.pathsep + os.environ['PATH'])
            def git(*args):
                return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.STDOUT)
            def run(*args, succeeds=True):
                result = subprocess.run(['python3', str(HELPER), '--repo', str(root), *args],
                                        capture_output=True, text=True, env=environment)
                self.assertEqual(result.returncode == 0, succeeds, result.stderr)
                return result.stdout
            git('init', '-b', 'main')
            git('config', 'user.email', 'test@example.invalid')
            git('config', 'user.name', 'Test')
            (root / '.gitignore').write_text('target/\n.worktrees/\nreview/\n.git-test-tools/\n')
            git('add', '.gitignore')
            git('-c', 'commit.gpgsign=false', 'commit', '-m', 'fixture')
            lane = root / '.worktrees' / 'baseline'
            run('create', 'baseline', '--owner', 'test', '--purpose', 'baseline')
            (lane / 'target').mkdir()
            (lane / 'target' / 'fresh').write_text('fresh build')
            run('state', str(lane), 'awaiting-review', '--owner', 'test', '--reason', 'taste',
                '--review', 'target/fresh')
            run('sweep', '--apply')
            self.assertTrue((lane / 'target' / 'fresh').exists())
            run('state', str(lane), 'retired', '--owner', 'test', '--reason', 'done', succeeds=False)
            run('state', str(lane), 'retired', '--owner', 'test', '--reason', 'done',
                '--stopped', '--preserved')
            run('sweep')
            self.assertTrue(lane.exists(), 'preview deleted a fresh retired checkout')
            (lane / 'untracked').write_text('keep')
            run('sweep', '--apply')
            self.assertTrue(lane.exists())
            (lane / 'untracked').unlink()
            (lane / 'review').mkdir()
            (lane / 'review' / 'image').write_text('evidence')
            run('sweep', '--apply')
            self.assertTrue((lane / 'review' / 'image').exists())
            (lane / 'review' / 'image').unlink()
            (lane / 'review').rmdir()
            (fake_bin / 'ps').write_text('#!/bin/sh\necho cargo\n')
            run('sweep', '--apply', succeeds=False)
            self.assertTrue(lane.exists())
            (fake_bin / 'ps').write_text('#!/bin/sh\nexit 1\n')
            run('sweep', '--apply', succeeds=False)
            self.assertTrue(lane.exists())
            (fake_bin / 'ps').write_text('#!/bin/sh\nexit 0\n')
            run('sweep', '--apply')
            self.assertFalse(lane.exists(), 'fresh retired worktree was not removed')
            run('create', 'unique', '--owner', 'test', '--purpose', 'unmerged')
            unique = root / '.worktrees' / 'unique'
            (unique / 'source').write_text('unique work')
            subprocess.check_call(['git', '-C', str(unique), 'add', 'source'])
            subprocess.check_call(['git', '-C', str(unique), '-c', 'commit.gpgsign=false',
                                   'commit', '-qm', 'unique'])
            run('state', str(unique), 'retired', '--owner', 'test', '--reason', 'done',
                '--stopped', '--preserved')
            self.assertIn('HEAD not preserved', run('sweep', '--apply'))
            self.assertTrue(unique.exists())
            registry = root / '.git' / 'awl-worktrees.json'
            registry.write_text('{}')
            self.assertIn('unknown', run('sweep', '--apply'))
            self.assertTrue(unique.exists())


if __name__ == '__main__':
    unittest.main()

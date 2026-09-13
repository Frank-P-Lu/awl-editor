#!/usr/bin/env python3
"""Create owned checkouts and retire explicitly released worktrees."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import re
import subprocess
import sys


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args]).decode().strip()


def roster(root):
    records = []
    for block in git(root, 'worktree', 'list', '--porcelain', '-z').split('\0\0'):
        fields = dict(line.split(' ', 1) if ' ' in line else (line, True)
                      for line in block.split('\0') if line)
        if 'worktree' in fields:
            records.append(fields)
    return records


def require_idle(root):
    # Fail closed when the process table cannot be read, including direct use
    # of the Python entry point rather than the sweep.sh wrapper.
    processes = subprocess.check_output(['ps', '-axo', 'comm=']).decode().splitlines()
    if any(Path(command.strip()).name in ('cargo', 'rustc') for command in processes):
        raise ValueError('build processes are running; refusing retirement')
    marker = root / '.orchestrator' / 'native-gate.marker'
    if marker.exists():
        match = re.search(r'\bpid=(\d+)', marker.read_text())
        if not match:
            raise ValueError('unreadable gate ownership; refusing retirement')
        try:
            os.kill(int(match[1]), 0)
        except ProcessLookupError:
            pass
        else:
            raise ValueError('native gate is running; refusing retirement')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', default=str(Path(__file__).resolve().parent.parent))
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('create')
    create.add_argument('name')
    create.add_argument('--ref', default='main')
    create.add_argument('--branch')
    create.add_argument('--owner', required=True)
    create.add_argument('--purpose', required=True)
    state = commands.add_parser('state')
    state.add_argument('path')
    state.add_argument('state', choices=['active', 'awaiting-review', 'reusable', 'retired'])
    state.add_argument('--owner', required=True)
    state.add_argument('--reason', required=True)
    state.add_argument('--review', default='')
    state.add_argument('--stopped', action='store_true')
    state.add_argument('--preserved', action='store_true')
    commands.add_parser('list')
    sweep = commands.add_parser('sweep')
    sweep.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    common = Path(git(args.repo, 'rev-parse', '--path-format=absolute', '--git-common-dir'))
    root = common.parent
    registry = common / 'awl-worktrees.json'
    with (common / 'awl-worktrees.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        data = json.loads(registry.read_text()) if registry.exists() else {}
        trees = roster(root)
        if args.command == 'sweep' and args.apply:
            require_idle(root)
        if args.command == 'create':
            if not re.fullmatch(r'[a-zA-Z0-9][a-zA-Z0-9_-]*', args.name):
                parser.error('name must contain only letters, digits, underscores and hyphens')
            path = root / '.worktrees' / args.name
            if path.parent.is_symlink():
                parser.error('.worktrees must not be a symlink')
            options = ['-b', args.branch] if args.branch else ['--detach']
            git(root, 'worktree', 'add', *options, str(path), args.ref)
            data[str(path)] = dict(state='active', owner=args.owner, reason=args.purpose, review='')
            print(path)
        elif args.command == 'state':
            path = str(Path(args.path).resolve())
            if path == str(root) or path not in [t['worktree'] for t in trees]:
                parser.error('path must be a registered secondary worktree')
            if args.state == 'retired' and not (args.stopped and args.preserved):
                parser.error('retirement requires --stopped and --preserved attestations')
            if args.state == 'awaiting-review' and not args.review:
                parser.error('awaiting-review requires --review with the review location')
            data[path] = dict(state=args.state, owner=args.owner, reason=args.reason, review=args.review)
        else:
            for tree in trees:
                path = Path(tree['worktree'])
                record = data.get(str(path), {})
                status = record.get('state', 'unknown')
                reason = record.get('reason', 'no lifecycle record; keep')
                eligible = status == 'retired' and path != root
                # Git's ordinary removal protects untracked/ignored review artifacts.
                # Only a conventional, non-symlink target directory is disposable.
                if eligible:
                    if not path.is_dir() or path.is_symlink() or 'locked' in tree:
                        eligible, reason = False, 'missing, symlinked or locked checkout'
                    elif git(path, 'status', '--porcelain', '--untracked-files=all'):
                        eligible, reason = False, 'uncommitted or untracked work'
                    elif subprocess.run(['git', '-C', str(root), 'merge-base', '--is-ancestor',
                                         tree['HEAD'], 'main'], capture_output=True).returncode:
                        eligible, reason = False, 'HEAD not preserved on main'
                    elif (path / 'target').is_symlink():
                        eligible, reason = False, 'target is a symlink'
                    else:
                        ignored = git(path, 'ls-files', '--others', '--ignored', '--exclude-standard').splitlines()
                        if any(not name.startswith('target/') for name in ignored):
                            eligible, reason = False, 'ignored files outside target need preservation'
                size = subprocess.run(['du', '-sk', str(path)], capture_output=True, text=True)
                kib = size.stdout.split()[0] if size.returncode == 0 else '?'
                location = 'canonical' if path.parent == root / '.worktrees' else 'external'
                print(f'{path}\t{kib} KiB\t{status}\t{location}\t'
                      f'{"eligible" if eligible else "keep"}: {reason}\t{record.get("review", "")}', flush=True)
                if args.command == 'sweep' and args.apply and eligible:
                    # Do not force Git removal: it must still reject newly dirty work.
                    import shutil
                    target = path / 'target'
                    if target.is_dir():
                        shutil.rmtree(target)
                    git(root, 'worktree', 'remove', str(path))
                    del data[str(path)]
        if args.command in ('create', 'state') or (args.command == 'sweep' and args.apply):
            temporary = registry.with_suffix('.tmp')
            temporary.write_text(json.dumps(data, indent=2) + '\n')
            temporary.replace(registry)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        sys.exit(f'worktree: {error}')

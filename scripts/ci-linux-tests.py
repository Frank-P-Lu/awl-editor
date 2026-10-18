#!/usr/bin/env python3
"""Run one bounded Linux correctness lane; exhaustive matrices stay in the full gate."""
from __future__ import annotations

import argparse
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
DEEP_TARGETS = frozenset({
    'caret_mono_grid_pixels', 'caret_punctuation_color', 'caret_punctuation_pixels',
})
PIXEL_REGRESSIONS = ('asterisk_in_bombora_does_not_invert',
                     'comma_and_roster_in_bowerbird_never_flip_black')
REQUIRED_CORE = frozenset({'fault_kill9', 'persistence_real_process', 'native_gate_canary', 'hermetic_canary'})


def run(command, env, **kwargs):
    return subprocess.run(command, cwd=ROOT, env=env, check=True, **kwargs)


def tested(command, env, minimum=1):
    passed = 0
    with subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True) as process:
        for line in process.stdout:
            print(line, end='', flush=True)
            match = re.search(r'test result: ok\. (\d+) passed;', line)
            if match:
                passed += int(match[1])
        status = process.wait()
    if status:
        raise subprocess.CalledProcessError(status, command)
    if passed < minimum:
        raise ValueError(f'only {passed} tests passed; required {minimum}: {command}')


def listed(command, env):
    result = run(command, env, capture_output=True, text=True)
    names = {line.removesuffix(': test') for line in result.stdout.splitlines()
             if line.endswith(': test')}
    if not names:
        raise ValueError(f'empty test selection: {command}')
    return names


def execute(convention):
    env = dict(os.environ, AWL_CONVENTION_FORCE=convention, AWL_MENU_BAR_FORCE="on")
    env.setdefault('RUST_TEST_THREADS', '2')
    phases = []

    def phase(name, operation):
        start = time.monotonic()
        print(f'linux-fast-phase convention={convention} phase={name} begin', flush=True)
        operation()
        elapsed = time.monotonic() - start
        phases.append((name, elapsed))
        print(f'linux-fast-phase convention={convention} phase={name} seconds={elapsed:.2f}', flush=True)

    with tempfile.TemporaryDirectory(prefix='awl-linux-fast-') as folder:
        scratch = Path(folder)
        artifacts = scratch / 'artifacts'
        cargo_json = scratch / 'cargo.jsonl'

        def compile_all():
            # Compile every native target, including the deferred matrices.
            with cargo_json.open('w') as output:
                run(['cargo', 'test', '--no-run', '--message-format=json'], env, stdout=output)
            run([sys.executable, str(ROOT / 'scripts/native-test-shards.py'),
                 'artifacts', str(cargo_json), str(artifacts)], env)

        phase('compile-all-targets', compile_all)
        rows = [line.split('=', 1) for line in artifacts.read_text().splitlines()]
        binary = next(value for key, value in rows if key == 'binary')
        integrations = {value for key, value in rows if key == 'integration'}
        if not DEEP_TARGETS <= integrations:
            raise ValueError(f'deep target roster missing: {sorted(DEEP_TARGETS - integrations)}')
        if not REQUIRED_CORE <= integrations:
            raise ValueError(f'core target roster missing: {sorted(REQUIRED_CORE - integrations)}')
        fast = sorted(integrations - DEEP_TARGETS)
        if not fast:
            raise ValueError('no core integration targets discovered')
        # New targets join fast CI automatically; only these three named targets
        # defer execution. The unfiltered nightly/prerelease gate runs all of them.
        unit_args = ['--skip', 'render::tests']
        units = listed([binary, '--list', *unit_args], env)
        menu_args = ['menubar', 'menu_bar', *unit_args]
        menus = listed([binary, '--list', *menu_args], env)
        canaries = listed(['cargo', 'test', '--test', 'caret_punctuation_color',
                           '--', '--list'], env)
        if not set(PIXEL_REGRESSIONS) <= canaries:
            raise ValueError(f'missing real-pixel regressions: {set(PIXEL_REGRESSIONS) - canaries}')
        print(f'linux-fast-roster convention={convention} units={len(units)} '
              f'integrations={len(fast)} menu_laws={len(menus)} '
              f'deep={",".join(sorted(DEEP_TARGETS))}', flush=True)
        phase('units', lambda: tested([binary, *unit_args], env))
        integration_command = ['cargo', 'test']
        for target in fast:
            integration_command.extend(['--test', target])
        phase('core-integrations', lambda: tested(integration_command, env))
        phase('caret-pixel-regressions', lambda: tested([
            'cargo', 'test', '--test', 'caret_punctuation_color', '--', '--exact', *PIXEL_REGRESSIONS], env, minimum=len(PIXEL_REGRESSIONS)))
        phase('menu-off-laws', lambda: tested([binary, *menu_args],
                                                dict(env, AWL_MENU_BAR_FORCE='off'), minimum=len(menus)))
    print(f'linux-fast: passed convention={convention}; targeted coverage, NOT a full-native receipt', flush=True)
    summary = os.environ.get('GITHUB_STEP_SUMMARY')
    if summary:
        with open(summary, 'a') as output:
            output.write(f'### Linux correctness: {convention}\n\n| Phase | Seconds |\n| --- | ---: |\n')
            for name, elapsed in phases:
                output.write(f'| {name} | {elapsed:.2f} |\n')
            output.write('\nAll native targets compiled. Exhaustive caret matrices run in nightly/prerelease.\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('convention', choices=['mac', 'linux'])
    args = parser.parse_args()
    try:
        execute(args.convention)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f'linux-fast: {error}')

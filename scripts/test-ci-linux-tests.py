#!/usr/bin/env python3
"""Prove fast-lane coverage selection, nonempty smoke and failure propagation."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent


class FastLane(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory(prefix='awl-fast-law-')
        self.addCleanup(self.folder.cleanup)
        self.root = Path(self.folder.name)
        (self.root / 'scripts').mkdir()
        (self.root / 'bin').mkdir()
        for name in ('ci-linux-tests.py', 'native-test-shards.py'):
            shutil.copy(ROOT / 'scripts' / name, self.root / 'scripts' / name)
        self.env = dict(os.environ, PATH=f'{self.root}/bin:' + os.environ['PATH'],
                        LAW_ROOT=str(self.root), LAW_LOG=str(self.root / 'calls.jsonl'))
        fake = f'''#!{sys.executable}
import json,os,pathlib,sys
root=pathlib.Path(os.environ['LAW_ROOT']); args=sys.argv[1:]; name=pathlib.Path(sys.argv[0]).name
with open(os.environ['LAW_LOG'],'a') as out:
 out.write(json.dumps([name,args,os.environ.get('AWL_CONVENTION_FORCE'),os.environ.get('AWL_MENU_BAR_FORCE')])+'\\n')
fault=os.environ.get('LAW_FAIL','')
if name=='cargo' and '--no-run' in args:
 if fault=='compile': sys.exit(7)
 targets=['caret_mono_grid_pixels','caret_punctuation_color','caret_punctuation_pixels','fault_kill9','persistence_real_process','native_gate_canary','hermetic_canary','new_core_target']
 if fault=='missing-deep': targets.remove('caret_mono_grid_pixels')
 if fault=='missing-core': targets.remove('persistence_real_process')
 print(json.dumps({{'reason':'compiler-artifact','executable':str(root/'bin/unit'),'target':{{'name':'awl','kind':['bin']}},'profile':{{'test':True}}}}))
 for target in targets:
  print(json.dumps({{'reason':'compiler-artifact','executable':'/unused','target':{{'name':target,'kind':['test']}},'profile':{{'test':True}}}}))
 sys.exit(0)
if '--list' in args:
 if name=='cargo':
  print('asterisk_in_bombora_does_not_invert: test')
  if fault!='missing-canary': print('comma_and_roster_in_bowerbird_never_flip_black: test')
 elif fault!='empty-units': print('menubar::law: test')
 sys.exit(0)
phase='menu' if name=='unit' and 'menubar' in args else 'unit' if name=='unit' else 'canary' if '--exact' in args else 'integration'
if fault==phase: sys.exit(7)
count = 0 if os.environ.get('LAW_EMPTY')==phase else 2 if phase=='canary' else 1
print(f'test result: ok. {{count}} passed; 0 failed; 0 ignored;')
'''
        for name in ('cargo', 'unit'):
            p = self.root / 'bin' / name
            p.write_text(fake)
            p.chmod(0o755)

    def lane(self, fault='', convention='mac'):
        return subprocess.run([sys.executable, str(self.root / 'scripts/ci-linux-tests.py'), convention],
                              env=dict(self.env, LAW_FAIL=fault), capture_output=True, text=True, timeout=15)

    def calls(self):
        return [json.loads(line) for line in (self.root / 'calls.jsonl').read_text().splitlines()]

    def test_every_new_integration_joins_fast_and_deep_matrices_stay_deferred(self):
        result = self.lane()
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls()
        integration = next(c for c in calls if c[0] == 'cargo' and 'new_core_target' in c[1])
        for target in ('fault_kill9', 'persistence_real_process', 'native_gate_canary', 'hermetic_canary'):
            self.assertIn(target, integration[1])
        for deep in ('caret_mono_grid_pixels', 'caret_punctuation_pixels', 'caret_punctuation_color'):
            self.assertNotIn(deep, integration[1])
        canary = next(c for c in calls if '--exact' in c[1])
        self.assertIn('asterisk_in_bombora_does_not_invert', canary[1])
        self.assertIn('comma_and_roster_in_bowerbird_never_flip_black', canary[1])
        self.assertIn('caret_punctuation_color', canary[1])
        menus = next(c for c in calls if c[0] == 'unit' and 'menubar' in c[1] and '--list' not in c[1])
        self.assertEqual(menus[3], 'off')
        normal = next(c for c in calls if c[0] == 'unit' and '--list' not in c[1] and 'menubar' not in c[1])
        self.assertEqual(normal[3], 'on')
        self.assertTrue(all(c[2] == 'mac' for c in calls))
        self.assertIn('NOT a full-native receipt', result.stdout)
        self.assertIn('phase=compile-all-targets', result.stdout)

    def test_each_real_execution_phase_fails_closed_and_stops_later_checks(self):
        for fault in ('compile', 'unit', 'integration', 'canary', 'menu'):
            with self.subTest(fault=fault):
                result = self.lane(fault)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn('linux-fast: passed', result.stdout)
                self.assertNotIn(f'phase={dict(unit="units", integration="core-integrations", canary="caret-pixel-regressions", menu="menu-off-laws", compile="compile-all-targets")[fault]} seconds=', result.stdout)

    def test_zero_passed_cannot_report_a_successful_selection(self):
        for phase in ('unit', 'integration', 'canary', 'menu'):
            with self.subTest(phase=phase):
                self.env['LAW_EMPTY'] = phase
                result = self.lane()
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn('linux-fast: passed', result.stdout)

    def test_missing_deep_core_or_canary_and_empty_unit_selection_refuse(self):
        for fault in ('missing-deep', 'missing-core', 'missing-canary', 'empty-units'):
            with self.subTest(fault=fault):
                self.assertNotEqual(self.lane(fault).returncode, 0)

    def test_linux_axis_and_nonempty_step_summary(self):
        summary = self.root / 'summary'
        self.env['GITHUB_STEP_SUMMARY'] = str(summary)
        result = self.lane(convention='linux')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(all(c[2] == 'linux' for c in self.calls()))
        self.assertIn('caret-pixel-regressions', summary.read_text())

    def test_actual_aggregate_rejects_failed_cancelled_skipped_or_missing_lane(self):
        ci = (ROOT / '.github/workflows/ci.yml').read_text()
        command = next(line.split('run: ', 1)[1] for line in ci.splitlines()
                       if 'run: test "$HEALTH_RESULT"' in line)
        for health, tests in [('success', 'success'), ('failure', 'success'),
                              ('success', 'failure'), ('success', 'cancelled'),
                              ('skipped', 'success'), ('success', '')]:
            result = subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', command],
                                    env=dict(os.environ, HEALTH_RESULT=health, TEST_RESULT=tests))
            self.assertEqual(result.returncode == 0, health == tests == 'success')


if __name__ == '__main__':
    unittest.main()

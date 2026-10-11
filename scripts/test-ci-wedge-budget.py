#!/usr/bin/env python3
"""Real descendants witness the hosted watchdog's process-group contract."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

SCRIPT = Path(__file__).resolve().parent / 'ci-wedge-budget.sh'

class BudgetLaws(unittest.TestCase):
    def run_case(self, mode, cancel=False):
        with tempfile.TemporaryDirectory(prefix='awl-wedge-law-') as raw:
            root = Path(raw)
            cargo = root / 'cargo'
            cargo.write_text('''#!/usr/bin/env python3
import os, signal, subprocess, sys, time
from pathlib import Path
root=Path(os.environ['LAW_ROOT'])
mode=os.environ['LAW_MODE']
if mode == 'fail': sys.exit(23)
if mode == 'pass': sys.exit(0)
child=subprocess.Popen([sys.executable,'-c',"import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); time.sleep(60)"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
(root/'child').write_text(str(child.pid))
if mode == 'early': sys.exit(0)
while True: time.sleep(.1)
''')
            cargo.chmod(0o755)
            env = dict(os.environ, PATH=str(root)+os.pathsep+os.environ['PATH'], LAW_ROOT=raw, LAW_MODE=mode)
            proc = subprocess.Popen(['bash', str(SCRIPT), 'mac', 'render::tests::', '0.7'], env=env, start_new_session=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            child = None
            try:
                if mode not in ('pass','fail'):
                    deadline=time.monotonic()+4
                    while not (root/'child').exists() and time.monotonic()<deadline: time.sleep(.02)
                    self.assertTrue((root/'child').exists(), 'fixture child started')
                    child=int((root/'child').read_text())
                if cancel: proc.send_signal(signal.SIGTERM)
                output, _ = proc.communicate(timeout=8)
                if child:
                    # Zombies are terminal: no executing child survives the wrapper.
                    state=subprocess.run(['ps','-o','stat=','-p',str(child)], capture_output=True,text=True).stdout.strip()
                    self.assertTrue(not state or state.startswith('Z'), f'descendant {child} still executing ({state}): {output}')
                return proc.returncode, output
            finally:
                if child:
                    try: os.kill(child,signal.SIGKILL)
                    except ProcessLookupError: pass
                try: os.killpg(proc.pid,signal.SIGKILL)
                except ProcessLookupError: pass
                proc.wait(timeout=3)
                proc.stdout.close()

    def test_deadline_reaps_the_actual_test_descendant(self):
        status, output=self.run_case('hang')
        self.assertEqual(status,137)
        self.assertIn('HUNG past',output)

    def test_cargo_exit_cannot_leave_a_test_descendant(self):
        self.assertEqual(self.run_case('early')[0],0)

    def test_caller_termination_cleans_descendants(self):
        self.assertEqual(self.run_case('hang',cancel=True)[0],143)

    def test_ordinary_exit_status_is_preserved(self):
        self.assertEqual(self.run_case('pass')[0],0)
        self.assertEqual(self.run_case('fail')[0],23)

if __name__ == '__main__': unittest.main()

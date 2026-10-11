#!/usr/bin/env python3
"""Real descendants witness the hosted watchdog's process-group contract."""
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

sys.dont_write_bytecode = True

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
if mode == 'kill': os.kill(os.getpid(), signal.SIGKILL)
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
                if mode not in ('pass','fail','kill'):
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

    def test_cancellation_during_spawn_retains_process_ownership(self):
        with tempfile.TemporaryDirectory(prefix='awl-wedge-spawn-law-') as raw:
            root = Path(raw)
            harness = root / 'spawn.py'
            harness.write_text("""import importlib.util, os, signal, subprocess, sys
from pathlib import Path
spec=importlib.util.spec_from_file_location('owner',sys.argv[1])
owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
real=subprocess.Popen
pid_file=Path(sys.argv[2])
def spawn(*args,**kwargs):
    child=real([sys.executable,'-c','import time; time.sleep(60)'],start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    pid_file.write_text(str(child.pid))
    owner.subprocess.Popen=real
    # Deliver cancellation after creation, before Popen returns to its owner.
    os.kill(os.getpid(),signal.SIGTERM)
    return child
owner.subprocess.Popen=spawn
sys.argv=['owner','mac','render::tests::','2']
status,_=owner.main()
sys.exit(status)
""")
            pid_file = root / 'pid'
            proc = subprocess.Popen(['python3','-B',str(harness),str(SCRIPT.parent/'ci-wedge-process.py'),str(pid_file)], start_new_session=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            child = None
            try:
                output, _ = proc.communicate(timeout=8)
                self.assertTrue(pid_file.exists(), output)
                child = int(pid_file.read_text())
                state = subprocess.run(['ps','-o','stat=','-p',str(child)], capture_output=True,text=True).stdout.strip()
                self.assertTrue(not state or state.startswith('Z'), f'spawned process {child} escaped cancellation ({state})')
                self.assertEqual(proc.returncode,143,output)
            finally:
                if child is None and pid_file.exists(): child=int(pid_file.read_text())
                if child:
                    try: os.killpg(child,signal.SIGKILL)
                    except ProcessLookupError: pass
                try: os.killpg(proc.pid,signal.SIGKILL)
                except ProcessLookupError: pass
                proc.wait(timeout=3)
                proc.stdout.close()

    def test_sigkill_before_deadline_is_not_a_watchdog_timeout(self):
        status, output = self.run_case('kill')
        self.assertEqual(status, 137)
        self.assertNotIn('HUNG past', output)
        self.assertIn('failed before watchdog deadline', output)

    def test_ordinary_exit_status_is_preserved(self):
        self.assertEqual(self.run_case('pass')[0],0)
        self.assertEqual(self.run_case('fail')[0],23)

if __name__ == '__main__': unittest.main()

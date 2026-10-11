#!/usr/bin/env python3
"""Check the survival oracle against real live, stopped and exited processes."""
import os
from pathlib import Path
import signal
import subprocess
import unittest


ORACLE = Path(__file__).with_name("native-gate-test-process.sh")


def running(pid, ps_stub=""):
    command = ps_stub + '. "$1"; native_gate_probe_running "$2"'
    return subprocess.run(
        ["bash", "-c", command, "probe", str(ORACLE), str(pid)],
        check=False,
    ).returncode == 0


class ProcessSurvival(unittest.TestCase):
    def test_live_descendant_is_a_survivor(self):
        with subprocess.Popen(["sleep", "30"]) as child:
            try:
                self.assertTrue(running(child.pid))
            finally:
                child.terminate()

    def test_stopped_descendant_is_a_survivor(self):
        with subprocess.Popen(["sleep", "30"]) as child:
            try:
                os.kill(child.pid, signal.SIGSTOP)
                self.assertTrue(running(child.pid))
            finally:
                os.kill(child.pid, signal.SIGCONT)
                child.terminate()

    def test_reaped_process_is_not_a_survivor(self):
        child = subprocess.Popen(["true"])
        child.wait()
        self.assertFalse(running(child.pid))

    @unittest.skipUnless(hasattr(os, "waitid") and hasattr(os, "WNOWAIT"),
                         "requires an unreaped exit observation")
    def test_zombie_has_closed_its_pipe_despite_kill_zero_succeeding(self):
        read_fd, write_fd = os.pipe()
        pid = os.fork()
        if pid == 0:
            os.close(read_fd)
            os._exit(0)
        os.close(write_fd)
        try:
            os.waitid(os.P_PID, pid, os.WEXITED | os.WNOWAIT)
            state = subprocess.check_output(
                ["ps", "-o", "stat=", "-p", str(pid)], text=True
            ).strip()
            self.assertTrue(state.startswith("Z"), state)
            os.kill(pid, 0)
            self.assertEqual(os.read(read_fd, 1), b"")
            self.assertFalse(running(pid))
        finally:
            os.close(read_fd)
            os.waitpid(pid, 0)

    def test_unavailable_state_does_not_hide_a_live_process(self):
        self.assertTrue(running(os.getpid(), "ps() { return 1; }; "))

    def test_empty_state_does_not_hide_a_live_process(self):
        self.assertTrue(running(os.getpid(), "ps() { return 0; }; "))


if __name__ == "__main__":
    unittest.main()

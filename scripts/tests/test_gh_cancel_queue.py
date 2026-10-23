import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "gh-cancel-queue.sh"


class CancelQueueTests(unittest.TestCase):
    bash = "bash"

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        directory = Path(self.temp.name)
        self.log = directory / "calls"
        mock = directory / "gh"
        mock.write_text(
            """#!/usr/bin/env bash
set -eu
printf '%s\\n' "$*" >> "$CALL_LOG"
if [[ "$1 $2" == "run list" ]]; then
  [[ "${LIST_FAIL:-0}" == 0 ]] || { echo "list failed" >&2; exit 1; }
  [[ "${EMPTY:-0}" == 0 ]] || exit 0
  case "$4" in
    in_progress) printf '101\\tin_progress\\tCI\\tmaster\\t2026-10-23\\n' ;;
    queued) printf '202\\tqueued\\tbuild-artifact\\tmaster\\t2026-10-23\\n' ;;
    waiting) printf '303\\twaiting\\tCD\\tmaster\\t2026-10-23\\n' ;;
  esac
elif [[ "$1 $2" == "run cancel" ]]; then
  [[ "$3" != "${CANCEL_FAIL:-none}" ]] || exit 1
else
  exit 2
fi
"""
        )
        mock.chmod(0o755)
        self.env = {
            **os.environ,
            "PATH": f"{directory}:{os.environ['PATH']}",
            "CALL_LOG": str(self.log),
        }

    def run_script(self, answers="", args=(), **env):
        return subprocess.run(
            [self.bash, str(SCRIPT), *args],
            input=answers,
            text=True,
            capture_output=True,
            env={**self.env, **env},
            timeout=10,
        )

    def cancellations(self):
        if not self.log.exists():
            return []
        return [
            line for line in self.log.read_text().splitlines()
            if line.startswith("run cancel ")
        ]

    def test_selected_numbers_and_duplicates(self):
        result = self.run_script("3,1 3\ny\n", args=("12",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.cancellations(), ["run cancel 101", "run cancel 303"])
        self.assertIn("--limit 12", self.log.read_text())
        self.assertIn("build-artifact", result.stdout)

    def test_all(self):
        result = self.run_script("all\ny\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            self.cancellations(),
            ["run cancel 101", "run cancel 202", "run cancel 303"],
        )

    def test_quit_empty_eof_and_declined_confirmation(self):
        for answers in ("q\n", "\n", "", "1\nn\n", "all\n"):
            with self.subTest(answers=answers):
                result = self.run_script(answers)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.cancellations(), [])

    def test_invalid_selection_never_partially_cancels(self):
        for selection in ("1 4", "all invalid", "0", "-1", "999999999999999999999"):
            with self.subTest(selection=selection):
                result = self.run_script(f"{selection}\ny\n")
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("invalid selection", result.stderr)
                self.assertEqual(self.cancellations(), [])

    def test_invalid_depth(self):
        result = self.run_script(args=("0",))
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.log.exists())

    def test_empty_queue(self):
        result = self.run_script(EMPTY="1")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("NO ACTIVE WORKFLOW RUNS", result.stdout)
        self.assertEqual(self.cancellations(), [])

    def test_listing_error_is_not_hidden(self):
        result = self.run_script("all\ny\n", LIST_FAIL="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("list failed", result.stderr)
        self.assertEqual(self.cancellations(), [])

    def test_cancellation_error_is_reported_and_remaining_runs_attempted(self):
        result = self.run_script("all\ny\n", CANCEL_FAIL="202")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("failed to cancel run 202", result.stderr)
        self.assertIn("2 cancellation requests accepted; 1 failed", result.stdout)
        self.assertEqual(len(self.cancellations()), 3)


class SystemBashCancelQueueTests(CancelQueueTests):
    bash = "/bin/bash"


if __name__ == "__main__":
    unittest.main(verbosity=2)

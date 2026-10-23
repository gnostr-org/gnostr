import os
from pathlib import Path
import subprocess
import unittest


HELPER = Path(__file__).resolve().parents[1] / "lib" / "script-ui.sh"


class ScriptUITests(unittest.TestCase):
    bash = "bash"

    def invoke(self, command, answers):
        return subprocess.run(
            [self.bash, "-euc", 'source "$1"; ' + command, "test", str(HELPER)],
            input=answers,
            capture_output=True,
            text=True,
            timeout=5,
            env=os.environ,
        )

    def test_selection_is_validated_and_deduplicated(self):
        result = self.invoke(
            'ui_select Runs alpha beta gamma; printf "SELECTED:%s\\n" "${UI_SELECTED[@]}"',
            "3,1 3\n",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("SELECTED:alpha\nSELECTED:gamma", result.stdout)
        self.assertNotIn("SELECTED:beta", result.stdout)

    def test_all_and_single_choice(self):
        result = self.invoke(
            'ui_select Runs alpha beta; printf "SELECTED:%s\\n" "${UI_SELECTED[@]}"',
            "all\n",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("SELECTED:alpha\nSELECTED:beta", result.stdout)
        result = self.invoke('ui_choose Mode build test; echo "VALUE:$UI_VALUE"', "2\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("VALUE:test", result.stdout)

    def test_invalid_or_multiple_single_choice_does_not_proceed(self):
        for command, answers in (
            ("ui_select Runs alpha beta", "1 3\n"),
            ("ui_select Runs alpha beta", "$(touch nope)\n"),
            ("ui_select Runs alpha beta", "999999999999999999\n"),
            ("ui_choose Mode build test", "all\n"),
        ):
            with self.subTest(answers=answers):
                result = self.invoke(command + '; echo PROCEEDED', answers)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn("PROCEEDED", result.stdout)

    def test_quit_blank_and_eof_never_proceed(self):
        for answers in ("q\n", "\n", ""):
            result = self.invoke('ui_select Runs alpha beta; echo PROCEEDED', answers)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("PROCEEDED", result.stdout)

    def test_confirmation_defaults_to_no(self):
        for answers in ("n\n", "\n", ""):
            result = self.invoke('ui_confirm Delete; echo PROCEEDED', answers)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn("PROCEEDED", result.stdout)
        result = self.invoke('ui_confirm Delete; echo PROCEEDED', "yes\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("PROCEEDED", result.stdout)

    def test_prompt_preserves_spaces_and_default(self):
        result = self.invoke('ui_prompt Path "my default"; echo "VALUE:$UI_VALUE"', "\n")
        self.assertIn("VALUE:my default", result.stdout)
        result = self.invoke('ui_prompt Path default; echo "VALUE:$UI_VALUE"', "a path\n")
        self.assertIn("VALUE:a path", result.stdout)


class SystemBashScriptUITests(ScriptUITests):
    bash = "/bin/bash"


if __name__ == "__main__":
    unittest.main(verbosity=2)

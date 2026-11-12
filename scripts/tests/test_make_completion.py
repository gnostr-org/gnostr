import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "scripts" / "gnostr-completion.sh"


class MakeCompletionTests(unittest.TestCase):
    bash = "bash"

    def complete(self, words, cwd=ROOT):
        result = subprocess.run(
            [
                self.bash, "-euc",
                'source "$1"; shift; COMP_WORDS=("$@"); '
                'COMP_CWORD=$((${#COMP_WORDS[@]} - 1)); '
                '_gnostr_make_complete; '
                'if ((${#COMPREPLY[@]})); then printf "%s\\n" "${COMPREPLY[@]}"; fi',
                "test", str(HELPER), *words,
            ],
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=5,
            env=os.environ,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        return result.stdout.splitlines()

    def test_generated_script_targets(self):
        self.assertIn(
            "script-cargo-test-workspace",
            self.complete(["make", "script-cargo-t"]),
        )
        expected = {
            "script-" + path.stem
            for path in (ROOT / "scripts").glob("*.sh")
        } | {"script-update", "script-release"}
        self.assertTrue(expected <= set(self.complete(["make", "script-"])))

    def test_static_targets_and_no_make_templates(self):
        self.assertIn("cargo-help", self.complete(["gmake", "cargo-h"]))
        targets = self.complete(["make", ""])
        self.assertEqual(len(targets), len(set(targets)))
        self.assertNotIn("SCRIPT_FILES", targets)
        self.assertNotIn("script-$(basename", targets)
        self.assertTrue(all("$" not in target for target in targets))

    def test_directory_option_from_path_with_spaces(self):
        with tempfile.TemporaryDirectory(prefix="make completion ") as directory:
            for option in (["-C", str(ROOT)], ["--directory=" + str(ROOT)]):
                with self.subTest(option=option):
                    self.assertIn(
                        "script-cargo-test-workspace",
                        self.complete(["make", *option, "script-cargo-t"], directory),
                    )

    def test_no_execution_of_make_expressions_or_recipes(self):
        with tempfile.TemporaryDirectory() as directory:
            makefile = Path(directory) / "custom.mk"
            makefile.write_text(
                "VALUE := $(shell touch executed)\n"
                "alpha beta:\n\ttouch executed\n"
                "define script_target\n"
                "script-$(basename $(notdir $(1))):\n"
                "\ttouch executed\nendef\n"
            )
            for option in (["-f", "custom.mk"], ["--file=" + str(makefile)]):
                self.assertEqual(
                    self.complete(["make", *option, ""], directory),
                    ["alpha", "beta"],
                )
            self.assertFalse((Path(directory) / "executed").exists())

    def test_no_targets_for_missing_makefile_or_option_argument(self):
        with tempfile.TemporaryDirectory() as directory:
            for words in (["make", ""], ["make", "-C", ""], ["make", "-f", ""]):
                with self.subTest(words=words):
                    self.assertEqual(self.complete(words, directory), [])


class SystemBashMakeCompletionTests(MakeCompletionTests):
    bash = "/bin/bash"


if __name__ == "__main__":
    unittest.main(verbosity=2)

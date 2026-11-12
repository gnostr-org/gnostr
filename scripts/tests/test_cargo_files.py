import os
from pathlib import Path
import subprocess
import tempfile
import unittest


HELPER = Path(__file__).resolve().parents[1] / "lib" / "cargo-files.sh"


class CargoFilesTests(unittest.TestCase):
    bash = "bash"

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "--quiet")
        for name in (
            "Cargo.toml", "Cargo.lock", "crate with spaces/Cargo.toml",
            "vendor/example/Cargo.toml", "crate/target/debug/Cargo.lock",
            "target/Cargo.toml", "ipfs/ffi/rust/Cargo.toml",
            "ipfs/test/rust-ffi-test/Cargo.toml", "ipfs/Cargo.lock",
        ):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture\n")
        self.git("update-index", "--add", "--cacheinfo", f"160000,{'1' * 40},ipfs")

    def git(self, *args):
        return subprocess.run(
            ["git", *args], cwd=self.root, check=True,
            capture_output=True, text=True,
        ).stdout

    def invoke(self, command, cwd=None):
        return subprocess.run(
            [self.bash, "-euo", "pipefail", "-c", 'source "$1"; ' + command, "test", str(HELPER)],
            cwd=cwd or self.root, env=os.environ,
            capture_output=True, text=True, timeout=10,
        )

    def test_discovery_and_staging_exclude_submodule_and_build_files(self):
        result = self.invoke(
            'refresh_cargo_files; managed_manifests; stage_cargo_files -f'
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            set(result.stdout.splitlines()),
            {"./Cargo.toml", "./crate with spaces/Cargo.toml"},
        )
        self.assertEqual(
            set(self.git("diff", "--cached", "--name-only").splitlines()),
            {"Cargo.toml", "Cargo.lock", "crate with spaces/Cargo.toml", "ipfs"},
        )
        self.assertEqual(
            (self.root / "ipfs/ffi/rust/Cargo.toml").read_text(), "fixture\n"
        )

    def test_discovery_keeps_ignored_files_for_forced_staging(self):
        (self.root / ".gitignore").write_text("Cargo.lock\n")
        result = self.invoke("stage_cargo_files -f")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Cargo.lock", self.git("diff", "--cached", "--name-only").splitlines())

    def test_refresh_picks_up_new_lockfiles(self):
        result = self.invoke(
            'refresh_cargo_files; printf "fixture\\n" > "crate with spaces/Cargo.lock"; stage_cargo_files'
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "crate with spaces/Cargo.lock",
            self.git("diff", "--cached", "--name-only").splitlines(),
        )

    def test_discovery_failure_does_not_continue(self):
        outside = self.root / "outside"
        outside.mkdir()
        result = self.invoke(
            'cd /; refresh_cargo_files; echo PROCEEDED', cwd=outside
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("could not discover", result.stderr)
        self.assertNotIn("PROCEEDED", result.stdout)


class SystemBashCargoFilesTests(CargoFilesTests):
    bash = "/bin/bash"


if __name__ == "__main__":
    unittest.main(verbosity=2)

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
INTERACTIVE_SCRIPTS = (
    "gh-workflow.sh", "gh-delete-all-workflow-runs.sh", "remove-big-commits.sh",
    "git-inspect-objects.sh", "cargo-yank.sh", "sccache-clear-cache.sh",
    "gnostr-p2p-fractal-clone.sh", "update", "update_versions.sh",
    "git-install-hooks.sh", "cargo-generate-lockfiles.sh",
    "cargo-install-feature-variants.sh", "cross.sh", "xcode-build.sh",
    "docker-gnostr-stack.sh", "act-test-matrix.sh", "cargo-cross.sh",
)

MOCK = r'''
import json
import os
from pathlib import Path
import sys

name = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["CALL_LOG"], "a") as log:
    log.write(json.dumps([name, *args]) + "\n")
if name == "rustc":
    print("host: aarch64-apple-darwin")
elif name == "uname":
    print("Darwin" if args == ["-s"] else "arm64")
elif name == "cross":
    print("cross 0.2.5")
elif name == "rustup":
    if args[:2] == ["target", "list"]:
        print(os.environ.get("INSTALLED_TARGETS", "aarch64-apple-darwin"))
    elif os.environ.get("INSTALL_FAIL") == "1":
        sys.exit(1)
elif name == "gh":
    if args[:2] == ["repo", "view"]:
        print("owner/repo")
    elif args[:2] == ["run", "list"]:
        status = args[args.index("-s") + 1] if "-s" in args else ""
        run_id = {"in_progress": 101, "queued": 202, "waiting": 303}.get(status)
        if run_id:
            print(f"{run_id}\t{status}\tCI\tmaster")
    elif args[:1] == ["api"]:
        if "DELETE" in args:
            if os.environ.get("DELETE_FAIL") == "1":
                print("deletion denied", file=sys.stderr)
                sys.exit(1)
        elif ".total_count" in args:
            print(250)
        elif "page=2" in args[1]:
            print("202\tcompleted\tCI\tmaster\t2026-10-22")
            print("203\tcompleted\tCI\tmaster\t2026-10-22")
        elif "page=3" in args[1]:
            print("303\tcompleted\tCI\tmaster\t2026-10-21")
elif name == "git":
    if args[:1] == ["for-each-ref"]:
        if any("objecttype" in arg for arg in args):
            print("refs/tags/one tag\nrefs/tags/two tag")
        else:
            print("refs/heads/one\nrefs/heads/two")
    elif args[:1] == ["symbolic-ref"]:
        print("master")
    elif args[:2] == ["remote", "get-url"]:
        print("https://example.invalid/repo.git")
    elif args[:1] == ["rev-parse"]:
        print("0" * 40)
    elif args[:1] == ["describe"]:
        print("test-tag")
elif name == "cargo":
    if os.environ.get("CARGO_FAIL") == "1":
        sys.exit(1)
elif name == "pkg-config":
    sys.exit(1)
'''


class InteractiveScriptTests(unittest.TestCase):
    bash = "bash"

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.log = self.directory / "calls"
        self.bin = self.directory / "bin"
        self.bin.mkdir()
        for name in (
            "gh", "git", "cargo", "rustc", "rustup", "cross", "uname",
            "sccache", "act", "docker", "gnostr", "sleep", "pkg-config",
        ):
            tool = self.bin / name
            tool.write_text(f"#!{sys.executable}\n" + MOCK)
            tool.chmod(0o755)
        self.env = {
            **os.environ,
            "PATH": f"{self.bin}:{os.environ['PATH']}",
            "CALL_LOG": str(self.log),
        }

    def run_script(self, name, answers="", args=(), **env):
        return subprocess.run(
            [self.bash, str(SCRIPTS / name), *args],
            cwd=ROOT,
            input=answers,
            capture_output=True,
            text=True,
            env={**self.env, **env},
            timeout=15,
        )

    def calls(self, name):
        if not self.log.exists():
            return []
        return [
            call for line in self.log.read_text().splitlines()
            if (call := json.loads(line))[0] == name
        ]

    def test_every_script_documents_interactive_mode_without_side_effects(self):
        for name in INTERACTIVE_SCRIPTS:
            with self.subTest(script=name):
                result = self.run_script(name, args=("--help",))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("--interactive", result.stdout)
        self.assertEqual(self.calls("cargo"), [])
        self.assertEqual(self.calls("gh"), [])

    def test_workflow_cancellation_selects_only_requested_run(self):
        result = self.run_script(
            "gh-workflow.sh", "\n\nall\n2\ny\n", ("--interactive",)
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            [call for call in self.calls("gh") if call[1:3] == ["run", "cancel"]],
            [["gh", "run", "cancel", "202"]],
        )

    def test_deletion_preserves_newest_page_and_only_deletes_selection(self):
        result = self.run_script(
            "gh-delete-all-workflow-runs.sh",
            "\n\n1\n2\ny\n",
            ("--interactive", "--repo", "owner/repo"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = self.calls("gh")
        deletes = [call for call in calls if "DELETE" in call]
        self.assertEqual(len(deletes), 1)
        self.assertIn("/repos/owner/repo/actions/runs/203", deletes[0])
        self.assertFalse(any("&page=1" in arg for call in calls for arg in call))
        first_delete = calls.index(deletes[0])
        self.assertTrue(all("page=" not in arg for call in calls[first_delete:] for arg in call))

    def test_deletion_worker_failure_is_reported(self):
        result = self.run_script(
            "gh-delete-all-workflow-runs.sh", args=("--repo", "owner/repo", "--limit", "1"),
            DELETE_FAIL="1",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("worker failed", result.stderr)

    def test_ref_selection_only_deletes_selected_local_ref(self):
        result = self.run_script("git-inspect-objects.sh", "2\ny\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            [call for call in self.calls("git") if call[1] == "update-ref"],
            [["git", "update-ref", "-d", "refs/tags/two"]],
        )

    def test_big_commit_selection_only_deletes_selected_branch(self):
        result = self.run_script(
            "remove-big-commits.sh", "2\ny\n", ("--interactive", "--local-branches")
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            [call for call in self.calls("git") if call[1:3] == ["branch", "-D"]],
            [["git", "branch", "-D", "two"]],
        )

    def test_hook_selection_installs_only_selected_hook(self):
        hooks = self.directory / "hooks"
        result = self.run_script(
            "git-install-hooks.sh", "1\n1\n\ny\n",
            ("--interactive", "--hooks-dir", str(hooks)),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([path.name for path in hooks.iterdir()], ["applypatch-msg"])

    def test_missing_target_installation_failure_never_starts_build(self):
        result = self.run_script(
            "cross.sh", args=("--target", "macos-x64", "--install-targets"),
            INSTALL_FAIL="1",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("failed to install x86_64-apple-darwin", result.stderr)
        self.assertEqual(self.calls("cargo"), [])

    def test_missing_target_without_install_prints_actionable_error(self):
        result = self.run_script(
            "cross.sh", args=("--target", "macos-x64", "--no-install-targets")
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("rustup target add x86_64-apple-darwin", result.stderr)
        self.assertEqual(self.calls("cargo"), [])
        self.assertFalse(any(call[1:3] == ["target", "add"] for call in self.calls("rustup")))

    def test_cross_dry_run_does_not_install_or_build(self):
        result = self.run_script("cross.sh", args=("--target", "macos-x64", "--dry-run"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("would run: rustup target add x86_64-apple-darwin", result.stdout)
        self.assertEqual(self.calls("cargo"), [])
        self.assertFalse(any(call[1:3] == ["target", "add"] for call in self.calls("rustup")))

    def test_cross_installs_before_building(self):
        result = self.run_script("cross.sh", args=("--target", "macos-x64"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(["rustup", "target", "add", "x86_64-apple-darwin"], self.calls("rustup"))
        self.assertEqual(len(self.calls("cargo")), 1)

    def test_interactive_missing_target_decline_never_installs_or_builds(self):
        result = self.run_script(
            "cross.sh", "1\n\n\n\ny\nn\n",
            ("--interactive", "--target", "macos-x64"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Rust target x86_64-apple-darwin is missing", result.stdout)
        self.assertEqual(self.calls("cargo"), [])
        self.assertFalse(any(call[1:3] == ["target", "add"] for call in self.calls("rustup")))

    def test_interactive_missing_target_accept_installs_then_builds(self):
        result = self.run_script(
            "cross.sh", "1\n\n\n\ny\ny\n",
            ("--interactive", "--target", "macos-x64"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(["rustup", "target", "add", "x86_64-apple-darwin"], self.calls("rustup"))
        self.assertEqual(len(self.calls("cargo")), 1)

    def test_xcode_interactive_list_only_shows_selected_project(self):
        result = self.run_script("xcode-build.sh", "4\n3\n\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("AppWithTool.xcodeproj", result.stdout)
        self.assertNotIn("Universal_App.xcodeproj", result.stdout)

    def test_yank_and_container_decline_do_not_mutate(self):
        for name, answers in (
            ("cargo-yank.sh", "example-crate\n1.2.3\nn\n"),
            ("docker-gnostr-stack.sh", "6\n\nn\n"),
            ("cargo-install-feature-variants.sh", "1\n\nn\n"),
        ):
            with self.subTest(script=name):
                result = self.run_script(name, answers, ("--interactive",))
                self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls("cargo"), [])
        self.assertEqual(self.calls("docker"), [])

    def test_cache_stats_does_not_stop_server(self):
        result = self.run_script("sccache-clear-cache.sh", "1\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls("sccache"), [["sccache", "--show-stats"]])

    def test_local_act_menu_passes_selected_toolchain(self):
        result = self.run_script("act-test-matrix.sh", "1\n2\n\ny\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("toolchain:nightly", self.calls("act")[0])

    def test_cargo_cross_menu_preserves_forwarded_act_options(self):
        result = self.run_script(
            "cargo-cross.sh", "\nrelease-test\n2\nsetup\ny\n",
            ("--interactive", "--verbose"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        call = self.calls("act")[0]
        self.assertIn("tag=release-test", call)
        self.assertIn("toolchain=nightly", call)
        self.assertIn("setup", call)
        self.assertIn("--verbose", call)

    def test_release_push_only_respects_crate_selection(self):
        result = self.run_script("update_versions.sh", "4\n1\ny\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        pushes = [call for call in self.calls("git") if call[1] == "push"]
        self.assertEqual(len(pushes), 1)
        self.assertIn("filetreelist/v", pushes[0][3])
        self.assertEqual(self.calls("cargo"), [])

    def test_update_quit_has_no_mutating_side_effects(self):
        result = self.run_script("update", "q\n", ("--interactive",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls("cargo"), [])
        self.assertEqual(self.calls("git"), [])


class SystemBashInteractiveScriptTests(InteractiveScriptTests):
    bash = "/bin/bash"


class MakeScriptTargetTests(unittest.TestCase):
    def test_list_and_argument_forwarding_without_running_builds(self):
        result = subprocess.run(
            ["make", "--no-print-directory", "-n", "script-cross", "ARGS=--interactive --dry-run"],
            cwd=ROOT, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('bash "scripts/cross.sh" --interactive --dry-run', result.stdout)
        result = subprocess.run(
            ["make", "--no-print-directory", "script-list"],
            cwd=ROOT, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("script-gh-cancel-queue", result.stdout)
        self.assertIn("script-update_versions", result.stdout)

    def test_generator_preserves_literal_make_syntax(self):
        generator = (SCRIPTS / "make_just.sh").read_text()
        block = generator.split("<<'SCRIPT_TARGETS_EOF'\n", 1)[1].split("\nSCRIPT_TARGETS_EOF", 1)[0]
        self.assertIn(block.strip(), (ROOT / "Makefile").read_text())


if __name__ == "__main__":
    unittest.main(verbosity=2)

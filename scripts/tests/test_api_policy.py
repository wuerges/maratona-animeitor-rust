import importlib.util
import json
import pathlib
import tempfile
import unittest
from unittest import mock

MODULE = pathlib.Path(__file__).resolve().parents[1] / "check_api.py"
spec = importlib.util.spec_from_file_location("check_api", MODULE)
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


def api():
    return {
        "openapi": "3.0.3",
        "info": {"title": "Example", "version": "2.1.0"},
        "paths": {"/events": {"get": {
            "operationId": "list_events", "description": "List events",
            "responses": {"200": {"description": "Event names", "content": {
                "application/json": {"schema": {"type": "array", "items": {"type": "string"}}}
            }}}
        }}}
    }


class PolicyTests(unittest.TestCase):
    def test_minimum_bumps_and_larger_versions(self):
        for level, minimum in [(0, (2, 1, 0)), (1, (2, 1, 1)), (2, (2, 2, 0)), (3, (3, 0, 0))]:
            policy.enforce_version((2, 1, 0), minimum, level)
            policy.enforce_version((2, 1, 0), (4, 0, 0), level)
            if level:
                with self.assertRaises(ValueError):
                    policy.enforce_version((2, 1, 0), (2, 1, 0), level)
        with self.assertRaises(ValueError):
            policy.enforce_version((2, 1, 0), (2, 1, 1), 2)
        with self.assertRaises(ValueError):
            policy.enforce_version((2, 1, 0), (2, 2, 0), 3)
        with self.assertRaises(ValueError):
            policy.enforce_version((2, 1, 0), (2, 0, 9), 0)

    def test_oasdiff_uses_pinned_container_and_read_only_mount(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(policy, "run", return_value="{}") as command:
            directory = pathlib.Path(temp)
            policy.oasdiff("diff", "/specs/base.json", "/specs/current.json", directory=directory)
            args = command.call_args.args
            self.assertEqual(args[:5], ("docker", "run", "--rm", "--network", "none"))
            self.assertIn("tufin/oasdiff:v1.30.0", args)
            self.assertIn(f"{directory.resolve()}:/specs:ro", args)
            self.assertIn("--user", args)
            self.assertEqual(args[-3:], ("diff", "/specs/base.json", "/specs/current.json"))
            policy.oasdiff("--version")
            self.assertNotIn("--volume", command.call_args.args)

    def test_stable_version_format(self):
        self.assertEqual(policy.version("2.1.0"), (2, 1, 0))
        for raw in ["2.1", "v2.1.0", "2.1.0-rc1"]:
            with self.assertRaises(ValueError):
                policy.version(raw)

    def test_real_oasdiff_classifications_and_mixed_changes(self):
        policy.check_tool()
        base = api()
        with tempfile.TemporaryDirectory() as temp:
            directory = pathlib.Path(temp)
            self.assertEqual(policy.compare(base, api(), directory, "unchanged"), 0)
            current = api()
            current["info"]["version"] = "2.2.0"
            self.assertEqual(policy.compare(base, current, directory, "version-only"), 0)
            current = api()
            current["paths"]["/events"]["get"]["description"] = "New documentation"
            self.assertEqual(policy.compare(base, current, directory, "documentation"), 1)
            current["paths"]["/new-events"] = current["paths"]["/events"]
            self.assertEqual(policy.compare(base, current, directory, "addition-and-docs"), 2)
            current["paths"].pop("/events")
            self.assertEqual(policy.compare(base, current, directory, "mixed-breaking"), 3)
            for report in ["breaking", "changelog", "diff", "contract"]:
                self.assertTrue((directory / f"mixed-breaking-{report}.json").is_file())

    def test_check_uses_git_base_even_after_candidate_snapshots_are_updated(self):
        policy.check_tool()
        original_run = policy.run
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            snapshots = root / "doc/openapi"
            snapshots.mkdir(parents=True)
            cargo = root / "Cargo.toml"
            cargo.write_text('[workspace.package]\nversion="2.1.0"\n')
            docs = root / "doc/internal-api-setup.md"
            docs.write_text("Original API documentation\n")
            candidate = api()

            def export(directory):
                directory.mkdir(parents=True, exist_ok=True)
                for name in policy.SPECS:
                    (directory / name).write_text(json.dumps(candidate, indent=2) + "\n")

            def command(*args, **kwargs):
                if args[0] == "cargo":
                    export(pathlib.Path(args[-1]))
                    return ""
                return original_run(*args, cwd=root)

            export(snapshots)
            original_run("git", "init", "-q", cwd=root)
            original_run("git", "add", ".", cwd=root)
            original_run("git", "-c", "user.name=API Test", "-c", "user.email=api@example.com", "commit", "-qm", "baseline", cwd=root)
            base = original_run("git", "rev-parse", "HEAD", cwd=root).strip()
            with mock.patch.object(policy, "ROOT", root), mock.patch.object(policy, "run", command), mock.patch("sys.argv", ["check_api", "--base", base]):
                policy.main()
                # Updating both tracked snapshots cannot hide an addition from the base comparison.
                candidate["paths"]["/new-events"] = candidate["paths"]["/events"]
                export(snapshots)
                with self.assertRaisesRegex(ValueError, "2.2.0"):
                    policy.main()
                cargo.write_text('[workspace.package]\nversion="2.2.0"\n')
                candidate["info"]["version"] = "2.2.0"
                export(snapshots)
                policy.main()
                # A Markdown-only API documentation edit still requires a patch bump.
                candidate = api()
                cargo.write_text('[workspace.package]\nversion="2.1.0"\n')
                export(snapshots)
                docs.write_text("Updated API documentation\n")
                with self.assertRaisesRegex(ValueError, "2.1.1"):
                    policy.main()
                cargo.write_text('[workspace.package]\nversion="2.1.1"\n')
                candidate["info"]["version"] = "2.1.1"
                export(snapshots)
                policy.main()

    def test_stale_snapshots_and_wrong_versions(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            generated, tracked = root / "generated", root / "tracked"
            generated.mkdir()
            tracked.mkdir()
            for name in policy.SPECS:
                data = json.dumps(api())
                (generated / name).write_text(data)
                (tracked / name).write_text(data)
            policy.validate_snapshots(generated, tracked, "2.1.0")
            with self.assertRaises(ValueError):
                policy.validate_snapshots(generated, tracked, "2.2.0")
            (tracked / "internal.json").write_text("{}")
            with self.assertRaisesRegex(ValueError, "stale"):
                policy.validate_snapshots(generated, tracked, "2.1.0")


if __name__ == "__main__":
    unittest.main()

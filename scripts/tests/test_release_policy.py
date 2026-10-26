import importlib.util
import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('check_release', pathlib.Path(__file__).resolve().parents[1] / 'check_release.py')
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class ReleaseTests(unittest.TestCase):
    def test_publish_guard_handles_annotated_remote_and_lightweight_local_tag(self):
        workflow = (pathlib.Path(__file__).resolve().parents[2] / '.github/workflows/release.yml').read_text()
        guard = textwrap.dedent(workflow.split("          import os,re,subprocess\n", 1)[1].split('          PY\n', 1)[0])
        guard = 'import os,re,subprocess\n' + guard
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            remote, checkout = root / 'remote', root / 'checkout'
            def git(*args, cwd=remote):
                return subprocess.check_output(['git', *args], cwd=cwd, text=True, stderr=subprocess.DEVNULL).strip()
            remote.mkdir()
            git('init')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.com', 'commit', '--allow-empty', '-m', 'initial')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.com', 'tag', '-a', 'v2.3.1', '-m', 'release')
            git('clone', str(remote), str(checkout))
            # Reproduce the publishing checkout's peeled, lightweight local tag.
            git('update-ref', 'refs/tags/v2.3.1', 'HEAD', cwd=checkout)
            local_tag = git('rev-parse', 'refs/tags/v2.3.1', cwd=checkout)
            self.assertNotEqual(local_tag, git('rev-parse', 'refs/tags/v2.3.1'))
            def check():
                return subprocess.run(['python3', '-c', guard], cwd=checkout,
                                      env={**os.environ, 'VERSION': '2.3.1'}, capture_output=True, text=True)
            self.assertEqual(check().returncode, 0)
            self.assertEqual(git('rev-parse', 'refs/tags/v2.3.1', cwd=checkout), local_tag)
            git('tag', 'v2.4.0-rc1')
            self.assertEqual(check().returncode, 0)
            git('tag', 'v2.4.0')
            result = check()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('A newer release tag exists', result.stderr)

    def test_dated_nonempty_notes(self):
        text = '## [Unreleased]\n\n## [2.3.0] - 2026-01-01\n\n### Added\n\n- Feature\n\n## [2.2.0]\n- Old\n'
        self.assertEqual(policy.release_notes(text, '2.3.0'), '### Added\n\n- Feature\n')
        for text in ['## [2.3.0]\n- Feature', '## [2.3.0] - 2026-01-01\n', '## [2.3.0] - 9999-01-01\n- Feature']:
            with self.assertRaises(ValueError):
                policy.release_notes(text, '2.3.0')

    def test_tag_and_lockfile(self):
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            (root / 'member').mkdir()
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=["member"]\n[workspace.package]\nversion="2.3.0"\n')
            (root / 'member/Cargo.toml').write_text('[package]\nname="example"\nversion.workspace=true\n')
            (root / 'CHANGELOG.md').write_text('## [2.3.0] - 2026-01-01\n- Feature\n')
            (root / 'Cargo.lock').write_text('[[package]]\nname="example"\nversion="2.3.0"\n')
            policy.validate(root, 'v2.3.0')
            with self.assertRaises(ValueError):
                policy.validate(root, 'v2.2.0')
            (root / 'Cargo.lock').write_text('[[package]]\nname="example"\nversion="2.2.0"\n')
            with self.assertRaises(ValueError):
                policy.validate(root, 'v2.3.0')

    def test_previous_release_is_highest_ancestor_below_current(self):
        with mock.patch.object(policy.subprocess, 'check_output', return_value='v2.2.0\nv2.9.0\nv2.10.0\nv3.0.0\nrelease/foo\n') as call:
            self.assertEqual(policy.previous_release('v2.10.0'), 'v2.9.0')
            self.assertIn('--merged', call.call_args.args[0])
        with mock.patch.object(policy.subprocess, 'check_output', return_value='v3.0.0\n'):
            with self.assertRaises(ValueError):
                policy.previous_release('v2.10.0')

    def test_invalid_versions(self):
        for value in ['2.3', '2.3.0-rc1', '02.3.0', 'v2.3.0']:
            with self.assertRaises(ValueError):
                policy.version(value)

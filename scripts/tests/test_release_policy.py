import importlib.util
import pathlib
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('check_release', pathlib.Path(__file__).resolve().parents[1] / 'check_release.py')
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class ReleaseTests(unittest.TestCase):
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

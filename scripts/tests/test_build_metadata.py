"""Exercise the actual Rust build script for native and Git-free builds."""
import os
import pathlib
import subprocess
import tempfile
import unittest


class BuildMetadataTests(unittest.TestCase):
    def test_revision_sources_and_missing_metadata(self):
        source = pathlib.Path(__file__).resolve().parents[2] / 'server/server-v2/build.rs'
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            binary = root / 'build-script'
            subprocess.run(['rustc', '--edition=2024', str(source), '-o', str(binary)], check=True)
            env = {k: v for k, v in os.environ.items() if k != 'ANIMEITOR_GIT_COMMIT'}
            def build(**extra):
                return subprocess.run([str(binary)], cwd=root, env={**env, **extra}, capture_output=True, text=True)
            unknown = build()
            self.assertEqual(unknown.returncode, 0)
            self.assertNotIn('cargo:rustc-env=ANIMEITOR_BUILD_COMMIT=', unknown.stdout)
            override = build(ANIMEITOR_GIT_COMMIT='ABCDEF0123456789' * 4)
            self.assertEqual(override.returncode, 0)
            self.assertIn('cargo:rustc-env=ANIMEITOR_BUILD_COMMIT=abcdef012345', override.stdout)
            self.assertNotEqual(build(ANIMEITOR_GIT_COMMIT='not-a-commit').returncode, 0)
            subprocess.run(['git', 'init', '-q'], cwd=root, check=True)
            subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.com',
                            'commit', '-q', '--allow-empty', '-m', 'initial'], cwd=root, check=True)
            commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
            self.assertIn(f'cargo:rustc-env=ANIMEITOR_BUILD_COMMIT={commit[:12]}', build().stdout)
            self.assertIn('cargo:rustc-env=ANIMEITOR_BUILD_COMMIT=abcdef012345',
                          build(ANIMEITOR_GIT_COMMIT='ABCDEF0123456789' * 4).stdout)

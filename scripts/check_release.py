#!/usr/bin/env python3
"""Validate release metadata; print the previous release for API comparison."""
import argparse
import datetime
import pathlib
import re
import subprocess
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]


def version(value):
    if not re.fullmatch(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', value):
        raise ValueError(f'invalid stable version: {value}')
    return tuple(map(int, value.split('.')))


def release_notes(changelog, value):
    match = re.search(r'^## \[' + re.escape(value) + r'\] - (\d{4}-\d{2}-\d{2})\s*\n(.*?)(?=^## |\Z)', changelog, re.M | re.S)
    if not match:
        raise ValueError(f'missing dated changelog section for {value}')
    date = datetime.date.fromisoformat(match[1])
    if date > datetime.date.today():
        raise ValueError('release date is in the future')
    if not re.search(r'^- \S', match[2], re.M):
        raise ValueError('release changelog must contain an entry')
    return match[2].strip() + '\n'


def validate(root, tag):
    manifest = tomllib.loads((root / 'Cargo.toml').read_text())
    value = manifest['workspace']['package']['version']
    version(value)
    if tag != f'v{value}':
        raise ValueError(f'tag {tag} does not match workspace version {value}')
    lock = tomllib.loads((root / 'Cargo.lock').read_text())['package']
    for member in manifest['workspace']['members']:
        package = tomllib.loads((root / member / 'Cargo.toml').read_text())['package']
        if package['version'] != {'workspace': True}:
            raise ValueError(f'{member} must inherit the workspace version')
        if not any(p['name'] == package['name'] and p['version'] == value and 'source' not in p for p in lock):
            raise ValueError(f'stale lockfile version for {package["name"]}')
    return release_notes((root / 'CHANGELOG.md').read_text(), value)


def previous_release(tag):
    tags = subprocess.check_output(['git', 'tag', '--merged', 'HEAD'], cwd=ROOT, text=True).splitlines()
    candidates = [t for t in tags if re.fullmatch(r'v\d+\.\d+\.\d+', t) and version(t[1:]) < version(tag[1:])]
    if not candidates:
        raise ValueError('no previous stable release ancestor; do not substitute HEAD')
    return max(candidates, key=lambda t: version(t[1:]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--notes', type=pathlib.Path)
    args = parser.parse_args()
    notes = validate(ROOT, args.tag)
    base = previous_release(args.tag)
    if args.notes:
        args.notes.write_text(notes)
    print(base)


if __name__ == '__main__':
    main()

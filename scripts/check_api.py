#!/usr/bin/env python3
"""Verify served OpenAPI snapshots and API version policy against a Git base."""
import argparse
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
OASDIFF_VERSION = "1.30.0"
OASDIFF_IMAGE = f"tufin/oasdiff:v{OASDIFF_VERSION}"
SPECS = ("internal.json", "public.json")
API_DOCS = {"doc/event-api.md", "doc/internal-api-setup.md", "doc/public-api-overview.md"}


def run(*args, cwd=ROOT):
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"command failed: {' '.join(map(str, args))}\n{result.stderr}")
    return result.stdout


def version(raw):
    if not re.fullmatch(r"\d+\.\d+\.\d+", raw):
        raise ValueError("API version must be a stable major.minor.patch version")
    return tuple(map(int, raw.split(".")))


def required_version(base, level):
    major, minor, patch = base
    return {0: base, 1: (major, minor, patch + 1), 2: (major, minor + 1, 0), 3: (major + 1, 0, 0)}[level]


def enforce_version(base, current, level):
    minimum = required_version(base, level)
    if current < minimum:
        raise ValueError(f"API change requires at least {'.'.join(map(str, minimum))}; found {'.'.join(map(str, current))}")


def oasdiff(*args, directory=None):
    command = ["docker", "run", "--rm", "--network", "none"]
    if directory is not None:
        command.extend(["--user", f"{os.getuid()}:{os.getgid()}", "--volume", f"{directory.resolve()}:/specs:ro"])
    return run(*command, OASDIFF_IMAGE, *args)


def check_tool():
    if oasdiff("--version").strip() not in {f"oasdiff version {OASDIFF_VERSION}", f"oasdiff version v{OASDIFF_VERSION}"}:
        raise ValueError(f"Docker image {OASDIFF_IMAGE} reported an unexpected oasdiff version")


def compare(base, current, directory, label):
    """Return severity and retain machine-readable reports for review."""
    # A version change must not classify itself as a contract change.
    documents = []
    for name, spec in [("base", base), ("current", current)]:
        value = json.loads(json.dumps(spec))
        value["info"]["version"] = "0.0.0"
        path = directory / f"{label}-{name}.json"
        path.write_text(json.dumps(value))
        documents.append(f"/specs/{path.name}")
    reports = {}
    for command, extra in [
        ("breaking", []), ("changelog", []), ("diff", []),
        ("contract", ["--exclude-elements", "description,examples,summary,title"]),
    ]:
        result = json.loads(oasdiff("diff" if command == "contract" else command, *documents, "--format", "json", "--allow-external-refs=false", *extra, directory=directory))
        reports[command] = result
        (directory / f"{label}-{command}.json").write_text(json.dumps(result, indent=2) + "\n")
    if reports["breaking"]:
        return 3
    if reports["changelog"] or reports["contract"]:
        return 2
    return 1 if reports["diff"] else 0


def validate_snapshots(generated, tracked, expected):
    for name in SPECS:
        if (generated / name).read_bytes() != (tracked / name).read_bytes():
            raise ValueError(f"stale OpenAPI snapshot: {name}; run make api-snapshots")
        if json.loads((generated / name).read_text())["info"]["version"] != expected:
            raise ValueError(f"{name} version does not match workspace package version")


def git_file(base, path):
    return run("git", "show", f"{base}:{path}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="HEAD", help="PR base commit or local comparison ref (default: HEAD)")
    parser.add_argument("--reports", type=pathlib.Path, default=ROOT / "target/api-reports")
    args = parser.parse_args()
    check_tool()
    current_raw = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    current = version(current_raw)
    args.reports = args.reports.resolve()
    args.reports.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="animeitor-openapi-") as temp:
        generated = pathlib.Path(temp)
        run("cargo", "run", "--quiet", "-p", "server-v2", "--bin", "export-openapi", "--", str(generated))
        validate_snapshots(generated, ROOT / "doc/openapi", current_raw)
        files = set(run("git", "ls-tree", "-r", "--name-only", args.base).splitlines())
        available = [f"doc/openapi/{name}" in files for name in SPECS]
        if not any(available):
            # One-time initialization. Later PR bases contain snapshots and cannot use this path.
            if current != (2, 1, 0):
                raise ValueError("initial API baseline must be package version 2.1.0")
            print("Validated initial 2.1.0 API baseline (base commit has no snapshots).")
            return
        if not all(available):
            raise ValueError("base commit contains an incomplete OpenAPI baseline")
        base_raw = tomllib.loads(git_file(args.base, "Cargo.toml"))["workspace"]["package"]["version"]
        base_version = version(base_raw)
        level = 0
        for name in SPECS:
            base_spec = json.loads(git_file(args.base, f"doc/openapi/{name}"))
            if base_spec["info"]["version"] != base_raw:
                raise ValueError("base snapshot version differs from base workspace version")
            level = max(level, compare(base_spec, json.loads((generated / name).read_text()), args.reports, name[:-5]))
        changed = set(run("git", "diff", "--name-only", args.base, "--").splitlines())
        if changed & API_DOCS:
            level = max(level, 1)
        enforce_version(base_version, current, level)
        print(f"API policy passed: {base_raw} -> {current_raw}; change level={['none', 'documentation', 'non-breaking', 'breaking'][level]}")
        print(f"oasdiff reports: {args.reports}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError, KeyError) as error:
        print(f"API check failed: {error}", file=sys.stderr)
        sys.exit(1)

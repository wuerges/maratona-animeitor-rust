# Project rules

## API versioning

The workspace package version in `Cargo.toml` is the single version source for
both public and internal OpenAPI documents. Commit
`7a309bdee523f906af2920e52429041c86086cb4` is the **2.1.0** changelog baseline.
The token roles, regex ownership, and capabilities implementation is recorded
in **2.2.0**.
Do not reset the version to 0.1.0 or maintain an independent API version.

For subsequent API changes:

- Breaking contract, authorization, or behavior changes require a major bump,
  resetting minor and patch to zero.
- Non-breaking additions or contract changes require a minor bump, resetting
  patch to zero.
- API documentation-only changes require a patch bump. Unrelated project
  documentation does not require an API bump.
- Mixed changes use the highest required bump. Larger bumps are permitted.

Regenerate committed specifications with `make api-snapshots`, run
`make api-check API_BASE=<base-commit>` using the Docker image **tufin/oasdiff:v1.30.0**, and record the
API changes in `CHANGELOG.md`. The compatibility check compares both APIs to
the base commit, not to newly regenerated snapshots. Commit snapshots and
version changes together; never bypass a failure by replacing the baseline.

Review behavioral and authorization changes manually: oasdiff can only detect
changes expressed in OpenAPI. Their version requirement applies even when the
schema comparison reports no change. Keep API documentation and examples
generic; do not include MOJ-specific mentions.

## Releases requested through AI

A request to make a patch or minor release authorizes preparing the release,
committing its files, creating an annotated version tag, and pushing the commit
and tag. It does not authorize production deployment. Follow [the release
procedure](doc/releases.md), using `Cargo.toml` as the single version source.

- Patch means increment patch; minor means increment minor and reset patch.
- Review all changes since the previous stable release tag, including behavior
  and authorization. If the API rules require a larger bump than requested,
  explain the mismatch and ask which permitted version to release before publishing.
- Do not include unrelated dirty work or overwrite an existing release tag.
- Finalize a dated changelog, update the lockfile and API snapshots, and run the
  documented checks before committing or tagging. Never replace the API baseline.
- Push the release commit before the tag. CI owns Docker publishing and GitHub
  release creation; do not run the manual Docker publishing target for this flow.
- Watch the release workflow to completion and report the tag, image digest,
  release URL, and any failures. Do not report success merely because a tag pushed.

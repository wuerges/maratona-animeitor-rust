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

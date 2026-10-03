# Making a release

Ask the coding agent “make a patch release” or “make a minor release.” This
includes committing and pushing the release and its annotated tag; production
updates require a separate request.

## One-time repository setup

Configure GitHub Actions secrets `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN`
with push access to the image repository. The optional Actions variable
`DOCKER_IMAGE_REPOSITORY` overrides `wuerges/animeitor`. Enable Actions and permit
the release publishing job to write repository contents for GitHub releases.
Require the `Release / validate`, API compatibility, and dependency policy checks
in branch protection. Restrict creation/deletion of `v*` tags to release operators.
These account settings are not configured by committing workflow files.

## Agent procedure

1. Inspect the working tree. Review commits since the highest preceding stable
   `vMAJOR.MINOR.PATCH` ancestor tag. Include previously unreleased changes, even
   if the workspace already has an unreleased version bump. Keep unrelated work
   out of the release commit.
2. Increment the current workspace version by the requested amount. Review API,
   behavioral, and authorization changes against the project versioning rules.
   A requested patch or minor must not bypass a required larger bump.
3. Move applicable Unreleased entries into `## [VERSION] - YYYY-MM-DD`, supplement
   them from the commit review, and leave an Unreleased heading for future work.
   Update workspace package entries in `Cargo.lock` using Cargo, without unrelated
   dependency upgrades. Run `make api-snapshots`.
4. Run:

   ```sh
   python3 scripts/check_release.py --tag vVERSION
   python3 -m unittest discover -s scripts/tests
   make test-databases
   cargo test --locked -p client-model -p offline-packager
   make cargo-deny
   make api-check API_BASE=PREVIOUS_RELEASE_TAG
   make rebuild-docker-image
   bash scripts/smoke_image.sh wuerges/animeitor:latest
   ```

   Docker is required; the API checker uses `tufin/oasdiff:v1.30.0`. The release
   checker prints the previous tag. Compare against its commit, never HEAD or
   newly regenerated snapshots. The Docker build also compiles the WASM client.
   The smoke test checks server startup, frontend HTML, authenticated capabilities,
   and CLI entry points; it does not replace contest behavior tests.
5. Review the diff, commit the version, lockfile, changelog, and both snapshots
   together, create an annotated `vVERSION` tag, then push the commit and tag.
6. Watch the Release workflow. It repeats validation, builds and smoke-tests an
   image, transfers that image to the publishing job, pushes version and latest
   tags, verifies registry image identity, records the digest, and creates a
   GitHub release. It rejects existing image version tags and prevents older
   releases from replacing latest. Failed publishing may need manual recovery;
   never delete or overwrite released tags to hide a failure.

Pull requests run release tests and the Docker smoke test without publishing
credentials. Version-changing PRs also require dated release metadata. Ordinary
PRs can retain Unreleased changelog entries; reviewing their completeness remains
an agent/reviewer responsibility.

The manual `make republish-docker-image` target remains available for legacy use,
but bypasses this workflow and should not be used for agent-driven releases.
Production deployment is separate: pin the version or digest, retain the previous
digest, and check storage/backups before restarting a running contest. Memory
storage loses state on restart. Frontend deployments outside the image must also
be verified separately.

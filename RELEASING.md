# Release Process

This document is the source of truth for releasing `best-claude-hud`.

## 1. Confirm the release candidate

1. Start from a clean `main` branch synchronized with `origin/main`.
2. Confirm the intended semantic version against all three public states:
   local/remote Git tags, GitHub Releases, and `npm view best-claude-hud`.
3. Complete implementation, regression tests, adversarial review when required,
   and the normal fix commit before preparing a release.
4. Push the fix commit only after explicit approval, then wait for every CI job
   to pass.

Do not treat a passing fix commit as release-ready while its manifests still
declare the previous version.

## 2. Prepare the version

Update all release metadata in one release-preparation commit:

- `Cargo.toml`
- the root `best-claude-hud` package entry in `Cargo.lock`
- `packaging/npm/package.json`
- `CHANGELOG.md`
- the release highlights in `.github/workflows/release.yml` when user-facing
  behavior changed

All three version declarations must match the intended tag without the `v`
prefix. The changelog date and release notes must describe the actual diff.

Run the complete local release gate:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --verbose
cargo build --release
npm --prefix packaging/npm run check
npm --prefix packaging/npm run test
npm --prefix packaging/npm run pack:dry-run
nix flake check --print-build-logs
nix build .#default --print-build-logs --no-link
```

Review the full diff and verification evidence, then obtain explicit approval
before committing. Use a Conventional Commit such as:

```text
release: prepare vX.Y.Z
```

Stop after the commit. Obtain separate approval before pushing it, and wait for
all CI jobs on the release-preparation commit to pass.

## 3. Create the tag and GitHub Release

Before tagging, verify all of the following:

- `main`, `origin/main`, and the intended release commit are identical.
- The working tree is clean.
- Cargo, Cargo.lock, npm, changelog, and release highlights agree on the version.
- The tag does not already exist locally or remotely.
- A GitHub Release and npm package do not already exist for the version.

Obtain explicit approval for the tag and release stage. Create an annotated tag
using the fixed project identity, then push only that tag:

```bash
git -c user.name=GaoSSR -c user.email=18220699480@163.com \
  tag -a vX.Y.Z -m "best-claude-hud vX.Y.Z"
git push origin vX.Y.Z
```

The tag push triggers `.github/workflows/release.yml`. Wait for it to build all
supported native archives, create npm tarballs and checksums, and publish the
GitHub Release. Its first job rejects any mismatch between the tag, Cargo
metadata and lockfile, and the npm manifest. Verify the workflow conclusion and
every expected release asset. Download the published assets into a temporary
directory and validate every `.sha256` file against its archive; an asset list
alone is not verification. Remove the temporary directory afterward.

Do not delete or move the tag if the workflow fails. Report the exact failure
and request approval for the recovery action.

## 4. Publish npm packages

Only after the GitHub Release and its npm tarballs are verified, obtain separate
approval and dispatch the trusted-publishing workflow:

```bash
gh workflow run "npm publish" \
  --repo GaoSSR/best-claude-hud \
  -f version=X.Y.Z
```

Wait for completion and verify the root version plus platform dist-tags through
the npm registry. If publishing is partial, inspect the versions that succeeded
and do not rerun the workflow unchanged: it publishes platform packages in a
fixed order and will stop at an already published version. Report the exact
partial state and obtain explicit approval for a scoped recovery; never publish
or deprecate additional versions speculatively.

## 5. Upgrade the local installation

Local installation is a final, separate approval checkpoint. After npm reports
the new version:

1. Detect the existing installation channel and executable path.
2. Upgrade through that same channel unless the user explicitly chooses another.
3. Do not change unrelated global packages or configuration.
4. Verify both `command -v best-claude-hud` and
   `best-claude-hud --version`.
5. Remove any temporary installer or diagnostic files created by the upgrade.

Issue or pull request comments and state changes are not part of the release;
handle them only after separate explicit approval.

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `use-release-plz` input and `release-plz-token` secret: optional release
  automation via release-plz. When enabled, a release PR is maintained on
  default-branch pushes and merging it publishes, tags, and creates the GitHub
  releases; the tag-gated release job is disabled.

- `ci-ok` aggregate gate job: a single always-run context that fails on any
  job failure or cancellation, intended as the one required check in branch
  protection. The release jobs now gate on it.

- Doctest execution in the docs job (`cargo test --doc`); nextest never runs
  doctests, so doc examples were previously unexercised.

- `use-audit-check` input (default `false`): publish cargo-audit results as a
  GitHub check run via rustsec/audit-check instead of the plain audit step.
  Requires the caller to grant `checks: write`; the job inherits the caller's
  permissions, so leaving it off keeps read-only callers valid.

- `release-plz-app-id` + `release-plz-app-private-key` secrets: the
  release-plz jobs mint a short-lived, scoped GitHub App installation token,
  preferred over a `release-plz-token` PAT.

- Merge-queue support: `merge_group` events take the comprehensive test path.

- zizmor (GitHub Actions security linter) added to pre-commit.

### Changed

- The `msrv` input now defaults to empty, which reads `rust-version` from
  `Cargo.toml` so the workflow cannot drift from the manifest.
- All checkouts set `persist-credentials: false` except the release-plz jobs,
  which intentionally keep their minted token for pushing.
- Caller inputs used in `run:` scripts now pass through `env` so they expand
  as shell data rather than script text (template-injection hardening).
- Replaced `pre-commit/action` (maintenance mode, pins an `actions/cache`
  version on a deprecated Node runtime) with inlined install/cache/run steps,
  and bumped `actions/setup-python` to v6.

### Fixed

- Removed the workflow-level `concurrency` block from `rust-ci.yml`: it is
  evaluated in the caller's run context, computed the same group as a caller
  using the conventional pattern, and GitHub canceled the run as a
  parent/child deadlock. Callers now own the concurrency policy (see
  `examples/ci.yml`).
- Nested jobs no longer request elevated permissions, which failed caller
  validation ("The nested job 'release' is requesting 'contents: write', but
  is only allowed 'contents: read'") whenever the caller ran with a read-only
  token, including fork PRs. Test jobs now downscope to `contents: read` and
  the release / release-plz jobs inherit the caller's grant. Callers that
  publish must grant `contents: write` on the caller job (release-plz also
  needs `pull-requests: write`).

## [0.1.0] - 2026-07-14

### Added

- `rust-ci.yml`: reusable all-in-one CI workflow for public Rust crates
  (pre-commit, lint, release build, bare-metal no_std build, unit + property +
  fuzz tests, miri, MSRV check, semver checks, security audit, docs, a
  PR-gated publish dry run, and a tag-gated crates.io publish).
- `examples/ci.yml`: example caller workflow reproducing a single-crate public
  CI.
- Repo CI that runs the pre-commit hooks and smoke-tests `rust-ci.yml` against
  a fixture crate on every pull request and push to `main`.
- Dependabot configuration with weekly grouped updates for GitHub Actions pins
  and the fixture crate's dependencies.
- MIT license.

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

### Fixed

- Nested jobs no longer request elevated permissions, which failed caller
  validation ("The nested job 'release' is requesting 'contents: write', but
  is only allowed 'contents: read'") whenever the caller ran with a read-only
  token, including fork PRs. Test jobs now downscope to `contents: read` and
  the release / release-plz jobs inherit the caller's grant. Callers that
  publish must grant `contents: write` on the caller job (release-plz also
  needs `pull-requests: write`).

## [1.0.0] - 2026-07-14

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

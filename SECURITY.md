# Security policy

## Reporting a vulnerability

Report security issues through GitHub's private vulnerability reporting: open
the [Security tab](https://github.com/luminartech/rust_workflow/security) and
choose **Report a vulnerability**. That opens a private advisory visible only
to the maintainers.

Please do not open a public issue for a security report.

## Why this repository is sensitive

This repository is not a library. It publishes a reusable GitHub Actions
workflow that other repositories call, so a change here executes in their CI.
Two things follow:

- **Consumers pin a mutable tag.** Callers reference
  `rust-ci.yml@v1`, and `v1` is a branch-like tag that moves. Re-pointing it
  changes every consumer's CI on their next run, with no action on their part
  and no review in their repository.
- **The release path handles publish credentials.** The release-plz jobs
  consume `cargo-registry-token`, `release-plz-app-id`,
  `release-plz-app-private-key` and `release-plz-token`. A workflow change that
  exfiltrates or misuses those can publish crate versions under the
  organization's name.

## Scope

In scope, and treated as a vulnerability here:

- Script or expression injection — anything that lets untrusted input (a branch
  name, a PR title, issue text) reach a `run:` block or an expression
  interpolation.
- Secret exposure — a secret written to logs, to an artifact, to a cache entry,
  or made reachable from a `pull_request_target`-style context.
- Excessive token permissions, or a job that keeps `contents: write` where
  `contents: read` would do.
- Cache or artifact poisoning that lets one job influence another's inputs.

Out of scope here, though still worth reporting upstream:

- Vulnerabilities in the third-party actions this workflow calls. Report those
  to their maintainers.
- A consumer repository's own configuration of this workflow.

## Known posture

Action pinning is currently mixed: `dtolnay/rust-toolchain` is pinned by commit
SHA, while `actions/checkout`, `actions/setup-python`, `actions/cache`,
`Swatinem/rust-cache`, `taiki-e/install-action`, `actions/upload-artifact` and
`codecov/codecov-action` are pinned by major-version tag, which upstream can
move. This is a deliberate trade-off against the churn of SHA-pinning
everything, not an oversight — but it is the right context for judging a report
about a compromised upstream action.

Workflows in this repository are linted by
[zizmor](https://github.com/zizmorcore/zizmor) and
[actionlint](https://github.com/rhysd/actionlint) on every pull request.

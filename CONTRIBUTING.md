# Contributing

Thanks for your interest in improving `rust_workflow`. This repo hosts
reusable GitHub Actions workflows for public Rust crates, so most changes are
YAML and documentation.

## Getting started

1. Fork the repository and create a topic branch.

1. Install [pre-commit](https://pre-commit.com) and set up the hooks:

   ```sh
   pre-commit install
   ```

The same hooks run in CI on every pull request, so running them locally
(`pre-commit run --all-files`) is the fastest way to catch problems.

## Making changes

- Workflow changes go in `.github/workflows/`. `rust-ci.yml` is consumed by
  other repositories via `workflow_call`, so treat its inputs, secrets, and
  defaults as a public API.
- Keep the input tables in `README.md` and the example caller in
  `examples/ci.yml` in sync with any input you add, remove, or change.
- Record user-facing changes in `CHANGELOG.md` under `Unreleased`.

## Testing workflow changes

Repo CI smoke-tests `rust-ci.yml` on every pull request by invoking it
against the fixture crate at the repository root, covering the lint, build,
no_std, unit-test, property-test, MSRV, and docs stages.

The fuzz, miri, semver-checks, security, and publish stages are not covered
by the smoke test. To exercise those end to end, point a consuming
repository's caller at your branch:

```yaml
uses: <your-fork>/rust_workflow/.github/workflows/rust-ci.yml@<your-branch>
```

then open a draft PR (or trigger `workflow_dispatch`) in that repository and
watch the run.

## Pull requests

- PR titles follow the Conventional Commit format described in
  `PULL_REQUEST_TEMPLATE.md`; the title becomes the merge commit summary.
- CI (the pre-commit hooks) must pass.
- Call out any breaking change to `rust-ci.yml` inputs or behavior in the PR
  description and the changelog.

## Versioning

Releases are git tags following [Semantic Versioning](https://semver.org).
Consumers pin a major tag (for example `@v1`), which moves to the latest
compatible release. Renaming or removing an input, changing a default in a way
that alters behavior, or removing a job is a breaking change and requires a
new major version.

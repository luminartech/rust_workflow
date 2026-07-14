# rust_workflow

Reusable GitHub Actions workflows for public Rust crates.

Today this repo provides one all-in-one CI workflow,
[`rust-ci.yml`](.github/workflows/rust-ci.yml). It is designed to grow: future
workflows (release-only, benchmark, cross-compile matrices) can be added
alongside it without breaking callers.

The `Cargo.toml` and `src/` at the repo root are a tiny fixture crate, not a
published library: repo CI smoke-tests `rust-ci.yml` against it on every pull
request.

## Usage

Add a thin caller workflow to your repo. The caller owns the triggers; the
reusable workflow owns the jobs. A full example that reproduces a single-crate
public CI (no_std build, fuzzing, miri, tag-gated crates.io publish) lives in
[`examples/ci.yml`](examples/ci.yml).

```yaml
# .github/workflows/ci.yml
name: CI
on:
  pull_request:
  push:
    branches: [main]
    tags: ['v*']

# Concurrency is caller-owned; the reusable workflow defines none of its own.
concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true

jobs:
  ci:
    uses: luminartech/rust_workflow/.github/workflows/rust-ci.yml@v1
    permissions:
      contents: write
    secrets:
      cargo-registry-token: ${{ secrets.CARGO_CI_TOKEN }}
    with:
      publish-repository: my-org/my_crate
```

Every stage runs by default. Disable the ones you do not need with the
`run-*` / `publish-crate` toggles, e.g. a pure-`std` crate turns off the
bare-metal build with `run-no-std: false`.

All test jobs downscope themselves to `contents: read`; only the release path
uses the caller's broader grant. `contents: write` is needed to create GitHub
releases (drop it, along with the registry token, if you disable publishing);
release-plz additionally needs `pull-requests: write`. The reusable workflow
never requests write scopes itself, so callers running with a read-only token
(fork PRs, for example) still pass workflow validation.

For branch protection, require the single `CI OK` check: it aggregates every
job (tolerating intentionally-skipped ones) and fails on any failure or
cancellation, so a job rename cannot silently drop protection. Repos using a
merge queue add `merge_group:` to the caller's triggers; merge-queue runs take
the comprehensive path as the final pre-merge gate.

## Run policy

Most stages are gated only by their toggle. Two behaviors are event-driven:

- **Comprehensive path.** Fuzz targets run for `fuzz-max-time` seconds normally
  and `fuzz-max-time-comprehensive` seconds on comprehensive runs; miri only
  runs on comprehensive runs. The comprehensive path auto-enables on `schedule`,
  on `merge_group` events, on pushes to `default-branch`, and on tag pushes. A
  manual dispatch can force it with `comprehensive-tests: true`.
- **Semver + publish.** Semver checks run on pull requests and `v*` tag
  pushes. The publish dry run happens on pull requests only, catching
  unpublishable changes before merge without needing a registry token. The
  release job runs only on `v*` tag pushes, subject to `publish-crate` and the
  optional `publish-repository` guard.

## Release automation (release-plz)

Set `use-release-plz: true` to hand versioning, changelogs, tags, and
publishing to [release-plz](https://github.com/release-plz/release-plz)
instead of the tag-gated release job (which is disabled while the flag is on).
On every `default-branch` push, a release PR is created or updated with
version bumps and changelog entries derived from conventional commits; merging
it publishes to crates.io, pushes the version tags, and creates the GitHub
releases. Pushes where everything is already published are a no-op, and the
publish half only runs after the rest of CI has passed.

Requirements for consumers:

- Grant the caller job `permissions: contents: write, pull-requests: write`.
- Pass a token so the release PR triggers CI (PRs opened with the default
  `github.token` do not). Preferred: `release-plz-app-id` +
  `release-plz-app-private-key`, from which each job mints a short-lived,
  scoped GitHub App installation token. Alternative: a `release-plz-token`
  PAT.
- Configure per-repo behavior with a `release-plz.toml` if needed.
- The publish step still runs in the `publish-environment` environment and
  honors the `publish-repository` guard; environment protection rules will
  gate every default-branch release run.

## Inputs

### Stage toggles (boolean, default `true` unless noted)

| Input                 | Stage                                                                              |
| --------------------- | ---------------------------------------------------------------------------------- |
| `run-pre-commit`      | pre-commit hooks                                                                   |
| `run-lint`            | rustfmt + clippy                                                                   |
| `run-build`           | release build                                                                      |
| `run-no-std`          | bare-metal no_std build + no_std clippy variants                                   |
| `run-unit-tests`      | unit/integration tests with coverage                                               |
| `run-property-tests`  | property tests                                                                     |
| `run-fuzz-tests`      | cargo-fuzz targets                                                                 |
| `run-miri-tests`      | miri (only executes on comprehensive runs)                                         |
| `run-msrv`            | MSRV build                                                                         |
| `run-semver-checks`   | cargo-semver-checks (PRs + tags only)                                              |
| `run-security`        | cargo-audit + cargo-deny                                                           |
| `use-audit-check`     | default `false`: cargo-audit as a check run; the caller must grant `checks: write` |
| `run-docs`            | rustdoc with warnings denied + doctests                                            |
| `run-publish-dry-run` | `cargo publish --dry-run` (PRs only)                                               |
| `publish-crate`       | crates.io publish + GitHub release (tags only)                                     |

### Run policy

| Input                 | Type    | Default | Notes                                 |
| --------------------- | ------- | ------- | ------------------------------------- |
| `comprehensive-tests` | boolean | `false` | Force the comprehensive path          |
| `default-branch`      | string  | `main`  | Branch whose pushes are comprehensive |

### Toolchain & targets

| Input               | Type   | Default              |
| ------------------- | ------ | -------------------- |
| `toolchain`         | string | `stable`             |
| `nightly-toolchain` | string | `nightly`            |
| `msrv`              | string | `''`                 |
| `no-std-target`     | string | `thumbv6m-none-eabi` |
| `alloc-feature`     | string | `alloc`              |

`msrv` defaults to empty, which reads `rust-version` from `Cargo.toml` so the
workflow cannot drift from what the manifest promises; set it only to test a
different toolchain than the manifest declares.

The no_std build is a canary, not a deployment target. Building for a `*-none`
target (where std does not exist) is the only way to prove that nothing in the
dependency graph pulls std back in; a plain `--no-default-features` build on a
hosted target cannot catch that. `thumbv6m-none-eabi` is the strictest common
denominator (32-bit, no atomic compare-and-swap), so code that builds there
builds essentially everywhere, embedded or desktop.

### Cargo invocations

| Input                         | Type   | Default                           | Notes                                                 |
| ----------------------------- | ------ | --------------------------------- | ----------------------------------------------------- |
| `package`                     | string | `''`                              | Crate for semver-checks + publish (empty = workspace) |
| `feature-flags`               | string | `--all-features`                  | Injected into build/test/clippy/docs/msrv             |
| `clippy-args`                 | string | `-D warnings -D clippy::pedantic` | Passed after `clippy --`                              |
| `unit-test-filter`            | string | `not test(prop_)`                 | nextest filter for unit tests                         |
| `property-test-filter`        | string | `test(prop_)`                     | nextest filter for property tests                     |
| `property-test-threads`       | number | `4`                               | Property-test thread count                            |
| `fuzz-targets`                | string | `''`                              | Space-separated targets (empty = `cargo fuzz list`)   |
| `fuzz-max-time`               | number | `6`                               | Per-target fuzz seconds, normal run                   |
| `fuzz-max-time-comprehensive` | number | `60`                              | Per-target fuzz seconds, comprehensive run            |
| `miri-args`                   | string | `''`                              | Extra args for `cargo miri test` (e.g. `-p my_crate`) |

### Artifact capture

| Input                          | Type    | Default         | Notes                                 |
| ------------------------------ | ------- | --------------- | ------------------------------------- |
| `upload-coverage`              | boolean | `true`          | Upload the LCOV report                |
| `coverage-artifact-name`       | string  | `coverage-lcov` |                                       |
| `coverage-retention-days`      | number  | `7`             |                                       |
| `upload-fuzz-artifacts`        | boolean | `true`          | Upload corpus/crashes on fuzz failure |
| `fuzz-artifact-retention-days` | number  | `30`            |                                       |
| `upload-docs`                  | boolean | `false`         | Upload generated rustdoc              |
| `docs-retention-days`          | number  | `7`             |                                       |

### Publish

| Input                    | Type    | Default     | Notes                               |
| ------------------------ | ------- | ----------- | ----------------------------------- |
| `publish-environment`    | string  | `crates-io` | GitHub environment gating publish   |
| `publish-repository`     | string  | `''`        | Restrict publish to this owner/repo |
| `generate-release-notes` | boolean | `true`      | Auto-generate GitHub release notes  |
| `use-release-plz`        | boolean | `false`     | Hand releases to release-plz        |

### Secrets

| Secret                        | Required | Notes                                                          |
| ----------------------------- | -------- | -------------------------------------------------------------- |
| `cargo-registry-token`        | no       | Token for `cargo publish` (needed only when publishing)        |
| `release-plz-token`           | no       | PAT for release-plz (falls back to github.token)               |
| `release-plz-app-id`          | no       | GitHub App ID; mints a short-lived token, preferred over a PAT |
| `release-plz-app-private-key` | no       | Private key of the release-plz GitHub App                      |

## Fixed by design

The runner (`ubuntu-latest`) and the common environment (`CARGO_TERM_COLOR`,
`RUST_BACKTRACE`, `CARGO_INCREMENTAL`) are not configurable. Open an issue if
a consumer needs one of these opened up.

Concurrency is the caller's job: a `concurrency` block in a called workflow is
evaluated in the caller's run context, computes the same group as the caller's
own block, and GitHub cancels the run as a parent/child deadlock. Define the
policy in your caller as shown in the usage example.

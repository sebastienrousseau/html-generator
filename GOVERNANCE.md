# Governance

This document describes how html-generator is run: who decides what, how
changes land, and how the project intends to grow its maintainer base.
It is intentionally lightweight and will formalise as the community
grows.

## Current model

html-generator is **maintainer-led**. As of the `0.x` line it has a single
lead maintainer, [Sebastien Rousseau](https://github.com/sebastienrousseau),
who is responsible for the roadmap, releases, and final decisions.

This is stated plainly rather than dressed up: a single maintainer is a
real bus-factor risk, and broadening the maintainer team is an explicit
`1.0` gate. Until then, the automation and the documented processes
below exist so the project does not depend on any one person's memory.

## Roles

- **Users** — file issues, ask in
  [Discussions](https://github.com/sebastienrousseau/html-generator/discussions),
  and open pull requests. No special status required.
- **Contributors** — anyone whose pull request has merged. Listed in the
  git history; thank you.
- **Maintainers** — hold merge rights and a release key. Currently one
  (the lead maintainer). Maintainers are bound by the same review and CI
  gates as everyone else.

### Becoming a maintainer

There is no rigid quota. A contributor who has landed several
non-trivial, high-quality changes, engaged constructively in review, and
shown good judgement on scope and compatibility may be invited to become
a maintainer by the lead maintainer. Reaching **two or more active
maintainers** is a stated goal on the road to `1.0`.

## How decisions are made

- **Day-to-day** (bug fixes, docs, dependency bumps): lazy consensus —
  proposed via PR, merged once CI is green and review is satisfied.
- **Notable changes** (public API, new features, MSRV bumps, new
  dependencies): discussed first in an issue or Discussion so the
  rationale and alternatives are on record before code is written.
- **Disagreements**: the lead maintainer is the tie-breaker while the
  project is single-maintainer. As the team grows this will move to
  maintainer consensus.

All changes, including a maintainer's own, go through a pull request and
the full CI suite. Direct pushes to `main` are not used.

## Compatibility & releases

- Versioning follows the `0.x` posture: under Cargo's SemVer rules a
  `0.0.x` release may carry breaking changes, and every one is
  documented in [`CHANGELOG.md`](CHANGELOG.md). Releases increment by
  `+0.0.1`.
- The minimum supported Rust version (MSRV) is recorded as
  `rust-version` in `Cargo.toml` and enforced in CI; changes to it are
  treated as notable and called out in the changelog.
- Releases are signed tags. The version-bearing files are checked by
  `scripts/verify-release-versions.sh` before a tag exists.

## Code of conduct & security

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
Security issues follow the private disclosure process in
[`SECURITY.md`](SECURITY.md) — please do not open public issues for
vulnerabilities.

## Changing this document

Amend `GOVERNANCE.md` via pull request like any other change. While the
project is single-maintainer, the lead maintainer approves governance
changes; once there are multiple maintainers, governance changes require
maintainer consensus.

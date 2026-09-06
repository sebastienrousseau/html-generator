# 0005. First-party crates are pinned exactly (`=0.0.X`)

- **Status:** accepted
- **Date:** 2026-07-26 (recorded 2026-09-06)

## Context

`noyalib` is a 0.0.x crate from the same maintainer. Under Cargo's
SemVer rules a 0.0.x release may break, and a caret requirement on
`0.0.15` would silently accept `0.0.16`. Consumers were taking upgrades
they never chose.

## Decision

`noyalib` is required as `=0.0.X`. A bump is a deliberate, tested change
to this crate, released with a changelog entry, and the pin and the
lockfile move together — checked by
`scripts/verify-release-versions.sh`.

## Consequences

- No surprise upgrades in consumers.
- Every noyalib release needs an html-generator release to reach
  consumers. The family accepts that cost; it is the same model
  noyalib's own satellites use.

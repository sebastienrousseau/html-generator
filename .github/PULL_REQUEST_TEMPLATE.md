<!-- SPDX-FileCopyrightText: 2026 html-generator -->
<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

## Summary

<!-- What does this change and why? One or two sentences. -->

## Related issue

<!-- e.g. "Closes #123". Use "N/A" if none. -->

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change (API or behaviour)
- [ ] Docs / CI / tooling only

## Checklist

- [ ] `cargo fmt --all --check` is clean
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` is clean
- [ ] `cargo test --all-features` passes (added/updated tests for the change)
- [ ] Public API changes are documented and reflected in `CHANGELOG.md`
- [ ] No new `unsafe` (the workspace is `#![forbid(unsafe_code)]`)
- [ ] Commits are signed (CI verifies this)
- [ ] Considered MSRV (1.86) and `no_std` impact, where relevant

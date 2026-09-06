# Security Policy

## Supported Versions

| Version | Supported |
|:--------|:---------:|
| 0.0.x   | Yes       |

## Reporting a Vulnerability

Report security vulnerabilities by emailing **sebastian.rousseau@gmail.com**.

Do not open a public issue for security reports.

Include:

- A description of the vulnerability.
- Steps to reproduce.
- Affected versions.
- Any suggested fix (optional).

Expect an initial response within 48 hours. A fix or mitigation plan will follow within 7 days of confirmation.

## Security Design

html-generator turns untrusted Markdown into HTML that a site will
serve. That makes injection the first-order risk and the reason for
most of the design below.

- `#![forbid(unsafe_code)]` — zero unsafe blocks, guaranteed by the
  compiler.
- No C dependencies, no FFI, no network I/O. File access happens only
  where the caller names a path.

### Injection

Raw HTML in Markdown is **off by default**: `HtmlConfig::allow_unsafe_html`
is `false`, so the Markdown renderer escapes embedded markup and nothing
a document author writes reaches the page as live HTML.

`sanitize_html` is a **second, separate switch and is also off by
default**. It has no effect on its own: it runs the output through
`ammonia`, an allow-list sanitiser, only when `allow_unsafe_html` is
`true`. A caller that turns raw HTML on and leaves sanitisation off is
serving untrusted markup verbatim, which is a deliberate choice the
caller owns. If you enable one, enable both.

Generated meta tags and structured data escape their values, so metadata
cannot break out of an attribute.

### Resource limits

| Limit | Default | Purpose |
|:---|:---|:---|
| `HtmlConfig::max_input_size` | 5 MiB | caps the Markdown a single call accepts; configurable per call |
| `MAX_HTML_SIZE` | 1 MB | caps the HTML the accessibility and SEO passes will rewrite; a compile-time constant |
| `MAX_BUFFER_SIZE` | 16 MiB | bounds the reader used for file and stdin input; a compile-time constant |

Only the first is configuration. These stop a single pathological
document, not a flood of them: a caller handling untrusted input at
volume still needs its own admission control.

### Fuzzing

`fuzz/` holds three libFuzzer targets: the whole Markdown pipeline,
front-matter extraction, and the accessibility pass (ARIA enrichment
plus WCAG validation, which rewrites markup by byte offset and is where
this crate's slice-boundary bugs have lived). A committed seed corpus
and every fixed-bug reproducer replay on each push; see
[`DEVELOPMENT.md`](DEVELOPMENT.md).

### A known upstream finding

Miri reports undefined behaviour inside `servo_arc 0.4.3`, reached
through `scraper` → `selectors` whenever a CSS selector is parsed. It is
a pointer derived from a zero-size retag in `Arc::from_header_and_iter`,
rejected under both of Miri's borrow models. It is not reachable as a
memory-safety failure in practice on any target this crate builds for,
and it is not fixable here: this crate contains no `unsafe` at all. It
is recorded rather than hidden, and it is why the Miri gate is scoped
(see [`DEVELOPMENT.md`](DEVELOPMENT.md)).

### Supply Chain

- `cargo-deny` (licences, advisories, sources) and `cargo-audit` in CI.
  Documented advisory exemptions live in `.cargo/audit.toml` and mirror
  `deny.toml`, each with the upstream reason.
- Dependency provenance recorded with `cargo-vet` (`supply-chain/`);
  exemptions are regenerated on every dependency change and the CI
  ratchet lets the count shrink but never grow.
- `noyalib` is pinned exactly (`=0.0.X`); a bump is a deliberate release
  of this crate.
- `Cargo.lock` committed for deterministic builds; CI builds `--locked`.
- All GitHub Actions SHA-pinned.
- REUSE/SPDX compliance linted in CI.

### Commit Integrity

All commits on the main branch are signed, and releases are signed
tags. The release-signing key is published in [`KEYS.asc`](KEYS.asc):

```text
4B7F16C909C7A8EE9BED338A4F047EDF5F90F638
```

Signing key `Sebastien Rousseau <sebastian.rousseau@gmail.com>`,
ed25519, signing-only, expires 2028-08-16. Verify the fingerprint out
of band before trusting it.

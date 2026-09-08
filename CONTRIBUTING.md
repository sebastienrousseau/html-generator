# Contributing to html-generator

Contributions are welcome. This guide covers the essentials.

## Prerequisites

- **Rust 1.80.0+** (the `rust-version` in `Cargo.toml`; Cargo refuses
  older toolchains).
- Git with [commit signing](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits)
  configured.
- For the full local gate: a nightly toolchain (Miri, cargo-fuzz,
  cargo-llvm-cov), `cargo-deny`, `cargo-vet`, `cargo-audit`, `uv` and
  `npx`. [`DEVELOPMENT.md`](DEVELOPMENT.md) lists what each is for.

## Getting Started

```sh
git clone https://github.com/sebastienrousseau/html-generator.git
cd html-generator
make
```

`make` runs `cargo check`, `cargo clippy`, and `cargo test` in sequence.

## Branch naming

Use Conventional Commits-flavoured prefixes. The branch prefix tells
reviewers what kind of change to expect before they open the diff:

| Prefix | Use when… | Example |
|---|---|---|
| `feat/` | adding user-visible behaviour or public API | `feat/typed-extraction` |
| `fix/` | fixing a bug whose behaviour change is observable | `fix/unescape-double-decode` |
| `perf/` | speeding up code without changing behaviour | `perf/single-pass-escape` |
| `refactor/` | restructuring code without behaviour change | `refactor/split-flatteners` |
| `docs/` | docs-only changes | `docs/adr-exact-pins` |
| `test/` | tests-only changes | `test/cover-context-arms` |
| `ci/` | CI / build / packaging changes | `ci/coverage-gate` |
| `chore/` | dependency bumps and similar housekeeping | `chore/bump-quick-xml-0.42` |

Release work lands on the branch named for the release it targets
(`feat/vX.Y.Z`); open PRs against `main`.

## Making changes

1. Fork the repository and create a branch using the prefixes above.
2. Write code. Match the local style of the file you're touching —
   read 2–3 neighbours before introducing a new pattern.
3. **Add or update tests in the same commit / PR** as the
   behaviour change. Never as a follow-up. A regression fix lands with
   the input that found it (`fuzz/regressions/` for fuzzer findings).
4. Run the full check suite:

```sh
make           # check + clippy + test
make fmt       # rustfmt --check
make lint      # markdownlint + codespell + REUSE
make deny      # cargo-deny supply-chain audit
make vet       # cargo-vet provenance (regenerate exemptions after a
dep change)
```

5. Commit with `git commit -S` (signed).

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/) format.
The scope is the module or subsystem touched:

```text
<type>(<scope>): <imperative summary>

<optional body explaining the why>

<optional footer with breaking-change notes, issue refs>
```

Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `ci`,
`chore`, `build`, `revert`.

Scopes: `accessibility`, `elements`, `emojis`, `error`, `generator`,
`math`, `minifier`, `performance`, `seo`, `utils`, `wasm`, `deps`,
`repo`, `fuzz`, `bench`.

Examples:

```text
fix(accessibility): rewrite offsets back to front after each edit
build(deps): noyalib =0.0.37 and comrak 0.44
test(minifier): cover every whitespace-collapse branch
docs(adr): add 0004 native minifier
```

Sign every commit (`git commit -S`). Unsigned commits won't be
merged. Set up GPG or SSH signing per the
[GitHub guide](https://docs.github.com/en/authentication/managing-commit-signature-verification).

## Pull requests

- Open against `main`.
- Title follows the same Conventional Commits format as commits.
- Body includes:
  - **What changed** in 1–3 bullets
  - **Why** in plain English
  - **Test plan** — what the reviewer should expect green
- Keep PRs focused. One logical change per PR; mechanical
  refactors and behaviour changes get separate PRs for blame
  hygiene. Structure cleanups never couple to code changes.
- CI must be green: clippy `-D warnings`, all tests, formatter,
  REUSE compliance, supply-chain audit and vet, coverage gate
  (98 % lines), Miri, fuzz corpus replay, docs lint.

## Code standards

- `#![forbid(unsafe_code)]`. **No `unsafe` blocks, ever.** See
  [ADR 0001](./docs/adr/0001-zero-unsafe-policy.md).
- All public items require documentation (`#![warn(missing_docs)]`,
  and `cargo doc` runs with warnings denied in CI).
- Public docstring rule: lead with one-line summary; include
  `# Examples` with working code; include `# Errors` for fallible
  functions; include `# Panics` if any path can panic.
- `cargo clippy --all-targets --all-features -- -D warnings` must pass.
- `cargo fmt --check` must pass.
- New behaviour ships with new tests *in the same commit*.
- New deps must come with a one-line rationale in the commit
  message body. A new lockfile entry is a code change.

## Architectural decisions

For changes that touch the generated HTML shape, the public API
surface, the dependency floor, or core invariants like the unsafe
policy: write an [ADR](./docs/adr/) before opening the PR. The
template lives at [`docs/adr/TEMPLATE.md`](./docs/adr/TEMPLATE.md).

The bar is "would I want a future contributor to read this before
proposing the opposite?" — if yes, ADR. If no, commit message
suffices.

## Reporting issues

Open an issue on GitHub. Include:

- A minimal Markdown or HTML fragment that reproduces the problem.
- Expected behaviour vs. actual behaviour.
- Rust version (`rustc --version`).
- html-generator version (`grep '^version' Cargo.toml`).

For security issues, **do not file a public issue.** See
[SECURITY.md](./SECURITY.md) for the disclosure process.

## License

By contributing, you agree that contributions are licensed under
the same dual license as the project: [MIT](LICENSE-MIT) or
[Apache 2.0](LICENSE-APACHE).

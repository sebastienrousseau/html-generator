<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Developing html-generator

The single entry point for working on this repository. User-facing
documentation lives in the [README](README.md) and [`docs/`](docs/);
contribution etiquette and review expectations live in
[`CONTRIBUTING.md`](CONTRIBUTING.md). This file is the *how*: toolchain,
tasks, and reproducing every CI gate locally.

## Toolchain

| What | Version | Why |
| :--- | :--- | :--- |
| Rust stable | `rust-version` in `Cargo.toml` or later | MSRV, enforced by Cargo and CI |
| Rust nightly | any recent | Miri, cargo-fuzz, coverage (`cargo-llvm-cov`) |
| cargo-deny, cargo-vet, cargo-audit | cargo-vet **0.10.2 or later** | supply-chain gates; earlier cargo-vet rejects a `trusted-publisher` entry |
| cargo-llvm-cov | latest | coverage gate |
| cargo-fuzz | latest, **installed from source** (`cargo install --locked cargo-fuzz`) | the prebuilt musl binary infers its own build triple as the fuzz target and dies on "sanitizer is incompatible with statically linked libc" |
| wasm-pack | latest | the wasm smoke tests, which native `cargo test` cannot run |
| uv (`uvx`) and npx | any | `reuse`, `codespell`, `markdownlint` for the docs lint |

```bash
git clone https://github.com/sebastienrousseau/html-generator
cd html-generator
make            # check + clippy + test — the default gate
```

## Task map

`make` targets are the canonical dev tasks (see the
[`Makefile`](Makefile) header for the full list):

| Task | Command |
| :--- | :--- |
| Everything a PR needs first | `make` |
| Full test suite | `make test` |
| Lints / formatting | `make clippy` / `make fmt` |
| Docs lint (markdownlint, codespell, REUSE) | `make lint` |
| Docs as CI builds them | `make doc` |
| Coverage gate | `make coverage` |
| Miri | `make miri` |
| Fuzz targets, corpus replay | `make fuzz` |
| All examples | `make examples` |
| Benches compile and run once | `make bench-smoke` |
| Version-bearing files agree | `make versions` |
| Supply chain | `make deny` / `make vet` / `make audit` |

## Reproducing the CI gates

CI has two workflows. [`ci.yml`](.github/workflows/ci.yml) calls the
shared pipelines from
[`sebastienrousseau/pipelines`](https://github.com/sebastienrousseau/pipelines);
[`quality.yml`](.github/workflows/quality.yml) holds the gates the
repository standard requires that the shared pipeline does not cover.

| CI job | Local reproduction | Gotcha |
| :--- | :--- | :--- |
| `ci` (fmt, clippy, test, cross-platform) | `make` | |
| `coverage-gate` | `make coverage` | nightly; 98 % lines, excluding `src/wasm.rs` |
| `miri` | `make miri` | scoped by `scripts/miri.sh`; the scraper-backed modules cannot be verified (see below) |
| `fuzz-replay` | `make fuzz` | needs a source-installed cargo-fuzz (see above) |
| `docs-lint` | `make lint` | British spellings are house style; see `.codespellrc` |
| `cargo-vet` | `make vet` | after a dep change: `cargo vet regenerate exemptions`; the count must not exceed `supply-chain/exemptions-baseline.txt` |
| `release-hygiene` | `make versions && make examples && make bench-smoke && make doc` | |
| `cargo-audit` | `make audit` | if a local cargo alias named `audit` shadows the subcommand, run `cargo-audit audit` |

## Coverage: the threshold and why

The gate is **98 % lines**, measured with `cargo llvm-cov --all-features`
and **excluding `src/wasm.rs`**.

That exclusion is the only one, and it is not a convenience. `wasm.rs`
is `#[wasm_bindgen]` glue behind `#[cfg(feature = "wasm")]`: its
functions exist to be called from JavaScript on a `wasm32` target, and a
native coverage run cannot execute a single line of them. Counting them
would make the number a measure of how much wasm glue the crate has
rather than of how well the library is tested. The module is verified
instead by `tests/wasm_smoke.rs` under `wasm-pack test --node --features
wasm`.

What remains uncovered inside the gate is mostly defensive: error arms
for conditions the surrounding code has already excluded, and the
approximate-offset fallbacks in the accessibility rewriter that only
fire on markup the anchoring pass could not locate.

## Miri: what it covers, and what it cannot

The crate is `#![forbid(unsafe_code)]`, so Miri is not policing this
crate's own code. It is here to check the *interaction* with
dependencies that use `unsafe` internally.

`scripts/miri.sh` runs it over `elements`, `emojis`, `error`, `math` and
`minifier`. Two groups of modules are excluded, and the reasons are
different:

- **`accessibility`, `seo`, `generator`, `utils`, `config` — blocked by
  a dependency.** Their tests parse CSS selectors through `scraper`,
  which goes `selectors` → `servo_arc 0.4.3`, whose
  `Arc::from_header_and_iter` writes through a pointer derived from a
  zero-size retag. Miri rejects it under **both** borrow models:
  Stacked Borrows reports "tag does not exist in the borrow stack"
  (`servo_arc` lib.rs:809) and Tree Borrows reports "write access is
  forbidden" (lib.rs:535). It fires on any `Selector::parse`. The fix
  belongs upstream in `servo_arc`; nothing here can make that call
  sound, and pretending otherwise by deleting the job would be worse
  than saying so.
- **`performance` — blocked by isolation.** Every test in it reads or
  writes a file, and Miri forbids `open` and `mkdir`. Nothing is left to
  check once those are skipped.

Elsewhere, individual filesystem tests carry
`#[cfg_attr(miri, ignore = "…")]` with the reason inline.

## Test layout

- `src/**` `#[cfg(test)]` — unit tests next to the code.
- `tests/unit_coverage.rs` — targeted tests for paths the module suites
  miss, grouped by why they were missed.
- `tests/integration_tests.rs` — the pipeline end to end.
- `tests/fixture_tests.rs` with `tests/fixtures/` — input/expected pairs.
- `tests/wasm_smoke.rs` — `wasm32` only, run by `wasm-pack`.
- `examples/` — fourteen runnable examples plus `support.rs`, a shared
  display helper the others `mod`-include; all fourteen run in CI.
- `benches/` — Criterion harnesses, smoke-run in CI.
- `fuzz/fuzz_targets/` — `fuzz_markdown`, `fuzz_front_matter`,
  `fuzz_accessibility`; `fuzz/corpus/<target>` is the committed seed set
  and `fuzz/regressions/<target>` holds every fixed-bug reproducer. Both
  replay per push. A crash found by fuzzing lands as a regression input
  in the same commit as its fix.

## Release model

Versions increment strictly by `+0.0.1`. Before tagging, run
`make versions`: it checks `Cargo.toml`, `Cargo.lock`, the `noyalib`
pin, `CITATION.cff`, the `CHANGELOG.md` heading and every install
snippet. Tags are signed (`git tag -s vX.Y.Z`); the key is in
[`KEYS.asc`](KEYS.asc). `release.yml` is tag-triggered.

## House rules

- CI must be green in the same session that turned it red.
- Commits are signed; releases are signed tags.
- Structure cleanups never couple to code changes.
- New behaviour lands with its test in the same commit; a regression
  fix lands with the input that found it.

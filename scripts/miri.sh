#!/usr/bin/env bash
set -euo pipefail
# Runs Miri over the modules it can actually verify.
#
# The crate is `#![forbid(unsafe_code)]`, so Miri is here to check the
# *interaction* with dependencies that use `unsafe` internally, not this
# crate's own code.
#
# `scraper` cannot be verified. Its selector parsing goes through
# `selectors` into `servo_arc 0.4.3`, whose `Arc::from_header_and_iter`
# writes through a pointer derived from a zero-size retag. Miri rejects
# it under **both** borrow models — Stacked Borrows
# ("tag does not exist in the borrow stack", servo_arc lib.rs:809) and
# Tree Borrows ("write access is forbidden", servo_arc lib.rs:535) — and
# it fires on any `Selector::parse`, which is most of this crate's
# accessibility, SEO and utility surface. The fix belongs upstream in
# servo_arc; nothing in this repository can make that call sound.
#
# `performance` and `emojis` are excluded for a different and simpler
# reason: every test in them reads or writes a file, and Miri's
# isolation forbids `open` and `mkdir`. Running them would report
# "0 passed; 15 ignored" — a green job that verified nothing, which is
# worse than an honest exclusion.
#
# So Miri runs over the modules that never touch scraper and do no file
# IO. Everything else is covered by the normal test suite, the fuzz
# targets and the coverage gate. Narrowing the gate is stated here
# rather than left as a silently passing job that verifies nothing.
#
# Usage: scripts/miri.sh [extra cargo-miri args]

MODULES=(elements error math minifier)

cd "$(cd "$(dirname -- "$0")/.." && pwd)"

echo "miri: covering ${MODULES[*]}"
echo "miri: skipping accessibility, seo, generator, utils, config (scraper) and performance, emojis (file IO) — see the note in this script"

for module in "${MODULES[@]}"; do
  echo "== $module"
  cargo +nightly miri test --lib "${module}::" "$@"
done

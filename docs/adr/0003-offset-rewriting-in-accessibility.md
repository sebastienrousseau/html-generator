# 0003. ARIA enrichment rewrites by byte offset, not by rebuilding the DOM

- **Status:** accepted
- **Date:** 2026-04-29 (recorded 2026-09-06)

## Context

`add_aria_attributes` needs to add attributes to elements inside HTML
the crate has already produced. The obvious implementation parses the
document, mutates the tree, and serialises it back. That normalises
everything: attribute order, quoting, self-closing forms, whitespace —
none of which the caller asked to change, and some of which downstream
tools and golden-file tests depend on.

## Decision

Compute replacements against the source string and apply them by byte
offset, leaving every byte the pass did not intend to change exactly as
it was. Offsets resolve in three tiers: an exact anchored match, an
approximate offset carried from the previous replacement, and a
last-resort first-occurrence search. A replacement that matches none of
the three is skipped rather than applied at a guessed position.

## Consequences

- Output is minimally different: only the attributes that were added.
- This is the most delicate code in the crate, and its failure mode is a
  panic on a slice boundary rather than wrong output. `fuzz_accessibility`
  exists for this function specifically, and the tiers are exercised by
  input that defeats each one in turn.
- A future rewrite to a real HTML5 tree-mutating pass would need to
  preserve formatting to replace this, which is a larger change than it
  first appears.

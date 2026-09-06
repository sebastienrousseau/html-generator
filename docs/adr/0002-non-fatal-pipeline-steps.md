# 0002. Only the Markdown parse is fatal; every later step degrades

- **Status:** accepted
- **Date:** 2026-04-04 (recorded 2026-09-06)

## Context

The pipeline runs eight steps. Early versions returned `Err` from any of
them, so a document whose table of contents could not be built produced
no HTML at all. For a static-site generator that is the wrong trade: the
page is still publishable without a TOC, and the build should say so
rather than stop.

## Decision

Step 1, Markdown to HTML, is fatal and returns `Err`. Every later step
records a `Diagnostic` — `Info`, `Warning` or `Error` — on
`HtmlOutput::diagnostics` and the pipeline continues.
`generate_html` discards the diagnostics for callers that do not want
them; `generate_html_with_diagnostics` returns both.

## Consequences

- A caller can log degradation and still publish.
- A step that changed nothing must not push a diagnostic, or the list
  becomes noise. Tests assert both directions for each step.
- "Did it work?" is no longer answerable from the `Result` alone. The
  contract is documented on both functions.

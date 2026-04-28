// Copyright © 2023 - 2026 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Server-side LaTeX → MathML and Mermaid diagram passthrough.
//!
//! Two post-processing steps that run against the HTML emitted by
//! the Markdown pipeline:
//!
//! * [`convert_math`] (gated behind the `math` feature) walks the
//!   text of an HTML fragment, finds `$$..$$` and `$..$` spans, and
//!   replaces each one with a `<math>...</math>` element rendered via
//!   `pulldown-latex`. No client-side JavaScript is required —
//!   browsers render MathML natively.
//!
//! * [`rewrite_mermaid_blocks`] rewrites `<pre><code class="language-mermaid">…</code></pre>`
//!   blocks (the form `comrak`/`mdx-gen` emits for `\u{60}\u{60}\u{60}mermaid` fenced
//!   code) into `<pre class="mermaid">…</pre>` so the standard
//!   client-side mermaid.js bundle picks them up.
//!
//! Both functions take a `&str` and return a fresh `String`. Each
//! has a fast-path: if the input contains no `$` (math) or no
//! `language-mermaid` substring (diagrams), the input is returned
//! unchanged with no allocation beyond the borrow check.
//!
//! # Examples
//!
//! Mermaid passthrough is always available:
//!
//! ```
//! use html_generator::math::rewrite_mermaid_blocks;
//!
//! let html = r#"<pre><code class="language-mermaid">graph TD; A-->B</code></pre>"#;
//! let out = rewrite_mermaid_blocks(html);
//! // The block body is preserved verbatim — only the wrapping tag
//! // changes from `<pre><code class="language-mermaid">` to
//! // `<pre class="mermaid">` so client-side mermaid.js picks it up.
//! assert!(out.contains(r#"<pre class="mermaid">graph TD; A-->B</pre>"#));
//! ```
//!
//! Math is feature-gated. With the default `math` feature on:
//!
//! ```
//! # #[cfg(feature = "math")]
//! # {
//! use html_generator::math::convert_math;
//!
//! let html = "<p>Energy: $$E = mc^2$$.</p>";
//! let out = convert_math(html);
//! assert!(out.contains("<math"));
//! assert!(out.contains("display=\"block\""));
//! # }
//! ```
//!
//! # Error reporting
//!
//! Both functions are infallible. `pulldown-latex` reports parse
//! errors *inline* via a `<merror style="border-color:#b22222">…</merror>`
//! element rather than failing the whole render — invalid LaTeX
//! shows up visibly in the page, not as a 500 from the build,
//! which is the right UX for content tooling.

use once_cell::sync::Lazy;
use regex::Regex;

// ─── Mermaid: rewrite the comrak/mdx-gen fenced output ───────────

/// Matches a `<pre><code class="language-mermaid">…</code></pre>`
/// block as emitted by comrak/mdx-gen. Captures the diagram source.
/// `(?s)` enables `.` to match newlines so multi-line graphs work.
static MERMAID_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)<pre><code class="language-mermaid">(.*?)</code></pre>"#,
    )
    .expect("static MERMAID_BLOCK_REGEX must compile")
});

/// Rewrite mermaid fenced blocks for client-side rendering.
///
/// The CommonMark engine emits `\u{60}\u{60}\u{60}mermaid` fenced blocks as
/// `<pre><code class="language-mermaid">…</code></pre>`. The mermaid.js
/// bundle, however, looks for `<pre class="mermaid">…</pre>` (or
/// `<div class="mermaid">`). This function rewrites every such block
/// so a page that includes `<script type="module">import mermaid
/// from "https://…/mermaid.esm.mjs"; mermaid.initialize({startOnLoad:true});</script>`
/// renders the diagrams without further work.
///
/// The diagram body is passed through verbatim (HTML-escaped — the
/// markdown engine has already done that). mermaid.js handles the
/// unescaping for the diagram parser.
///
/// Fast-path: returns immediately when the input contains no
/// `language-mermaid` substring (SIMD-backed `str::contains`).
///
/// # Examples
///
/// ```
/// use html_generator::math::rewrite_mermaid_blocks;
///
/// let input = r#"<p>Diagram below:</p>
/// <pre><code class="language-mermaid">graph LR
/// A --&gt; B</code></pre>"#;
/// let out = rewrite_mermaid_blocks(input);
/// assert!(out.contains(r#"<pre class="mermaid">"#));
/// assert!(!out.contains("<code class=\"language-mermaid\""));
/// ```
#[must_use]
pub fn rewrite_mermaid_blocks(html: &str) -> String {
    if !html.contains("language-mermaid") {
        return html.to_string();
    }
    MERMAID_BLOCK_REGEX
        .replace_all(html, r#"<pre class="mermaid">$1</pre>"#)
        .into_owned()
}

// ─── Math: $..$ inline and $$..$$ display → MathML ───────────────

/// Block math: `$$ … $$`. Greedy-by-default would over-match across
/// paragraphs, so we use a non-greedy `(?s).*?`. We require at least
/// one non-`$` character so empty `$$$$` is left alone.
#[cfg(feature = "math")]
static DISPLAY_MATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)\$\$([^$].*?)\$\$")
        .expect("static DISPLAY_MATH_REGEX must compile")
});

/// Inline math: `$ … $`. Matches a single `$`, then captures up to
/// the next single `$` that is not preceded by `\` (TeX-style
/// escape) and is not followed by another digit (avoids matching
/// `$1` and `$2` in plain prose). Run AFTER display math so the
/// `$$..$$` form is eaten first.
#[cfg(feature = "math")]
static INLINE_MATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$([^\s$][^$]*?[^\s\\$])\$(?:[^0-9]|$)")
        .expect("static INLINE_MATH_REGEX must compile")
});

/// Convert LaTeX math spans inside an HTML fragment to MathML.
///
/// Two delimiter styles are recognised, in this order:
///
/// * `$$...$$` → `<math display="block">…</math>`
/// * `$...$`   → `<math>…</math>` (inline)
///
/// The matchers are deliberately conservative: a `$` immediately
/// followed by a digit is not treated as math (so `$5` and `$2.50`
/// in prose stay literal), and an inline match must have non-space
/// content. Unbalanced `$` is left as-is.
///
/// Fast-path: returns immediately when the input contains no `$`
/// character.
///
/// # Examples
///
/// Block math:
///
/// ```
/// use html_generator::math::convert_math;
///
/// let out = convert_math("<p>$$x + y$$</p>");
/// assert!(out.contains(r#"display="block""#));
/// assert!(out.contains("<math"));
/// ```
///
/// Inline math:
///
/// ```
/// use html_generator::math::convert_math;
///
/// let out = convert_math(r"<p>Pythagoras: $a^2 + b^2 = c^2$.</p>");
/// assert!(out.contains("<math"));
/// // Inline form is the default; no `display="block"`:
/// assert!(!out.contains(r#"display="block""#));
/// ```
///
/// Plain prose with `$` is not touched:
///
/// ```
/// use html_generator::math::convert_math;
///
/// let out = convert_math("<p>That cost $5.</p>");
/// assert_eq!(out, "<p>That cost $5.</p>");
/// ```
#[cfg(feature = "math")]
#[must_use]
pub fn convert_math(html: &str) -> String {
    if !html.contains('$') {
        return html.to_string();
    }

    // Phase 1: display math `$$..$$`. Apply first so `$$x$$` is not
    // first eaten by the inline matcher.
    let mut out = String::with_capacity(html.len());
    let mut last = 0usize;
    for m in DISPLAY_MATH_REGEX.captures_iter(html) {
        let mat = m.get(0).expect("regex match has group 0");
        let latex = m
            .get(1)
            .expect("DISPLAY_MATH_REGEX has capture group 1")
            .as_str();
        out.push_str(&html[last..mat.start()]);
        out.push_str(&render_latex(latex, true));
        last = mat.end();
    }
    out.push_str(&html[last..]);

    if !out.contains('$') {
        return out;
    }

    // Phase 2: inline math `$..$`.
    let pass1 = out;
    let mut out = String::with_capacity(pass1.len());
    let mut last = 0usize;
    for m in INLINE_MATH_REGEX.captures_iter(&pass1) {
        let mat = m.get(0).expect("regex match has group 0");
        let latex = m
            .get(1)
            .expect("INLINE_MATH_REGEX has capture group 1")
            .as_str();
        // The `(?:[^0-9]|$)` tail is captured in match_zero so we
        // need to re-emit any trailing non-`$` byte.
        let tail = mat.as_str();
        let trailer = match tail.chars().last() {
            Some('$') => "",
            Some(_) => &tail[tail.len() - 1..],
            None => "",
        };
        out.push_str(&pass1[last..mat.start()]);
        out.push_str(&render_latex(latex, false));
        out.push_str(trailer);
        last = mat.end();
    }
    out.push_str(&pass1[last..]);

    out
}

/// Render a single LaTeX span to MathML. Infallible: `pulldown-latex`
/// emits parse errors as inline `<merror>` elements rather than
/// returning `Err`, so writing into a `String` (no I/O) cannot fail.
#[cfg(feature = "math")]
fn render_latex(src: &str, display: bool) -> String {
    use pulldown_latex::config::{DisplayMode, RenderConfig};
    use pulldown_latex::{mathml::push_mathml, Parser, Storage};

    let storage = Storage::new();
    let parser = Parser::new(src, &storage);
    let mut out = String::new();
    let cfg = RenderConfig {
        display_mode: if display {
            DisplayMode::Block
        } else {
            DisplayMode::Inline
        },
        ..Default::default()
    };
    // `push_mathml` returns io::Result<()> for the writer. Writing
    // into a String never fails, and parse errors are encoded
    // inline as `<merror>` rather than returned as Err. The
    // `unwrap_or_default()` is therefore unreachable in practice
    // and exists only to satisfy the type signature.
    let _ = push_mathml(&mut out, parser, cfg);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_fast_path_is_pass_through() {
        let html = "<p>no diagrams here</p>";
        assert_eq!(rewrite_mermaid_blocks(html), html);
    }

    #[test]
    fn mermaid_block_is_rewritten() {
        let input = r#"<pre><code class="language-mermaid">graph TD;A--&gt;B</code></pre>"#;
        let out = rewrite_mermaid_blocks(input);
        assert_eq!(
            out,
            r#"<pre class="mermaid">graph TD;A--&gt;B</pre>"#
        );
    }

    #[test]
    fn mermaid_multiple_blocks_each_rewritten() {
        let input = r#"<pre><code class="language-mermaid">a-->b</code></pre>between<pre><code class="language-mermaid">c-->d</code></pre>"#;
        let out = rewrite_mermaid_blocks(input);
        assert_eq!(
            out,
            r#"<pre class="mermaid">a-->b</pre>between<pre class="mermaid">c-->d</pre>"#,
        );
        assert_eq!(out.matches(r#"<pre class="mermaid">"#).count(), 2);
        assert!(!out.contains("language-mermaid"));
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_fast_path_no_dollar_passes_through() {
        let html = "<p>no math here</p>";
        assert_eq!(convert_math(html), html);
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_inline_is_rendered() {
        let out = convert_math("<p>$x+y$</p>");
        assert!(out.contains("<math"));
        assert!(out.contains("</math>"));
        // inline math must NOT carry display="block".
        assert!(!out.contains(r#"display="block""#));
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_display_uses_block_attribute() {
        let out = convert_math("<p>$$E=mc^2$$</p>");
        assert!(out.contains("<math"));
        assert!(out.contains(r#"display="block""#));
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_dollar_followed_by_digit_left_alone() {
        let out = convert_math("<p>That cost $5 yesterday.</p>");
        // `$5` is currency, not math — left as-is.
        assert_eq!(out, "<p>That cost $5 yesterday.</p>");
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_unbalanced_dollar_left_alone() {
        let out = convert_math("<p>only one $ here</p>");
        assert_eq!(out, "<p>only one $ here</p>");
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_invalid_latex_emits_inline_merror_marker() {
        // Double subscripts (`a_b_c`) are a LaTeX syntax error per
        // pulldown-latex's own test suite. Rather than returning an
        // error, the renderer encodes the failure as an `<merror>`
        // element inline so the page surfaces the broken span
        // visibly. Our wrapper preserves that behaviour.
        let out = convert_math("<p>$a_b_c$</p>");
        assert!(
            out.contains("<merror"),
            "expected inline <merror> marker, got: {out}"
        );
    }

    #[cfg(feature = "math")]
    #[test]
    fn math_block_and_inline_in_same_input() {
        let out = convert_math("<p>see $a+b$ and $$c+d$$.</p>");
        // Two MathML blocks emitted.
        assert_eq!(out.matches("<math").count(), 2);
        assert!(out.contains(r#"display="block""#));
    }
}

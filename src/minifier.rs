// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 html-generator contributors

//! Dependency-free HTML minification.
//!
//! Replaces the `minify-html` crate, which reached
//! `rkyv 0.7` (RUSTSEC-2026-0235, out-of-bounds reads) through
//! `lightningcss` -> `parcel_sourcemap`. No upstream fix exists:
//! `parcel_sourcemap` 2.1.1 is the latest release and pins
//! `rkyv ^0.7.38`.
//!
//! # What it does
//!
//! - drops comments, except conditional comments (`<!--[if ...]>`),
//!   which are markup, not commentary
//! - drops runs of whitespace between tags where the whitespace cannot
//!   affect rendering
//! - collapses other whitespace runs to a single space
//! - leaves `<pre>`, `<textarea>`, `<script>` and `<style>` bodies byte
//!   for byte
//!
//! # What it deliberately does not do
//!
//! The previous configuration also set `minify_css` and `minify_js`.
//! Those are what pulled in `lightningcss`, and re-implementing a CSS
//! or JS minifier is a different problem with a much larger surface for
//! getting it wrong. Style and script bodies are passed through
//! unchanged. HTML structure is where the bulk of the saving is for
//! generated documents, and correctness matters more here than the last
//! few percent.
//!
//! # Why whitespace handling is element-aware
//!
//! Removing whitespace between tags is only safe when it cannot be
//! rendered. Between block-level elements it never can:
//!
//! ```text
//! <body>  <p>x</p>  </body>   ->   <body><p>x</p></body>
//! ```
//!
//! Between inline elements it can, and dropping it changes what the
//! reader sees:
//!
//! ```text
//! <span>a</span> <span>b</span>   ->   "a b", not "ab"
//! ```
//!
//! So whitespace is dropped only when the tag on at least one side is
//! block-level, and collapsed to a single space otherwise.

use crate::Result;

/// Elements whose surrounding whitespace cannot affect rendering.
///
/// Block-level and structural elements only. Anything not listed is
/// treated as inline, which is the conservative direction: an unknown
/// element keeps its whitespace.
const BLOCK_ELEMENTS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "body",
    "br",
    "canvas",
    "caption",
    "col",
    "colgroup",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "legend",
    "li",
    "main",
    "meta",
    "nav",
    "ol",
    "option",
    "p",
    "pre",
    "script",
    "section",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "ul",
];

/// Elements whose content is preserved byte for byte.
const RAW_TEXT_ELEMENTS: &[&str] =
    &["pre", "textarea", "script", "style"];

/// Minify an HTML document.
///
/// # Errors
///
/// Never fails on well-formed input; the `Result` keeps the signature
/// compatible with callers that already handle one.
pub(crate) fn minify(input: &str) -> Result<String> {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    // Element name of the most recent tag, for the whitespace policy.
    let mut prev_tag: Option<String> = None;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Comment?
            if input[i..].starts_with("<!--") {
                let is_conditional = input[i..].starts_with("<!--[if");
                let end = input[i..]
                    .find("-->")
                    .map_or(bytes.len(), |e| i + e + 3);
                if is_conditional {
                    // Conditional comments are markup: keep verbatim.
                    out.push_str(&input[i..end]);
                }
                i = end;
                continue;
            }

            // A tag. Copy it through unchanged — attributes are never
            // rewritten, which is what `allow_removing_spaces_between_
            // attributes = false` and the unquoted-value setting asked
            // for.
            let Some(rel_end) = input[i..].find('>') else {
                // Unterminated tag: emit the remainder as-is rather
                // than guess.
                out.push_str(&input[i..]);
                break;
            };
            let end = i + rel_end + 1;
            let tag = &input[i..end];
            out.push_str(tag);

            let name = tag_name(tag);

            // A raw-text element's body is copied verbatim.
            if let Some(ref n) = name {
                if RAW_TEXT_ELEMENTS.contains(&n.as_str())
                    && !tag.starts_with("</")
                    && !tag.ends_with("/>")
                {
                    let close = format!("</{n}");
                    if let Some(rel) = input[end..].find(&close) {
                        out.push_str(&input[end..end + rel]);
                        i = end + rel;
                        prev_tag = name;
                        continue;
                    }
                }
            }

            prev_tag = name;
            i = end;
            continue;
        }

        // A text run, up to the next tag.
        let next = input[i..].find('<').map_or(bytes.len(), |n| i + n);
        let text = &input[i..next];
        i = next;

        if text.trim().is_empty() {
            // Pure whitespace. Droppable only when a block-level tag
            // sits on one side of it.
            let next_tag = tag_name_at(input, i);
            let droppable = is_block(prev_tag.as_deref())
                || is_block(next_tag.as_deref());
            if !droppable && !text.is_empty() {
                out.push(' ');
            }
            continue;
        }

        // Text with content: collapse internal whitespace runs, and
        // keep a single leading/trailing space where one was present —
        // that space is rendered.
        out.push_str(&collapse_whitespace(text));
    }

    Ok(out)
}

/// Collapse every run of whitespace to one space, preserving whether
/// the run touched either end.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_ws = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(ch);
            in_ws = false;
        }
    }
    out
}

/// Lower-cased element name of a tag, if it has one.
fn tag_name(tag: &str) -> Option<String> {
    let body = tag
        .trim_start_matches('<')
        .trim_start_matches('/')
        .trim_end_matches('>')
        .trim_end_matches('/');
    if body.starts_with('!') || body.starts_with('?') {
        return None; // doctype or processing instruction
    }
    let name: String = body
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name.to_ascii_lowercase())
    }
}

/// Element name of the tag starting at `idx`, if one starts there.
fn tag_name_at(input: &str, idx: usize) -> Option<String> {
    if !input[idx..].starts_with('<') {
        return None;
    }
    let end = input[idx..].find('>')? + idx + 1;
    tag_name(&input[idx..end])
}

fn is_block(name: Option<&str>) -> bool {
    // No tag on that side means a document boundary, where whitespace
    // is not rendered either.
    // `map_or` rather than `is_none_or`: the latter is stable only
    // from 1.82 and this crate's MSRV is 1.80.
    name.map_or(true, |n| BLOCK_ELEMENTS.contains(&n))
}

#[cfg(test)]
mod tests {
    use super::minify;

    fn m(s: &str) -> String {
        minify(s).expect("minify")
    }

    #[test]
    fn collapses_whitespace_between_block_elements() {
        assert_eq!(
            m("<html>  <body>    <p>Test</p>  </body>  </html>"),
            "<html><body><p>Test</p></body></html>"
        );
    }

    #[test]
    fn keeps_whitespace_between_inline_elements() {
        // The reason the policy is element-aware. Dropping this space
        // renders "ab" instead of "a b".
        assert_eq!(
            m("<span>a</span> <span>b</span>"),
            "<span>a</span> <span>b</span>"
        );
        assert_eq!(
            m("<em>x</em>   <em>y</em>"),
            "<em>x</em> <em>y</em>"
        );
    }

    #[test]
    fn drops_comments_but_keeps_conditional_ones() {
        assert_eq!(
            m("<html><!-- note --><body><p>T</p></body></html>"),
            "<html><body><p>T</p></body></html>"
        );
        let cond = "<!--[if IE]><p>old</p><![endif]-->";
        assert!(
            m(cond).contains("[if IE]"),
            "conditional comments are markup and must survive"
        );
    }

    #[test]
    fn preserves_raw_text_bodies_byte_for_byte() {
        let pre = "<pre>  two  spaces\n  and a newline</pre>";
        assert_eq!(m(pre), pre, "pre content must not be touched");

        let script = "<script>const a = 1;   const b = 2;</script>";
        assert_eq!(m(script), script, "script bodies pass through");

        let style = "<style>body {  color : red ; }</style>";
        assert_eq!(m(style), style, "style bodies pass through");

        let ta = "<textarea>  keep   me  </textarea>";
        assert_eq!(m(ta), ta, "textarea content is user-visible");
    }

    #[test]
    fn preserves_utf8_and_in_text_spacing() {
        assert_eq!(
            m("<html><body><p>Test 你好 🦀</p></body></html>"),
            "<html><body><p>Test 你好 🦀</p></body></html>"
        );
    }

    #[test]
    fn collapses_runs_inside_text() {
        assert_eq!(m("<p>a     b</p>"), "<p>a b</p>");
    }

    #[test]
    fn leaves_entities_alone() {
        let s = "<div>&lt;Special&gt; &amp; Characters</div>";
        assert_eq!(m(s), s);
    }

    #[test]
    fn leaves_attributes_untouched() {
        let s = r#"<a href="x.html"  class="a b"  data-x='1'>t</a>"#;
        assert_eq!(m(s), s, "attribute text is never rewritten");
    }

    #[test]
    fn keeps_the_doctype() {
        let s = "<!DOCTYPE html><html><body><p>x</p></body></html>";
        assert_eq!(m(s), s);
    }

    #[test]
    fn handles_unterminated_tag_without_panicking() {
        // Emitting the remainder verbatim beats guessing at a fix.
        assert_eq!(m("<p>ok</p><div"), "<p>ok</p><div");
    }

    #[test]
    fn empty_and_whitespace_only_inputs() {
        assert_eq!(m(""), "");
        assert_eq!(m("   \n  "), "");
    }

    #[test]
    fn actually_reduces_size_on_a_realistic_document() {
        let src =
            "<html>\n  <head>\n    <title>T</title>\n  </head>\n  \
                   <body>\n    <!-- hi -->\n    <p>Hello</p>\n  \
                   </body>\n</html>";
        let out = m(src);
        assert!(
            out.len() < src.len(),
            "{} !< {}",
            out.len(),
            src.len()
        );
        assert!(!out.contains("<!--"), "comments removed");
    }
}

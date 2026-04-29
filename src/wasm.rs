// Copyright © 2023 - 2026 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WebAssembly bindings.
//!
//! Compiled in only when the crate is built with `--features wasm`
//! and a `wasm32-*` target. Exposes a small JS-friendly surface so
//! the same Markdown-to-accessible-HTML pipeline that runs server-
//! side can render directly inside Cloudflare Workers, Vercel Edge,
//! browser previews, and Node-via-WASI without extra plumbing.
//!
//! Three functions are exported:
//!
//! * [`generate_html_wasm`] — the simplest entry point. Takes a
//!   Markdown string, returns the HTML fragment. Uses
//!   [`crate::HtmlConfig::default`] under the hood (ARIA on, TOC
//!   off, JSON-LD off, no full-document wrap, no minify).
//! * [`generate_html_full_document_wasm`] — wraps the output in a
//!   complete HTML5 document.
//! * [`generate_html_with_options_wasm`] — accepts a JSON string
//!   describing the subset of [`crate::HtmlConfig`] flags that make
//!   sense in a browser/edge runtime (no file I/O, no async).
//!
//! Errors are surfaced as JS exceptions: the bindings return
//! `Result<String, JsValue>` and `wasm-bindgen` materialises the
//! `Err(JsValue)` as a thrown JS `Error` on the JS side.
//!
//! # Examples
//!
//! ```ignore
//! // doctest is `ignore`d because it only compiles when the `wasm`
//! // feature is on and the target is `wasm32`. The `cargo test`
//! // target runs on native; the WASM smoke test lives in
//! // `tests/wasm_smoke.rs` and is driven by `wasm-bindgen-test`.
//! use wasm_bindgen::prelude::*;
//! use html_generator::wasm::generate_html_wasm;
//!
//! let html: Result<String, JsValue> = generate_html_wasm("# Hi");
//! assert!(html.unwrap().contains("<h1>"));
//! ```
//!
//! Build for the browser:
//!
//! ```text
//! cargo build --release --target wasm32-unknown-unknown --features wasm
//! ```
//!
//! Or with `wasm-pack` for an npm-publishable bundle:
//!
//! ```text
//! wasm-pack build --target web --features wasm
//! ```

use wasm_bindgen::prelude::*;

use crate::{generate_html, HtmlConfig};

/// Render Markdown to an accessible HTML fragment.
///
/// This is the simplest WASM entry point — no configuration, the
/// pipeline runs with [`HtmlConfig::default`] (ARIA on, TOC off,
/// JSON-LD off, no full-document wrap).
///
/// # Errors
///
/// Returns a `JsValue` carrying the [`crate::error::HtmlError`]
/// display string when the underlying Markdown render fails.
/// `wasm-bindgen` materialises this on the JS side as a thrown
/// JavaScript `Error`.
#[wasm_bindgen(js_name = generateHtml)]
pub fn generate_html_wasm(markdown: &str) -> Result<String, JsValue> {
    generate_html(markdown, &HtmlConfig::default())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Render Markdown wrapped in a full HTML5 document skeleton.
///
/// Convenience wrapper that flips `generate_full_document` on
/// before delegating to [`generate_html`]. The output starts with
/// `<!DOCTYPE html>` and is suitable for direct write-out from a
/// Worker / Edge function.
///
/// # Errors
///
/// Same as [`generate_html_wasm`].
#[wasm_bindgen(js_name = generateHtmlFullDocument)]
pub fn generate_html_full_document_wasm(
    markdown: &str,
) -> Result<String, JsValue> {
    let cfg = HtmlConfig {
        generate_full_document: true,
        ..HtmlConfig::default()
    };
    generate_html(markdown, &cfg)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Render Markdown with a JSON-encoded subset of [`HtmlConfig`].
///
/// The JSON object accepts these keys (all optional, all booleans
/// or strings, defaults match [`HtmlConfig::default`]):
///
/// * `add_aria_attributes` *(bool)* — inject ARIA labels and roles.
/// * `generate_toc` *(bool)* — replace the `[[TOC]]` placeholder
///   with a generated table of contents.
/// * `generate_structured_data` *(bool)* — emit a `<script
///   type="application/ld+json">` JSON-LD block.
/// * `generate_full_document` *(bool)* — wrap output in
///   `<!DOCTYPE html>`.
/// * `enable_math` *(bool)* — render `$..$` and `$$..$$` to MathML
///   (requires the `math` feature, on by default).
/// * `enable_diagrams` *(bool)* — rewrite mermaid fenced blocks for
///   client-side mermaid.js.
/// * `language` *(string)* — BCP 47 language code for the `<html>`
///   `lang` attribute when full-document mode is on.
/// * `minify_output` *(bool)* — minify the final HTML.
///
/// File-I/O fields (`encoding`, `max_buffer_size`) are accepted for
/// shape compatibility but ignored: WASM has no host filesystem.
///
/// # Errors
///
/// Returns a `JsValue` carrying:
///
/// * `"invalid options JSON: …"` if `options_json` does not parse
///   as a JSON object.
/// * The [`crate::error::HtmlError`] display string when the
///   underlying render fails.
#[wasm_bindgen(js_name = generateHtmlWithOptions)]
pub fn generate_html_with_options_wasm(
    markdown: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    let opts: serde_json::Value = serde_json::from_str(options_json)
        .map_err(|e| {
            JsValue::from_str(&format!("invalid options JSON: {e}"))
        })?;

    let mut cfg = HtmlConfig::default();
    if let Some(b) =
        opts.get("add_aria_attributes").and_then(|v| v.as_bool())
    {
        cfg.add_aria_attributes = b;
    }
    if let Some(b) = opts.get("generate_toc").and_then(|v| v.as_bool())
    {
        cfg.generate_toc = b;
    }
    if let Some(b) = opts
        .get("generate_structured_data")
        .and_then(|v| v.as_bool())
    {
        cfg.generate_structured_data = b;
    }
    if let Some(b) =
        opts.get("generate_full_document").and_then(|v| v.as_bool())
    {
        cfg.generate_full_document = b;
    }
    if let Some(b) = opts.get("enable_math").and_then(|v| v.as_bool()) {
        cfg.enable_math = b;
    }
    if let Some(b) =
        opts.get("enable_diagrams").and_then(|v| v.as_bool())
    {
        cfg.enable_diagrams = b;
    }
    if let Some(b) = opts.get("minify_output").and_then(|v| v.as_bool())
    {
        cfg.minify_output = b;
    }
    if let Some(s) = opts.get("language").and_then(|v| v.as_str()) {
        cfg.language = s.to_string();
    }

    generate_html(markdown, &cfg)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

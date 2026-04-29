// Copyright © 2023 - 2026 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! WebAssembly smoke tests.
//!
//! Run only on `wasm32` targets with the `wasm` feature, via:
//!
//! ```text
//! wasm-pack test --node --features wasm
//! ```
//!
//! The `cfg` guards keep this file inert under regular `cargo test`,
//! so the native test suite stays unaffected.

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use html_generator::wasm::{
    generate_html_full_document_wasm, generate_html_wasm,
    generate_html_with_options_wasm,
};
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn renders_basic_markdown() {
    let html = generate_html_wasm("# Hello").unwrap();
    assert!(html.contains("<h1"));
    assert!(html.contains("Hello"));
}

#[wasm_bindgen_test]
fn full_document_wraps_doctype() {
    let html = generate_html_full_document_wasm("# Hi").unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<body"));
}

#[wasm_bindgen_test]
fn options_json_disables_aria() {
    let html = generate_html_with_options_wasm(
        "<button>Save</button>",
        r#"{"add_aria_attributes": false}"#,
    )
    .unwrap();
    // ARIA was disabled, so the button keeps its bare form.
    assert!(html.contains("<button>Save</button>"));
}

#[wasm_bindgen_test]
fn options_json_invalid_returns_error_value() {
    let result = generate_html_with_options_wasm("# x", "not-json");
    assert!(result.is_err());
}

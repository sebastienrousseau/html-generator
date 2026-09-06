// SPDX-License-Identifier: Apache-2.0 OR MIT
//! The whole Markdown-to-HTML pipeline over arbitrary input.
//!
//! The property is totality: any input either produces HTML or a
//! `HtmlError`. Nothing panics, nothing hangs. The diagnostics variant
//! runs too, with the optional steps on, so sanitisation, diagrams,
//! minification and heading extraction all see hostile input.
#![no_main]

use html_generator::{
    generator::generate_html_with_diagnostics, HtmlConfig,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(markdown) = std::str::from_utf8(data) else {
        return;
    };

    let _ = generate_html_with_diagnostics(markdown, &HtmlConfig::default());

    let everything = HtmlConfig {
        enable_diagrams: true,
        minify_output: true,
        generate_toc: true,
        ..HtmlConfig::default()
    };
    let _ = generate_html_with_diagnostics(markdown, &everything);
});

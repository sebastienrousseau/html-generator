// SPDX-License-Identifier: Apache-2.0 OR MIT
//! ARIA enrichment and WCAG validation over arbitrary HTML.
//!
//! `add_aria_attributes` rewrites markup by byte offset, which is where
//! this crate's slice-boundary bugs have lived; the validator then walks
//! the result. Both must be total on any input, and the enriched output
//! must still be something the validator can read.
#![no_main]

use html_generator::accessibility::{
    add_aria_attributes, validate_wcag, AccessibilityConfig,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(html) = std::str::from_utf8(data) else {
        return;
    };
    let config = AccessibilityConfig::default();

    if let Ok(enriched) = add_aria_attributes(html, None) {
        let _ = validate_wcag(&enriched, &config, None);
    }
    let _ = validate_wcag(html, &config, None);
});

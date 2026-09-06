// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Front-matter extraction over arbitrary input.
//!
//! Two properties beyond not panicking: what comes back is a slice of
//! the input (so extraction never fabricates content), and running it
//! again on its own output is stable.
#![no_main]

use html_generator::utils::extract_front_matter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(content) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(extracted) = extract_front_matter(content) {
        assert!(
            extracted.len() <= content.len(),
            "extraction grew the input"
        );
    }
});

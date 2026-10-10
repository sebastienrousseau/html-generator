// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 HTML Generator. All rights reserved.

//! Stack depth of email autolinking (GHSA-xg9p-p4jc-c46g).
//!
//! The native pipeline renders through `mdx-gen` with comrak's
//! autolink extension on. comrak 0.54 and earlier linked bare email
//! addresses by recursing once per address, so a paragraph of a few
//! thousand of them overflowed a worker thread's stack. A stack
//! overflow aborts the whole process rather than panicking, so on an
//! affected `mdx-gen` (0.0.5, comrak 0.52) this test binary dies and
//! `cargo test` fails. `mdx-gen` 0.0.6 requires comrak 0.55, which
//! links them in a loop.

// The wasm32 build renders with comrak directly and has no threads.
#![cfg(not(target_arch = "wasm32"))]

use html_generator::{generate_html, HtmlConfig};
use std::thread;

/// A server pool's worker-thread stack: the advisory measured about
/// 2,000 emails as enough to overflow it.
const WORKER_STACK: usize = 512 * 1024;

/// Comfortably past that threshold.
const EMAILS: usize = 3_000;

#[test]
fn bare_email_autolinks_do_not_overflow_a_worker_stack() {
    let markdown = "a@b.co ".repeat(EMAILS);

    let html = thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || generate_html(&markdown, &HtmlConfig::default()))
        .expect("spawn the render thread")
        .join()
        .expect("the render thread panicked")
        .expect("the render failed");

    assert_eq!(html.matches("href=\"mailto:a@b.co\"").count(), EMAILS);
}

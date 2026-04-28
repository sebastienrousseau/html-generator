// Copyright © 2023 - 2026 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Side-by-side throughput comparison against three popular Rust
//! Markdown engines on the same realistic 8 KB blog-post payload.
//!
//! Each engine renders the exact same input string; the harness
//! measures wall-clock per iteration so the README can quote
//! comparable numbers (not estimates) for the v0.0.5 performance
//! section.
//!
//! Engines compared:
//!
//! * `html_generator::generate_html` — the full pipeline (Markdown +
//!   front-matter + ARIA + TOC + JSON-LD + minification when enabled).
//! * `comrak` — a 100% CommonMark / GFM parser; the underlying engine
//!   `mdx-gen` (and therefore `html-generator`) wraps. Comparing
//!   against raw `comrak` exposes the cost of the post-processing
//!   layers we add on top.
//! * `pulldown-cmark` — a pull-parser tuned for raw throughput; the
//!   "fastest plain Markdown engine in Rust" benchmark anchor.
//! * `markdown-it` — Rust port of the JS `markdown-it` library; widely
//!   used by SSGs that need plugin extensibility.
//!
//! Run with `cargo bench --bench competitors`.

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Same realistic blog-post payload as `benchmark_generate_html_realistic`
/// in `html_benchmark.rs`: ~8 KB, 30 sections, headings, lists, tables,
/// fenced code, and links. Front matter is included for the
/// html-generator path; competitor engines that do not handle YAML
/// front-matter receive the same stripped string so they are not
/// charged for an extension they do not implement.
fn payload_with_front_matter() -> String {
    let mut md = String::with_capacity(8 * 1024);
    md.push_str("---\ntitle: Perf Bench\nauthor: Bench\n---\n\n");
    md.push_str("[[TOC]]\n\n");
    for i in 0..30 {
        md.push_str(&format!("## Section {i}\n\nA paragraph with *italic*, **bold**, and `inline code` plus a [link](https://example.com/{i}).\n\n"));
        md.push_str("- list item one\n- list item two with **bold**\n- list item three\n\n");
        md.push_str(&format!("```rust\nfn section_{i}() -> i32 {{\n    {i} * 2\n}}\n```\n\n"));
    }
    md.push_str("| A | B | C |\n|---|---|---|\n");
    for i in 0..20 {
        md.push_str(&format!("| r{i}a | r{i}b | r{i}c |\n"));
    }
    md
}

/// Same payload with the front-matter and `[[TOC]]` placeholder
/// stripped, since competitor engines do not consume those tokens
/// (they are html-generator extensions).
fn payload_plain() -> String {
    let full = payload_with_front_matter();
    // Strip the YAML front matter and the TOC placeholder so
    // competitors are not charged for emitting them.
    let after_front = full
        .split_once("---\n\n")
        .map_or(full.as_str(), |(_, rest)| rest);
    after_front.replacen("[[TOC]]\n\n", "", 1)
}

fn bench_html_generator(c: &mut Criterion) {
    let md = payload_with_front_matter();
    let config = html_generator::HtmlConfig {
        add_aria_attributes: true,
        generate_toc: true,
        generate_structured_data: true,
        ..html_generator::HtmlConfig::default()
    };
    let _ = c.bench_function(
        "competitors/html_generator_full_pipeline",
        |b| {
            b.iter(|| {
                html_generator::generate_html(
                    black_box(&md),
                    black_box(&config),
                )
            })
        },
    );
}

fn bench_comrak(c: &mut Criterion) {
    let md = payload_plain();
    let mut opts = comrak::Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    let _ = c.bench_function(
        "competitors/comrak_default_extensions",
        |b| {
            b.iter(|| {
                comrak::markdown_to_html(
                    black_box(&md),
                    black_box(&opts),
                )
            })
        },
    );
}

fn bench_pulldown_cmark(c: &mut Criterion) {
    let md = payload_plain();
    let opts = pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS;
    let _ = c.bench_function("competitors/pulldown_cmark_html", |b| {
        b.iter(|| {
            let parser =
                pulldown_cmark::Parser::new_ext(black_box(&md), opts);
            let mut out = String::with_capacity(md.len() * 2);
            pulldown_cmark::html::push_html(&mut out, parser);
            out
        })
    });
}

fn bench_markdown_it(c: &mut Criterion) {
    let md = payload_plain();
    let mut parser = markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    let _ = c
        .bench_function("competitors/markdown_it_with_extras", |b| {
            b.iter(|| parser.parse(black_box(&md)).render())
        });
}

criterion_group!(
    competitors,
    bench_html_generator,
    bench_comrak,
    bench_pulldown_cmark,
    bench_markdown_it
);
criterion_main!(competitors);

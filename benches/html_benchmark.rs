#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use html_generator::{
    accessibility::add_aria_attributes, generate_html,
    performance::minify_html, seo::generate_meta_tags,
    utils::extract_front_matter,
};
use std::hint::black_box;

fn benchmark_generate_html(c: &mut Criterion) {
    let markdown_input = r#"# Benchmark Heading
This is a test content for benchmarking HTML generation."#;
    let config = html_generator::HtmlConfig::default();
    let _ = c.bench_function("generate_html", |b| {
        b.iter(|| generate_html(black_box(markdown_input), &config))
    });
}

fn benchmark_minify_html(c: &mut Criterion) {
    let html_input =
        r#"<html><head></head><body><h1>Test</h1></body></html>"#;
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), html_input).unwrap();
    let _ = c.bench_function("minify_html", |b| {
        b.iter(|| minify_html(black_box(temp_file.path())))
    });
}

fn benchmark_add_aria_attributes(c: &mut Criterion) {
    let html_input = r#"<button>Click Me</button>"#;
    let _ = c.bench_function("add_aria_attributes", |b| {
        b.iter(|| add_aria_attributes(black_box(html_input), None))
    });
}

fn benchmark_generate_meta_tags(c: &mut Criterion) {
    let html_input = r#"<html><head><title>Page Title</title></head><body><p>Content</p></body></html>"#;
    let _ = c.bench_function("generate_meta_tags", |b| {
        b.iter(|| generate_meta_tags(black_box(html_input)))
    });
}

fn benchmark_extract_front_matter(c: &mut Criterion) {
    let input = r#"---
title: Test
---
# Content
This is the main content."#;
    let _ = c.bench_function("extract_front_matter", |b| {
        b.iter(|| extract_front_matter(black_box(input)))
    });
}

/// Realistic blog-post-sized Markdown (~8 KB, 30+ headings, tables,
/// code blocks, lists, links). Exercises the whole pipeline under a
/// payload that html5ever setup cost no longer dominates — this is
/// where regex/selector savings become visible.
fn benchmark_generate_html_realistic(c: &mut Criterion) {
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

    let config = html_generator::HtmlConfig {
        add_aria_attributes: true,
        generate_toc: true,
        generate_structured_data: true,
        ..html_generator::HtmlConfig::default()
    };

    let _ = c.bench_function("generate_html_realistic", |b| {
        b.iter(|| generate_html(black_box(&md), black_box(&config)))
    });
}

criterion_group!(
    benches,
    benchmark_generate_html,
    benchmark_generate_html_realistic,
    benchmark_minify_html,
    benchmark_add_aria_attributes,
    benchmark_generate_meta_tags,
    benchmark_extract_front_matter
);
criterion_main!(benches);

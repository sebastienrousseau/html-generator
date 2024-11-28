//! SEO functionality examples for the HTML Generator library.
//!
//! This module demonstrates the usage of SEO-related features including:
//! - Meta tag generation
//! - Structured data (JSON-LD) generation
//! - SEO optimization techniques

use html_generator::seo::{
    generate_meta_tags, generate_structured_data, MetaTagsBuilder,
    StructuredDataConfig,
};
use html_generator::HtmlError;
use std::collections::HashMap;

/// Macro for consistent result handling and error reporting
macro_rules! print_result {
    ($result:expr, $type:expr) => {
        match $result {
            Ok(data) => println!("Generated {}: \n{data}", $type),
            Err(error) => {
                eprintln!("Failed to generate {}: {error}", $type);
                return Err(error);
            }
        }
    };
}

/// Main entry point for the SEO examples.
///
/// Runs through various examples demonstrating SEO functionality including
/// meta tag generation and structured data implementation.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
fn main() -> Result<(), HtmlError> {
    println!("\n🧪 html-generator SEO Examples\n");

    generate_meta_tags_simple_example()?;
    generate_meta_tags_builder_example()?;
    generate_structured_data_example()?;
    generate_structured_data_advanced_example()?;

    println!("\n🎉 All SEO examples completed successfully!");
    Ok(())
}

/// Demonstrates basic meta tags generation using the simple API.
///
/// This example shows how to generate meta tags from HTML content
/// using the `generate_meta_tags` function.
///
/// # Errors
///
/// Returns an error if meta tag generation fails.
fn generate_meta_tags_simple_example() -> Result<(), HtmlError> {
    println!("🦀 Generate Meta Tags (Simple) Example");
    println!("---------------------------------------------");

    let html = r#"
        <html>
            <head><title>Test Page</title></head>
            <body><p>This is a test page.</p></body>
        </html>
    "#;

    print_result!(generate_meta_tags(html), "Meta Tags");
    Ok(())
}

/// Demonstrates advanced meta tags generation using the builder pattern.
///
/// This example shows how to use `MetaTagsBuilder` for more control over
/// meta tag generation.
///
/// # Errors
///
/// Returns an error if meta tag generation fails.
fn generate_meta_tags_builder_example() -> Result<(), HtmlError> {
    println!("\n🦀 Generate Meta Tags (Builder) Example");
    println!("---------------------------------------------");

    let meta_tags = MetaTagsBuilder::new()
        .with_title("Test Page")
        .with_description("This is a test page.")
        .add_meta_tag("keywords", "test,example,seo")
        .add_meta_tag("author", "Test Author")
        .build()?;

    println!("Generated Meta Tags: \n{meta_tags}");
    Ok(())
}

/// Demonstrates basic structured data generation.
///
/// This example shows how to generate JSON-LD structured data
/// from HTML content using default configuration.
///
/// # Errors
///
/// Returns an error if structured data generation fails.
fn generate_structured_data_example() -> Result<(), HtmlError> {
    println!("\n🦀 Generate Structured Data Example");
    println!("---------------------------------------------");

    let html = r#"
        <html>
            <head><title>Test Page</title></head>
            <body><p>This is a test page.</p></body>
        </html>
    "#;

    print_result!(
        generate_structured_data(html, None),
        "Structured Data"
    );
    Ok(())
}

/// Demonstrates advanced structured data generation with custom configuration.
///
/// This example shows how to generate JSON-LD structured data with
/// custom types and additional data.
///
/// # Errors
///
/// Returns an error if structured data generation fails.
fn generate_structured_data_advanced_example() -> Result<(), HtmlError>
{
    println!("\n🦀 Generate Structured Data (Advanced) Example");
    println!("---------------------------------------------");

    let html = r#"
        <html>
            <head><title>Test Article</title></head>
            <body><p>This is a test article.</p></body>
        </html>
    "#;

    let additional_data = HashMap::from([
        ("author".to_string(), "Test Author".to_string()),
        ("datePublished".to_string(), "2024-03-15".to_string()),
    ]);

    let config = StructuredDataConfig {
        page_type: "Article".to_string(),
        additional_types: vec!["WebPage".to_string()],
        additional_data: Some(additional_data),
    };

    print_result!(
        generate_structured_data(html, Some(config)),
        "Advanced Structured Data"
    );
    Ok(())
}

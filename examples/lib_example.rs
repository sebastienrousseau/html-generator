// src/examples/lib_example.rs

#![allow(missing_docs)]

use html_generator::{
    add_aria_attributes, async_generate_html,
    error::{ErrorKind, HtmlError},
    generate_html, generate_meta_tags, generate_structured_data,
    HtmlConfig, Result,
};

/// Entry point for the html-generator library usage examples.
///
/// This function demonstrates various ways to use the `html_generator` library,
/// including HTML generation, accessibility features, performance optimizations, and SEO.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🧪 html-generator Library Examples\n");

    basic_html_generation_example()?;
    accessibility_example()?;
    performance_optimization_example().await?;
    seo_optimization_example()?;

    println!("\n🎉 All examples completed successfully!");

    Ok(())
}

/// Demonstrates basic HTML generation from Markdown content.
fn basic_html_generation_example() -> Result<()> {
    println!("🦀 Basic HTML Generation Example");
    println!("---------------------------------------------");

    let markdown = "# Welcome to html-generator!";
    let config = HtmlConfig::default();
    let html = generate_html(markdown, &config)?;

    println!("Generated HTML: \n{}", html);

    Ok(())
}

/// Demonstrates the use of accessibility functions.
fn accessibility_example() -> Result<()> {
    println!("\n🦀 Accessibility Example");
    println!("---------------------------------------------");

    let html = "<button>Click me</button>";

    // Map the error from `add_aria_attributes` to `HtmlError::Error`
    let updated_html =
        add_aria_attributes(html, None).map_err(|e| {
            HtmlError::accessibility(
                ErrorKind::MissingAriaAttributes,
                e.to_string(),
                None,
            )
        })?;

    println!("Updated HTML with ARIA attributes: \n{}", updated_html);
    Ok(())
}

/// Demonstrates performance optimization by minifying HTML and asynchronous generation.
async fn performance_optimization_example() -> Result<()> {
    println!("\n🦀 Performance Optimization Example");
    println!("---------------------------------------------");

    let markdown = "# Performance matters!";
    let html = async_generate_html(markdown).await?;
    println!("Generated HTML: \n{}", html);

    Ok(())
}

/// Demonstrates SEO optimization by generating meta tags and structured data.
fn seo_optimization_example() -> Result<()> {
    println!("\n🦀 SEO Optimization Example");
    println!("---------------------------------------------");

    let html = "<h1>Example Article</h1><p>This is an example article for SEO optimization.</p>";

    // Use a closure to convert the error type to HtmlError::SeoError, which expects a String
    let meta_tags = generate_meta_tags(html)
        .map_err(|e| HtmlError::MinificationError(e.to_string()))?;
    let structured_data = generate_structured_data(html, None)
        .map_err(|e| HtmlError::MinificationError(e.to_string()))?;

    println!("Generated Meta Tags: \n{}", meta_tags);
    println!("Generated Structured Data: \n{}", structured_data);

    Ok(())
}

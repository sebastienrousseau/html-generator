//! # Basic Example: HTML Generator
//!
//! This example demonstrates the fundamental functionality of the `html-generator` library,
//! converting simple Markdown input into optimised HTML output using the default configuration.
//!
//! ## Features Highlighted
//! - Basic Markdown to HTML conversion
//! - Display of generated HTML output

use html_generator::{generate_html, HtmlConfig};

/// Entry point for the basic example of HTML Generator.
///
/// This example demonstrates the transformation of Markdown into HTML using the default
/// configuration provided by the library.
///
/// # Errors
/// Returns an error if the Markdown to HTML conversion fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Welcome to the Basic HTML Generator Example!");
    println!("---------------------------------------------");

    // Define simple Markdown content
    let markdown = "# Welcome to HTML Generator\n\nEffortlessly convert Markdown into HTML.";

    // Use the default HTML configuration
    let config = HtmlConfig::default();

    // Generate HTML from Markdown
    let result = generate_html(markdown, &config);

    match result {
        Ok(html) => {
            println!("    ✅ Successfully generated HTML:\n{}", html);
        }
        Err(e) => {
            println!("    ❌ Failed to generate HTML: {}", e);
        }
    }

    println!("\n🎉 Basic HTML generation completed!");

    Ok(())
}

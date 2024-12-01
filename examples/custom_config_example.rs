// src/examples/custom_config_example.rs
#![allow(missing_docs)]

use html_generator::{generate_html, HtmlConfig, Result as HtmlResult};

/// Demonstrates the use of a custom configuration for HTML generation.
fn main() -> HtmlResult<()> {
    println!(
        "
🧪 Custom Configuration Example
"
    );
    println!("---------------------------------------------");

    // Markdown content
    let markdown = r#"# Custom Configuration
This demonstrates a custom configuration for HTML generation."#;

    // Customise the HTML configuration
    let config = HtmlConfig::builder()
        .with_language("en-GB")
        .with_syntax_highlighting(true, Some("monokai".to_string()))
        .build()?;

    // Generate HTML with custom configuration
    let html = generate_html(markdown, &config)?;
    println!(
        "    ✅ Generated HTML with custom configuration:
{}",
        html
    );

    println!(
        "
🎉 Custom configuration example completed successfully!"
    );

    Ok(())
}

// src/examples/generator_example.rs
#![allow(missing_docs)]

use html_generator::{
    generator::generate_html,
    generator::markdown_to_html_with_extensions, HtmlConfig,
    Result as HtmlResult,
};

/// Entry point for the html-generator HTML generation examples.
///
/// This function runs various examples demonstrating the process of converting Markdown
/// to HTML using different configurations and error handling scenarios.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 html-generator Markdown to HTML Examples\n");

    generate_html_basic_example()?;
    markdown_to_html_with_extensions_example()?;
    empty_markdown_example()?;
    invalid_markdown_example()?;
    complex_markdown_example()?;
    markdown_conversion_error_example()?;

    println!(
        "\n🎉 All HTML generation examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates basic Markdown to HTML generation.
fn generate_html_basic_example() -> HtmlResult<()> {
    println!("🦀 Basic HTML Generation Example");
    println!("---------------------------------------------");

    let markdown = "# Hello, world!\nThis is a test.";
    let config = HtmlConfig::default();
    let result = generate_html(markdown, &config);

    match result {
        Ok(html) => {
            println!("    ✅ Successfully generated HTML: \n{}", html);
        }
        Err(e) => {
            println!("    ❌ Failed to generate HTML: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates Markdown to HTML generation with extensions.
fn markdown_to_html_with_extensions_example() -> HtmlResult<()> {
    println!("\n🦀 Markdown to HTML with Extensions Example");
    println!("---------------------------------------------");

    let markdown = r#"
~~strikethrough~~
| Header 1 | Header 2 |
| -------- | -------- |
| Row 1    | Row 2    |
"#;
    let result = markdown_to_html_with_extensions(markdown);

    match result {
        Ok(html) => {
            println!("    ✅ Successfully generated HTML with extensions: \n{}", html);
        }
        Err(e) => {
            println!(
                "    ❌ Failed to generate HTML with extensions: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates conversion of empty Markdown.
fn empty_markdown_example() -> HtmlResult<()> {
    println!("\n🦀 Empty Markdown Example");
    println!("---------------------------------------------");

    let markdown = "";
    let config = HtmlConfig::default();
    let result = generate_html(markdown, &config);

    match result {
        Ok(html) => {
            println!("    ✅ Successfully handled empty Markdown. Result: \n{}", html);
        }
        Err(e) => {
            println!("    ❌ Failed to handle empty Markdown: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates handling of invalid Markdown.
fn invalid_markdown_example() -> HtmlResult<()> {
    println!("\n🦀 Invalid Markdown Example");
    println!("---------------------------------------------");

    let markdown = "# Unclosed header\nSome **unclosed bold";
    let config = HtmlConfig::default();
    let result = generate_html(markdown, &config);

    match result {
        Ok(html) => {
            println!("    ✅ Successfully generated HTML from invalid Markdown: \n{}", html);
        }
        Err(e) => {
            println!("    ❌ Failed to handle invalid Markdown: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates conversion of complex Markdown content.
fn complex_markdown_example() -> HtmlResult<()> {
    println!("\n🦀 Complex Markdown Example");
    println!("---------------------------------------------");

    let markdown = r#"
# Header

## Subheader

Some `inline code` and a [link](https://example.com).

```rust
fn main() {
    println!("Hello, world!");
}
```

1. First item
2. Second item
"#;
    let config = HtmlConfig::default();
    let result = generate_html(markdown, &config);

    match result {
        Ok(html) => {
            println!(
                "    ✅ Successfully generated complex HTML: \n{}",
                html
            );
        }
        Err(e) => {
            println!("    ❌ Failed to handle complex Markdown: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates handling of Markdown conversion errors.
fn markdown_conversion_error_example() -> HtmlResult<()> {
    println!("\n🦀 Markdown Conversion Error Example");
    println!("---------------------------------------------");

    let markdown = "# Invalid Markdown with an unhandled case";

    // Simulate a markdown conversion failure
    let result = markdown_to_html_with_extensions(markdown);

    match result {
        Ok(_) => {
            println!("    ❌ Unexpected success in converting invalid Markdown");
        }
        Err(e) => {
            println!("    ✅ Successfully caught Markdown Conversion Error: {}", e);
        }
    }

    Ok(())
}

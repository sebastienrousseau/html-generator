// src/examples/performance_example.rs

#![allow(missing_docs)]

use html_generator::performance::{
    async_generate_html, generate_html, minify_html,
};
use html_generator::HtmlError;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), HtmlError> {
    println!("\n🧪 html-generator Performance Examples\n");

    minify_html_example()?;
    async_generate_html_example().await?;
    generate_html_example()?;

    println!("\n🎉 All performance examples completed successfully!");

    Ok(())
}

/// Demonstrates the use of the `minify_html` function.
fn minify_html_example() -> Result<(), HtmlError> {
    println!("🦀 Minify HTML Example");
    println!("---------------------------------------------");

    let path = Path::new("index.html");

    // Attempt to minify the HTML file
    match minify_html(path) {
        Ok(minified_html) => {
            println!("Minified HTML: \n{}", minified_html);
        }
        Err(e) => {
            println!("Failed to minify HTML: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates the asynchronous generation of HTML from Markdown.
async fn async_generate_html_example() -> Result<(), HtmlError> {
    println!("\n🦀 Async Generate HTML Example");
    println!("---------------------------------------------");

    let markdown = "# Hello\n\nThis is an async test.";
    match async_generate_html(markdown).await {
        Ok(html) => println!("Generated HTML: \n{}", html),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

/// Demonstrates the synchronous generation of HTML from Markdown.
fn generate_html_example() -> Result<(), HtmlError> {
    println!("\n🦀 Generate HTML Example");
    println!("---------------------------------------------");

    let markdown = "# Hello\n\nThis is a test.";
    match generate_html(markdown) {
        Ok(html) => println!("Generated HTML: \n{}", html),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

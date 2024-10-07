// src/examples/utils_example.rs

#![allow(missing_docs)]

use html_generator::utils::{
    extract_front_matter, format_header_with_id_class,
    generate_table_of_contents,
};
use html_generator::HtmlError;

fn main() -> Result<(), HtmlError> {
    println!("\n🧪 html-generator Utils Examples\n");

    extract_front_matter_example()?;
    format_header_with_id_class_example()?;
    generate_table_of_contents_example()?;

    println!("\n🎉 All utils examples completed successfully!");

    Ok(())
}

/// Demonstrates extracting front matter from Markdown content.
fn extract_front_matter_example() -> Result<(), HtmlError> {
    println!("🦀 Extract Front Matter Example");
    println!("---------------------------------------------");

    let content =
        "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
    match extract_front_matter(content) {
        Ok(remaining_content) => {
            println!("Remaining Content: \n{}", remaining_content);
        }
        Err(e) => {
            println!("Failed to extract front matter: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates formatting a header with an ID and class.
fn format_header_with_id_class_example() -> Result<(), HtmlError> {
    println!("\n🦀 Format Header with ID and Class Example");
    println!("---------------------------------------------");

    let header = "<h2>Hello, World!</h2>";
    match format_header_with_id_class(header, None, None) {
        Ok(formatted_header) => {
            println!("Formatted Header: \n{}", formatted_header);
        }
        Err(e) => {
            println!("Failed to format header: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates generating a table of contents from HTML content.
fn generate_table_of_contents_example() -> Result<(), HtmlError> {
    println!("\n🦀 Generate Table of Contents Example");
    println!("---------------------------------------------");

    let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p><h3>Sub-subtitle</h3>";
    match generate_table_of_contents(html) {
        Ok(toc) => {
            println!("Generated Table of Contents: \n{}", toc);
        }
        Err(e) => {
            println!("Failed to generate table of contents: {}", e);
        }
    }

    Ok(())
}

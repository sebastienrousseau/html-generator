// src/examples/seo_example.rs

#![allow(missing_docs)]

use html_generator::seo::{
    generate_meta_tags, generate_structured_data,
};
use html_generator::HtmlError;

fn main() -> Result<(), HtmlError> {
    println!("\n🧪 html-generator SEO Examples\n");

    generate_meta_tags_example()?;
    generate_structured_data_example()?;

    println!("\n🎉 All SEO examples completed successfully!");

    Ok(())
}

/// Demonstrates the generation of meta tags for SEO purposes.
fn generate_meta_tags_example() -> Result<(), HtmlError> {
    println!("🦀 Generate Meta Tags Example");
    println!("---------------------------------------------");

    let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
    match generate_meta_tags(html) {
        Ok(meta_tags) => {
            println!("Generated Meta Tags: \n{}", meta_tags);
        }
        Err(e) => {
            println!("Failed to generate meta tags: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates the generation of structured data (JSON-LD) for SEO purposes.
fn generate_structured_data_example() -> Result<(), HtmlError> {
    println!("\n🦀 Generate Structured Data Example");
    println!("---------------------------------------------");

    let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
    match generate_structured_data(html) {
        Ok(structured_data) => {
            println!(
                "Generated Structured Data: \n{}",
                structured_data
            );
        }
        Err(e) => {
            println!("Failed to generate structured data: {}", e);
        }
    }

    Ok(())
}

// src/examples/accessibility_example.rs
#![allow(missing_docs)]

use html_generator::{
    accessibility::AccessibilityError,
    accessibility::{add_aria_attributes, validate_wcag},
};

/// Entry point for the html-generator accessibility handling examples.
///
/// This function runs various examples demonstrating error creation, conversion,
/// and handling for different scenarios in the html-generator library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 html-generator Accessibility Examples\n");

    aria_attribute_error_example()?;
    wcag_validation_error_example()?;
    html_processing_error_example()?;
    html_too_large_error_example()?;
    malformed_html_error_example()?;

    println!("\n🎉 All accessibility examples completed successfully!");

    Ok(())
}

/// Demonstrates handling of invalid ARIA attribute errors.
fn aria_attribute_error_example() -> Result<(), AccessibilityError> {
    println!("🦀 Invalid ARIA Attribute Example");
    println!("---------------------------------------------");

    let invalid_html =
        r#"<div aria-invalid="unsupported_value">Content</div>"#;
    let result = add_aria_attributes(invalid_html);

    match result {
        Ok(_) => {
            println!(
                "    ❌ Unexpected success in adding ARIA attributes"
            )
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught Invalid ARIA Attribute Error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of WCAG validation errors.
fn wcag_validation_error_example() -> Result<(), AccessibilityError> {
    println!("\n🦀 WCAG Validation Error Example");
    println!("---------------------------------------------");

    let invalid_html = r#"<img src="image.jpg">"#; // Missing alt text
    match validate_wcag(invalid_html) {
        Ok(_) => {
            println!(
                "    ❌ Unexpected success in passing WCAG validation"
            )
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught WCAG Validation Error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of HTML processing errors.
fn html_processing_error_example() -> Result<(), AccessibilityError> {
    println!("\n🦀 HTML Processing Error Example");
    println!("---------------------------------------------");

    let malformed_html = "<div><button>Unclosed button";
    match add_aria_attributes(malformed_html) {
        Ok(_) => {
            println!("    ❌ Unexpected success in processing malformed HTML")
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught HTML Processing Error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of HTML too large errors.
fn html_too_large_error_example() -> Result<(), AccessibilityError> {
    println!("\n🦀 HTML Too Large Error Example");
    println!("---------------------------------------------");

    let large_html = "a".repeat(1_000_001); // Exceeds MAX_HTML_SIZE
    match add_aria_attributes(&large_html) {
        Ok(_) => {
            println!(
                "    ❌ Unexpected success in processing large HTML"
            )
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught HTML Too Large Error: {}",
                e
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of malformed HTML errors.
fn malformed_html_error_example() -> Result<(), AccessibilityError> {
    println!("\n🦀 Malformed HTML Error Example");
    println!("---------------------------------------------");

    let malformed_html = "<div><span>Unclosed span";
    match add_aria_attributes(malformed_html) {
        Ok(_) => {
            println!("    ❌ Unexpected success in processing malformed HTML")
        }
        Err(e) => {
            println!(
                "    ✅ Successfully caught Malformed HTML Error: {}",
                e
            );
        }
    }

    Ok(())
}

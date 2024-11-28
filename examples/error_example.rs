// src/examples/error_example.rs
#![allow(missing_docs)]

use html_generator::error::{ErrorKind, HtmlError, SeoErrorKind};

/// Entry point for the html-generator error handling examples.
///
/// This function runs various examples demonstrating error creation, conversion,
/// and handling for different scenarios in the html-generator library.
///
/// # Errors
///
/// Returns an error if any of the example functions fail.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 html-generator Error Handling Examples\n");

    regex_error_example()?;
    front_matter_error_example()?;
    header_formatting_error_example()?;
    io_error_example()?;
    selector_parse_error_example()?;
    minification_error_example()?;
    markdown_conversion_error_example()?;
    seo_optimization_error_example()?;
    accessibility_error_example()?;
    missing_html_element_error_example()?;
    invalid_structured_data_example()?;
    invalid_input_example()?;
    input_too_large_error_example()?;
    utf8_conversion_error_example()?;
    validation_error_example()?;
    unexpected_error_example()?;

    println!(
        "\n🎉 All error handling examples completed successfully!"
    );

    Ok(())
}

/// Demonstrates handling of regex compilation errors.
fn regex_error_example() -> Result<(), HtmlError> {
    println!("🦀 Regex Compilation Error Example");
    println!("---------------------------------------------");

    let invalid_regex = "(unclosed group";
    let result = regex::Regex::new(invalid_regex);

    match result {
        Ok(_) => {
            println!("    ❌ Unexpected success in compiling regex")
        }
        Err(e) => {
            let error = HtmlError::RegexCompilationError(e);
            println!(
                "    ✅ Successfully caught Regex Error: {}",
                error
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of front matter extraction errors.
fn front_matter_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Front Matter Extraction Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::FrontMatterExtractionError(
        "Failed to extract front matter".to_string(),
    );
    println!("    ✅ Created Front Matter Error: {}", error);

    Ok(())
}

/// Demonstrates handling of header formatting errors.
fn header_formatting_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Header Formatting Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::HeaderFormattingError(
        "Header is invalid".to_string(),
    );
    println!("    ✅ Created Header Formatting Error: {}", error);

    Ok(())
}

/// Demonstrates handling of IO errors.
fn io_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 IO Error Example");
    println!("---------------------------------------------");

    let io_error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found",
    );
    let error = HtmlError::Io(io_error);
    println!("    ✅ Created IO Error: {}", error);

    Ok(())
}

/// Demonstrates handling of selector parse errors.
fn selector_parse_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Selector Parse Error Example");
    println!("---------------------------------------------");

    let selector = "div[invalid]";
    let result = scraper::Selector::parse(selector);

    match result {
        Ok(_) => {
            println!("    ❌ Unexpected success in parsing selector")
        }
        Err(e) => {
            let error = HtmlError::SelectorParseError(
                selector.to_string(),
                e.to_string(),
            );
            println!(
                "    ✅ Successfully caught Selector Parse Error: {}",
                error
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of HTML minification errors.
fn minification_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Minification Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::MinificationError(
        "Failed to minify HTML".to_string(),
    );
    println!("    ✅ Created Minification Error: {}", error);

    Ok(())
}

/// Demonstrates handling of Markdown conversion errors.
fn markdown_conversion_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Markdown Conversion Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::markdown_conversion(
        "Failed to convert markdown".to_string(),
        None,
    );
    println!("    ✅ Created Markdown Conversion Error: {}", error);

    Ok(())
}

/// Demonstrates handling of SEO optimization errors.
fn seo_optimization_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 SEO Optimization Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::Seo {
        message: "SEO issue occurred".to_string(),
        element: Some("meta".to_string()),
        kind: SeoErrorKind::Other,
    };
    println!("    ✅ Created SEO Optimization Error: {}", error);

    Ok(())
}

/// Demonstrates handling of accessibility errors.
fn accessibility_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Accessibility Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::Accessibility {
        message: "Failed to add ARIA attributes".to_string(),
        kind: ErrorKind::Other,
        wcag_guideline: Some("1.1.1".to_string()),
    };
    println!("    ✅ Created Accessibility Error: {}", error);

    Ok(())
}

/// Demonstrates handling of missing HTML element errors.
fn missing_html_element_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Missing HTML Element Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::MissingHtmlElement("title".to_string());
    println!("    ✅ Created Missing HTML Element Error: {}", error);

    Ok(())
}

/// Demonstrates handling of invalid structured data errors.
fn invalid_structured_data_example() -> Result<(), HtmlError> {
    println!("\n🦀 Invalid Structured Data Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::InvalidStructuredData(
        "Invalid JSON-LD format".to_string(),
    );
    println!("    ✅ Created Invalid Structured Data Error: {}", error);

    Ok(())
}

/// Demonstrates handling of invalid input errors.
fn invalid_input_example() -> Result<(), HtmlError> {
    println!("\n🦀 Invalid Input Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::InvalidInput("Input not valid".to_string());
    println!("    ✅ Created Invalid Input Error: {}", error);

    Ok(())
}

/// Demonstrates handling of input too large errors.
fn input_too_large_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Input Too Large Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::InputTooLarge(1_024_001);
    println!("    ✅ Created Input Too Large Error: {}", error);

    Ok(())
}

/// Demonstrates handling of UTF-8 conversion errors.
fn utf8_conversion_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 UTF-8 Conversion Error Example");
    println!("---------------------------------------------");

    let invalid_utf8 = vec![0, 159, 146, 150];
    match String::from_utf8(invalid_utf8) {
        Ok(_) => {
            println!("    ❌ Unexpected success in UTF-8 conversion")
        }
        Err(e) => {
            let error = HtmlError::Utf8ConversionError(e);
            println!(
                "    ✅ Successfully caught UTF-8 Conversion Error: {}",
                error
            );
        }
    }

    Ok(())
}

/// Demonstrates handling of validation errors.
fn validation_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Validation Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::ValidationError(
        "Data does not meet schema".to_string(),
    );
    println!("    ✅ Created Validation Error: {}", error);

    Ok(())
}

/// Demonstrates handling of unexpected errors.
fn unexpected_error_example() -> Result<(), HtmlError> {
    println!("\n🦀 Unexpected Error Example");
    println!("---------------------------------------------");

    let error = HtmlError::UnexpectedError(
        "An unexpected issue occurred".to_string(),
    );
    println!("    ✅ Created Unexpected Error: {}", error);

    Ok(())
}

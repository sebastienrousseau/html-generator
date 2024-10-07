//! SEO-related functionality for HTML processing.
//!
//! This module provides functions for generating meta tags and structured data
//! to improve the Search Engine Optimization (SEO) of web pages.

use crate::error::{HtmlError, Result};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use scraper::{Html, Selector};
use std::borrow::Cow;

const MAX_HTML_SIZE: usize = 1_000_000; // 1MB

/// Escapes HTML entities in the given string using regex.
///
/// This function replaces the characters `&`, `<`, `>`, `"`, and `'`
/// with their respective HTML entity codes.
///
/// # Arguments
///
/// * `s` - The input string that needs HTML escaping.
///
/// # Returns
///
/// A `Cow<str>` that either holds a reference to the original string or an
/// escaped version if any replacements were made.
///
/// # Example
///
/// ```
/// use html_generator::seo::escape_html;
/// let input = "Hello & welcome to <Rust>!";
/// let escaped = escape_html(input);
/// assert_eq!(escaped, "Hello &amp; welcome to &lt;Rust&gt;!");
/// ```
pub fn escape_html(s: &str) -> Cow<str> {
    lazy_static! {
        // Precompiled regex for matching HTML special characters
        static ref HTML_ESCAPES: Regex = Regex::new(r#"[&<>"']"#).unwrap();
    }

    // Replace matched HTML special characters with their corresponding entities
    HTML_ESCAPES.replace_all(s, |caps: &Captures| match &caps[0] {
        "&" => "&amp;",
        "<" => "&lt;",
        ">" => "&gt;",
        "\"" => "&quot;",
        "'" => "&#x27;",
        _ => unreachable!(),
    })
}

/// Generates meta tags for SEO purposes.
///
/// This function parses the provided HTML, extracts relevant information,
/// and generates meta tags for title and description.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - A string containing the generated meta tags, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The HTML input is too large (> 1MB).
/// * The HTML selectors fail to parse.
/// * Required HTML elements (title, description) are missing.
///
/// # Examples
///
/// ```
/// use html_generator::seo::generate_meta_tags;
///
/// let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
/// let meta_tags = generate_meta_tags(html).unwrap();
/// assert!(meta_tags.contains(r#"<meta name="title" content="Test Page">"#));
/// assert!(meta_tags.contains(r#"<meta name="description" content="This is a test page.">"#));
/// ```
pub fn generate_meta_tags(html: &str) -> Result<String> {
    if html.len() > MAX_HTML_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let document = Html::parse_document(html);
    let mut meta_tags = String::with_capacity(200);

    let title = extract_title(&document)?;
    let description = extract_description(&document)?;
    let escaped_title = escape_html(&title);
    let escaped_description = escape_html(&description);

    meta_tags.push_str(&format!(
        r#"<meta name="title" content="{}">"#,
        escaped_title
    ));
    meta_tags.push_str(&format!(
        r#"<meta name="description" content="{}">"#,
        escaped_description
    ));
    meta_tags
        .push_str(r#"<meta property="og:type" content="website">"#);

    Ok(meta_tags)
}

/// Generates structured data (JSON-LD) for SEO purposes.
///
/// This function creates a JSON-LD script tag with basic webpage information
/// extracted from the provided HTML content.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - A string containing the generated JSON-LD script, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The HTML input is too large (> 1MB).
/// * The HTML selectors fail to parse.
/// * Required HTML elements (title, description) are missing.
///
/// # Examples
///
/// ```
/// use html_generator::seo::generate_structured_data;
///
/// let html = r#"<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>"#;
/// let structured_data = generate_structured_data(html).unwrap();
/// assert!(structured_data.contains(r#""@type": "WebPage""#));
/// assert!(structured_data.contains(r#""name": "Test Page""#));
/// assert!(structured_data.contains(r#""description": "This is a test page.""#));
/// ```
pub fn generate_structured_data(html: &str) -> Result<String> {
    if html.len() > MAX_HTML_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let document = Html::parse_document(html);

    let title = extract_title(&document)?;
    let description = extract_description(&document)?;

    let structured_data = format!(
        r#"<script type="application/ld+json">
        {{
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "{}",
            "description": "{}"
        }}
        </script>"#,
        escape_html(&title),
        escape_html(&description)
    );

    Ok(structured_data)
}

fn extract_title(document: &Html) -> Result<String> {
    let title_selector = Selector::parse("title").map_err(|e| {
        HtmlError::SelectorParseError(
            "title".to_string(),
            e.to_string(),
        )
    })?;

    // Extract the raw inner HTML without escaping
    document
        .select(&title_selector)
        .next()
        .map(|t| t.text().collect::<String>()) // Use .text() instead of .inner_html()
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("title".to_string())
        })
}

fn extract_description(document: &Html) -> Result<String> {
    let meta_description_selector =
        Selector::parse("meta[name='description']").map_err(|e| {
            HtmlError::SelectorParseError(
                "meta description".to_string(),
                e.to_string(),
            )
        })?;

    let p_selector = Selector::parse("p").map_err(|e| {
        HtmlError::SelectorParseError("p".to_string(), e.to_string())
    })?;

    // First, try to find a meta description
    if let Some(meta) =
        document.select(&meta_description_selector).next()
    {
        if let Some(content) = meta.value().attr("content") {
            return Ok(content.to_string()); // Use the raw content, no escaping here
        }
    }

    // If no meta description, fall back to the first paragraph
    document
        .select(&p_selector)
        .next()
        .map(|p| p.text().collect::<String>()) // Use .text() to get raw text
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("description".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_meta_tags() {
        let html = "<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>";
        let result = generate_meta_tags(html);
        assert!(result.is_ok());
        let meta_tags = result.unwrap();
        assert!(meta_tags
            .contains(r#"<meta name="title" content="Test Page">"#));
        assert!(meta_tags.contains(r#"<meta name="description" content="This is a test page.">"#));
    }

    #[test]
    fn test_generate_structured_data() {
        let html = "<html><head><title>Test Page</title></head><body><p>This is a test page.</p></body></html>";
        let result = generate_structured_data(html);
        assert!(result.is_ok());
        let structured_data = result.unwrap();
        assert!(structured_data.contains(r#""@type": "WebPage""#));
        assert!(structured_data.contains(r#""name": "Test Page""#));
        assert!(structured_data
            .contains(r#""description": "This is a test page.""#));
    }

    #[test]
    fn test_generate_meta_tags_missing_title() {
        let html =
            "<html><body><p>This is a test page.</p></body></html>";
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_structured_data_missing_description() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        let result = generate_structured_data(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_meta_tags_with_special_characters() {
        let html = r#"<html><head><title>Test & Page</title></head><body><p>This is a "test" page.</p></body></html>"#;
        let result = generate_meta_tags(html);
        assert!(result.is_ok());
        let meta_tags = result.unwrap();
        println!("Generated meta tags: {}", meta_tags); // Debug print
        assert!(meta_tags.contains(
            r#"<meta name="title" content="Test &amp; Page">"#
        ));
        assert!(meta_tags.contains(r#"<meta name="description" content="This is a &quot;test&quot; page.">"#));
    }

    #[test]
    fn test_generate_meta_tags_with_meta_description() {
        let html = r#"<html><head><title>Test Page</title><meta name="description" content="Meta description"></head><body><p>This is a test page.</p></body></html>"#;
        let result = generate_meta_tags(html);
        assert!(result.is_ok());
        let meta_tags = result.unwrap();
        assert!(meta_tags.contains(
            r#"<meta name="description" content="Meta description">"#
        ));
    }

    #[test]
    fn test_input_too_large() {
        let large_html = "a".repeat(MAX_HTML_SIZE + 1);
        assert!(matches!(
            generate_meta_tags(&large_html),
            Err(HtmlError::InputTooLarge(_))
        ));
        assert!(matches!(
            generate_structured_data(&large_html),
            Err(HtmlError::InputTooLarge(_))
        ));
    }

    #[test]
    fn test_escape_html() {
        let input = "This is <a test> & a 'quote' \"string\"";
        let result = escape_html(input);
        assert_eq!(result, "This is &lt;a test&gt; &amp; a &#x27;quote&#x27; &quot;string&quot;");
    }

    #[test]
    fn test_escape_html_no_special_characters() {
        let input = "This is just a normal string.";
        let result = escape_html(input);
        assert_eq!(result, "This is just a normal string.");
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Title</title></head><body></body></html>";
        let document = Html::parse_document(html);
        let result = extract_title(&document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test Title");
    }

    #[test]
    fn test_extract_title_missing() {
        let html = "<html><head></head><body></body></html>";
        let document = Html::parse_document(html);
        let result = extract_title(&document);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_extract_description() {
        let html = "<html><head><meta name='description' content='This is a test description'></head><body></body></html>";
        let document = Html::parse_document(html);
        let result = extract_description(&document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a test description");
    }

    #[test]
    fn test_extract_description_fallback_to_paragraph() {
        let html = "<html><body><p>This is a fallback description.</p></body></html>";
        let document = Html::parse_document(html);
        let result = extract_description(&document);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a fallback description.");
    }

    #[test]
    fn test_extract_description_missing() {
        let html = "<html><body></body></html>";
        let document = Html::parse_document(html);
        let result = extract_description(&document);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_meta_tags_empty_html() {
        let html = "";
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_structured_data_empty_html() {
        let html = "";
        let result = generate_structured_data(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_meta_tags_only_meta_description() {
        let html = r#"<html><head><meta name="description" content="Meta description"></head><body></body></html>"#;
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_meta_tags_only_title() {
        let html = r#"<html><head><title>Test Title</title></head><body></body></html>"#;
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_structured_data_with_special_characters() {
        let html = r#"<html><head><title>Test & Page</title></head><body><p>This is a "test" page.</p></body></html>"#;
        let result = generate_structured_data(html);
        assert!(result.is_ok());
        let structured_data = result.unwrap();
        assert!(
            structured_data.contains(r#""name": "Test &amp; Page""#)
        );
        assert!(structured_data.contains(
            r#""description": "This is a &quot;test&quot; page.""#
        ));
    }

    #[test]
    fn test_generate_meta_tags_malformed_html() {
        let html = r#"<html><head><title>Test Page"#;
        let result = generate_meta_tags(html);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HtmlError::MissingHtmlElement(_)
        ));
    }

    #[test]
    fn test_generate_meta_tags_multiple_titles() {
        let html = r#"
    <html>
        <head>
            <title>First Title</title>
            <title>Second Title</title>
        </head>
        <body><p>This is a test page.</p></body>
    </html>
    "#;
        let result = generate_meta_tags(html);
        assert!(result.is_ok());
        let meta_tags = result.unwrap();
        assert!(meta_tags
            .contains(r#"<meta name="title" content="First Title">"#));
    }
}

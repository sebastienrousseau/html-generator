// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Utility functions for HTML and Markdown processing.
//!
//! This module provides various utility functions for tasks such as
//! extracting front matter from Markdown content and formatting HTML headers.

use crate::error::{HtmlError, Result};
use crate::seo::escape_html;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;
use serde_json::Value;
use std::collections::HashMap;

static FRONT_MATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ms)^---\s*\n(.*?)\n---\s*\n")
        .expect("static FRONT_MATTER_REGEX must compile")
});

static TOML_FRONT_MATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ms)^\+\+\+\s*\n(.*?)\n\+\+\+\s*\n")
        .expect("static TOML_FRONT_MATTER_REGEX must compile")
});

static HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<(h[1-6])(?:\s[^>]*)?>(.+?)</h[1-6]>")
        .expect("static HEADER_REGEX must compile")
});

/// Maximum allowed input size (in bytes) to prevent DOS attacks
const MAX_INPUT_SIZE: usize = 1_000_000; // 1 MB

/// Extracts front matter from Markdown content.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to process.
///
/// # Returns
///
/// * `Result<String>` - The content with front matter removed, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The input is empty or exceeds the maximum allowed size.
/// * The front matter is invalidly formatted.
///
/// # Examples
///
/// ```
/// use html_generator::utils::extract_front_matter;
///
/// let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
/// let result = extract_front_matter(content).unwrap();
/// assert_eq!(result, "# Hello, world!\n\nThis is a test.");
/// ```
pub fn extract_front_matter(content: &str) -> Result<String> {
    if content.is_empty() {
        return Err(HtmlError::InvalidInput("Empty input".to_string()));
    }
    if content.len() > MAX_INPUT_SIZE {
        return Err(HtmlError::InputTooLarge(content.len()));
    }

    if content.starts_with("---") {
        if let Some(captures) = FRONT_MATTER_REGEX.captures(content) {
            // Group 1 is the mandatory `(.*?)` — if `captures`
            // matched, the group is always present.
            let front_matter = captures
                .get(1)
                .expect("front-matter regex group 1 is mandatory")
                .as_str();

            for line in front_matter.lines() {
                let trimmed = line.trim();
                // Skip blank lines and YAML comments
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if !trimmed.contains(':') {
                    return Err(HtmlError::InvalidFrontMatterFormat(
                        format!(
                            "Invalid line in front matter: {}",
                            line
                        ),
                    ));
                }
            }

            let remaining_content =
                &content[captures.get(0).map_or(0, |m| m.end())..];
            Ok(remaining_content.trim().to_string())
        } else {
            Err(HtmlError::InvalidFrontMatterFormat(
                "Invalid front matter format".to_string(),
            ))
        }
    } else {
        Ok(content.to_string())
    }
}

/// Extracts and parses front matter from content, supporting YAML (`---`),
/// TOML (`+++`), and JSON (`{`...`}`) delimiters.
///
/// Returns a tuple of (parsed front matter as JSON Value, remaining content).
/// If no front matter is found, returns (`Value::Null`, original content).
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to process.
///
/// # Returns
///
/// * `Result<(Value, String)>` - Parsed front matter and remaining content, or an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The input is empty or exceeds the maximum allowed size.
/// * The front matter is invalidly formatted or cannot be parsed.
///
/// # Examples
///
/// ```
/// use html_generator::utils::extract_front_matter_data;
/// use serde_json::json;
///
/// let content = "---\ntitle: My Page\n---\n# Hello, world!";
/// let (data, rest) = extract_front_matter_data(content).unwrap();
/// assert_eq!(data["title"], "My Page");
/// assert_eq!(rest, "# Hello, world!");
/// ```
pub fn extract_front_matter_data(
    content: &str,
) -> Result<(Value, String)> {
    if content.is_empty() {
        return Err(HtmlError::InvalidInput("Empty input".to_string()));
    }
    if content.len() > MAX_INPUT_SIZE {
        return Err(HtmlError::InputTooLarge(content.len()));
    }

    // YAML front matter (---)
    if content.starts_with("---") {
        if let Some(captures) = FRONT_MATTER_REGEX.captures(content) {
            let raw = captures
                .get(1)
                .expect("front-matter regex group 1 is mandatory")
                .as_str();

            let map = parse_yaml_to_map(raw)?;
            let remaining =
                &content[captures.get(0).map_or(0, |m| m.end())..];
            return Ok((
                Value::Object(map),
                remaining.trim().to_string(),
            ));
        }
        return Err(HtmlError::InvalidFrontMatterFormat(
            "Invalid YAML front matter format".to_string(),
        ));
    }

    // TOML front matter (+++)
    if content.starts_with("+++") {
        if let Some(captures) =
            TOML_FRONT_MATTER_REGEX.captures(content)
        {
            let raw = captures
                .get(1)
                .expect("TOML front-matter regex group 1 is mandatory")
                .as_str();

            let map = parse_toml_to_map(raw)?;
            let remaining =
                &content[captures.get(0).map_or(0, |m| m.end())..];
            return Ok((
                Value::Object(map),
                remaining.trim().to_string(),
            ));
        }
        return Err(HtmlError::InvalidFrontMatterFormat(
            "Invalid TOML front matter format".to_string(),
        ));
    }

    // JSON front matter ({...})
    if content.starts_with('{') {
        if let Some(end) = find_matching_brace(content) {
            let json_str = &content[..=end];
            let value: Value =
                serde_json::from_str(json_str).map_err(|e| {
                    HtmlError::InvalidFrontMatterFormat(format!(
                        "Invalid JSON front matter: {e}"
                    ))
                })?;
            let remaining = content[end + 1..].trim_start();
            return Ok((value, remaining.to_string()));
        }
        return Err(HtmlError::InvalidFrontMatterFormat(
            "Unmatched opening brace in JSON front matter".to_string(),
        ));
    }

    // No front matter found
    Ok((Value::Null, content.to_string()))
}

/// Parses a YAML front matter block into a serde_json Map using
/// the vendored `yaml_safe` crate (a pure-Rust, `forbid(unsafe_code)`
/// implementation; see `crates/yaml_safe/src/lib.rs` for vendor
/// rationale).
fn parse_yaml_to_map(
    raw: &str,
) -> Result<serde_json::Map<String, Value>> {
    let value: Value = yaml_safe::from_str(raw).map_err(|e| {
        HtmlError::InvalidFrontMatterFormat(format!(
            "Invalid YAML front matter: {e}"
        ))
    })?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(HtmlError::InvalidFrontMatterFormat(
            "YAML front matter must be a mapping".to_string(),
        )),
    }
}

/// Parses a TOML front matter block into a serde_json Map using the `toml` crate.
fn parse_toml_to_map(
    raw: &str,
) -> Result<serde_json::Map<String, Value>> {
    let toml_value: toml::Value = toml::from_str(raw).map_err(|e| {
        HtmlError::InvalidFrontMatterFormat(format!(
            "Invalid TOML front matter: {e}"
        ))
    })?;
    // Convert toml::Value -> serde_json::Value via serialization round-trip
    let json_value: Value =
        serde_json::to_value(toml_value).map_err(|e| {
            HtmlError::InvalidFrontMatterFormat(format!(
                "Failed to convert TOML to JSON: {e}"
            ))
        })?;
    // `toml::from_str::<toml::Value>` only succeeds on a table
    // document, so the round-trip through serde_json yields an
    // `Object` every time.
    match json_value {
        Value::Object(map) => Ok(map),
        _ => Err(HtmlError::InvalidFrontMatterFormat(
            "TOML document must parse as a table".to_string(),
        )),
    }
}

/// Finds the index of the closing `}` that matches the opening `{` at index 0.
fn find_matching_brace(content: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut prev_backslash = false;

    for (i, ch) in content.char_indices() {
        if in_string {
            if ch == '\\' && !prev_backslash {
                prev_backslash = true;
                continue;
            }
            if ch == '"' && !prev_backslash {
                in_string = false;
            }
            prev_backslash = false;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        prev_backslash = false;
    }
    None
}

/// Formats a header with an ID and class.
///
/// # Arguments
///
/// * `header` - A string slice that holds the HTML header to process.
/// * `id_generator` - An optional function that generates the ID from the header content.
/// * `class_generator` - An optional function that generates the class from the header content.
///
/// # Returns
///
/// * `Result<String>` - The formatted HTML header, or an error.
///
/// # Examples
///
/// ```
/// use html_generator::utils::format_header_with_id_class;
///
/// let header = "<h2>Hello, World!</h2>";
/// let result = format_header_with_id_class(header, None, None).unwrap();
/// assert_eq!(result, "<h2 id=\"hello-world\" class=\"hello-world\">Hello, World!</h2>");
/// ```
pub fn format_header_with_id_class(
    header: &str,
    id_generator: Option<fn(&str) -> String>,
    class_generator: Option<fn(&str) -> String>,
) -> Result<String> {
    let captures = HEADER_REGEX.captures(header).ok_or_else(|| {
        HtmlError::InvalidHeaderFormat(
            "Invalid header format".to_string(),
        )
    })?;

    // Groups 1 and 2 are both mandatory (`(h[1-6])` and `(.+?)`); if
    // `captures` returned Some, they are always present.
    let tag = captures
        .get(1)
        .expect("header regex group 1 is mandatory")
        .as_str();

    let text_content = captures
        .get(2)
        .expect("header regex group 2 is mandatory")
        .as_str();

    let id = id_generator.map_or_else(
        || generate_id(text_content),
        |generator| generator(text_content),
    );
    let class = class_generator.map_or_else(
        || generate_id(text_content),
        |generator| generator(text_content),
    );

    Ok(format!(
        r#"<{} id="{}" class="{}">{}</{}>"#,
        tag, id, class, text_content, tag
    ))
}

/// Generates a table of contents from HTML content.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content to process.
///
/// # Returns
///
/// * `Result<String>` - The generated table of contents as an HTML string, or an error.
///
/// # Examples
///
/// ```
/// use html_generator::utils::generate_table_of_contents;
///
/// let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p>";
/// let result = generate_table_of_contents(html).unwrap();
/// assert_eq!(result, r#"<ul><li class="toc-h1"><a href="\#title">Title</a></li><li class="toc-h2"><a href="\#subtitle">Subtitle</a></li></ul>"#);
/// ```
pub fn generate_table_of_contents(html: &str) -> Result<String> {
    if html.is_empty() {
        return Err(HtmlError::InvalidInput("Empty input".to_string()));
    }
    if html.len() > MAX_INPUT_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let mut toc = String::new();
    toc.push_str("<ul>");

    for captures in HEADER_REGEX.captures_iter(html) {
        if let Some(tag) = captures.get(1) {
            let content = captures.get(2).map_or("", |m| m.as_str());
            let id = generate_id(content);
            toc.push_str(&format!(
                r#"<li class="toc-{}"><a href="\#{}">{}</a></li>"#,
                tag.as_str(),
                id,
                escape_html(content)
            ));
        }
    }

    toc.push_str("</ul>");
    Ok(toc)
}

/// Check if an ARIA role is valid for a given element.
///
/// # Arguments
///
/// * `role` - The ARIA role to validate.
/// * `element` - The HTML element to validate.
///
/// # Returns
///
/// * `bool` - Whether the role is valid for the element.
pub fn is_valid_aria_role(role: &str, element: &ElementRef) -> bool {
    static VALID_ROLES: Lazy<HashMap<&'static str, Vec<&'static str>>> =
        Lazy::new(|| {
            let mut roles = HashMap::new();
            let _ =
                roles.insert("a", vec!["link", "button", "menuitem"]);
            let _ = roles.insert("button", vec!["button"]);
            let _ =
                roles.insert("div", vec!["alert", "tooltip", "dialog"]);
            let _ = roles.insert(
                "input",
                vec!["textbox", "radio", "checkbox", "searchbox"],
            );
            roles
        });

    if let Some(valid_roles) = VALID_ROLES.get(element.value().name()) {
        valid_roles.contains(&role)
    } else {
        false
    }
}

/// Validates a language code.
///
/// # Arguments
///
/// * `lang` - The language code to validate.
///
/// # Returns
///
/// * `bool` - Whether the language code is valid.
pub fn is_valid_language_code(lang: &str) -> bool {
    let parts: Vec<&str> = lang.split('-').collect();
    if parts.is_empty() || parts[0].len() < 2 || parts[0].len() > 3 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_lowercase())
}

/// Generates a slug-like ID from the given content.
///
/// Walks the input once: alphanumerics pass through as lowercase, any
/// other character becomes a single `-` (duplicates collapsed), and
/// trailing/leading dashes are trimmed. Allocates exactly one `String`.
fn generate_id(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut last_dash = true;
    for ch in content.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        let _ = out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    /// Tests for `extract_front_matter` function.
    mod extract_front_matter_tests {
        use super::*;

        #[test]
        fn test_valid_front_matter() {
            let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
            let result = extract_front_matter(content);
            let extracted = result.expect("valid front matter");
            assert_eq!(extracted, "# Hello, world!\n\nThis is a test.");
        }

        #[test]
        fn test_no_front_matter() {
            let content = "# Hello, world!\n\nThis is a test without front matter.";
            let result = extract_front_matter(content);
            let extracted =
                result.expect("valid no-front-matter input");
            assert_eq!(extracted, content);
        }

        #[test]
        fn test_empty_input() {
            let content = "";
            let result = extract_front_matter(content);
            assert!(matches!(result, Err(HtmlError::InvalidInput(_))));
        }

        #[test]
        fn test_exceeding_max_input_size() {
            let content = "a".repeat(MAX_INPUT_SIZE + 1);
            let result = extract_front_matter(&content);
            assert!(matches!(result, Err(HtmlError::InputTooLarge(_))));
        }

        #[test]
        fn test_invalid_front_matter_format() {
            let content =
                "---\ntitle: value\ninvalid_line\n---\nContent";
            let result = extract_front_matter(content);
            assert!(matches!(
                result,
                Err(HtmlError::InvalidFrontMatterFormat(_))
            ));
        }

        #[test]
        fn test_valid_front_matter_with_extra_content() {
            let content = "---\ntitle: Page\n---\n\n# Title\n\nContent";
            let result = extract_front_matter(content);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "# Title\n\nContent");
        }

        #[test]
        fn test_extract_front_matter_with_mid_document_delimiter() {
            let content = "# Title\nContent\n---\nkey: value\n---";
            let result = extract_front_matter(content);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), content);
        }
    }

    /// Tests for `format_header_with_id_class` function.
    mod format_header_with_id_class_tests {
        use super::*;

        #[test]
        fn test_valid_header_default_generators() {
            let header = "<h2>Hello, World!</h2>";
            let result =
                format_header_with_id_class(header, None, None);
            let formatted = result.expect("valid header");
            assert_eq!(formatted, "<h2 id=\"hello-world\" class=\"hello-world\">Hello, World!</h2>");
        }

        #[test]
        fn test_custom_id_and_class_generators() {
            let header = "<h3>Test Header</h3>";
            fn id_gen(content: &str) -> String {
                format!(
                    "custom-{}",
                    content.to_lowercase().replace(' ', "-")
                )
            }
            fn class_gen(_: &str) -> String {
                "custom-class".to_string()
            }
            let result = format_header_with_id_class(
                header,
                Some(id_gen),
                Some(class_gen),
            );
            let formatted =
                result.expect("valid header with custom generators");
            assert_eq!(formatted, "<h3 id=\"custom-test-header\" class=\"custom-class\">Test Header</h3>");
        }

        #[test]
        fn test_invalid_header_format() {
            let header = "<p>Not a header</p>";
            let result =
                format_header_with_id_class(header, None, None);
            assert!(matches!(
                result,
                Err(HtmlError::InvalidHeaderFormat(_))
            ));
        }

        #[test]
        fn test_header_with_nested_tags() {
            let header = "<h2><span>Nested Header</span></h2>";
            let result =
                format_header_with_id_class(header, None, None);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<h2 id=\"span-nested-header-span\" class=\"span-nested-header-span\"><span>Nested Header</span></h2>"
            );
        }

        #[test]
        fn test_format_header_with_long_content() {
            let header = format!("<h1>{}</h1>", "a".repeat(300));
            let result =
                format_header_with_id_class(&header, None, None);
            assert!(result.is_ok());
        }

        #[test]
        fn test_header_with_special_characters() {
            let header = "<h3>Special & Header!</h3>";
            let result =
                format_header_with_id_class(header, None, None);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "<h3 id=\"special-header\" class=\"special-header\">Special & Header!</h3>"
            );
        }
    }

    /// Tests for `generate_table_of_contents` function.
    mod generate_table_of_contents_tests {
        use super::*;

        #[test]
        fn test_valid_html_with_headers() {
            let html = "<h1>Title</h1><h2>Subtitle</h2>";
            let result = generate_table_of_contents(html);
            let toc = result.expect("valid headers produce a TOC");
            assert_eq!(
                toc,
                r#"<ul><li class="toc-h1"><a href="\#title">Title</a></li><li class="toc-h2"><a href="\#subtitle">Subtitle</a></li></ul>"#
            );
        }

        #[test]
        fn test_html_without_headers() {
            let html = "<p>No headers here.</p>";
            let result = generate_table_of_contents(html);
            let toc =
                result.expect("no headers still yields an empty TOC");
            assert_eq!(toc, "<ul></ul>");
        }

        #[test]
        fn test_empty_html() {
            let html = "";
            let result = generate_table_of_contents(html);
            assert!(matches!(result, Err(HtmlError::InvalidInput(_))));
        }

        #[test]
        fn test_large_html_content() {
            let html = "<h1>Header</h1>".repeat(1000);
            let result = generate_table_of_contents(&html);
            assert!(result.is_ok());
        }

        #[test]
        fn test_generate_table_of_contents_with_malformed_html() {
            let html = "<h1>Title<h2>Subtitle";
            let result = generate_table_of_contents(html);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "<ul></ul>");
        }

        #[test]
        fn test_generate_table_of_contents_with_attributes() {
            let html = r#"<h1 class="header-class">Header</h1>"#;
            let result = generate_table_of_contents(html);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                r#"<ul><li class="toc-h1"><a href="\#header">Header</a></li></ul>"#
            );
        }
    }

    /// Tests for ARIA validation and utilities.
    mod aria_validation_tests {
        use super::*;

        #[test]
        fn test_valid_aria_role_for_button() {
            let html =
                Html::parse_fragment("<button role='button'></button>");
            let element = html
                .select(&scraper::Selector::parse("button").unwrap())
                .next()
                .unwrap();
            assert!(is_valid_aria_role("button", &element));
        }

        #[test]
        fn test_invalid_aria_role_for_button() {
            let html =
                Html::parse_fragment("<button role='link'></button>");
            let element = html
                .select(&scraper::Selector::parse("button").unwrap())
                .next()
                .unwrap();
            assert!(!is_valid_aria_role("link", &element));
        }

        #[test]
        fn test_missing_required_aria_properties() {
            let html =
                Html::parse_fragment(r#"<div role="slider"></div>"#);
            let element = html
                .select(&scraper::Selector::parse("div").unwrap())
                .next()
                .unwrap();
            let missing = crate::accessibility::utils::get_missing_required_aria_properties(&element);
            assert_eq!(
                missing.unwrap(),
                vec![
                    "aria-valuenow".to_string(),
                    "aria-valuemin".to_string(),
                    "aria-valuemax".to_string()
                ]
            );
        }

        #[test]
        fn test_get_missing_required_aria_properties_valid_role() {
            let html = Html::parse_fragment(
                r#"<div role="slider" aria-valuenow="10" aria-valuemin="0" aria-valuemax="100"></div>"#,
            );
            let element = html
                .select(&scraper::Selector::parse("div").unwrap())
                .next()
                .unwrap();
            let missing = crate::accessibility::utils::get_missing_required_aria_properties(&element);
            assert!(missing.is_none());
        }

        #[test]
        fn test_get_missing_required_aria_properties_unknown_role() {
            let html =
                Html::parse_fragment(r#"<div role="unknown"></div>"#);
            let element = html
                .select(&scraper::Selector::parse("div").unwrap())
                .next()
                .unwrap();
            let missing = crate::accessibility::utils::get_missing_required_aria_properties(&element);
            assert!(missing.is_none());
        }
    }

    /// Tests for utility functions.
    mod utility_function_tests {
        use super::*;

        #[test]
        fn test_generate_id() {
            let content = "Test Header!";
            let result = generate_id(content);
            assert_eq!(result, "test-header");
        }

        #[test]
        fn test_generate_id_with_special_characters() {
            let content = "Header--with??special**chars";
            let result = generate_id(content);
            assert_eq!(result, "header-with-special-chars");
        }

        #[test]
        fn test_generate_id_with_leading_trailing_whitespace() {
            let content = "  Test Header  ";
            let result = generate_id(content);
            assert_eq!(result, "test-header");
        }

        #[test]
        fn test_generate_id_with_numeric_content() {
            let content = "12345";
            let result = generate_id(content);
            assert_eq!(result, "12345");
        }

        #[test]
        fn test_is_valid_language_code() {
            assert!(is_valid_language_code("en"));
            assert!(is_valid_language_code("en-US"));
            assert!(!is_valid_language_code("E"));
            assert!(!is_valid_language_code("123"));
        }

        #[test]
        fn test_is_valid_language_code_long_code() {
            assert!(is_valid_language_code("en-US-variant-123"));
        }

        #[test]
        fn test_is_valid_language_code_non_ascii() {
            assert!(!is_valid_language_code("日本語"));
        }

        /// Additional tests for `extract_front_matter` function.
        #[test]
        fn test_extract_front_matter_empty_delimiters() {
            let content = "------\n# Missing proper front matter";
            let result = extract_front_matter(content);
            assert!(matches!(
                result,
                Err(HtmlError::InvalidFrontMatterFormat(_))
            ));
        }

        #[test]
        fn test_extract_front_matter_large_content_valid_front_matter()
        {
            let large_content = format!(
                "---\nkey: value\n---\n{}",
                "Content".repeat(5000)
            );
            let result = extract_front_matter(&large_content);
            assert!(result.is_ok());
        }

        /// Additional tests for `format_header_with_id_class` function.
        #[test]
        fn test_format_header_with_malformed_html() {
            let header = "<h2 Missing closing>";
            let result =
                format_header_with_id_class(header, None, None);
            assert!(matches!(
                result,
                Err(HtmlError::InvalidHeaderFormat(_))
            ));
        }

        #[test]
        fn test_format_header_with_inline_styles() {
            let header =
                r#"<h2 style="color: red;">Styled Header</h2>"#;
            let result =
                format_header_with_id_class(header, None, None);
            assert!(result.is_ok());
            assert_eq!(
            result.unwrap(),
            "<h2 id=\"styled-header\" class=\"styled-header\">Styled Header</h2>"
        );
        }

        /// Additional tests for `generate_table_of_contents` function.
        #[test]
        fn test_toc_with_nested_headers() {
            let html = "<div><h1>Outer</h1><h2>Inner</h2></div>";
            let result = generate_table_of_contents(html);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                r#"<ul><li class="toc-h1"><a href="\#outer">Outer</a></li><li class="toc-h2"><a href="\#inner">Inner</a></li></ul>"#
            );
        }

        #[test]
        fn test_toc_with_malformed_and_valid_headers() {
            let html = "<h1>Valid</h1><h2 Malformed>";
            let result = generate_table_of_contents(html);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                r#"<ul><li class="toc-h1"><a href="\#valid">Valid</a></li></ul>"#
            );
        }

        /// Additional tests for `is_valid_aria_role` function.
        #[test]
        fn test_unsupported_html_element() {
            let html = Html::parse_fragment(
                "<unsupported role='custom'></unsupported>",
            );
            let element = html
                .select(
                    &scraper::Selector::parse("unsupported").unwrap(),
                )
                .next()
                .unwrap();
            assert!(!is_valid_aria_role("custom", &element));
        }

        /// Additional tests for `is_valid_language_code` function.
        #[test]
        fn test_is_valid_language_code_with_mixed_case() {
            assert!(!is_valid_language_code("eN-uS"));
            assert!(!is_valid_language_code("En#Us"));
        }

        /// Additional tests for `generate_id` function.
        #[test]
        fn test_generate_id_empty_content() {
            let content = "";
            let result = generate_id(content);
            assert_eq!(result, "");
        }

        #[test]
        fn test_generate_id_whitespace_content() {
            let content = "   ";
            let result = generate_id(content);
            assert_eq!(result, "");
        }

        #[test]
        fn test_generate_id_symbols_only() {
            let content = "!@#$%^&*()";
            let result = generate_id(content);
            assert_eq!(result, "");
        }
    }
}

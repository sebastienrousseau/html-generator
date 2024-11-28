//! Utility functions for HTML and Markdown processing.
//!
//! This module provides various utility functions for tasks such as
//! extracting front matter from Markdown content and formatting HTML headers.

use crate::error::{HtmlError, Result};
use once_cell::sync::Lazy;
use regex::Regex;

static FRONT_MATTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ms)^---\s*\n(.*?)\n---\s*\n")
        .expect("Failed to compile FRONT_MATTER_REGEX")
});

static HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<(h[1-6])>(.+?)</h[1-6]>")
        .expect("Failed to compile HEADER_REGEX")
});

static CONSECUTIVE_HYPHENS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-{2,}")
        .expect("Failed to compile CONSECUTIVE_HYPHENS_REGEX")
});

/// Maximum allowed input size (in bytes) to prevent DOS attacks
const MAX_INPUT_SIZE: usize = 1_000_000; // 1 MB

/// Extracts front matter from Markdown content.
///
/// This function removes the front matter (if present) from the given content
/// and returns the rest of the content. If no front matter is present, it returns
/// the original content.
///
/// The front matter should be in the following format:
/// ```markdown
/// ---
/// key1: value1
/// key2: value2
/// ---
/// ```
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
            let remaining_content = &content[captures
                .get(0)
                .ok_or_else(|| {
                    HtmlError::InvalidFrontMatterFormat(
                        "Missing front matter match".to_string(),
                    )
                })?
                .end()..];
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

/// Formats a header with an ID and class.
///
/// This function takes an HTML header and adds an id and class attribute
/// based on the header's content.
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
/// # Errors
///
/// This function will return an error if the header is invalidly formatted.
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

    let tag = captures
        .get(1)
        .ok_or_else(|| {
            HtmlError::InvalidHeaderFormat(
                "Missing header tag".to_string(),
            )
        })?
        .as_str();
    let content = captures
        .get(2)
        .ok_or_else(|| {
            HtmlError::InvalidHeaderFormat(
                "Missing header content".to_string(),
            )
        })?
        .as_str();

    let id = id_generator.map_or_else(
        || generate_id(content),
        |generator| generator(content),
    );

    let class = class_generator.map_or_else(
        || generate_id(content),
        |generator| generator(content),
    );

    Ok(format!(
        r#"<{} id="{}" class="{}">{}</{}>"#,
        tag, id, class, content, tag
    ))
}

/// Generates a table of contents from HTML content.
///
/// This function extracts all headers (h1-h6) from the provided HTML content
/// and generates a table of contents as an HTML unordered list.
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
/// let html = "<h1>Title</h1><p>Some content</p><h2>Subtitle</h2><p>More content</p><h3>Sub-subtitle</h3>";
/// let result = generate_table_of_contents(html).unwrap();
/// assert_eq!(result, r#"<ul><li class="toc-h1"><a href="\#title">Title</a></li><li class="toc-h2"><a href="\#subtitle">Subtitle</a></li><li class="toc-h3"><a href="\#sub-subtitle">Sub-subtitle</a></li></ul>"#);
/// ```
pub fn generate_table_of_contents(html: &str) -> Result<String> {
    if html.is_empty() {
        return Err(HtmlError::InvalidInput("Empty input".to_string()));
    }
    if html.len() > MAX_INPUT_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let mut toc = String::with_capacity(html.len() / 10);
    toc.push_str("<ul>");

    for captures in HEADER_REGEX.captures_iter(html) {
        let tag = captures
            .get(1)
            .ok_or_else(|| {
                HtmlError::InvalidHeaderFormat(
                    "Missing tag in header".to_string(),
                )
            })?
            .as_str();
        let content = captures
            .get(2)
            .ok_or_else(|| {
                HtmlError::InvalidHeaderFormat(
                    "Missing content in header".to_string(),
                )
            })?
            .as_str();
        let id = generate_id(content);

        toc.push_str(&format!(
            r#"<li class="toc-{}"><a href="\#{}">{}</a></li>"#,
            tag, id, content
        ));
    }

    toc.push_str("</ul>");
    Ok(toc)
}

/// Generates an ID from the given content.
fn generate_id(content: &str) -> String {
    CONSECUTIVE_HYPHENS_REGEX
        .replace_all(
            &content
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "-"),
            "-",
        )
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter() {
        let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
        let result = extract_front_matter(content);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(extracted) = result {
            assert_eq!(extracted, "# Hello, world!\n\nThis is a test.");
        }
    }

    #[test]
    fn test_extract_front_matter_no_front_matter() {
        let content =
            "# Hello, world!\n\nThis is a test without front matter.";
        let result = extract_front_matter(content);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(extracted) = result {
            assert_eq!(extracted, content);
        }
    }

    #[test]
    fn test_extract_front_matter_empty_input() {
        let content = "";
        let result = extract_front_matter(content);
        assert!(matches!(result, Err(HtmlError::InvalidInput(_))));
    }

    #[test]
    fn test_format_header_with_id_class() {
        let header = "<h2>Hello, World!</h2>";
        let result = format_header_with_id_class(header, None, None);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(formatted) = result {
            assert_eq!(
                formatted,
                r#"<h2 id="hello-world" class="hello-world">Hello, World!</h2>"#
            );
        }
    }

    #[test]
    fn test_format_header_with_custom_generators() {
        let header = "<h3>Test Header</h3>";
        let id_gen = |content: &str| {
            format!(
                "custom-{}",
                content.to_lowercase().replace(' ', "-")
            )
        };
        let class_gen = |_: &str| "custom-class".to_string();
        let result = format_header_with_id_class(
            header,
            Some(id_gen),
            Some(class_gen),
        );
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(formatted) = result {
            assert_eq!(
                formatted,
                r#"<h3 id="custom-test-header" class="custom-class">Test Header</h3>"#
            );
        }
    }

    #[test]
    fn test_format_header_with_special_characters() {
        let header = "<h3>Test: Special & Characters</h3>";
        let result = format_header_with_id_class(header, None, None);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(formatted) = result {
            assert_eq!(
                formatted,
                r#"<h3 id="test-special-characters" class="test-special-characters">Test: Special & Characters</h3>"#
            );
        }
    }

    #[test]
    fn test_format_header_with_consecutive_hyphens() {
        let header = "<h4>Multiple---Hyphens</h4>";
        let result = format_header_with_id_class(header, None, None);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(formatted) = result {
            assert_eq!(
                formatted,
                r#"<h4 id="multiple-hyphens" class="multiple-hyphens">Multiple---Hyphens</h4>"#
            );
        }
    }

    #[test]
    fn test_format_header_with_invalid_format() {
        let header = "<p>Not a header</p>";
        let result = format_header_with_id_class(header, None, None);
        assert!(matches!(
            result,
            Err(HtmlError::InvalidHeaderFormat(_))
        ));
    }

    #[test]
    fn test_generate_table_of_contents() {
        let html = "<h1>Title</h1><h2>Subtitle</h2>";
        let result = generate_table_of_contents(html);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(toc) = result {
            assert_eq!(
                toc,
                r#"<ul><li class="toc-h1"><a href="\#title">Title</a></li><li class="toc-h2"><a href="\#subtitle">Subtitle</a></li></ul>"#
            );
        }
    }

    #[test]
    fn test_generate_table_of_contents_empty_input() {
        let html = "";
        let result = generate_table_of_contents(html);
        assert!(matches!(result, Err(HtmlError::InvalidInput(_))));
    }

    #[test]
    fn test_generate_table_of_contents_no_headers() {
        let html = "<p>This is a paragraph without any headers.</p>";
        let result = generate_table_of_contents(html);
        assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result);
        if let Ok(toc) = result {
            assert_eq!(toc, "<ul></ul>");
        }
    }
}

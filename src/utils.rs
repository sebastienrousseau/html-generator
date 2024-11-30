//! Utility functions for HTML and Markdown processing.
//!
//! This module provides various utility functions for tasks such as
//! extracting front matter from Markdown content and formatting HTML headers.

use crate::error::{HtmlError, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;
use std::collections::HashMap;

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
            // Extract the front matter
            let front_matter = captures
                .get(1)
                .ok_or_else(|| {
                    HtmlError::InvalidFrontMatterFormat(
                        "Missing front matter match".to_string(),
                    )
                })?
                .as_str();

            // Validate the front matter content
            for line in front_matter.lines() {
                if !line.trim().contains(':') {
                    return Err(HtmlError::InvalidFrontMatterFormat(
                        format!(
                            "Invalid line in front matter: {}",
                            line
                        ),
                    ));
                }
            }

            // Extract remaining content
            let remaining_content =
                &content[captures.get(0).unwrap().end()..];
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

/// Check if an ARIA role is valid for a given element.
pub fn is_valid_aria_role(role: &str, element: &ElementRef) -> bool {
    static VALID_ROLES: Lazy<HashMap<&'static str, Vec<&'static str>>> =
        Lazy::new(|| {
            let mut roles = HashMap::new();
            _ = roles.insert("a", vec!["link", "button", "menuitem"]);
            _ = roles.insert("button", vec!["button"]);
            _ = roles.insert("div", vec!["alert", "tooltip", "dialog"]);
            _ = roles.insert(
                "input",
                vec!["textbox", "radio", "checkbox", "searchbox"],
            );
            // Add other elements and roles as necessary
            roles
        });

    if let Some(valid_roles) = VALID_ROLES.get(element.value().name()) {
        valid_roles.contains(&role)
    } else {
        false // If the element isn't in the map, return false
    }
}

/// Validate a language code using basic BCP 47 rules.
pub fn is_valid_language_code(lang: &str) -> bool {
    let parts: Vec<&str> = lang.split('-').collect();
    if parts.is_empty() || parts[0].len() < 2 || parts[0].len() > 3 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_lowercase())
}

/// Get missing required ARIA properties for an element.
pub fn get_missing_required_aria_properties(
    element: &ElementRef,
) -> Option<Vec<String>> {
    let mut missing = Vec::new();
    if let Some(role) = element.value().attr("role") {
        match role {
            "slider" => {
                if element.value().attr("aria-valuenow").is_none() {
                    missing.push("aria-valuenow".to_string());
                }
                if element.value().attr("aria-valuemin").is_none() {
                    missing.push("aria-valuemin".to_string());
                }
                if element.value().attr("aria-valuemax").is_none() {
                    missing.push("aria-valuemax".to_string());
                }
            }
            "combobox" => {
                if element.value().attr("aria-expanded").is_none() {
                    missing.push("aria-expanded".to_string());
                }
            }
            _ => {}
        }
    }
    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
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
    use scraper::Html;

    /// Tests for `extract_front_matter` function.
    mod extract_front_matter_tests {
        use super::*;

        #[test]
        fn test_valid_front_matter() {
            let content = "---\ntitle: My Page\n---\n# Hello, world!\n\nThis is a test.";
            let result = extract_front_matter(content);
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(extracted) = result {
                assert_eq!(
                    extracted,
                    "# Hello, world!\n\nThis is a test."
                );
            }
        }

        #[test]
        fn test_no_front_matter() {
            let content = "# Hello, world!\n\nThis is a test without front matter.";
            let result = extract_front_matter(content);
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(extracted) = result {
                assert_eq!(extracted, content);
            }
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
            // Input with an invalid front matter line (missing `:`).
            let content =
                "---\ntitle: value\ninvalid_line\n---\nContent";
            let result = extract_front_matter(content);
            assert!(
        matches!(result, Err(HtmlError::InvalidFrontMatterFormat(_))),
        "Expected InvalidFrontMatterFormat error, but got: {:?}",
        result
    );
        }

        #[test]
        fn test_valid_front_matter_with_extra_content() {
            let content = "---\ntitle: Page\n---\n\n# Title\n\nContent";
            let result = extract_front_matter(content);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "# Title\n\nContent");
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
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(formatted) = result {
                assert_eq!(
                    formatted,
                    r#"<h2 id="hello-world" class="hello-world">Hello, World!</h2>"#
                );
            }
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
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(formatted) = result {
                assert_eq!(
                    formatted,
                    r#"<h3 id="custom-test-header" class="custom-class">Test Header</h3>"#
                );
            }
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
        fn test_header_with_special_characters() {
            let header = "<h3>Special & Header!</h3>";
            let result =
                format_header_with_id_class(header, None, None);
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                r#"<h3 id="special-header" class="special-header">Special & Header!</h3>"#
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
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(toc) = result {
                assert_eq!(
                    toc,
                    r#"<ul><li class="toc-h1"><a href="\#title">Title</a></li><li class="toc-h2"><a href="\#subtitle">Subtitle</a></li></ul>"#
                );
            }
        }

        #[test]
        fn test_html_without_headers() {
            let html = "<p>No headers here.</p>";
            let result = generate_table_of_contents(html);
            assert!(
                result.is_ok(),
                "Expected Ok, got Err: {:?}",
                result
            );
            if let Ok(toc) = result {
                assert_eq!(toc, "<ul></ul>");
            }
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
            let missing =
                get_missing_required_aria_properties(&element);
            assert_eq!(
                missing.unwrap(),
                vec![
                    "aria-valuenow".to_string(),
                    "aria-valuemin".to_string(),
                    "aria-valuemax".to_string()
                ]
            );
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
        fn test_is_valid_language_code() {
            assert!(is_valid_language_code("en"));
            assert!(is_valid_language_code("en-US"));
            assert!(!is_valid_language_code("E"));
            assert!(!is_valid_language_code("123"));
        }
    }
}

//! Search Engine Optimization (SEO) functionality for HTML processing.
//!
//! This module provides tools for improving the SEO of web pages through automated
//! meta tag generation and structured data implementation. It includes features for:
//! - Meta tag generation for improved search engine visibility
//! - Structured data (JSON-LD) generation for rich search results
//! - HTML content analysis for SEO optimization
//! - Safe HTML entity escaping
//!
//! # Examples
//!
//! ```rust
//! use html_generator::seo::{MetaTagsBuilder, generate_structured_data};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let html = r#"<html><head><title>My Page</title></head><body><p>Content</p></body></html>"#;
//!
//! // Generate meta tags
//! let meta_tags = MetaTagsBuilder::new()
//!     .with_title("My Page")
//!     .with_description("Page content")
//!     .build()?;
//!
//! // Generate structured data
//! let structured_data = generate_structured_data(html, None)?;
//! # Ok(())
//! # }
//! ```

use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::error::{HtmlError, Result, SeoErrorKind};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use scraper::{Html, Selector};

// Constants
/// Maximum allowed size for HTML input (1MB)
const MAX_HTML_SIZE: usize = 1_000_000;
/// Default page type for structured data
const DEFAULT_PAGE_TYPE: &str = "WebPage";
/// Schema.org context URL
const SCHEMA_ORG_CONTEXT: &str = "https://schema.org";
/// Default OpenGraph type
const DEFAULT_OG_TYPE: &str = "website";

// Compile regular expressions at compile time
lazy_static! {
    /// Regular expression for matching HTML special characters
    static ref HTML_ESCAPES: Regex = Regex::new(r#"[&<>"']"#)
        .expect("Failed to compile HTML escapes regex");

    /// Regular expression for extracting meta description
    static ref META_DESC_SELECTOR: Selector = Selector::parse("meta[name='description']")
        .expect("Failed to compile meta description selector");

    /// Regular expression for extracting title
    static ref TITLE_SELECTOR: Selector = Selector::parse("title")
        .expect("Failed to compile title selector");

    /// Regular expression for extracting paragraphs
    static ref PARAGRAPH_SELECTOR: Selector = Selector::parse("p")
        .expect("Failed to compile paragraph selector");
}

/// Configuration options for structured data generation.
#[derive(Debug, Clone)]
pub struct StructuredDataConfig {
    /// Additional key-value pairs to include in the structured data
    pub additional_data: Option<HashMap<String, String>>,
    /// The type of webpage (e.g., "WebPage", "Article", "Product")
    pub page_type: String,
    /// Additional schema.org types to include
    pub additional_types: Vec<String>,
}

impl Default for StructuredDataConfig {
    fn default() -> Self {
        Self {
            additional_data: None,
            page_type: String::from(DEFAULT_PAGE_TYPE),
            additional_types: Vec::new(),
        }
    }
}

impl StructuredDataConfig {
    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The page type is empty
    /// - Any additional type is empty
    fn validate(&self) -> Result<()> {
        validate_page_type(&self.page_type)?;

        if self.additional_types.iter().any(String::is_empty) {
            return Err(HtmlError::seo(
                SeoErrorKind::InvalidStructuredData,
                "Additional types cannot be empty",
                None,
            ));
        }
        Ok(())
    }
}

/// Builder for constructing meta tags.
#[derive(Debug, Default)]
pub struct MetaTagsBuilder {
    /// Title for the meta tags
    title: Option<String>,
    /// Description for the meta tags
    description: Option<String>,
    /// OpenGraph type
    og_type: String,
    /// Additional meta tags
    additional_tags: Vec<(String, String)>,
}

impl MetaTagsBuilder {
    /// Creates a new `MetaTagsBuilder` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            og_type: String::from(DEFAULT_OG_TYPE),
            additional_tags: Vec::new(),
        }
    }

    /// Sets the title for the meta tags.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description for the meta tags.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Adds an additional meta tag.
    #[must_use]
    pub fn add_meta_tag(
        mut self,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.additional_tags.push((name.into(), content.into()));
        self
    }

    /// Adds multiple meta tags at once.
    #[must_use]
    pub fn add_meta_tags<I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        self.additional_tags.extend(tags);
        self
    }

    /// Builds the meta tags string.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields (title or description) are missing.
    pub fn build(self) -> Result<String> {
        let title = self.title.ok_or_else(|| {
            HtmlError::seo(
                SeoErrorKind::MissingTitle,
                "Meta title is required",
                None,
            )
        })?;

        let description = self.description.ok_or_else(|| {
            HtmlError::seo(
                SeoErrorKind::MissingDescription,
                "Meta description is required",
                None,
            )
        })?;

        let mut meta_tags = String::with_capacity(500);

        // Add required meta tags
        meta_tags.push_str(&format!(
            r#"<meta name="title" content="{}">"#,
            escape_html(&title)
        ));
        meta_tags.push_str(&format!(
            r#"<meta name="description" content="{}">"#,
            escape_html(&description)
        ));
        meta_tags.push_str(&format!(
            r#"<meta property="og:type" content="{}">"#,
            escape_html(&self.og_type)
        ));

        // Add additional meta tags
        for (name, content) in self.additional_tags {
            meta_tags.push_str(&format!(
                r#"<meta name="{}" content="{}">"#,
                escape_html(&name),
                escape_html(&content)
            ));
        }

        Ok(meta_tags)
    }
}

/// Validates that a page type is not empty.
///
/// # Errors
///
/// Returns an error if the page type is empty.
fn validate_page_type(page_type: &str) -> Result<()> {
    if page_type.is_empty() {
        return Err(HtmlError::seo(
            SeoErrorKind::InvalidStructuredData,
            "Page type cannot be empty",
            None,
        ));
    }
    Ok(())
}

/// Escapes HTML special characters in a string.
///
/// This function replaces special characters with their HTML entity equivalents:
/// - `&` becomes `&amp;`
/// - `<` becomes `&lt;`
/// - `>` becomes `&gt;`
/// - `"` becomes `&quot;`
/// - `'` becomes `&#x27;`
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// Returns a `Cow<str>` containing either the original string if no escaping was
/// needed, or a new string with escaped characters.
///
/// # Examples
///
/// ```
/// use html_generator::seo::escape_html;
///
/// let input = r#"<script>alert("Hello & goodbye")</script>"#;
/// let escaped = escape_html(input);
/// assert_eq!(
///     escaped,
///     r#"&lt;script&gt;alert(&quot;Hello &amp; goodbye&quot;)&lt;/script&gt;"#
/// );
/// ```
#[must_use]
pub fn escape_html(s: &str) -> Cow<str> {
    HTML_ESCAPES.replace_all(s, |caps: &Captures| match &caps[0] {
        "&" => "&amp;",
        "<" => "&lt;",
        ">" => "&gt;",
        "\"" => "&quot;",
        "'" => "&#x27;",
        _ => unreachable!("Regex only matches [&<>\"']"),
    })
}

/// Generates meta tags for SEO purposes.
///
/// # Arguments
///
/// * `html` - The HTML content to analyze
///
/// # Returns
///
/// Returns a `Result` containing the generated meta tags as a string.
///
/// # Errors
///
/// Returns an error if:
/// - The HTML input is too large (> 1MB)
/// - Required elements (title, description) are missing
///
/// # Examples
///
/// ```
/// use html_generator::seo::generate_meta_tags;
///
/// let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
/// let meta_tags = generate_meta_tags(html)?;
/// # Ok::<(), html_generator::error::HtmlError>(())
/// ```
pub fn generate_meta_tags(html: &str) -> Result<String> {
    if html.len() > MAX_HTML_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let document = Html::parse_document(html);
    let title = extract_title(&document)?;
    let description = extract_description(&document)?;

    MetaTagsBuilder::new()
        .with_title(title)
        .with_description(description)
        .build()
}

/// Generates structured data (JSON-LD) for SEO purposes.
///
/// # Arguments
///
/// * `html` - The HTML content to analyze
/// * `config` - Optional configuration for structured data generation
///
/// # Returns
///
/// Returns a `Result` containing the generated JSON-LD script as a string.
///
/// # Errors
///
/// Returns an error if:
/// - The HTML input is too large (> 1MB)
/// - Required elements are missing
/// - JSON serialization fails
/// - Configuration validation fails
///
/// # Examples
///
/// ```
/// use html_generator::seo::generate_structured_data;
///
/// let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
/// let structured_data = generate_structured_data(html, None)?;
/// # Ok::<(), html_generator::error::HtmlError>(())
/// ```
pub fn generate_structured_data(
    html: &str,
    config: Option<StructuredDataConfig>,
) -> Result<String> {
    if html.len() > MAX_HTML_SIZE {
        return Err(HtmlError::InputTooLarge(html.len()));
    }

    let document = Html::parse_document(html);
    let config = config.unwrap_or_default();
    config.validate()?;

    let title = extract_title(&document)?;
    let description = extract_description(&document)?;

    let mut json = if config.additional_types.is_empty() {
        json!({
            "@context": SCHEMA_ORG_CONTEXT,
            "@type": config.page_type,
            "name": title,
            "description": description,
        })
    } else {
        let mut types = vec![config.page_type];
        types.extend(config.additional_types.into_iter());
        json!({
            "@context": SCHEMA_ORG_CONTEXT,
            "@type": types,
            "name": title,
            "description": description,
        })
    };

    // Add any additional data
    if let Some(additional_data) = config.additional_data {
        for (key, value) in additional_data {
            json[key] = json!(value);
        }
    }

    Ok(format!(
        r#"<script type="application/ld+json">
{}
</script>"#,
        serde_json::to_string_pretty(&json).map_err(|e| {
            HtmlError::InvalidStructuredData(e.to_string())
        })?
    ))
}

// Private helper functions
fn extract_title(document: &Html) -> Result<String> {
    document
        .select(&TITLE_SELECTOR)
        .next()
        .map(|t| t.text().collect::<String>())
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("title".to_string())
        })
}

fn extract_description(document: &Html) -> Result<String> {
    // Try meta description first
    if let Some(meta) = document.select(&META_DESC_SELECTOR).next() {
        if let Some(content) = meta.value().attr("content") {
            return Ok(content.to_string());
        }
    }

    // Fall back to first paragraph
    document
        .select(&PARAGRAPH_SELECTOR)
        .next()
        .map(|p| p.text().collect::<String>())
        .ok_or_else(|| {
            HtmlError::MissingHtmlElement("description".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case as case;

    /// Tests for MetaTagsBuilder functionality
    mod meta_tags_builder {
        use super::*;

        #[test]
        fn builds_basic_meta_tags() {
            let meta_tags = MetaTagsBuilder::new()
                .with_title("Test Title")
                .with_description("Test Description")
                .add_meta_tag("keywords", "test,keywords")
                .build()
                .unwrap();

            assert!(meta_tags.contains(
                r#"<meta name="title" content="Test Title">"#
            ));
            assert!(meta_tags.contains(r#"<meta name="description" content="Test Description">"#));
            assert!(meta_tags.contains(
                r#"<meta name="keywords" content="test,keywords">"#
            ));
        }

        #[test]
        fn handles_multiple_meta_tags() {
            let tags = vec![
                ("keywords".to_string(), "test,tags".to_string()),
                ("robots".to_string(), "index,follow".to_string()),
            ];
            let meta_tags = MetaTagsBuilder::new()
                .with_title("Test")
                .with_description("Test")
                .add_meta_tags(tags)
                .build()
                .unwrap();

            assert!(
                meta_tags.contains(r#"keywords" content="test,tags"#)
            );
            assert!(
                meta_tags.contains(r#"robots" content="index,follow"#)
            );
        }

        #[test]
        fn fails_without_title() {
            let result = MetaTagsBuilder::new()
                .with_description("Test Description")
                .build();

            assert!(matches!(
                result,
                Err(HtmlError::Seo {
                    kind: SeoErrorKind::MissingTitle,
                    ..
                })
            ));
        }

        #[test]
        fn fails_without_description() {
            let result =
                MetaTagsBuilder::new().with_title("Test Title").build();

            assert!(matches!(
                result,
                Err(HtmlError::Seo {
                    kind: SeoErrorKind::MissingDescription,
                    ..
                })
            ));
        }

        #[test]
        fn escapes_special_characters_in_meta_tags() {
            let meta_tags = MetaTagsBuilder::new()
                .with_title("Test & Title")
                .with_description("Test < Description >")
                .build()
                .unwrap();

            assert!(meta_tags.contains(r#"content="Test &amp; Title"#));
            assert!(meta_tags
                .contains(r#"content="Test &lt; Description &gt;"#));
        }
    }

    /// Tests for HTML escaping functionality
    mod html_escaping {
        use super::*;

        #[case("<>&\"'" => "&lt;&gt;&amp;&quot;&#x27;" ; "escapes all special characters")]
        #[case("Normal text" => "Normal text" ; "leaves normal text unchanged")]
        #[case("" => "" ; "handles empty string")]
        fn escape_html_cases(input: &str) -> String {
            escape_html(input).into_owned()
        }

        #[test]
        fn escapes_mixed_content() {
            let input = "Text with <tags> & \"quotes\" 'here'";
            let expected = "Text with &lt;tags&gt; &amp; &quot;quotes&quot; &#x27;here&#x27;";
            assert_eq!(escape_html(input), expected);
        }
    }

    /// Tests for structured data functionality
    mod structured_data {
        use super::*;

        #[test]
        fn generates_basic_structured_data() {
            let html = r"<html><head><title>Test</title></head><body><p>Description</p></body></html>";
            let result = generate_structured_data(html, None).unwrap();

            let json_content = extract_json_from_script(&result);
            let parsed: serde_json::Value =
                serde_json::from_str(&json_content).unwrap();

            assert_eq!(parsed["@type"], "WebPage");
            assert_eq!(parsed["name"], "Test");
            assert_eq!(parsed["description"], "Description");
        }

        #[test]
        fn generates_multiple_types() {
            let html = r"<html><head><title>Test</title></head><body><p>Description</p></body></html>";
            let config = StructuredDataConfig {
                page_type: "Article".to_string(),
                additional_types: vec!["WebPage".to_string()],
                additional_data: Some(HashMap::from([(
                    "author".to_string(),
                    "Test Author".to_string(),
                )])),
            };

            let result =
                generate_structured_data(html, Some(config)).unwrap();
            let json_content = extract_json_from_script(&result);
            let parsed: serde_json::Value =
                serde_json::from_str(&json_content).unwrap();

            assert_eq!(
                parsed["@type"],
                serde_json::json!(["Article", "WebPage"]),
                "Expected @type to include multiple types"
            );
            assert_eq!(
                parsed["author"], "Test Author",
                "Expected author to be included"
            );
        }

        #[test]
        fn validates_config() {
            let empty_type = StructuredDataConfig {
                page_type: "".to_string(),
                ..Default::default()
            };
            assert!(empty_type.validate().is_err());

            let empty_additional = StructuredDataConfig {
                additional_types: vec!["".to_string()],
                ..Default::default()
            };
            assert!(empty_additional.validate().is_err());
        }

        /// Helper function to extract JSON content from script tags
        fn extract_json_from_script(script: &str) -> String {
            let json_start =
                script.find('{').expect("JSON should start with '{'");
            let json_end =
                script.rfind('}').expect("JSON should end with '}'");
            script[json_start..=json_end].to_string()
        }
    }

    /// Tests for input validation and limits
    mod input_validation {
        use super::*;

        #[test]
        fn enforces_size_limit_for_meta_tags() {
            let large_html = "a".repeat(MAX_HTML_SIZE + 1);
            assert!(matches!(
                generate_meta_tags(&large_html),
                Err(HtmlError::InputTooLarge(_))
            ));
        }

        #[test]
        fn enforces_size_limit_for_structured_data() {
            let large_html = "a".repeat(MAX_HTML_SIZE + 1);
            assert!(matches!(
                generate_structured_data(&large_html, None),
                Err(HtmlError::InputTooLarge(_))
            ));
        }

        #[test]
        fn handles_missing_title() {
            let html =
                r"<html><body><p>No title here</p></body></html>";
            assert!(matches!(
                generate_meta_tags(html),
                Err(HtmlError::MissingHtmlElement(ref e)) if e == "title"
            ));
        }

        #[test]
        fn handles_missing_description() {
            let html =
                r"<html><head><title>Title only</title></head></html>";
            assert!(matches!(
                generate_meta_tags(html),
                Err(HtmlError::MissingHtmlElement(ref e)) if e == "description"
            ));
        }
    }
}

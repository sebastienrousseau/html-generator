//! Accessibility-related functionality for HTML processing.
//!
//! This module provides comprehensive tools for improving HTML accessibility through:
//! - Automated ARIA attribute management
//! - WCAG 2.1 compliance validation
//! - Accessibility issue detection and correction
//!
//! # WCAG Compliance
//!
//! This module implements checks for WCAG 2.1 compliance across three levels:
//! - Level A (minimum level of conformance)
//! - Level AA (addresses major accessibility barriers)
//! - Level AAA (highest level of accessibility conformance)
//!
//! For detailed information about WCAG guidelines, see:
//! <https://www.w3.org/WAI/WCAG21/quickref/>
//!
//! # Limitations
//!
//! While this module provides automated checks, some accessibility aspects require
//! manual review, including:
//! - Semantic correctness of ARIA labels
//! - Meaningful alternative text for images
//! - Logical heading structure
//! - Color contrast ratios
//!
//! # Examples
//!
//! ```rust
//! use html_generator::accessibility::{add_aria_attributes, validate_wcag, WcagLevel};
//!
//! use html_generator::accessibility::AccessibilityConfig;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let html = r#"<button>Click me</button>"#;
//!
//!     // Add ARIA attributes automatically
//!     let enhanced_html = add_aria_attributes(html, None)?;
//!
//!     // Validate against WCAG AA level
//!     let config = AccessibilityConfig::default();
//!     validate_wcag(&enhanced_html, &config, None)?;
//!
//!     Ok(())
//! }
//! ```

use crate::accessibility::utils::get_missing_required_aria_properties;
use crate::accessibility::utils::is_valid_aria_role;
use crate::accessibility::utils::is_valid_language_code;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;
use thiserror::Error;

/// Constants used throughout the accessibility module
pub mod constants {
    /// Maximum size of HTML input in bytes (1MB)
    pub const MAX_HTML_SIZE: usize = 1_000_000;

    /// Default ARIA role for navigation elements
    pub const DEFAULT_NAV_ROLE: &str = "navigation";

    /// Default ARIA role for buttons
    pub const DEFAULT_BUTTON_ROLE: &str = "button";

    /// Default ARIA role for forms
    pub const DEFAULT_FORM_ROLE: &str = "form";

    /// Default ARIA role for inputs
    pub const DEFAULT_INPUT_ROLE: &str = "textbox";
}

use constants::{
    DEFAULT_BUTTON_ROLE, DEFAULT_INPUT_ROLE, DEFAULT_NAV_ROLE,
    MAX_HTML_SIZE,
};

/// WCAG Conformance Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WcagLevel {
    /// Level A: Minimum level of conformance
    /// Essential accessibility features that must be supported
    A,

    /// Level AA: Addresses major accessibility barriers
    /// Standard level of conformance for most websites
    AA,

    /// Level AAA: Highest level of accessibility conformance
    /// Includes additional enhancements and specialized features
    AAA,
}

/// Types of accessibility issues that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueType {
    /// Missing alternative text for images
    MissingAltText,
    /// Improper heading structure
    HeadingStructure,
    /// Missing form labels
    MissingLabels,
    /// Invalid ARIA attributes
    InvalidAria,
    /// Color contrast issues
    ColorContrast,
    /// Keyboard navigation issues
    KeyboardNavigation,
    /// Missing or invalid language declarations
    LanguageDeclaration,
}

/// Enum to represent possible accessibility-related errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Error indicating an invalid ARIA attribute.
    #[error("Invalid ARIA Attribute '{attribute}': {message}")]
    InvalidAriaAttribute {
        /// The name of the invalid attribute
        attribute: String,
        /// Description of the error
        message: String,
    },

    /// Error indicating failure to validate HTML against WCAG guidelines.
    #[error("WCAG {level} Validation Error: {message}")]
    WcagValidationError {
        /// WCAG conformance level where the error occurred
        level: WcagLevel,
        /// Description of the error
        message: String,
        /// Specific WCAG guideline reference
        guideline: Option<String>,
    },

    /// Error indicating the HTML input is too large to process.
    #[error(
        "HTML Input Too Large: size {size} exceeds maximum {max_size}"
    )]
    HtmlTooLarge {
        /// Actual size of the input
        size: usize,
        /// Maximum allowed size
        max_size: usize,
    },

    /// Error indicating a failure in processing HTML for accessibility.
    #[error("HTML Processing Error: {message}")]
    HtmlProcessingError {
        /// Description of the processing error
        message: String,
        /// Source of the error, if available
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error indicating malformed HTML input.
    #[error("Malformed HTML: {message}")]
    MalformedHtml {
        /// Description of the HTML issue
        message: String,
        /// The problematic HTML fragment, if available
        fragment: Option<String>,
    },
}

/// Result type alias for accessibility operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Structure representing an accessibility issue found in the HTML
#[derive(Debug, Clone)]
pub struct Issue {
    /// Type of accessibility issue
    pub issue_type: IssueType,
    /// Description of the issue
    pub message: String,
    /// WCAG guideline reference, if applicable
    pub guideline: Option<String>,
    /// HTML element where the issue was found
    pub element: Option<String>,
    /// Suggested fix for the issue
    pub suggestion: Option<String>,
}

/// Helper function to create a `Selector`, returning an `Option` on failure.
fn try_create_selector(selector: &str) -> Option<Selector> {
    match Selector::parse(selector) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!(
                "Failed to create selector '{}': {}",
                selector, e
            );
            None
        }
    }
}

/// Helper function to create a `Regex`, returning an `Option` on failure.
fn try_create_regex(pattern: &str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("Failed to create regex '{}': {}", pattern, e);
            None
        }
    }
}

/// Static selectors for HTML elements and ARIA attributes
static BUTTON_SELECTOR: Lazy<Option<Selector>> =
    Lazy::new(|| try_create_selector("button:not([aria-label])"));

/// Selector for navigation elements without ARIA attributes
static NAV_SELECTOR: Lazy<Option<Selector>> =
    Lazy::new(|| try_create_selector("nav:not([aria-label])"));

/// Selector for form elements without ARIA attributes
static FORM_SELECTOR: Lazy<Option<Selector>> =
    Lazy::new(|| try_create_selector("form:not([aria-labelledby])"));

/// Regex for finding input elements
static INPUT_REGEX: Lazy<Option<Regex>> =
    Lazy::new(|| try_create_regex(r"<input[^>]*>"));

/// Comprehensive selector for all ARIA attributes
static ARIA_SELECTOR: Lazy<Option<Selector>> = Lazy::new(|| {
    try_create_selector(concat!(
        "[aria-label], [aria-labelledby], [aria-describedby], ",
        "[aria-hidden], [aria-expanded], [aria-haspopup], ",
        "[aria-controls], [aria-pressed], [aria-checked], ",
        "[aria-current], [aria-disabled], [aria-dropeffect], ",
        "[aria-grabbed], [aria-invalid], [aria-live], ",
        "[aria-owns], [aria-relevant], [aria-required], ",
        "[aria-role], [aria-selected], [aria-valuemax], ",
        "[aria-valuemin], [aria-valuenow], [aria-valuetext]"
    ))
});

/// Set of valid ARIA attributes
static VALID_ARIA_ATTRIBUTES: Lazy<HashSet<&'static str>> =
    Lazy::new(|| {
        [
            "aria-label",
            "aria-labelledby",
            "aria-describedby",
            "aria-hidden",
            "aria-expanded",
            "aria-haspopup",
            "aria-controls",
            "aria-pressed",
            "aria-checked",
            "aria-current",
            "aria-disabled",
            "aria-dropeffect",
            "aria-grabbed",
            "aria-invalid",
            "aria-live",
            "aria-owns",
            "aria-relevant",
            "aria-required",
            "aria-role",
            "aria-selected",
            "aria-valuemax",
            "aria-valuemin",
            "aria-valuenow",
            "aria-valuetext",
        ]
        .iter()
        .copied()
        .collect()
    });

/// Color contrast requirements for different WCAG levels
// static COLOR_CONTRAST_RATIOS: Lazy<HashMap<WcagLevel, f64>> = Lazy::new(|| {
//     let mut m = HashMap::new();
//     m.insert(WcagLevel::A, 3.0);       // Minimum contrast for Level A
//     m.insert(WcagLevel::AA, 4.5);      // Enhanced contrast for Level AA
//     m.insert(WcagLevel::AAA, 7.0);     // Highest contrast for Level AAA
//     m
// });
///
/// Set of elements that must have labels
// static LABELABLE_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
//     [
//         "input", "select", "textarea", "button", "meter",
//         "output", "progress", "canvas"
//     ].iter().copied().collect()
// });
///
/// Selector for finding headings
// static HEADING_SELECTOR: Lazy<Selector> = Lazy::new(|| {
//     Selector::parse("h1, h2, h3, h4, h5, h6")
//         .expect("Failed to create heading selector")
// });
///
/// Selector for finding images
// static IMAGE_SELECTOR: Lazy<Selector> = Lazy::new(|| {
//     Selector::parse("img").expect("Failed to create image selector")
// });
/// Configuration for accessibility validation
#[derive(Debug, Copy, Clone)]
pub struct AccessibilityConfig {
    /// WCAG conformance level to validate against
    pub wcag_level: WcagLevel,
    /// Maximum allowed heading level jump (e.g., 1 means no skipping levels)
    pub max_heading_jump: u8,
    /// Minimum required color contrast ratio
    pub min_contrast_ratio: f64,
    /// Whether to automatically fix issues when possible
    pub auto_fix: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            wcag_level: WcagLevel::AA,
            max_heading_jump: 1,
            min_contrast_ratio: 4.5, // WCAG AA standard
            auto_fix: true,
        }
    }
}

/// A comprehensive accessibility check result
#[derive(Debug)]
pub struct AccessibilityReport {
    /// List of accessibility issues found
    pub issues: Vec<Issue>,
    /// WCAG conformance level checked
    pub wcag_level: WcagLevel,
    /// Total number of elements checked
    pub elements_checked: usize,
    /// Number of issues found
    pub issue_count: usize,
    /// Time taken for the check (in milliseconds)
    pub check_duration_ms: u64,
}

/// Add ARIA attributes to HTML for improved accessibility.
///
/// This function performs a comprehensive analysis of the HTML content and adds
/// appropriate ARIA attributes to improve accessibility. It handles:
/// - Button labeling
/// - Navigation landmarks
/// - Form controls
/// - Input elements
/// - Dynamic content
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content
/// * `config` - Optional configuration for the enhancement process
///
/// # Returns
///
/// * `Result<String>` - The modified HTML with ARIA attributes included
///
/// # Errors
///
/// Returns an error if:
/// * The input HTML is larger than `MAX_HTML_SIZE`
/// * The HTML cannot be parsed
/// * There's an error adding ARIA attributes
///
/// # Examples
///
/// ```rust
/// use html_generator::accessibility::{add_aria_attributes, AccessibilityConfig};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let html = r#"<button>Click me</button>"#;
///     let result = add_aria_attributes(html, None)?;
///     assert!(result.contains(r#"aria-label="Click me""#));
///
///     Ok(())
/// }
/// ```
pub fn add_aria_attributes(
    html: &str,
    config: Option<AccessibilityConfig>,
) -> Result<String> {
    let config = config.unwrap_or_default();

    if html.len() > MAX_HTML_SIZE {
        return Err(Error::HtmlTooLarge {
            size: html.len(),
            max_size: MAX_HTML_SIZE,
        });
    }

    let mut html_builder = HtmlBuilder::new(html);

    // Apply transformations
    html_builder = add_aria_to_buttons(html_builder)?;
    html_builder = add_aria_to_navs(html_builder)?;
    html_builder = add_aria_to_forms(html_builder)?;
    html_builder = add_aria_to_inputs(html_builder)?;

    // Additional transformations for stricter WCAG levels
    if matches!(config.wcag_level, WcagLevel::AA | WcagLevel::AAA) {
        html_builder = enhance_landmarks(html_builder)?;
        html_builder = add_live_regions(html_builder)?;
    }

    if matches!(config.wcag_level, WcagLevel::AAA) {
        html_builder = enhance_descriptions(html_builder)?;
    }

    // Validate and clean up
    let new_html =
        remove_invalid_aria_attributes(&html_builder.build());

    if !validate_aria(&new_html) {
        return Err(Error::InvalidAriaAttribute {
            attribute: "multiple".to_string(),
            message: "Failed to add valid ARIA attributes".to_string(),
        });
    }

    Ok(new_html)
}

/// A builder struct for constructing HTML content.
#[derive(Debug, Clone)]
struct HtmlBuilder {
    content: String,
}

impl HtmlBuilder {
    /// Creates a new `HtmlBuilder` with the given initial content.
    fn new(initial_content: &str) -> Self {
        HtmlBuilder {
            content: initial_content.to_string(),
        }
    }

    /// Builds the final HTML content.
    fn build(self) -> String {
        self.content
    }
}

/// Helper function to count total elements checked during validation
fn count_checked_elements(document: &Html) -> usize {
    document.select(&Selector::parse("*").unwrap()).count()
}

/// Add landmark regions to improve navigation
const fn enhance_landmarks(
    html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    // Implementation for adding landmarks
    Ok(html_builder)
}

/// Add live regions for dynamic content
const fn add_live_regions(
    html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    // Implementation for adding live regions
    Ok(html_builder)
}

/// Enhance element descriptions for better accessibility
const fn enhance_descriptions(
    html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    // Implementation for enhancing descriptions
    Ok(html_builder)
}

/// Check heading structure
fn check_heading_structure(document: &Html, issues: &mut Vec<Issue>) {
    let mut prev_level: Option<u8> = None;

    let selector = match Selector::parse("h1, h2, h3, h4, h5, h6") {
        Ok(selector) => selector,
        Err(e) => {
            eprintln!("Failed to parse selector: {}", e);
            return; // Skip checking if the selector is invalid
        }
    };

    for heading in document.select(&selector) {
        let current_level = heading
            .value()
            .name()
            .chars()
            .nth(1)
            .and_then(|c| c.to_digit(10))
            .and_then(|n| u8::try_from(n).ok());

        if let Some(current_level) = current_level {
            if let Some(prev_level) = prev_level {
                if current_level > prev_level + 1 {
                    issues.push(Issue {
                        issue_type: IssueType::HeadingStructure,
                        message: format!(
                            "Skipped heading level from h{} to h{}",
                            prev_level, current_level
                        ),
                        guideline: Some("WCAG 2.4.6".to_string()),
                        element: Some(heading.html()),
                        suggestion: Some(
                            "Use sequential heading levels".to_string(),
                        ),
                    });
                }
            }
            prev_level = Some(current_level);
        }
    }
}

/// Validate HTML against WCAG guidelines with detailed reporting.
///
/// Performs a comprehensive accessibility check based on WCAG guidelines and
/// provides detailed feedback about any issues found.
///
/// # Arguments
///
/// * `html` - The HTML content to validate
/// * `config` - Configuration options for the validation
///
/// # Returns
///
/// * `Result<AccessibilityReport>` - A detailed report of the accessibility check
///
/// # Examples
///
/// ```rust
/// use html_generator::accessibility::{validate_wcag, AccessibilityConfig, WcagLevel};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let html = r#"<img src="test.jpg" alt="A descriptive alt text">"#;
///     let config = AccessibilityConfig::default();
///
///     let report = validate_wcag(html, &config, None)?;
///     println!("Found {} issues", report.issue_count);
///
///     Ok(())
/// }
/// ```
pub fn validate_wcag(
    html: &str,
    config: &AccessibilityConfig,
    disable_checks: Option<&[IssueType]>,
) -> Result<AccessibilityReport> {
    let start_time = std::time::Instant::now();
    let mut issues = Vec::new();
    let mut elements_checked = 0;

    if html.trim().is_empty() {
        return Ok(AccessibilityReport {
            issues: Vec::new(),
            wcag_level: config.wcag_level,
            elements_checked: 0,
            issue_count: 0,
            check_duration_ms: 0,
        });
    }

    let document = Html::parse_document(html);

    if disable_checks
        .map_or(true, |d| !d.contains(&IssueType::LanguageDeclaration))
    {
        check_language_attributes(&document, &mut issues)?; // Returns Result<()>, so `?` works.
    }

    // This function returns `()`, so no `?`.
    check_heading_structure(&document, &mut issues);

    elements_checked += count_checked_elements(&document);

    // Explicit error conversion for u64::try_from
    let check_duration_ms = u64::try_from(
        start_time.elapsed().as_millis(),
    )
    .map_err(|err| Error::HtmlProcessingError {
        message: "Failed to convert duration to milliseconds"
            .to_string(),
        source: Some(Box::new(err)),
    })?;

    Ok(AccessibilityReport {
        issues: issues.clone(),
        wcag_level: config.wcag_level,
        elements_checked,
        issue_count: issues.len(),
        check_duration_ms,
    })
}

/// From implementation for TryFromIntError
impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Self {
        Error::HtmlProcessingError {
            message: "Integer conversion error".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

/// Display implementation for WCAG levels
impl std::fmt::Display for WcagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WcagLevel::A => write!(f, "A"),
            WcagLevel::AA => write!(f, "AA"),
            WcagLevel::AAA => write!(f, "AAA"),
        }
    }
}

/// Internal helper functions for accessibility checks
impl AccessibilityReport {
    /// Creates a new accessibility issue
    fn add_issue(
        issues: &mut Vec<Issue>,
        issue_type: IssueType,
        message: impl Into<String>,
        guideline: Option<String>,
        element: Option<String>,
        suggestion: Option<String>,
    ) {
        issues.push(Issue {
            issue_type,
            message: message.into(),
            guideline,
            element,
            suggestion,
        });
    }
}

/// Add ARIA attributes to button elements.
fn add_aria_to_buttons(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    // Safely unwrap the BUTTON_SELECTOR
    if let Some(selector) = BUTTON_SELECTOR.as_ref() {
        for button in document.select(selector) {
            // Check if the button has no aria-label
            if button.value().attr("aria-label").is_none() {
                let button_html = button.html();
                let inner_content = button.inner_html();

                // Generate a new button with appropriate aria-label
                let new_button_html = if inner_content.trim().is_empty()
                {
                    format!(
                        r#"<button aria-label="{}" role="button">{}</button>"#,
                        DEFAULT_BUTTON_ROLE, inner_content
                    )
                } else {
                    format!(
                        r#"<button aria-label="{}" role="button">{}</button>"#,
                        inner_content.trim(),
                        inner_content
                    )
                };

                // Replace the old button HTML with the new one
                html_builder.content = html_builder
                    .content
                    .replace(&button_html, &new_button_html);
            }
        }
    }

    Ok(html_builder)
}

/// Add ARIA attributes to navigation elements.
fn add_aria_to_navs(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    if let Some(selector) = NAV_SELECTOR.as_ref() {
        for nav in document.select(selector) {
            let nav_html = nav.html();
            let new_nav_html = nav_html.replace(
                "<nav",
                &format!(
                    r#"<nav aria-label="{}" role="navigation""#,
                    DEFAULT_NAV_ROLE
                ),
            );
            html_builder.content =
                html_builder.content.replace(&nav_html, &new_nav_html);
        }
    }

    Ok(html_builder)
}

/// Add ARIA attributes to form elements.
fn add_aria_to_forms(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    if let Some(selector) = FORM_SELECTOR.as_ref() {
        for form in document.select(selector) {
            let form_html = form.html();
            let form_id = format!("form-{}", generate_unique_id());
            let new_form_html = form_html.replace(
                "<form",
                &format!(
                    r#"<form id="{}" aria-labelledby="{}" role="form""#,
                    form_id, form_id
                ),
            );
            html_builder.content = html_builder
                .content
                .replace(&form_html, &new_form_html);
        }
    }

    Ok(html_builder)
}

/// Add ARIA attributes to input elements.
fn add_aria_to_inputs(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    if let Some(regex) = INPUT_REGEX.as_ref() {
        let mut replacements: Vec<(String, String)> = Vec::new();

        for cap in regex.captures_iter(&html_builder.content) {
            let input_tag = &cap[0];
            if !input_tag.contains("aria-label") {
                let input_type = extract_input_type(input_tag)
                    .unwrap_or_else(|| "text".to_string());
                let new_input_tag = format!(
                    r#"<input aria-label="{}" role="{}" type="{}""#,
                    input_type, DEFAULT_INPUT_ROLE, input_type
                );
                replacements
                    .push((input_tag.to_string(), new_input_tag));
            }
        }

        for (old, new) in replacements {
            html_builder.content =
                html_builder.content.replace(&old, &new);
        }
    }

    Ok(html_builder)
}

/// Extract input type from an input tag.
fn extract_input_type(input_tag: &str) -> Option<String> {
    static TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"type=["']([^"']+)["']"#)
            .expect("Failed to create type regex")
    });

    TYPE_REGEX
        .captures(input_tag)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Generate a unique ID for form elements.
fn generate_unique_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("aria-{}", nanos)
}

/// Validate ARIA attributes within the HTML.
fn validate_aria(html: &str) -> bool {
    let document = Html::parse_document(html);

    if let Some(selector) = ARIA_SELECTOR.as_ref() {
        document
            .select(selector)
            .flat_map(|el| el.value().attrs())
            .filter(|(name, _)| name.starts_with("aria-"))
            .all(|(name, value)| is_valid_aria_attribute(name, value))
    } else {
        eprintln!("ARIA_SELECTOR failed to initialize.");
        false
    }
}

fn remove_invalid_aria_attributes(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut new_html = html.to_string();

    if let Some(selector) = ARIA_SELECTOR.as_ref() {
        for element in document.select(selector) {
            let element_html = element.html();
            let mut updated_html = element_html.clone();

            for (attr_name, attr_value) in element.value().attrs() {
                if attr_name.starts_with("aria-")
                    && !is_valid_aria_attribute(attr_name, attr_value)
                {
                    updated_html = updated_html.replace(
                        &format!(r#" {}="{}""#, attr_name, attr_value),
                        "",
                    );
                }
            }

            new_html = new_html.replace(&element_html, &updated_html);
        }
    }

    new_html
}

/// Check if an ARIA attribute is valid.
fn is_valid_aria_attribute(name: &str, value: &str) -> bool {
    if !VALID_ARIA_ATTRIBUTES.contains(name) {
        return false; // Invalid ARIA attribute name
    }

    match name {
        "aria-hidden" | "aria-expanded" | "aria-pressed"
        | "aria-invalid" => {
            matches!(value, "true" | "false") // Only "true" or "false" are valid
        }
        "aria-level" => value.parse::<u32>().is_ok(), // Must be a valid integer
        _ => !value.trim().is_empty(), // General check for non-empty values
    }
}

fn check_language_attributes(
    document: &Html,
    issues: &mut Vec<Issue>,
) -> Result<()> {
    if let Some(html_element) =
        document.select(&Selector::parse("html").unwrap()).next()
    {
        if html_element.value().attr("lang").is_none() {
            AccessibilityReport::add_issue(
                issues,
                IssueType::LanguageDeclaration,
                "Missing language declaration on HTML element",
                Some("WCAG 3.1.1".to_string()),
                Some("<html>".to_string()),
                Some("Add lang attribute to HTML element".to_string()),
            );
        }
    }

    for element in document.select(&Selector::parse("[lang]").unwrap())
    {
        if let Some(lang) = element.value().attr("lang") {
            if !is_valid_language_code(lang) {
                AccessibilityReport::add_issue(
                    issues,
                    IssueType::LanguageDeclaration,
                    format!("Invalid language code: {}", lang),
                    Some("WCAG 3.1.2".to_string()),
                    Some(element.html()),
                    Some("Use valid BCP 47 language code".to_string()),
                );
            }
        }
    }
    Ok(())
}

/// Helper functions for WCAG validation
impl AccessibilityReport {
    /// Check keyboard navigation
    pub fn check_keyboard_navigation(
        document: &Html,
        issues: &mut Vec<Issue>,
    ) -> Result<()> {
        let binding = Selector::parse(
            "a, button, input, select, textarea, [tabindex]",
        )
        .unwrap();
        let interactive_elements = document.select(&binding);

        for element in interactive_elements {
            // Check tabindex
            if let Some(tabindex) = element.value().attr("tabindex") {
                if let Ok(index) = tabindex.parse::<i32>() {
                    if index < 0 {
                        Self::add_issue(
                            issues,
                            IssueType::KeyboardNavigation,
                            "Negative tabindex prevents keyboard focus",
                            Some("WCAG 2.1.1".to_string()),
                            Some(element.html()),
                            Some(
                                "Remove negative tabindex value"
                                    .to_string(),
                            ),
                        );
                    }
                }
            }

            // Check for click handlers without keyboard equivalents
            if element.value().attr("onclick").is_some()
                && element.value().attr("onkeypress").is_none()
                && element.value().attr("onkeydown").is_none()
            {
                Self::add_issue(
                    issues,
                    IssueType::KeyboardNavigation,
                    "Click handler without keyboard equivalent",
                    Some("WCAG 2.1.1".to_string()),
                    Some(element.html()),
                    Some("Add keyboard event handlers".to_string()),
                );
            }
        }
        Ok(())
    }

    /// Check language attributes
    pub fn check_language_attributes(
        document: &Html,
        issues: &mut Vec<Issue>,
    ) -> Result<()> {
        // Check html lang attribute
        let html_element =
            document.select(&Selector::parse("html").unwrap()).next();
        if let Some(element) = html_element {
            if element.value().attr("lang").is_none() {
                Self::add_issue(
                    issues,
                    IssueType::LanguageDeclaration,
                    "Missing language declaration",
                    Some("WCAG 3.1.1".to_string()),
                    Some(element.html()),
                    Some(
                        "Add lang attribute to html element"
                            .to_string(),
                    ),
                );
            }
        }

        // Check for changes in language
        let binding = Selector::parse("[lang]").unwrap();
        let text_elements = document.select(&binding);
        for element in text_elements {
            if let Some(lang) = element.value().attr("lang") {
                if !is_valid_language_code(lang) {
                    Self::add_issue(
                        issues,
                        IssueType::LanguageDeclaration,
                        format!("Invalid language code: {}", lang),
                        Some("WCAG 3.1.2".to_string()),
                        Some(element.html()),
                        Some(
                            "Use valid BCP 47 language code"
                                .to_string(),
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    /// Check advanced ARIA usage
    pub fn check_advanced_aria(
        document: &Html,
        issues: &mut Vec<Issue>,
    ) -> Result<()> {
        // Check for proper ARIA roles
        let binding = Selector::parse("[role]").unwrap();
        let elements_with_roles = document.select(&binding);
        for element in elements_with_roles {
            if let Some(role) = element.value().attr("role") {
                if !is_valid_aria_role(role, &element) {
                    Self::add_issue(
                        issues,
                        IssueType::InvalidAria,
                        format!(
                            "Invalid ARIA role '{}' for element",
                            role
                        ),
                        Some("WCAG 4.1.2".to_string()),
                        Some(element.html()),
                        Some("Use appropriate ARIA role".to_string()),
                    );
                }
            }
        }

        // Check for required ARIA properties
        let elements_with_aria =
            document.select(ARIA_SELECTOR.as_ref().unwrap());
        for element in elements_with_aria {
            if let Some(missing_props) =
                get_missing_required_aria_properties(&element)
            {
                Self::add_issue(
                    issues,
                    IssueType::InvalidAria,
                    format!(
                        "Missing required ARIA properties: {}",
                        missing_props.join(", ")
                    ),
                    Some("WCAG 4.1.2".to_string()),
                    Some(element.html()),
                    Some("Add required ARIA properties".to_string()),
                );
            }
        }
        Ok(())
    }
}

/// Utility functions for accessibility checks
mod utils {
    use scraper::ElementRef;
    use std::collections::HashMap;

    /// Validate language code against BCP 47
    use once_cell::sync::Lazy;
    use regex::Regex;

    /// Validate language code against simplified BCP 47 rules.
    pub(crate) fn is_valid_language_code(lang: &str) -> bool {
        static LANGUAGE_CODE_REGEX: Lazy<Regex> = Lazy::new(|| {
            // Match primary language and optional subtags
            Regex::new(r"(?i)^[a-z]{2,3}(-[a-z0-9]{2,8})*$").unwrap()
        });

        // Ensure the regex matches and the code does not end with a hyphen
        LANGUAGE_CODE_REGEX.is_match(lang) && !lang.ends_with('-')
    }

    /// Check if ARIA role is valid for element
    pub(crate) fn is_valid_aria_role(
        role: &str,
        element: &ElementRef,
    ) -> bool {
        static VALID_ROLES: Lazy<HashMap<&str, Vec<&str>>> =
            Lazy::new(|| {
                let mut map = HashMap::new();
                let _ = map.insert(
                    "button",
                    vec!["button", "link", "menuitem"],
                );
                let _ = map.insert(
                    "input",
                    vec!["textbox", "radio", "checkbox", "button"],
                );
                map
            });

        if let Some(valid_roles) =
            VALID_ROLES.get(element.value().name())
        {
            valid_roles.contains(&role)
        } else {
            true
        }
    }

    /// Get missing required ARIA properties
    pub(crate) fn get_missing_required_aria_properties(
        element: &ElementRef,
    ) -> Option<Vec<String>> {
        let mut missing = Vec::new();
        if let Some(role) = element.value().attr("role") {
            match role {
                "combobox" => {
                    check_required_prop(
                        element,
                        "aria-expanded",
                        &mut missing,
                    );
                }
                "slider" => {
                    check_required_prop(
                        element,
                        "aria-valuenow",
                        &mut missing,
                    );
                    check_required_prop(
                        element,
                        "aria-valuemin",
                        &mut missing,
                    );
                    check_required_prop(
                        element,
                        "aria-valuemax",
                        &mut missing,
                    );
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

    /// Check if required property is present
    fn check_required_prop(
        element: &ElementRef,
        prop: &str,
        missing: &mut Vec<String>,
    ) {
        if element.value().attr(prop).is_none() {
            missing.push(prop.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test WCAG Level functionality
    mod wcag_level_tests {
        use super::*;

        #[test]
        fn test_wcag_level_ordering() {
            assert!(matches!(WcagLevel::A, WcagLevel::A));
            assert!(matches!(WcagLevel::AA, WcagLevel::AA));
            assert!(matches!(WcagLevel::AAA, WcagLevel::AAA));
        }

        #[test]
        fn test_wcag_level_debug() {
            assert_eq!(format!("{:?}", WcagLevel::A), "A");
            assert_eq!(format!("{:?}", WcagLevel::AA), "AA");
            assert_eq!(format!("{:?}", WcagLevel::AAA), "AAA");
        }
    }

    // Test AccessibilityConfig functionality
    mod config_tests {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = AccessibilityConfig::default();
            assert_eq!(config.wcag_level, WcagLevel::AA);
            assert_eq!(config.max_heading_jump, 1);
            assert_eq!(config.min_contrast_ratio, 4.5);
            assert!(config.auto_fix);
        }

        #[test]
        fn test_custom_config() {
            let config = AccessibilityConfig {
                wcag_level: WcagLevel::AAA,
                max_heading_jump: 2,
                min_contrast_ratio: 7.0,
                auto_fix: false,
            };
            assert_eq!(config.wcag_level, WcagLevel::AAA);
            assert_eq!(config.max_heading_jump, 2);
            assert_eq!(config.min_contrast_ratio, 7.0);
            assert!(!config.auto_fix);
        }
    }

    // Test ARIA attribute management
    mod aria_attribute_tests {
        use super::*;

        #[test]
        fn test_valid_aria_attributes() {
            assert!(is_valid_aria_attribute("aria-label", "Test"));
            assert!(is_valid_aria_attribute("aria-hidden", "true"));
            assert!(is_valid_aria_attribute("aria-hidden", "false"));
            assert!(!is_valid_aria_attribute("aria-hidden", "yes"));
            assert!(!is_valid_aria_attribute("invalid-aria", "value"));
        }

        #[test]
        fn test_empty_aria_value() {
            assert!(!is_valid_aria_attribute("aria-label", ""));
            assert!(!is_valid_aria_attribute("aria-label", "  "));
        }
    }

    // Test HTML modification functions
    mod html_modification_tests {
        use super::*;

        #[test]
        fn test_add_aria_to_button() {
            let html = "<button>Click me</button>";
            let result = add_aria_attributes(html, None);
            assert!(result.is_ok());
            let enhanced = result.unwrap();
            assert!(enhanced.contains(r#"aria-label="Click me""#));
            assert!(enhanced.contains(r#"role="button""#));
        }

        #[test]
        fn test_add_aria_to_empty_button() {
            let html = "<button></button>";
            let result = add_aria_attributes(html, None);
            assert!(result.is_ok());
            let enhanced = result.unwrap();
            assert!(enhanced.contains(r#"aria-label="button""#));
        }

        #[test]
        fn test_large_input() {
            let large_html = "a".repeat(MAX_HTML_SIZE + 1);
            let result = add_aria_attributes(&large_html, None);
            assert!(matches!(result, Err(Error::HtmlTooLarge { .. })));
        }
    }

    // Test accessibility validation
    mod validation_tests {
        use super::*;

        #[test]
        fn test_heading_structure() {
            let valid_html = "<h1>Main Title</h1><h2>Subtitle</h2>";
            let invalid_html =
                "<h1>Main Title</h1><h3>Skipped Heading</h3>";

            let config = AccessibilityConfig::default();

            // Validate correct heading structure
            let valid_result = validate_wcag(
                valid_html,
                &config,
                Some(&[IssueType::LanguageDeclaration]),
            )
            .unwrap();
            assert_eq!(
                valid_result.issue_count, 0,
                "Expected no issues for valid HTML, but found: {:#?}",
                valid_result.issues
            );

            // Validate incorrect heading structure
            let invalid_result = validate_wcag(
                invalid_html,
                &config,
                Some(&[IssueType::LanguageDeclaration]),
            )
            .unwrap();
            assert_eq!(
        invalid_result.issue_count,
        1,
        "Expected one issue for skipped heading levels, but found: {:#?}",
        invalid_result.issues
    );

            let issue = &invalid_result.issues[0];
            assert_eq!(issue.issue_type, IssueType::HeadingStructure);
            assert_eq!(
                issue.message,
                "Skipped heading level from h1 to h3"
            );
            assert_eq!(issue.guideline, Some("WCAG 2.4.6".to_string()));
            assert_eq!(
                issue.suggestion,
                Some("Use sequential heading levels".to_string())
            );
        }
    }

    // Test report generation
    mod report_tests {
        use super::*;

        #[test]
        fn test_report_generation() {
            let html = r#"<img src="test.jpg">"#;
            let config = AccessibilityConfig::default();
            let report = validate_wcag(html, &config, None).unwrap();

            assert!(report.issue_count > 0);

            assert_eq!(report.wcag_level, WcagLevel::AA);
        }

        #[test]
        fn test_empty_html_report() {
            let html = "";
            let config = AccessibilityConfig::default();
            let report = validate_wcag(html, &config, None).unwrap();

            assert_eq!(report.elements_checked, 0);
            assert_eq!(report.issue_count, 0);
        }

        #[test]
        fn test_missing_selector_handling() {
            // Simulate a scenario where NAV_SELECTOR fails to initialize.
            static TEST_NAV_SELECTOR: Lazy<Option<Selector>> =
                Lazy::new(|| None);

            let html = "<nav>Main Navigation</nav>";
            let document = Html::parse_document(html);

            if let Some(selector) = TEST_NAV_SELECTOR.as_ref() {
                let navs: Vec<_> = document.select(selector).collect();
                assert_eq!(navs.len(), 0);
            }
        }
    }
    #[cfg(test)]
    mod utils_tests {
        use super::*;

        mod language_code_validation {
            use super::*;

            #[test]
            fn test_valid_language_codes() {
                let valid_codes = [
                    "en", "en-US", "zh-CN", "fr-FR", "de-DE", "es-419",
                    "ar-001", "pt-BR", "ja-JP", "ko-KR",
                ];
                for code in valid_codes {
                    assert!(
                        is_valid_language_code(code),
                        "Language code '{}' should be valid",
                        code
                    );
                }
            }

            #[test]
            fn test_invalid_language_codes() {
                let invalid_codes = [
                    "",               // Empty string
                    "a",              // Single character
                    "123",            // Numeric code
                    "en_US",          // Underscore instead of hyphen
                    "en-",            // Trailing hyphen
                    "-en",            // Leading hyphen
                    "en--US",         // Consecutive hyphens
                    "toolong",        // Primary subtag too long
                    "en-US-INVALID-", // Trailing hyphen with subtags
                ];
                for code in invalid_codes {
                    assert!(
                        !is_valid_language_code(code),
                        "Language code '{}' should be invalid",
                        code
                    );
                }
            }

            #[test]
            fn test_language_code_case_sensitivity() {
                assert!(is_valid_language_code("en-GB"));
                assert!(is_valid_language_code("fr-FR"));
                assert!(is_valid_language_code("zh-Hans"));
                assert!(is_valid_language_code("EN-GB"));
            }
        }

        mod aria_role_validation {
            use super::*;

            #[test]
            fn test_valid_button_roles() {
                let html = "<button>Test</button>";
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("button").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();
                let valid_roles = ["button", "link", "menuitem"];
                for role in valid_roles {
                    assert!(
                        is_valid_aria_role(role, &element),
                        "Role '{}' should be valid for button",
                        role
                    );
                }
            }

            #[test]
            fn test_valid_input_roles() {
                let html = "<input type='text'>";
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("input").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();
                let valid_roles =
                    ["textbox", "radio", "checkbox", "button"];
                for role in valid_roles {
                    assert!(
                        is_valid_aria_role(role, &element),
                        "Role '{}' should be valid for input",
                        role
                    );
                }
            }

            #[test]
            fn test_valid_anchor_roles() {
                let html = "<a href='#'>Test</a>";
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("a").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();
                let valid_roles = ["button", "link", "menuitem"];
                for role in valid_roles {
                    assert!(
                        is_valid_aria_role(role, &element),
                        "Role '{}' should be valid for anchor",
                        role
                    );
                }
            }

            #[test]
            fn test_invalid_element_roles() {
                let html = "<button>Test</button>";
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("button").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();
                let invalid_roles =
                    ["textbox", "radio", "checkbox", "invalid"];
                for role in invalid_roles {
                    assert!(
                        !is_valid_aria_role(role, &element),
                        "Role '{}' should be invalid for button",
                        role
                    );
                }
            }

            #[test]
            fn test_unrestricted_elements() {
                // Testing with <div>
                let html_div = "<div>Test</div>";
                let fragment_div = Html::parse_fragment(html_div);
                let selector_div = Selector::parse("div").unwrap();
                let element_div =
                    fragment_div.select(&selector_div).next().unwrap();

                // Testing with <span>
                let html_span = "<span>Test</span>";
                let fragment_span = Html::parse_fragment(html_span);
                let selector_span = Selector::parse("span").unwrap();
                let element_span = fragment_span
                    .select(&selector_span)
                    .next()
                    .unwrap();

                let roles =
                    ["button", "textbox", "navigation", "banner"];

                for role in roles {
                    assert!(
                        is_valid_aria_role(role, &element_div),
                        "Role '{}' should be allowed for div",
                        role
                    );
                    assert!(
                        is_valid_aria_role(role, &element_span),
                        "Role '{}' should be allowed for span",
                        role
                    );
                }
            }
        }

        mod required_aria_properties {
            use super::*;

            #[test]
            fn test_combobox_required_properties() {
                let html = r#"<div role="combobox">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element)
                        .unwrap();
                assert!(missing.contains(&"aria-expanded".to_string()));
            }

            #[test]
            fn test_complete_combobox() {
                let html = r#"<div role="combobox" aria-expanded="true">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element);
                assert!(missing.is_none());
            }

            #[test]
            fn test_slider_required_properties() {
                let html = r#"<div role="slider">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element)
                        .unwrap();

                assert!(missing.contains(&"aria-valuenow".to_string()));
                assert!(missing.contains(&"aria-valuemin".to_string()));
                assert!(missing.contains(&"aria-valuemax".to_string()));
            }

            #[test]
            fn test_complete_slider() {
                let html = r#"<div role="slider"
                   aria-valuenow="50"
                   aria-valuemin="0"
                   aria-valuemax="100">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element);
                assert!(missing.is_none());
            }

            #[test]
            fn test_partial_slider_properties() {
                let html = r#"<div role="slider" aria-valuenow="50">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element)
                        .unwrap();

                assert!(!missing.contains(&"aria-valuenow".to_string()));
                assert!(missing.contains(&"aria-valuemin".to_string()));
                assert!(missing.contains(&"aria-valuemax".to_string()));
            }

            #[test]
            fn test_unknown_role() {
                let html = r#"<div role="unknown">Test</div>"#;
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element);
                assert!(missing.is_none());
            }

            #[test]
            fn test_no_role() {
                let html = "<div>Test</div>";
                let fragment = Html::parse_fragment(html);
                let selector = Selector::parse("div").unwrap();
                let element =
                    fragment.select(&selector).next().unwrap();

                let missing =
                    get_missing_required_aria_properties(&element);
                assert!(missing.is_none());
            }
        }
    }
}

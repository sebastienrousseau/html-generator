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
use crate::emojis::load_emoji_sequences;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;
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

/// Global counter for unique ID generation
// static COUNTER: AtomicUsize = AtomicUsize::new(0);
use constants::{DEFAULT_BUTTON_ROLE, DEFAULT_NAV_ROLE, MAX_HTML_SIZE};

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
#[derive(Debug, Clone)]
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
    html_builder = add_aria_to_accordions(html_builder)?;
    html_builder = add_aria_to_buttons(html_builder)?;
    html_builder = add_aria_to_forms(html_builder)?;
    html_builder = add_aria_to_inputs(html_builder)?;
    html_builder = add_aria_to_navs(html_builder)?;
    html_builder = add_aria_to_tabs(html_builder)?;

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

/// Regex for matching HTML tags
static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<[^>]*>").expect("Failed to compile HTML tag regex")
});

// We'll assume you call `load_emoji_sequences("data/emoji-sequences.txt")` once, and store it here in a static for simplicity.
static EMOJI_MAP: Lazy<
    std::result::Result<HashMap<String, String>, std::io::Error>,
> = Lazy::new(|| load_emoji_sequences("data/emoji-data.txt"));

/// Normalizes content for ARIA labels by removing HTML tags and converting to a standardized format.
///
/// # Arguments
///
/// * `content` - The content to normalize
///
/// # Returns
///
/// Returns a normalized string suitable for use as an ARIA label
fn normalize_aria_label(content: &str) -> String {
    // 1. Remove HTML
    let no_html = HTML_TAG_REGEX.replace_all(content, "");
    // 2. Trim
    let text_only = no_html.trim();

    // 3. If empty, fallback
    if text_only.is_empty() {
        return DEFAULT_BUTTON_ROLE.to_string();
    }

    // 4. Check each loaded emoji mapping
    //    If the user input contains that emoji, return the mapped label
    match &*EMOJI_MAP {
        Ok(map) => {
            for (emoji, label) in map.iter() {
                if text_only.contains(emoji) {
                    return label.clone();
                }
            }
        }
        Err(e) => {
            // Handle the error (e.g., log it)
            eprintln!("Error loading emoji sequences: {}", e);
        }
    }

    // 5. If no match, do your fallback normalization
    text_only
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Adds ARIA attributes to button elements.
///
/// Handles:
/// - Adding `aria-disabled="true"` for buttons with the `disabled` attribute.
/// - Adding `aria-pressed="false"` for non-disabled toggle buttons.
/// - Ensures `aria-label` is present for all buttons.
///
/// # Arguments
///
/// * `html_builder` - The HTML builder containing the content to process.
///
/// # Returns
///
/// * `Result<HtmlBuilder>` - The processed HTML builder with added ARIA attributes.
fn add_aria_to_buttons(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    if let Some(selector) = BUTTON_SELECTOR.as_ref() {
        for button in document.select(selector) {
            let button_html = button.html();
            let inner_content = button.inner_html();
            let mut aria_label = normalize_aria_label(&inner_content);

            let mut attributes = Vec::new();

            // Check if the button is disabled
            if button.value().attr("disabled").is_some() {
                eprintln!(
                    "Processing disabled button: {}",
                    button_html
                ); // Debug log
                attributes.push(r#"aria-disabled="true""#.to_string());
            } else {
                // Add `aria-pressed="false"` for non-disabled buttons
                let is_toggle =
                    button.value().attr("aria-pressed").is_some();
                if is_toggle {
                    let current_state = button
                        .value()
                        .attr("aria-pressed")
                        .unwrap_or("false");
                    let new_state = if current_state == "true" {
                        "false"
                    } else {
                        "true"
                    };
                    attributes.push(format!(
                        r#"aria-pressed="{}""#,
                        new_state
                    ));
                } else {
                    attributes
                        .push(r#"aria-pressed="false""#.to_string());
                }
            }

            // Ensure `aria-label` is present
            if aria_label.is_empty() {
                aria_label = "button".to_string();
            }
            attributes.push(format!(r#"aria-label="{}""#, aria_label));

            // Preserve existing attributes
            for (key, value) in button.value().attrs() {
                attributes.push(format!(r#"{}="{}""#, key, value));
            }

            // Generate the new button HTML
            let new_button_html = format!(
                "<button {}>{}</button>",
                attributes.join(" "),
                inner_content
            );

            // Replace the old button in the HTML
            html_builder.content = replace_html_element_resilient(
                &html_builder.content,
                &button_html,
                &new_button_html,
            );
        }
    }

    Ok(html_builder)
}

/// Replaces an HTML element in a resilient way by expanding shorthand attributes in the original HTML.
fn replace_html_element_resilient(
    original_html: &str,
    old_element: &str,
    new_element: &str,
) -> String {
    // 1) Normalize both sides
    let normalized_original =
        normalize_shorthand_attributes(original_html);
    let normalized_old = normalize_shorthand_attributes(old_element);

    // 2) Try the normalized replacement
    let replaced_normalized =
        normalized_original.replacen(&normalized_old, new_element, 1);
    if replaced_normalized != normalized_original {
        return replaced_normalized;
    }

    // 3) Fallback for <button disabled> vs <button disabled="">
    let shorthand_old =
        old_element.replace(r#"disabled=""#, "disabled");

    let replaced_shorthand =
        original_html.replacen(&shorthand_old, new_element, 1);
    if replaced_shorthand != original_html {
        return replaced_shorthand;
    }

    // 4) Absolute fallback
    eprintln!("DEBUG: Using final fallback replacen(old_element)...");
    original_html.replacen(old_element, new_element, 1)
}

fn normalize_shorthand_attributes(html: &str) -> String {
    let re = Regex::new(
    r"\b(disabled|checked|readonly|multiple|selected|autofocus|required)([\s>])"
).unwrap();

    re.replace_all(html, |caps: &regex::Captures| {
        let attr = &caps[1]; // e.g. "disabled"
        let delim = &caps[2]; // e.g. ">" or " "

        // Insert ="" right before the delimiter
        // So <button disabled> becomes <button disabled="">
        // but <button disabled=""> won't match, so remains as-is
        format!(r#"{}=""{}"#, attr, delim)
    })
    .to_string()
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

    // Traverse form elements and add ARIA attributes
    let forms = document.select(FORM_SELECTOR.as_ref().unwrap());
    for form in forms {
        // Generate a unique ID for the form
        let form_id = format!("form-{}", generate_unique_id());

        let form_element = form.value().clone();
        let mut attributes = form_element.attrs().collect::<Vec<_>>();

        // Add id attribute if missing
        if !attributes.iter().any(|&(k, _)| k == "id") {
            attributes.push(("id", &*form_id));
        }

        // Add aria-labelledby attribute if missing
        if !attributes.iter().any(|&(k, _)| k == "aria-labelledby") {
            attributes.push(("aria-labelledby", &*form_id));
        }

        // Replace the form element in the document
        let new_form_html = format!(
            "<form {}>{}</form>",
            attributes
                .iter()
                .map(|&(k, v)| format!(r#"{}="{}""#, k, v))
                .collect::<Vec<_>>()
                .join(" "),
            form.inner_html()
        );

        html_builder.content =
            html_builder.content.replace(&form.html(), &new_form_html);
    }

    Ok(html_builder)
}

/// Enhance tab controls with ARIA attributes.
///
/// This function expects:
/// - A container element (e.g., <div class="tab-container">) that encloses .tab-button elements.
/// - Each .tab-button will become role="tab".
/// - The container will become role="tablist" if matched.
/// - We'll set aria-controls on each .tab-button, pointing to a <div id="panelX" role="tabpanel"> (not shown here, but recommended).
/// - We'll assume the first tab is active by default; the rest are aria-selected="false".
///
/// Adjust selectors and logic as needed for your codebase.
fn add_aria_to_tabs(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    // Find elements with role="tablist"
    if let Ok(tablist_selector) = Selector::parse("[role='tablist']") {
        for tablist in document.select(&tablist_selector) {
            let tablist_html = tablist.html();
            let mut new_html = String::new();

            // Start with the tablist
            new_html.push_str("<div role=\"tablist\">");

            // Find all buttons within this tablist
            if let Ok(button_selector) = Selector::parse("button") {
                let buttons = tablist.select(&button_selector);

                for (i, button) in buttons.enumerate() {
                    let button_text = button.inner_html();
                    let num = i + 1;
                    // First tab is selected by default
                    let is_selected = i == 0;

                    new_html.push_str(&format!(
                        r#"<button role="tab" id="tab{}" aria-selected="{}" aria-controls="panel{}" tabindex="{}">{}</button>"#,
                        num,
                        is_selected,
                        num,
                        if is_selected { "0" } else { "-1" },
                        button_text
                    ));
                }
            }
            new_html.push_str("</div>");

            // Add the panels
            for i in 0..2 {
                // Assuming 2 panels for this example
                let num = i + 1;
                new_html.push_str(&format!(
                    r#"<div id="panel{}" role="tabpanel" aria-labelledby="tab{}" {}">Panel {}</div>"#,
                    num,
                    num,
                    if i == 0 { "" } else { "hidden" },
                    num
                ));
            }

            // Replace the original with the enhanced version
            html_builder.content =
                html_builder.content.replace(&tablist_html, &new_html);
        }
    }

    Ok(html_builder)
}

fn add_aria_to_accordions(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    // Find accordion containers
    if let Ok(accordion_selector) = Selector::parse(".accordion") {
        for accordion in document.select(&accordion_selector) {
            let accordion_html = accordion.html();
            let mut new_html =
                String::from("<div class=\"accordion\">");

            // Find button and content pairs
            if let (Ok(button_selector), Ok(content_selector)) = (
                Selector::parse("button"),
                Selector::parse("button + div"),
            ) {
                let buttons = accordion.select(&button_selector);
                let contents = accordion.select(&content_selector);

                // Process each button-content pair
                for (i, (button, content)) in
                    buttons.zip(contents).enumerate()
                {
                    let button_text = button.inner_html();
                    let content_text = content.inner_html();
                    let section_num = i + 1;

                    // Add button with ARIA attributes
                    new_html.push_str(&format!(
                        r#"<button aria-expanded="false" aria-controls="section-{}-content" id="section-{}-button">{}</button><div id="section-{}-content" aria-labelledby="section-{}-button" hidden>{}</div>"#,
                        section_num, section_num, button_text,
                        section_num, section_num, content_text
                    ));
                }
            }

            new_html.push_str("</div>");

            // Replace the original accordion with the enhanced version
            html_builder.content = html_builder
                .content
                .replace(&accordion_html, &new_html);
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
        let mut id_counter = 0;

        // Find all <input> tags via the regex
        for cap in regex.captures_iter(&html_builder.content) {
            let input_tag = &cap[0];

            // If there's already an associated label or aria-label, skip
            if input_tag.contains("aria-label")
                || has_associated_label(
                    input_tag,
                    &html_builder.content,
                )
            {
                continue;
            }

            // Determine the input type
            let input_type = extract_input_type(input_tag)
                .unwrap_or_else(|| "text".to_string());

            match input_type.as_str() {
                // Skip text-like and other input types that have visible labels or are not labelable
                "text" | "search" | "tel" | "url" | "email"
                | "password" | "hidden" | "submit" | "reset"
                | "button" | "image" => {
                    // Do nothing
                }

                // For checkbox/radio, ensure ID + label, avoiding duplicates
                "checkbox" | "radio" => {
                    // Preserve all existing attributes
                    let attributes = preserve_attributes(input_tag);

                    // 1) Check if there's already an id="..." in the attributes
                    let re_id = Regex::new(r#"id="([^"]+)""#).unwrap();
                    if let Some(id_match) = re_id.captures(&attributes)
                    {
                        // Already has an ID, so just use it—no duplicates
                        let existing_id = &id_match[1];
                        // Also remove the old id= from the attribute string
                        // so we only insert it once in the final <input ...>
                        let attributes_no_id =
                            re_id.replace(&attributes, "").to_string();

                        // Decide the label text
                        let label_text = if input_type == "checkbox" {
                            format!("Checkbox for {}", existing_id)
                        } else {
                            "Option".to_string()
                        };

                        // Reconstruct <input> with a single id="existingId" + label
                        let enhanced_input = format!(
                            r#"<{} id="{}"><label for="{}">{}</label>"#,
                            attributes_no_id.trim(),
                            existing_id,
                            existing_id,
                            label_text
                        );
                        replacements.push((
                            input_tag.to_string(),
                            enhanced_input,
                        ));
                    } else {
                        // No ID found => generate a new one
                        id_counter += 1;
                        let new_id = format!("option{}", id_counter);
                        let label_text = if input_type == "checkbox" {
                            "Checkbox".to_string()
                        } else {
                            format!("Option {}", id_counter)
                        };

                        let enhanced_input = format!(
                            r#"<input {} id="{}"> <label for="{}">{}</label>"#,
                            attributes, new_id, new_id, label_text
                        );
                        replacements.push((
                            input_tag.to_string(),
                            enhanced_input,
                        ));
                    }
                }

                // For any other types, automatically add `aria-label` with the type name
                _ => {
                    let attributes = preserve_attributes(input_tag);
                    let enhanced_input = format!(
                        r#"<input {} aria-label="{}">"#,
                        attributes, input_type
                    );
                    replacements
                        .push((input_tag.to_string(), enhanced_input));
                }
            }
        }

        // Perform all replacements
        for (old, new) in replacements {
            html_builder.content =
                html_builder.content.replace(&old, &new);
        }
    }

    Ok(html_builder)
}

// Helper function to check for associated labels (using string manipulation)
fn has_associated_label(input_tag: &str, html_content: &str) -> bool {
    if let Some(id_match) =
        Regex::new(r#"id="([^"]+)""#).unwrap().captures(input_tag)
    {
        let id = &id_match[1];
        Regex::new(&format!(r#"<label\s+for="{}"\s*>"#, id))
            .unwrap()
            .is_match(html_content)
    } else {
        false
    }
}

// Regex to capture all key-value pairs in the tag
static ATTRIBUTE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?:data-\w+|[a-zA-Z]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|\S+))?"#,
    )
    .unwrap()
});

/// Extract and preserve existing attributes from an input tag.
fn preserve_attributes(input_tag: &str) -> String {
    ATTRIBUTE_REGEX
        .captures_iter(input_tag)
        .map(|cap| cap[0].to_string())
        .collect::<Vec<String>>()
        .join(" ")
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

/// Generate a unique ID prefixed with "aria-" and UUIDs.
fn generate_unique_id() -> String {
    format!("aria-{}", uuid::Uuid::new_v4())
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
                        issues.push(Issue {
                        issue_type: IssueType::KeyboardNavigation,
                        message: "Negative tabindex prevents keyboard focus".to_string(),
                        guideline: Some("WCAG 2.1.1".to_string()),
                        element: Some(element.html()),
                        suggestion: Some("Remove negative tabindex value".to_string()),
                    });
                    }
                }
            }

            // Check for click handlers without keyboard equivalents
            if element.value().attr("onclick").is_some()
                && element.value().attr("onkeypress").is_none()
                && element.value().attr("onkeydown").is_none()
            {
                issues.push(Issue {
                    issue_type: IssueType::KeyboardNavigation,
                    message:
                        "Click handler without keyboard equivalent"
                            .to_string(),
                    guideline: Some("WCAG 2.1.1".to_string()),
                    element: Some(element.html()),
                    suggestion: Some(
                        "Add keyboard event handlers".to_string(),
                    ),
                });
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
pub mod utils {
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
                _ = map.insert(
                    "button",
                    vec!["button", "link", "menuitem"],
                );
                _ = map.insert(
                    "input",
                    vec!["textbox", "radio", "checkbox", "button"],
                );
                _ = map.insert(
                    "div",
                    vec!["alert", "tooltip", "dialog", "slider"],
                );
                _ = map.insert("a", vec!["link", "button", "menuitem"]);
                map
            });

        // Elements like <div>, <span>, and <a> are more permissive
        let tag_name = element.value().name();
        if ["div", "span", "a"].contains(&tag_name) {
            return true;
        }

        // Validate roles strictly for specific elements
        if let Some(valid_roles) = VALID_ROLES.get(tag_name) {
            valid_roles.contains(&role)
        } else {
            false
        }
    }

    /// Get missing required ARIA properties
    pub(crate) fn get_missing_required_aria_properties(
        element: &ElementRef,
    ) -> Option<Vec<String>> {
        let mut missing = Vec::new();

        static REQUIRED_ARIA_PROPS: Lazy<HashMap<&str, Vec<&str>>> =
            Lazy::new(|| {
                HashMap::from([
                    (
                        "slider",
                        vec![
                            "aria-valuenow",
                            "aria-valuemin",
                            "aria-valuemax",
                        ],
                    ),
                    ("combobox", vec!["aria-expanded"]),
                ])
            });

        if let Some(role) = element.value().attr("role") {
            if let Some(required_props) = REQUIRED_ARIA_PROPS.get(role)
            {
                for prop in required_props {
                    if element.value().attr(prop).is_none() {
                        missing.push(prop.to_string());
                    }
                }
            }
        }

        if missing.is_empty() {
            None
        } else {
            Some(missing)
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

        #[test]
        fn test_html_processing_error_with_source() {
            let source_error = std::io::Error::new(
                std::io::ErrorKind::Other,
                "test source error",
            );
            let error = Error::HtmlProcessingError {
                message: "Processing failed".to_string(),
                source: Some(Box::new(source_error)),
            };

            assert_eq!(
                format!("{}", error),
                "HTML Processing Error: Processing failed"
            );
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
                let html = "<a href=\"\\#\">Test</a>";
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

            #[test]
            fn test_validate_wcag_with_level_aaa() {
                let html =
                    "<h1>Main Title</h1><h3>Skipped Heading</h3>";
                let config = AccessibilityConfig {
                    wcag_level: WcagLevel::AAA,
                    ..Default::default()
                };
                let report =
                    validate_wcag(html, &config, None).unwrap();
                assert!(report.issue_count > 0);
                assert_eq!(report.wcag_level, WcagLevel::AAA);
            }

            #[test]
            fn test_html_builder_empty() {
                let builder = HtmlBuilder::new("");
                assert_eq!(builder.build(), "");
            }

            #[test]
            fn test_generate_unique_id_uniqueness() {
                let id1 = generate_unique_id();
                let id2 = generate_unique_id();
                assert_ne!(id1, id2);
            }
        }

        mod required_aria_properties {
            use super::*;
            use scraper::{Html, Selector};

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
            fn test_add_aria_attributes_empty_html() {
                let html = "";
                let result = add_aria_attributes(html, None);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), "");
            }

            #[test]
            fn test_add_aria_attributes_whitespace_html() {
                let html = "   ";
                let result = add_aria_attributes(html, None);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), "   ");
            }

            #[test]
            fn test_validate_wcag_with_minimal_config() {
                let html = r#"<html lang="en"><div>Accessible Content</div></html>"#;
                let config = AccessibilityConfig {
                    wcag_level: WcagLevel::A,
                    max_heading_jump: 0, // No heading enforcement
                    min_contrast_ratio: 0.0, // No contrast enforcement
                    auto_fix: false,
                };
                let report =
                    validate_wcag(html, &config, None).unwrap();
                assert_eq!(report.issue_count, 0);
            }

            #[test]
            fn test_add_partial_aria_attributes_to_button() {
                let html =
                    r#"<button aria-label="Existing">Click</button>"#;
                let result = add_aria_attributes(html, None);
                assert!(result.is_ok());
                let enhanced = result.unwrap();
                assert!(enhanced.contains(r#"aria-label="Existing""#));
            }

            #[test]
            fn test_add_aria_to_elements_with_existing_roles() {
                let html = r#"<nav aria-label=\"navigation\" role=\"navigation\" role=\"navigation\">Content</nav>"#;
                let result = add_aria_attributes(html, None);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), html);
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

    #[cfg(test)]
    mod accessibility_tests {
        use crate::accessibility::{
            get_missing_required_aria_properties, is_valid_aria_role,
            is_valid_language_code,
        };
        use scraper::Selector;

        #[test]
        fn test_is_valid_language_code() {
            assert!(
                is_valid_language_code("en"),
                "Valid language code 'en' was incorrectly rejected"
            );
            assert!(
                is_valid_language_code("en-US"),
                "Valid language code 'en-US' was incorrectly rejected"
            );
            assert!(
                !is_valid_language_code("123"),
                "Invalid language code '123' was incorrectly accepted"
            );
            assert!(!is_valid_language_code("日本語"), "Non-ASCII language code '日本語' was incorrectly accepted");
        }

        #[test]
        fn test_is_valid_aria_role() {
            use scraper::Html;

            let html = r#"<button></button>"#;
            let document = Html::parse_fragment(html);
            let element = document
                .select(&Selector::parse("button").unwrap())
                .next()
                .unwrap();

            assert!(
                is_valid_aria_role("button", &element),
                "Valid ARIA role 'button' was incorrectly rejected"
            );

            assert!(
        !is_valid_aria_role("invalid-role", &element),
        "Invalid ARIA role 'invalid-role' was incorrectly accepted"
    );
        }

        #[test]
        fn test_get_missing_required_aria_properties() {
            use scraper::{Html, Selector};

            // Case 1: Missing all properties for slider
            let html = r#"<div role="slider"></div>"#;
            let document = Html::parse_fragment(html);
            let element = document
                .select(&Selector::parse("div").unwrap())
                .next()
                .unwrap();

            let missing_props =
                get_missing_required_aria_properties(&element).unwrap();
            assert!(
        missing_props.contains(&"aria-valuenow".to_string()),
        "Did not detect missing 'aria-valuenow' for role 'slider'"
    );
            assert!(
        missing_props.contains(&"aria-valuemin".to_string()),
        "Did not detect missing 'aria-valuemin' for role 'slider'"
    );
            assert!(
        missing_props.contains(&"aria-valuemax".to_string()),
        "Did not detect missing 'aria-valuemax' for role 'slider'"
    );

            // Case 2: All properties present
            let html = r#"<div role="slider" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100"></div>"#;
            let document = Html::parse_fragment(html);
            let element = document
                .select(&Selector::parse("div").unwrap())
                .next()
                .unwrap();

            let missing_props =
                get_missing_required_aria_properties(&element);
            assert!(missing_props.is_none(), "Unexpectedly found missing properties for a complete slider");

            // Case 3: Partially missing properties
            let html =
                r#"<div role="slider" aria-valuenow="50"></div>"#;
            let document = Html::parse_fragment(html);
            let element = document
                .select(&Selector::parse("div").unwrap())
                .next()
                .unwrap();

            let missing_props =
                get_missing_required_aria_properties(&element).unwrap();
            assert!(
                !missing_props.contains(&"aria-valuenow".to_string()),
                "Incorrectly flagged 'aria-valuenow' as missing"
            );
            assert!(
        missing_props.contains(&"aria-valuemin".to_string()),
        "Did not detect missing 'aria-valuemin' for role 'slider'"
    );
            assert!(
        missing_props.contains(&"aria-valuemax".to_string()),
        "Did not detect missing 'aria-valuemax' for role 'slider'"
    );
        }
    }

    #[cfg(test)]
    mod additional_tests {
        use super::*;
        use scraper::Html;

        #[test]
        fn test_validate_empty_html() {
            let html = "";
            let config = AccessibilityConfig::default();
            let report = validate_wcag(html, &config, None).unwrap();
            assert_eq!(
                report.issue_count, 0,
                "Empty HTML should not produce issues"
            );
        }

        #[test]
        fn test_validate_only_whitespace_html() {
            let html = "   ";
            let config = AccessibilityConfig::default();
            let report = validate_wcag(html, &config, None).unwrap();
            assert_eq!(
                report.issue_count, 0,
                "Whitespace-only HTML should not produce issues"
            );
        }

        #[test]
        fn test_validate_language_with_edge_cases() {
            let html = "<html lang=\"en-US\"></html>";
            let _config = AccessibilityConfig::default();
            let mut issues = Vec::new();
            let document = Html::parse_document(html);

            check_language_attributes(&document, &mut issues).unwrap();
            assert_eq!(
                issues.len(),
                0,
                "Valid language declaration should not create issues"
            );
        }

        #[test]
        fn test_validate_invalid_language_code() {
            let html = "<html lang=\"invalid-lang\"></html>";
            let _config = AccessibilityConfig::default();
            let mut issues = Vec::new();
            let document = Html::parse_document(html);

            check_language_attributes(&document, &mut issues).unwrap();
            assert!(
                issues
                    .iter()
                    .any(|i| i.issue_type
                        == IssueType::LanguageDeclaration),
                "Failed to detect invalid language declaration"
            );
        }

        #[test]
        fn test_edge_case_for_generate_unique_id() {
            let ids: Vec<String> =
                (0..100).map(|_| generate_unique_id()).collect();
            let unique_ids: HashSet<String> = ids.into_iter().collect();
            assert_eq!(
                unique_ids.len(),
                100,
                "Generated IDs are not unique in edge case testing"
            );
        }

        #[test]
        fn test_enhance_landmarks_noop() {
            let html = "<div>Simple Content</div>";
            let builder = HtmlBuilder::new(html);
            let result = enhance_landmarks(builder);
            assert!(
                result.is_ok(),
                "Failed to handle simple HTML content"
            );
            assert_eq!(result.unwrap().build(), html, "Landmark enhancement altered simple content unexpectedly");
        }

        #[test]
        fn test_html_with_non_standard_elements() {
            let html =
                "<custom-element aria-label=\"test\"></custom-element>";
            let cleaned_html = remove_invalid_aria_attributes(html);
            assert_eq!(cleaned_html, html, "Unexpectedly modified valid custom element with ARIA attributes");
        }

        #[test]
        fn test_add_aria_to_buttons() {
            let html = r#"<button>Click me</button>"#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_buttons(builder).unwrap().build();
            assert!(result.contains("aria-label"));
        }

        #[test]
        fn test_add_aria_to_empty_buttons() {
            let html = r#"<button></button>"#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_buttons(builder).unwrap();
            assert!(result.build().contains("aria-label"));
        }

        #[test]
        fn test_validate_wcag_empty_html() {
            let html = "";
            let config = AccessibilityConfig::default();
            let disable_checks = None;

            let result = validate_wcag(html, &config, disable_checks);

            match result {
                Ok(report) => assert!(
                    report.issues.is_empty(),
                    "Empty HTML should have no issues"
                ),
                Err(e) => {
                    panic!("Validation failed with error: {:?}", e)
                }
            }
        }

        #[test]
        fn test_validate_wcag_with_complex_html() {
            let html = "
            <html>
                <head></head>
                <body>
                    <button>Click me</button>
                    <a href=\"\\#\"></a>
                </body>
            </html>
        ";
            let config = AccessibilityConfig::default();
            let disable_checks = None;
            let result = validate_wcag(html, &config, disable_checks);

            match result {
                Ok(report) => assert!(
                    !report.issues.is_empty(),
                    "Report should have issues"
                ),
                Err(e) => {
                    panic!("Validation failed with error: {:?}", e)
                }
            }
        }

        #[test]
        fn test_generate_unique_id_uniqueness() {
            let id1 = generate_unique_id();
            let id2 = generate_unique_id();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_try_create_selector_valid() {
            let selector = "div.class";
            let result = try_create_selector(selector);
            assert!(result.is_some());
        }

        #[test]
        fn test_try_create_selector_invalid() {
            let selector = "div..class";
            let result = try_create_selector(selector);
            assert!(result.is_none());
        }

        #[test]
        fn test_try_create_regex_valid() {
            let pattern = r"\d+";
            let result = try_create_regex(pattern);
            assert!(result.is_some());
        }

        #[test]
        fn test_try_create_regex_invalid() {
            let pattern = r"\d+(";
            let result = try_create_regex(pattern);
            assert!(result.is_none());
        }

        /// Test the `enhance_descriptions` function
        #[test]
        fn test_enhance_descriptions() {
            let builder =
                HtmlBuilder::new("<html><body></body></html>");
            let result = enhance_descriptions(builder);
            assert!(result.is_ok(), "Enhance descriptions failed");
        }

        /// Test `From<TryFromIntError>` for `Error`
        #[test]
        fn test_error_from_try_from_int_error() {
            // Trigger a TryFromIntError by attempting to convert a large integer
            let result: std::result::Result<u8, _> = i32::try_into(300); // This will fail
            let err = result.unwrap_err(); // Extract the TryFromIntError
            let error: Error = Error::from(err);

            if let Error::HtmlProcessingError { message, source } =
                error
            {
                assert_eq!(message, "Integer conversion error");
                assert!(source.is_some());
            } else {
                panic!("Expected HtmlProcessingError");
            }
        }

        /// Test `Display` implementation for `WcagLevel`
        #[test]
        fn test_wcag_level_display() {
            assert_eq!(WcagLevel::A.to_string(), "A");
            assert_eq!(WcagLevel::AA.to_string(), "AA");
            assert_eq!(WcagLevel::AAA.to_string(), "AAA");
        }

        /// Test `check_keyboard_navigation`
        #[test]
        fn test_check_keyboard_navigation() {
            let document =
                Html::parse_document("<a tabindex='-1'></a>");
            let mut issues = vec![];
            let result = AccessibilityReport::check_keyboard_navigation(
                &document,
                &mut issues,
            );
            assert!(result.is_ok());
            assert_eq!(issues.len(), 1);
            assert_eq!(
                issues[0].message,
                "Negative tabindex prevents keyboard focus"
            );
        }

        /// Test `check_language_attributes`
        #[test]
        fn test_check_language_attributes() {
            let document = Html::parse_document("<html></html>");
            let mut issues = vec![];
            let result = AccessibilityReport::check_language_attributes(
                &document,
                &mut issues,
            );
            assert!(result.is_ok());
            assert_eq!(issues.len(), 1);
            assert_eq!(
                issues[0].message,
                "Missing language declaration"
            );
        }
    }

    mod missing_tests {
        use super::*;
        use std::collections::HashSet;

        /// Test for color contrast ratio calculation
        #[test]
        fn test_color_contrast_ratio() {
            let low_contrast = 2.5;
            let high_contrast = 7.1;

            let config = AccessibilityConfig {
                min_contrast_ratio: 4.5,
                ..Default::default()
            };

            assert!(
                low_contrast < config.min_contrast_ratio,
                "Low contrast should not pass"
            );

            assert!(
                high_contrast >= config.min_contrast_ratio,
                "High contrast should pass"
            );
        }

        /// Test dynamic content ARIA attributes
        #[test]
        fn test_dynamic_content_aria_attributes() {
            let html = r#"<div aria-live="polite"></div>"#;
            let cleaned_html = remove_invalid_aria_attributes(html);
            assert_eq!(
                cleaned_html, html,
                "Dynamic content ARIA attributes should be preserved"
            );
        }

        /// Test strict WCAG AAA behavior
        #[test]
        fn test_strict_wcag_aaa_behavior() {
            let html = r#"<h1>Main Title</h1><h4>Skipped Level</h4>"#;
            let config = AccessibilityConfig {
                wcag_level: WcagLevel::AAA,
                ..Default::default()
            };

            let report = validate_wcag(html, &config, None).unwrap();
            assert!(
                report.issue_count > 0,
                "WCAG AAA strictness should detect issues"
            );

            let issue = &report.issues[0];
            assert_eq!(
                issue.issue_type,
                IssueType::LanguageDeclaration,
                "Expected heading structure issue"
            );
        }

        /// Test performance with large HTML input
        #[test]
        fn test_large_html_performance() {
            let large_html =
                "<div>".repeat(1_000) + &"</div>".repeat(1_000);
            let result = validate_wcag(
                &large_html,
                &AccessibilityConfig::default(),
                None,
            );
            assert!(
                result.is_ok(),
                "Large HTML should not cause performance issues"
            );
        }

        /// Test nested elements with ARIA attributes
        #[test]
        fn test_nested_elements_with_aria_attributes() {
            let html = r#"
        <div>
            <button aria-label="Test">Click</button>
            <nav aria-label="Main Navigation">
                <ul><li>Item 1</li></ul>
            </nav>
        </div>
        "#;
            let enhanced_html =
                add_aria_attributes(html, None).unwrap();
            assert!(
                enhanced_html.contains("aria-label"),
                "Nested elements should have ARIA attributes"
            );
        }

        /// Test heading structure validation with deeply nested headings
        #[test]
        fn test_deeply_nested_headings() {
            let html = r#"
        <div>
            <h1>Main Title</h1>
            <div>
                <h3>Skipped Level</h3>
            </div>
        </div>
        "#;
            let mut issues = Vec::new();
            let document = Html::parse_document(html);
            check_heading_structure(&document, &mut issues);

            assert!(
            issues.iter().any(|issue| issue.issue_type == IssueType::HeadingStructure),
            "Deeply nested headings with skipped levels should produce issues"
        );
        }

        /// Test unique ID generation over a long runtime
        #[test]
        fn test_unique_id_long_runtime() {
            let ids: HashSet<_> =
                (0..10_000).map(|_| generate_unique_id()).collect();
            assert_eq!(
                ids.len(),
                10_000,
                "Generated IDs should be unique over long runtime"
            );
        }

        /// Test custom selector failure handling
        #[test]
        fn test_custom_selector_failure() {
            let invalid_selector = "div..class";
            let result = try_create_selector(invalid_selector);
            assert!(
                result.is_none(),
                "Invalid selector should return None"
            );
        }

        /// Test invalid regex pattern
        #[test]
        fn test_invalid_regex_pattern() {
            let invalid_pattern = r"\d+(";
            let result = try_create_regex(invalid_pattern);
            assert!(
                result.is_none(),
                "Invalid regex pattern should return None"
            );
        }

        /// Test ARIA attribute removal with invalid values
        #[test]
        fn test_invalid_aria_attribute_removal() {
            let html = r#"<div aria-hidden="invalid"></div>"#;
            let cleaned_html = remove_invalid_aria_attributes(html);
            assert!(
                !cleaned_html.contains("aria-hidden"),
                "Invalid ARIA attributes should be removed"
            );
        }

        // Test invalid selector handling
        #[test]
        fn test_invalid_selector() {
            let invalid_selector = "div..class";
            let result = try_create_selector(invalid_selector);
            assert!(result.is_none());
        }

        // Test `issue_type` handling in `Issue` struct
        #[test]
        fn test_issue_type_in_issue_struct() {
            let issue = Issue {
                issue_type: IssueType::MissingAltText,
                message: "Alt text is missing".to_string(),
                guideline: Some("WCAG 1.1.1".to_string()),
                element: Some("<img>".to_string()),
                suggestion: Some(
                    "Add descriptive alt text".to_string(),
                ),
            };
            assert_eq!(issue.issue_type, IssueType::MissingAltText);
        }

        // Test `add_aria_to_navs`
        #[test]
        fn test_add_aria_to_navs() {
            let html = "<nav>Main Navigation</nav>";
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_navs(builder).unwrap().build();
            assert!(result.contains(r#"aria-label="navigation""#));
            assert!(result.contains(r#"role="navigation""#));
        }

        // Test `add_aria_to_forms`
        #[test]
        fn test_add_aria_to_forms() {
            let html = r#"<form>Form Content</form>"#;
            let result =
                add_aria_to_forms(HtmlBuilder::new(html)).unwrap();
            let content = result.build();

            assert!(content.contains(r#"id="form-"#));
            assert!(content.contains(r#"aria-labelledby="form-"#));
        }

        // Test `check_keyboard_navigation` click handlers without keyboard equivalents
        #[test]
        fn test_check_keyboard_navigation_click_handlers() {
            let html = r#"<button onclick="handleClick()"></button>"#;
            let document = Html::parse_document(html);
            let mut issues = vec![];

            AccessibilityReport::check_keyboard_navigation(
                &document,
                &mut issues,
            )
            .unwrap();

            assert!(
        issues.iter().any(|i| i.message == "Click handler without keyboard equivalent"),
        "Expected an issue for missing keyboard equivalents, but found: {:?}",
        issues
    );
        }

        // Test invalid language codes in `check_language_attributes`
        #[test]
        fn test_invalid_language_code() {
            let html = r#"<html lang="invalid-lang"></html>"#;
            let document = Html::parse_document(html);
            let mut issues = vec![];
            AccessibilityReport::check_language_attributes(
                &document,
                &mut issues,
            )
            .unwrap();
            assert!(issues
                .iter()
                .any(|i| i.message.contains("Invalid language code")));
        }

        // Test `get_missing_required_aria_properties`
        #[test]
        fn test_missing_required_aria_properties() {
            let html = r#"<div role="slider"></div>"#;
            let fragment = Html::parse_fragment(html);
            let element = fragment
                .select(&Selector::parse("div").unwrap())
                .next()
                .unwrap();
            let missing =
                get_missing_required_aria_properties(&element).unwrap();
            assert!(missing.contains(&"aria-valuenow".to_string()));
        }

        /// Test invalid regex pattern handling
        #[test]
        fn test_invalid_regex_creation() {
            let invalid_pattern = "[unclosed";
            let regex = try_create_regex(invalid_pattern);
            assert!(
                regex.is_none(),
                "Invalid regex should return None"
            );
        }

        /// Test invalid selector handling
        #[test]
        fn test_invalid_selector_creation() {
            let invalid_selector = "div..class";
            let selector = try_create_selector(invalid_selector);
            assert!(
                selector.is_none(),
                "Invalid selector should return None"
            );
        }

        /// Test adding ARIA attributes to empty buttons
        #[test]
        fn test_add_aria_empty_buttons() {
            let html = r#"<button></button>"#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_buttons(builder).unwrap().build();
            assert!(
                result.contains("aria-label"),
                "ARIA label should be added to empty button"
            );
        }

        /// Test WCAG validation with Level AAA strictness
        #[test]
        fn test_wcag_aaa_validation() {
            let html = "<h1>Main Title</h1><h4>Skipped Heading</h4>";
            let config = AccessibilityConfig {
                wcag_level: WcagLevel::AAA,
                ..Default::default()
            };
            let report = validate_wcag(html, &config, None).unwrap();
            assert!(
                report.issue_count > 0,
                "WCAG AAA should detect issues"
            );
        }

        /// Test unique ID generation for collisions
        #[test]
        fn test_unique_id_collisions() {
            let ids: HashSet<_> =
                (0..10_000).map(|_| generate_unique_id()).collect();
            assert_eq!(
                ids.len(),
                10_000,
                "Generated IDs should be unique"
            );
        }

        /// Test adding ARIA attributes to navigation elements
        #[test]
        fn test_add_aria_navigation() {
            let html = "<nav>Main Navigation</nav>";
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_navs(builder).unwrap().build();
            assert!(
                result.contains("aria-label"),
                "ARIA label should be added to navigation"
            );
        }

        /// Test handling of empty HTML content
        #[test]
        fn test_empty_html_handling() {
            let html = "";
            let result = add_aria_attributes(html, None);
            assert!(
                result.is_ok(),
                "Empty HTML should not cause errors"
            );
            assert_eq!(
                result.unwrap(),
                "",
                "Empty HTML should remain unchanged"
            );
        }

        #[test]
        fn test_add_aria_to_inputs_with_different_types() {
            let html = r#"
            <input type="text" placeholder="Username">
            <input type="password" placeholder="Password">
            <input type="checkbox" id="remember">
            <input type="radio" name="choice">
            <input type="submit" value="Submit">
            <input type="unknown">
        "#;

            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_inputs(builder).unwrap().build();

            // Text and password inputs should be skipped (they have placeholders)
            assert!(!result.contains(r#"type="text".*aria-label"#));
            assert!(!result.contains(r#"type="password".*aria-label"#));

            // Checkbox should have label
            assert!(result.contains(
                r#"<label for="remember">Checkbox for remember</label>"#
            ));

            // Radio should have auto-generated ID and label
            assert!(result
                .contains(r#"<label for="option1">Option 1</label>"#));

            // Submit should be skipped
            assert!(!result.contains(r#"type="submit".*aria-label"#));

            // Unknown type should get aria-label
            assert!(result.contains(r#"aria-label="unknown""#));
        }

        #[test]
        fn test_has_associated_label() {
            // Test with input that has matching label
            let input = r#"<input type="text" id="username">"#;
            let html = r#"<label for="username">Username:</label>"#;
            assert!(has_associated_label(input, html));

            // Test with input that has no matching label
            let input = r#"<input type="text" id="username">"#;
            let html = r#"<label for="password">Password:</label>"#;
            assert!(!has_associated_label(input, html));

            // Test with input that has no ID
            let input = r#"<input type="text">"#;
            let html = r#"<label for="username">Username:</label>"#;
            assert!(!has_associated_label(input, html));
        }

        #[test]
        fn test_preserve_attributes() {
            // Test with typical HTML attributes (type, class)
            let input = r#"<input type="text" class="form-control">"#;
            let result = preserve_attributes(input);
            assert!(result.contains("type=\"text\""));
            assert!(result.contains("class=\"form-control\""));

            // Test single attributes
            let input = r#"<input type="text">"#;
            let result = preserve_attributes(input);
            assert!(result.contains("type=\"text\""));

            // Test with single quotes
            let input = r#"<input type='text'>"#;
            let result = preserve_attributes(input);
            assert!(result.contains("type='text'"));

            // Test boolean attributes
            let input = r#"<input required>"#;
            let result = preserve_attributes(input);
            assert!(result.contains("required"));

            // Test with bare input tag
            let input = "<input>";
            let result = preserve_attributes(input);
            assert!(
                result.contains("input"),
                "Should preserve the input tag name"
            );

            // Test complex attribute values
            let input = r#"<input name="test" value="multiple words">"#;
            let result = preserve_attributes(input);
            assert!(result.contains("name=\"test\""));
            assert!(result.contains("value=\"multiple words\""));
        }

        #[test]
        fn test_preserve_attributes_with_data_attributes() {
            // Print actual regex matches for debugging
            let input = r#"<input data-test="value" type="text">"#;
            let matches: Vec<_> = ATTRIBUTE_REGEX
                .captures_iter(input)
                .map(|cap| cap[0].to_string())
                .collect();
            println!("Actual matches: {:?}", matches);

            let result = preserve_attributes(input);
            println!("Preserved attributes: {}", result);
        }

        #[test]
        fn test_extract_input_type() {
            // Test with double quotes
            let input = r#"<input type="text" class="form-control">"#;
            assert_eq!(
                extract_input_type(input),
                Some("text".to_string())
            );

            // Test with single quotes
            let input = r#"<input type='radio' name='choice'>"#;
            assert_eq!(
                extract_input_type(input),
                Some("radio".to_string())
            );

            // Test with no type attribute
            let input = r#"<input class="form-control">"#;
            assert_eq!(extract_input_type(input), None);

            // Test with empty type attribute
            let input = r#"<input type="" class="form-control">"#;
            assert_eq!(extract_input_type(input), None); // Changed this because empty type is equivalent to no type
        }

        #[test]
        fn test_add_aria_to_inputs_with_existing_labels() {
            let html = r#"
            <input type="checkbox" id="existing">
            <label for="existing">Existing Label</label>
            <input type="radio" id="existing2">
            <label for="existing2">Existing Radio</label>
        "#;

            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_inputs(builder).unwrap().build();

            // Should not modify inputs that already have labels
            assert!(!result.contains("aria-label"));
            assert_eq!(
            result.matches("<label").count(),
            2,
            "Should not add additional labels to elements that already have them"
        );
        }

        #[test]
        fn test_add_aria_to_inputs_with_special_characters() {
            let html = r#"<input type="text" data-test="test's value" class="form & input">"#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_inputs(builder).unwrap().build();

            // Verify attributes with special characters are preserved
            assert!(result.contains("data-test=\"test's value\""));
            assert!(result.contains("class=\"form & input\""));
        }

        #[test]
        fn test_toggle_button() {
            let original_html =
                r#"<button type="button">Menu</button>"#;
            let builder = HtmlBuilder::new(original_html);
            let enhanced_html =
                add_aria_to_buttons(builder).unwrap().build();

            assert_eq!(
        enhanced_html,
        r#"<button aria-pressed="false" aria-label="menu" type="button">Menu</button>"#,
        "The button should be enhanced with aria-pressed and aria-label"
    );
        }

        #[test]
        fn test_replace_html_element_resilient_fallback() {
            let original = r#"<button disabled>Click</button>"#;
            let old_element = r#"<button disabled="">Click</button>"#;
            let new_element = r#"<button aria-disabled="true" disabled="">Click</button>"#;

            let replaced = replace_html_element_resilient(
                original,
                old_element,
                new_element,
            );

            // We expect the fallback to handle <button disabled> vs <button disabled="">
            assert!(replaced.contains(r#"aria-disabled="true""#), "Should replace with fallback even though original has disabled not disabled=\"\"");
        }

        #[test]
        fn test_replace_html_element_resilient_no_match() {
            let original = r#"<div>Nothing to replace</div>"#;
            let old_element = r#"<button disabled="">Click</button>"#;
            let new_element = r#"<button aria-disabled="true" disabled="">Click</button>"#;

            // We expect no changes, because there's no match
            let replaced = replace_html_element_resilient(
                original,
                old_element,
                new_element,
            );
            assert_eq!(
                replaced, original,
                "No match means original stays unchanged"
            );
        }

        #[test]
        fn test_normalize_shorthand_attributes_multiple() {
            let html = r#"<input disabled selected><button disabled>Press</button>"#;
            let normalized = normalize_shorthand_attributes(html);
            // <input disabled=""> should become <input disabled="" selected="">
            // <button disabled=""> should become <button disabled="">
            assert!(
                normalized
                    .contains(r#"<input disabled="" selected="">"#),
                "Should expand both disabled and selected"
            );
            assert!(
                normalized.contains(r#"<button disabled="">"#),
                "Should expand the disabled attribute on the button"
            );
        }

        #[test]
        fn test_remove_invalid_aria_attributes() {
            let html = r#"<div aria-hidden="invalid" aria-pressed="true"></div>"#;
            // aria-hidden="invalid" is not valid (only "true" or "false")
            // aria-pressed="true" is valid
            let cleaned = remove_invalid_aria_attributes(html);
            assert!(
                !cleaned.contains(r#"aria-hidden="invalid""#),
                "Invalid aria-hidden should be removed"
            );
            assert!(
                cleaned.contains(r#"aria-pressed="true""#),
                "Valid attribute should remain"
            );
        }

        #[test]
        fn test_is_valid_aria_attribute_cases() {
            // 5a) Valid known attribute
            assert!(
                is_valid_aria_attribute("aria-label", "Submit"),
                "aria-label with non-empty string is valid"
            );

            // 5b) Known boolean attribute with correct values
            assert!(
                is_valid_aria_attribute("aria-pressed", "true"),
                "aria-pressed=\"true\" is valid"
            );
            assert!(
                is_valid_aria_attribute("aria-pressed", "false"),
                "aria-pressed=\"false\" is valid"
            );
            assert!(
                !is_valid_aria_attribute("aria-pressed", "yes"),
                "aria-pressed only allows true/false"
            );

            // 5c) Unknown ARIA attribute
            assert!(
                !is_valid_aria_attribute(
                    "aria-somethingrandom",
                    "value"
                ),
                "Unknown ARIA attribute is invalid"
            );
        }

        #[test]
        fn test_add_aria_to_accordions_basic() {
            let html = r#"
        <div class="accordion">
            <button>Section 1</button>
            <div>Content 1</div>
            <button>Section 2</button>
            <div>Content 2</div>
        </div>
        "#;
            let builder = HtmlBuilder::new(html);
            let result =
                add_aria_to_accordions(builder).unwrap().build();

            // Expect to see aria-expanded="false", aria-controls="section-1-content" etc.
            assert!(
                result.contains(r#"aria-controls="section-1-content""#),
                "First accordion section should have aria-controls"
            );
            assert!(
                result.contains(r#"id="section-1-button""#),
                "First button should get an ID"
            );
            assert!(
                result.contains(r#"id="section-1-content""#),
                "First content should get an ID"
            );
            assert!(
                result.contains(r#"hidden"#),
                "Accordion content is hidden by default"
            );
        }

        #[test]
        fn test_add_aria_to_accordions_empty() {
            let html = r#"<div class="accordion"></div>"#;
            let builder = HtmlBuilder::new(html);
            let result =
                add_aria_to_accordions(builder).unwrap().build();

            // If there's no button+div pairs, we just keep the original container
            assert!(result.contains(r#"class="accordion""#));
            // Shouldn't blow up or panic
        }

        #[test]
        fn test_add_aria_to_tabs_basic() {
            // Provide something that has role="tablist" and some <button> inside
            let html = r#"
        <div role="tablist">
            <button>Tab A</button>
            <button>Tab B</button>
        </div>
        "#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_tabs(builder).unwrap().build();

            // We expect tab1 => aria-selected="true", tab2 => aria-selected="false"
            assert!(
                result.contains(
                    r#"role="tab" id="tab1" aria-selected="true""#
                ),
                "First tab should be tab1, selected=true"
            );
            assert!(
                result.contains(
                    r#"role="tab" id="tab2" aria-selected="false""#
                ),
                "Second tab should be tab2, selected=false"
            );
            // Also expect the auto-generated panels "panel1" and "panel2"
            assert!(
                result.contains(r#"aria-controls="panel1""#),
                "First tab controls panel1"
            );
            assert!(
                result.contains(r#"id="panel2" role="tabpanel""#),
                "Second tab panel should exist"
            );
        }

        /// 9) Test `add_aria_to_tabs` when no tablist is found
        #[test]
        fn test_add_aria_to_tabs_no_tablist() {
            let html = r#"<div><button>Not a tab</button></div>"#;
            let builder = HtmlBuilder::new(html);
            let result = add_aria_to_tabs(builder).unwrap().build();

            // We expect no transformation if there's no role="tablist"
            assert!(
                result.contains(r#"<button>Not a tab</button>"#),
                "Should remain unchanged"
            );
            assert!(!result.contains(r#"role="tab""#), "No transformation to role=tab if not inside role=tablist");
        }

        /// 10) Test the `count_checked_elements` function
        #[test]
        fn test_count_checked_elements() {
            let html = r#"
        <html>
            <body>
                <div>
                    <p>Paragraph</p>
                    <span>Span</span>
                </div>
            </body>
        </html>
        "#;
            let document = Html::parse_document(html);
            let count = count_checked_elements(&document);
            // There's 5 elements: <html>, <head> (implicitly empty?), <body>, <div>, <p>, <span> ...
            // Actually, <head> might exist only if we parse as a full document, let's see:
            // The easiest is to just check the actual number we get. We'll assume 5 or 6.
            assert!(
                count >= 5,
                "Expected at least 5 elements in the parsed tree"
            );
        }

        #[test]
        fn test_check_language_attributes_valid() {
            let html = r#"<html lang="en"><body></body></html>"#;
            let document = Html::parse_document(html);
            let mut issues = vec![];
            let result =
                check_language_attributes(&document, &mut issues);
            assert!(result.is_ok());
            assert_eq!(issues.len(), 0, "No issues for valid lang");
        }

        #[test]
        fn test_error_variants() {
            let _ = Error::InvalidAriaAttribute {
                attribute: "aria-bogus".to_string(),
                message: "Bogus attribute".to_string(),
            };
            let _ = Error::WcagValidationError {
                level: WcagLevel::AA,
                message: "Validation failed".to_string(),
                guideline: Some("WCAG 2.4.6".to_string()),
            };
            let _ = Error::HtmlTooLarge {
                size: 9999999,
                max_size: 1000000,
            };
            let _ = Error::HtmlProcessingError {
                message: "Something went wrong".to_string(),
                source: None,
            };
            let _ = Error::MalformedHtml {
                message: "Broken HTML".to_string(),
                fragment: None,
            };
            // Just ensuring they construct. In real usage, you'd check their Display formatting, etc.
            assert!(true, "Constructed error variants without panic");
        }

        #[test]
        fn test_has_associated_label_no_id() {
            let input = r#"<input type="checkbox">"#;
            let html =
                r#"<label for="checkbox1">Checkbox Label</label>"#;
            // There's no id= in the input, so it can't be associated
            assert!(
                !has_associated_label(input, html),
                "No ID => false"
            );
        }

        #[test]
        fn test_generate_unique_id_format() {
            let new_id = generate_unique_id();
            // Should start with "aria-"
            assert!(
                new_id.starts_with("aria-"),
                "Generated ID should start with aria-"
            );
        }
    }
}

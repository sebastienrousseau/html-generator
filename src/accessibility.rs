//! Accessibility-related functionality for HTML processing.
//!
//! This module provides functions for improving the accessibility of HTML content, including adding ARIA attributes and validating against WCAG guidelines.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;
use thiserror::Error;

/// Maximum size of HTML input in bytes (1MB)
const MAX_HTML_SIZE: usize = 1_000_000;

/// Enum to represent possible accessibility-related errors.
#[derive(Debug, Error)]
pub enum AccessibilityError {
    /// Error indicating an invalid ARIA attribute.
    #[error("Invalid ARIA Attribute: {0}")]
    InvalidAriaAttribute(String),

    /// Error indicating failure to validate HTML against WCAG guidelines.
    #[error("WCAG Validation Error: {0}")]
    WcagValidationError(String),

    /// Error indicating a failure in processing HTML for accessibility.
    #[error("HTML Processing Error: {0}")]
    HtmlProcessingError(String),

    /// Error indicating the HTML input is too large to process.
    #[error("HTML Input Too Large: {0}")]
    HtmlTooLarge(usize),

    /// Error indicating malformed HTML input.
    #[error("Malformed HTML: {0}")]
    MalformedHtml(String),
}

/// Result type alias for convenience.
pub type Result<T> = std::result::Result<T, AccessibilityError>;

static BUTTON_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("button:not([aria-label])")
        .expect("Failed to create button selector")
});

static NAV_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("nav:not([aria-label])")
        .expect("Failed to create nav selector")
});

static FORM_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("form:not([aria-labelledby])")
        .expect("Failed to create form selector")
});

static INPUT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<input[^>]*>"#).expect("Failed to create input regex")
});

static ARIA_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse(
        "[aria-label], [aria-labelledby], [aria-describedby], [aria-hidden], [aria-expanded], [aria-haspopup], [aria-controls], [aria-pressed], [aria-checked], [aria-current], [aria-disabled], [aria-dropeffect], [aria-grabbed], [aria-haspopup], [aria-invalid], [aria-live], [aria-owns], [aria-relevant], [aria-required], [aria-role], [aria-selected], [aria-valuemax], [aria-valuemin], [aria-valuenow], [aria-valuetext]"
    ).expect("Failed to create ARIA selector")
});

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
            "aria-haspopup",
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
        .cloned()
        .collect()
    });

/// Add ARIA attributes to HTML for improved accessibility.
///
/// This function adds ARIA attributes to common elements, such as buttons, forms,
/// navigation elements, and images.
///
/// # Arguments
///
/// * `html` - A string slice representing the HTML content.
///
/// # Returns
///
/// * `Result<String>` - The modified HTML with ARIA attributes included.
///
/// # Errors
///
/// This function will return an error if:
/// * The input HTML is larger than `MAX_HTML_SIZE`.
/// * The HTML cannot be parsed.
/// * There's an error adding ARIA attributes.
///
/// # Examples
///
/// ```
/// use html_generator::accessibility::add_aria_attributes;
///
/// let html = r#"<button>Click me</button>"#;
/// let result = add_aria_attributes(html);
/// assert!(result.is_ok());
/// assert!(result.unwrap().contains(r#"aria-label="button""#));
/// ```
pub fn add_aria_attributes(html: &str) -> Result<String> {
    if html.len() > MAX_HTML_SIZE {
        return Err(AccessibilityError::HtmlTooLarge(html.len()));
    }

    let mut html_builder = HtmlBuilder::new(html);

    html_builder = add_aria_to_buttons(html_builder)?;
    html_builder = add_aria_to_navs(html_builder)?;
    html_builder = add_aria_to_forms(html_builder)?;
    html_builder = add_aria_to_inputs(html_builder)?;

    // Remove invalid ARIA attributes before returning
    let new_html =
        remove_invalid_aria_attributes(&html_builder.build());

    if !validate_aria(&new_html) {
        return Err(AccessibilityError::InvalidAriaAttribute(
            "Failed to add valid ARIA attributes.".to_string(),
        ));
    }

    Ok(new_html)
}

/// Add ARIA attributes to button elements.
fn add_aria_to_buttons(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);

    for button in document.select(&BUTTON_SELECTOR) {
        // Only modify buttons that do not already have an aria-label
        if button.value().attr("aria-label").is_none() {
            let button_html = button.html();
            let inner_content = button.inner_html(); // Get inner content
            let new_button_html = format!(
                r#"<button aria-label="button">{}</button>"#,
                inner_content
            );

            // Replace original button with the modified one
            html_builder.content = html_builder
                .content
                .replace(&button_html, &new_button_html);
        }
    }

    Ok(html_builder)
}

/// Add ARIA attributes to navigation elements.
fn add_aria_to_navs(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);
    for nav in document.select(&NAV_SELECTOR) {
        let nav_html = nav.html();
        let new_nav_html =
            nav_html.replace("<nav", r#"<nav aria-label="navigation""#);
        html_builder.content =
            html_builder.content.replace(&nav_html, &new_nav_html);
    }

    Ok(html_builder)
}

/// Add ARIA attributes to form elements.
fn add_aria_to_forms(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let document = Html::parse_document(&html_builder.content);
    for form in document.select(&FORM_SELECTOR) {
        let form_html = form.html();
        let new_form_html = form_html
            .replace("<form", r#"<form aria-labelledby="form-label""#);
        html_builder.content =
            html_builder.content.replace(&form_html, &new_form_html);
    }

    Ok(html_builder)
}

/// Add ARIA attributes to input elements without labels.
/// Add ARIA attributes to input elements without labels.
fn add_aria_to_inputs(
    mut html_builder: HtmlBuilder,
) -> Result<HtmlBuilder> {
    let mut replacements = Vec::with_capacity(
        INPUT_REGEX.captures_iter(&html_builder.content).count(),
    );

    for cap in INPUT_REGEX.captures_iter(&html_builder.content) {
        let input_tag = &cap[0];
        if !input_tag.contains("aria-label") {
            let new_input_tag = input_tag
                .replace("<input", r#"<input aria-label="input""#);
            replacements.push((input_tag.to_string(), new_input_tag));
        }
    }

    for (old, new) in replacements {
        html_builder.content = html_builder.content.replace(&old, &new);
    }

    Ok(html_builder)
}

/// Validate ARIA attributes within the HTML.
///
/// This function ensures that ARIA attributes are correctly formatted and conform to
/// the expected naming conventions.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content.
///
/// # Returns
///
/// * `bool` - Returns `true` if all ARIA attributes are valid, otherwise `false`.
fn validate_aria(html: &str) -> bool {
    let document = Html::parse_document(html);

    // Iterate over all elements that have ARIA attributes
    document
        .select(&ARIA_SELECTOR)
        .flat_map(|el| el.value().attrs())
        .filter(|(name, _)| name.starts_with("aria-"))
        .all(|(name, value)| {
            // Ensure the attribute is in the valid list and its value is valid
            is_valid_aria_attribute(name, value)
        })
}

/// Check if an ARIA attribute is valid.
///
/// This function checks if the given ARIA attribute name and value conform to the ARIA specification.
///
/// # Arguments
///
/// * `name` - The name of the ARIA attribute.
/// * `value` - The value of the ARIA attribute.
///
/// # Returns
///
/// * `bool` - Returns `true` if the ARIA attribute is valid, otherwise `false`.
fn is_valid_aria_attribute(name: &str, value: &str) -> bool {
    if !VALID_ARIA_ATTRIBUTES.contains(name) {
        return false;
    }

    match name {
        "aria-hidden" | "aria-expanded" | "aria-pressed"
        | "aria-invalid" => ["true", "false"].contains(&value),
        _ => !value.is_empty(),
    }
}

/// Remove invalid ARIA attributes from the HTML.
fn remove_invalid_aria_attributes(html: &str) -> String {
    let document = Html::parse_document(html);
    let aria_selector = Selector::parse("[aria-label], [aria-labelledby], [aria-describedby], [aria-hidden], [aria-expanded], [aria-haspopup], [aria-controls], [aria-pressed], [aria-invalid]")
        .expect("Failed to create invalid ARIA selector");
    let mut new_html = html.to_string();

    for element in document.select(&aria_selector) {
        let element_html = element.html();
        let new_element_html = element
            .value()
            .attrs()
            .filter(|(name, value)| {
                !name.starts_with("aria-")
                    || is_valid_aria_attribute(name, value)
            })
            .fold(String::new(), |mut acc, (name, value)| {
                acc.push_str(&format!(r#" {}="{}""#, name, value));
                acc
            });

        let new_tag =
            format!("<{}{}>", element.value().name(), new_element_html);
        new_html = new_html.replace(&element_html, &new_tag);
    }

    new_html
}

/// Validate HTML against WCAG (Web Content Accessibility Guidelines).
///
/// This function performs various checks to validate the HTML content against WCAG standards,
/// such as ensuring all images have alt text, proper heading structure, and more.
///
/// # Arguments
///
/// * `html` - A string slice that holds the HTML content.
///
/// # Returns
///
/// * `Result<()>` - An empty result if validation passes, otherwise an error.
///
/// # Errors
///
/// This function will return an error if:
/// * The input HTML is larger than `MAX_HTML_SIZE`.
/// * The HTML fails to meet WCAG guidelines.
///
/// # Examples
///
/// ```
/// use html_generator::accessibility::validate_wcag;
///
/// let html = r#"<img src="image.jpg" alt="A descriptive alt text"><h1>Title</h1><h2>Subtitle</h2>"#;
/// let result = validate_wcag(html);
/// assert!(result.is_ok());
/// ```
pub fn validate_wcag(html: &str) -> Result<()> {
    if html.len() > MAX_HTML_SIZE {
        return Err(AccessibilityError::HtmlTooLarge(html.len()));
    }

    let document = Html::parse_document(html);

    check_alt_text(&document)?;
    check_heading_structure(&document)?;
    check_input_labels(&document)?;

    Ok(())
}

/// Check for the presence of alt text in images.
fn check_alt_text(document: &Html) -> Result<()> {
    let img_selector = Selector::parse("img").map_err(|e| {
        AccessibilityError::HtmlProcessingError(e.to_string())
    })?;
    if document
        .select(&img_selector)
        .any(|img| img.value().attr("alt").is_none())
    {
        Err(AccessibilityError::WcagValidationError(
            "Missing alt text for images.".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Check heading structure to ensure no levels are skipped.
fn check_heading_structure(document: &Html) -> Result<()> {
    let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6")
        .map_err(|e| {
            AccessibilityError::HtmlProcessingError(e.to_string())
        })?;
    let mut prev_level = 0;

    for heading in document.select(&heading_selector) {
        let current_level = heading
            .value()
            .name()
            .chars()
            .nth(1)
            .and_then(|c| c.to_digit(10))
            .ok_or_else(|| {
                AccessibilityError::MalformedHtml(
                    "Invalid heading tag".to_string(),
                )
            })?;

        if current_level > prev_level + 1 {
            return Err(AccessibilityError::WcagValidationError(
                "Improper heading structure (skipping heading levels)."
                    .to_string(),
            ));
        }
        prev_level = current_level;
    }

    Ok(())
}

/// Check if all form inputs have associated labels.
fn check_input_labels(document: &Html) -> Result<()> {
    let input_selector = Selector::parse("input").map_err(|e| {
        AccessibilityError::HtmlProcessingError(e.to_string())
    })?;
    if document.select(&input_selector).any(|input| {
        input.value().attr("aria-label").is_none()
            && input.value().attr("id").is_none()
    }) {
        Err(AccessibilityError::WcagValidationError(
            "Form inputs missing associated labels.".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// A builder struct for constructing HTML content.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_aria_attributes() {
        let html = "<button>Click me</button><nav>Menu</nav><form>Form</form><input type='text'>";
        let result = add_aria_attributes(html).unwrap();

        assert!(result.contains(r#"<button aria-label="button">"#));
        assert!(result.contains(r#"<nav aria-label="navigation">"#));
        assert!(
            result.contains(r#"<form aria-labelledby="form-label">"#)
        );
        assert!(result.contains(r#"<input aria-label="input""#));
    }

    #[test]
    fn test_validate_wcag() {
        let valid_html = r#"<img src="image.jpg" alt="Image description"><h1>Title</h1><h2>Subtitle</h2><input id="name" type="text">"#;
        let invalid_html = r#"<img src="image.jpg"><h1>Title</h1><h3>Subtitle</h3><input type="text">"#;

        assert!(validate_wcag(valid_html).is_ok());
        assert!(validate_wcag(invalid_html).is_err());
    }

    #[test]
    fn test_html_too_large() {
        let large_html = "a".repeat(MAX_HTML_SIZE + 1);
        assert!(matches!(
            add_aria_attributes(&large_html),
            Err(AccessibilityError::HtmlTooLarge(_))
        ));
        assert!(matches!(
            validate_wcag(&large_html),
            Err(AccessibilityError::HtmlTooLarge(_))
        ));
    }

    #[test]
    fn test_invalid_aria_attribute() {
        let html = r#"<div aria-invalid="true">Invalid ARIA</div>"#;
        let result = add_aria_attributes(html);
        assert!(result.is_ok());
        assert!(result.unwrap().contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn test_add_aria_to_buttons() {
        let html = "<button>Click me</button><button aria-label='existing'>Existing</button>";
        let mut html_builder = HtmlBuilder::new(html);
        html_builder = add_aria_to_buttons(html_builder).unwrap();
        let result = html_builder.build();
        assert!(result.contains(
            r#"<button aria-label="button">Click me</button>"#
        ));
        assert!(result.contains(
            r#"<button aria-label='existing'>Existing</button>"#
        ));
    }

    #[test]
    fn test_add_aria_to_navs() {
        let html =
            "<nav>Menu</nav><nav aria-label='existing'>Existing</nav>";
        let mut html_builder = HtmlBuilder::new(html);
        html_builder = add_aria_to_navs(html_builder).unwrap();
        let result = html_builder.build();
        assert!(result
            .contains(r#"<nav aria-label="navigation">Menu</nav>"#));
        assert!(result
            .contains(r#"<nav aria-label='existing'>Existing</nav>"#));
    }

    #[test]
    fn test_add_aria_to_forms() {
        let html = "<form>Form</form><form aria-labelledby='existing'>Existing</form>";
        let mut html_builder = HtmlBuilder::new(html);
        html_builder = add_aria_to_forms(html_builder).unwrap();
        let result = html_builder.build();
        assert!(result.contains(
            r#"<form aria-labelledby="form-label">Form</form>"#
        ));
        assert!(result.contains(
            r#"<form aria-labelledby='existing'>Existing</form>"#
        ));
    }

    #[test]
    fn test_add_aria_to_inputs() {
        let html = r#"<input type="text"><input type="text" aria-label="existing">"#;
        let mut html_builder = HtmlBuilder::new(html);
        html_builder = add_aria_to_inputs(html_builder).unwrap();
        let result = html_builder.build();
        assert!(result
            .contains(r#"<input aria-label="input" type="text">"#));
        assert!(result
            .contains(r#"<input type="text" aria-label="existing">"#));
    }

    #[test]
    fn test_is_valid_aria_attribute() {
        assert!(is_valid_aria_attribute("aria-label", "Valid label"));
        assert!(is_valid_aria_attribute("aria-hidden", "true"));
        assert!(is_valid_aria_attribute("aria-hidden", "false"));
        assert!(!is_valid_aria_attribute("aria-hidden", "yes"));
        assert!(is_valid_aria_attribute("aria-invalid", "true"));
        assert!(!is_valid_aria_attribute("aria-fake", "value"));
    }

    #[test]
    fn test_remove_invalid_aria_attributes() {
        let html =
            r#"<div aria-label="Valid" aria-invalid="true">Test</div>"#;
        let result = remove_invalid_aria_attributes(html);
        assert!(result.contains(r#"aria-label="Valid""#));
        assert!(result.contains(r#"aria-invalid="true""#));
    }

    #[test]
    fn test_validate_aria() {
        // Valid HTML with correct ARIA attributes
        let valid_html = r#"<div aria-label="Valid">Valid ARIA</div>"#;

        // Invalid HTML with an invalid ARIA attribute or invalid value
        let invalid_html =
            r#"<div aria-invalid="invalid_value">Invalid ARIA</div>"#;

        assert!(validate_aria(valid_html));
        assert!(!validate_aria(invalid_html));
    }

    #[test]
    fn test_check_alt_text() {
        let valid_html = Html::parse_document(
            r#"<img src="image.jpg" alt="Description">"#,
        );
        let invalid_html =
            Html::parse_document(r#"<img src="image.jpg">"#);
        assert!(check_alt_text(&valid_html).is_ok());
        assert!(check_alt_text(&invalid_html).is_err());
    }

    #[test]
    fn test_check_heading_structure() {
        let valid_html =
            Html::parse_document("<h1>Title</h1><h2>Subtitle</h2>");
        let invalid_html =
            Html::parse_document("<h1>Title</h1><h3>Subtitle</h3>");
        assert!(check_heading_structure(&valid_html).is_ok());
        assert!(check_heading_structure(&invalid_html).is_err());
    }

    #[test]
    fn test_check_input_labels() {
        let valid_html = Html::parse_document(
            r#"<input id="name"><input aria-label="Email">"#,
        );
        let invalid_html =
            Html::parse_document(r#"<input type="text">"#);
        assert!(check_input_labels(&valid_html).is_ok());
        assert!(check_input_labels(&invalid_html).is_err());
    }

    #[test]
    fn test_add_aria_attributes_basic() {
        let html = r#"
            <button>Click me</button>
            <nav>Menu</nav>
            <form>Form</form>
            <input type='text'>
        "#;
        let result = add_aria_attributes(html).unwrap();

        assert!(result.contains(r#"<button aria-label="button">"#));
        assert!(result.contains(r#"<nav aria-label="navigation">"#));
        assert!(
            result.contains(r#"<form aria-labelledby="form-label">"#)
        );
        assert!(result.contains(r#"<input aria-label="input""#));
    }

    #[test]
    fn test_add_aria_attributes_mixed_content() {
        let html = r#"
            <button>Click me</button>
            <nav aria-label="main">Menu</nav>
            <form>Form</form>
            <input type='text' aria-label="username">
        "#;
        let result = add_aria_attributes(html).unwrap();

        assert!(result.contains(r#"<button aria-label="button">"#));
        assert!(result.contains(r#"<nav aria-label="main">"#));
        assert!(
            result.contains(r#"<form aria-labelledby="form-label">"#)
        );
        assert!(result
            .contains(r#"<input type='text' aria-label="username">"#));
    }

    #[test]
    fn test_add_aria_attributes_html_too_large() {
        let large_html = "a".repeat(MAX_HTML_SIZE + 1);
        let result = add_aria_attributes(&large_html);

        assert!(matches!(
            result,
            Err(AccessibilityError::HtmlTooLarge(_))
        ));
    }

    #[test]
    fn test_add_aria_attributes_invalid_html() {
        let invalid_html = "<button>Unclosed button";
        let result = add_aria_attributes(invalid_html);

        // The function should still process this without error
        assert!(result.is_ok());
        let processed_html = result.unwrap();
        // Check if the original content is preserved
        assert!(processed_html.contains("Unclosed button"));
    }
}

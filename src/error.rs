//! Error types for HTML generation and processing.
//!
//! This module defines custom error types used throughout the HTML generation library.
//! It provides a centralized location for all error definitions, making it easier to manage and handle errors consistently across the codebase.

use std::io;
use thiserror::Error;

/// Enum to represent various errors that can occur during HTML generation, processing, or optimization.
#[derive(Error, Debug)]
pub enum HtmlError {
    /// Error that occurs when a regular expression fails to compile.
    ///
    /// This variant contains the underlying error from the `regex` crate.
    #[error("Failed to compile regex: {0}")]
    RegexCompilationError(#[from] regex::Error),

    /// Error indicating failure in extracting front matter from the input content.
    ///
    /// This variant is used when there is an issue parsing the front matter of a document.
    /// The associated string provides details about the error.
    #[error("Failed to extract front matter: {0}")]
    FrontMatterExtractionError(String),

    /// Error indicating a failure in formatting an HTML header.
    ///
    /// This variant is used when the header cannot be formatted correctly. The associated string provides more details.
    #[error("Failed to format header: {0}")]
    HeaderFormattingError(String),

    /// Error that occurs when parsing a selector fails.
    ///
    /// This variant is used when a CSS or HTML selector cannot be parsed.
    /// The first string is the selector, and the second string provides additional context.
    #[error("Failed to parse selector '{0}': {1}")]
    SelectorParseError(String, String),

    /// Error indicating failure to minify HTML content.
    ///
    /// This variant is used when there is an issue during the HTML minification process. The associated string provides details.
    #[error("Failed to minify HTML: {0}")]
    MinificationError(String),

    /// Error that occurs during the conversion of Markdown to HTML.
    ///
    /// This variant is used when the Markdown conversion process encounters an issue. The associated string provides more information.
    #[error("Failed to convert Markdown to HTML: {message}")]
    MarkdownConversion {
        /// The error message
        message: String,
        /// The source error, if available
        #[source]
        source: Option<io::Error>,
    },

    /// Errors that occur during HTML minification.
    #[error("HTML minification failed: {message}")]
    Minification {
        /// The error message
        message: String,
        /// The source error, if available
        size: Option<usize>,
        /// The source error, if available
        #[source]
        source: Option<io::Error>,
    },

    /// SEO-related errors.
    #[error("SEO optimization failed: {kind}: {message}")]
    Seo {
        /// The kind of SEO error
        kind: SeoErrorKind,
        /// The error message
        message: String,
        /// The problematic element, if available
        element: Option<String>,
    },

    /// Accessibility-related errors.
    #[error("Accessibility check failed: {kind}: {message}")]
    Accessibility {
        /// The kind of accessibility error
        kind: AccessibilityErrorKind,
        /// The error message
        message: String,
        /// The relevant WCAG guideline, if available
        wcag_guideline: Option<String>,
    },

    /// Error indicating that a required HTML element is missing.
    ///
    /// This variant is used when a necessary HTML element (like a title tag) is not found.
    #[error("Missing required HTML element: {0}")]
    MissingHtmlElement(String),

    /// Error that occurs when structured data is invalid.
    ///
    /// This variant is used when JSON-LD or other structured data does not meet the expected format or requirements.
    #[error("Invalid structured data: {0}")]
    InvalidStructuredData(String),

    /// Input/Output errors
    ///
    /// This variant is used when an IO operation fails (e.g., reading or writing files).
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error indicating an invalid input.
    ///
    /// This variant is used when the input content is invalid or does not meet the expected criteria.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Error indicating an invalid front matter format.
    ///
    /// This variant is used when the front matter of a document does not follow the expected format.
    #[error("Invalid front matter format: {0}")]
    InvalidFrontMatterFormat(String),

    /// Error indicating an input that is too large.
    ///
    /// This variant is used when the input content exceeds a certain size limit.
    #[error("Input too large: size {0} bytes")]
    InputTooLarge(usize),

    /// Error indicating an invalid header format.
    ///
    /// This variant is used when an HTML header does not conform to the expected format.
    #[error("Invalid header format: {0}")]
    InvalidHeaderFormat(String),

    /// Error that occurs when converting from UTF-8 fails.
    ///
    /// This variant wraps errors that occur when converting a byte sequence to a UTF-8 string.
    #[error("UTF-8 conversion error: {0}")]
    Utf8ConversionError(#[from] std::string::FromUtf8Error),

    /// Error indicating a failure during parsing.
    ///
    /// This variant is used for general parsing errors where the specific source of the issue isn't covered by other variants.
    #[error("Parsing error: {0}")]
    ParsingError(String),

    /// Errors that occur during template rendering.
    #[error("Template rendering failed: {message}")]
    TemplateRendering {
        /// The error message
        message: String,
        /// The source error, if available
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Error indicating a validation failure.
    ///
    /// This variant is used when a validation step fails, such as schema validation or data integrity checks.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// A catch-all error for unexpected failures.
    ///
    /// This variant is used for errors that do not fit into other categories.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Types of SEO-related errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SeoErrorKind {
    /// Missing required meta tags
    MissingMetaTags,
    /// Invalid structured data
    InvalidStructuredData,
    /// Missing title
    MissingTitle,
    /// Missing description
    MissingDescription,
    /// Other SEO-related errors
    Other,
}

/// Types of accessibility-related errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AccessibilityErrorKind {
    /// Missing ARIA attributes
    MissingAriaAttributes,
    /// Invalid ARIA attribute values
    InvalidAriaValue,
    /// Missing alternative text
    MissingAltText,
    /// Incorrect heading structure
    HeadingStructure,
    /// Missing form labels
    MissingFormLabels,
    /// Other accessibility-related errors
    Other,
}

impl std::fmt::Display for AccessibilityErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessibilityErrorKind::MissingAriaAttributes => {
                write!(f, "Missing ARIA attributes")
            }
            AccessibilityErrorKind::InvalidAriaValue => {
                write!(f, "Invalid ARIA attribute values")
            }
            AccessibilityErrorKind::MissingAltText => {
                write!(f, "Missing alternative text")
            }
            AccessibilityErrorKind::HeadingStructure => {
                write!(f, "Incorrect heading structure")
            }
            AccessibilityErrorKind::MissingFormLabels => {
                write!(f, "Missing form labels")
            }
            AccessibilityErrorKind::Other => {
                write!(f, "Other accessibility-related errors")
            }
        }
    }
}

impl std::fmt::Display for SeoErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeoErrorKind::MissingMetaTags => {
                write!(f, "Missing required meta tags")
            }
            SeoErrorKind::InvalidStructuredData => {
                write!(f, "Invalid structured data")
            }
            SeoErrorKind::MissingTitle => write!(f, "Missing title"),
            SeoErrorKind::MissingDescription => {
                write!(f, "Missing description")
            }
            SeoErrorKind::Other => {
                write!(f, "Other SEO-related errors")
            }
        }
    }
}

impl HtmlError {
    /// Creates a new InvalidInput error
    pub fn invalid_input(
        message: impl Into<String>,
        _input: Option<String>,
    ) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Creates a new InputTooLarge error
    pub fn input_too_large(size: usize) -> Self {
        Self::InputTooLarge(size)
    }

    /// Creates a new Seo error
    pub fn seo(
        kind: SeoErrorKind,
        message: impl Into<String>,
        element: Option<String>,
    ) -> Self {
        Self::Seo {
            kind,
            message: message.into(),
            element,
        }
    }

    /// Creates a new Accessibility error
    pub fn accessibility(
        kind: AccessibilityErrorKind,
        message: impl Into<String>,
        wcag_guideline: Option<String>,
    ) -> Self {
        Self::Accessibility {
            kind,
            message: message.into(),
            wcag_guideline,
        }
    }

    /// Creates a new MarkdownConversion error
    pub fn markdown_conversion(
        message: impl Into<String>,
        source: Option<io::Error>,
    ) -> Self {
        Self::MarkdownConversion {
            message: message.into(),
            source,
        }
    }
}

/// Type alias for a result using the `HtmlError` error type.
///
/// This type alias makes it more convenient to work with Results throughout the library,
/// reducing boilerplate and improving readability.
pub type Result<T> = std::result::Result<T, HtmlError>;

#[cfg(test)]
mod tests {
    use super::*;

    // Basic Error Creation Tests
    mod basic_errors {
        use super::*;

        #[test]
        fn test_regex_compilation_error() {
            let regex_error =
                regex::Error::Syntax("invalid regex".to_string());
            let error: HtmlError = regex_error.into();
            assert!(matches!(
                error,
                HtmlError::RegexCompilationError(_)
            ));
            assert!(error
                .to_string()
                .contains("Failed to compile regex"));
        }

        #[test]
        fn test_front_matter_extraction_error() {
            let error = HtmlError::FrontMatterExtractionError(
                "Missing delimiter".to_string(),
            );
            assert_eq!(
                error.to_string(),
                "Failed to extract front matter: Missing delimiter"
            );
        }

        #[test]
        fn test_header_formatting_error() {
            let error = HtmlError::HeaderFormattingError(
                "Invalid header level".to_string(),
            );
            assert_eq!(
                error.to_string(),
                "Failed to format header: Invalid header level"
            );
        }

        #[test]
        fn test_selector_parse_error() {
            let error = HtmlError::SelectorParseError(
                "div>".to_string(),
                "Unexpected end".to_string(),
            );
            assert_eq!(
                error.to_string(),
                "Failed to parse selector 'div>': Unexpected end"
            );
        }

        #[test]
        fn test_minification_error() {
            let error = HtmlError::MinificationError(
                "Syntax error".to_string(),
            );
            assert_eq!(
                error.to_string(),
                "Failed to minify HTML: Syntax error"
            );
        }
    }

    // Structured Error Tests
    mod structured_errors {
        use super::*;

        #[test]
        fn test_markdown_conversion_with_source() {
            let source =
                io::Error::new(io::ErrorKind::Other, "source error");
            let error = HtmlError::markdown_conversion(
                "Conversion failed",
                Some(source),
            );
            assert!(error
                .to_string()
                .contains("Failed to convert Markdown to HTML"));
        }

        #[test]
        fn test_markdown_conversion_without_source() {
            let error = HtmlError::markdown_conversion(
                "Conversion failed",
                None,
            );
            assert!(error.to_string().contains("Conversion failed"));
        }

        #[test]
        fn test_minification_with_size_and_source() {
            let error = HtmlError::Minification {
                message: "Too large".to_string(),
                size: Some(1024),
                source: Some(io::Error::new(
                    io::ErrorKind::Other,
                    "IO error",
                )),
            };
            assert!(error
                .to_string()
                .contains("HTML minification failed"));
        }
    }

    // SEO Error Tests
    mod seo_errors {
        use super::*;

        #[test]
        fn test_seo_error_missing_meta_tags() {
            let error = HtmlError::seo(
                SeoErrorKind::MissingMetaTags,
                "Required meta tags missing",
                Some("head".to_string()),
            );
            assert!(error
                .to_string()
                .contains("Missing required meta tags"));
        }

        #[test]
        fn test_seo_error_without_element() {
            let error = HtmlError::seo(
                SeoErrorKind::MissingTitle,
                "Title not found",
                None,
            );
            assert!(error.to_string().contains("Missing title"));
        }

        #[test]
        fn test_all_seo_error_kinds() {
            let kinds = [
                SeoErrorKind::MissingMetaTags,
                SeoErrorKind::InvalidStructuredData,
                SeoErrorKind::MissingTitle,
                SeoErrorKind::MissingDescription,
                SeoErrorKind::Other,
            ];
            for kind in kinds {
                assert!(!kind.to_string().is_empty());
            }
        }
    }

    // Accessibility Error Tests
    mod accessibility_errors {
        use super::*;

        #[test]
        fn test_accessibility_error_with_guideline() {
            let error = HtmlError::accessibility(
                AccessibilityErrorKind::MissingAltText,
                "Images must have alt text",
                Some("WCAG 1.1.1".to_string()),
            );
            assert!(error
                .to_string()
                .contains("Missing alternative text"));
        }

        #[test]
        fn test_accessibility_error_without_guideline() {
            let error = HtmlError::accessibility(
                AccessibilityErrorKind::InvalidAriaValue,
                "Invalid ARIA value",
                None,
            );
            assert!(error
                .to_string()
                .contains("Invalid ARIA attribute values"));
        }

        #[test]
        fn test_all_accessibility_error_kinds() {
            let kinds = [
                AccessibilityErrorKind::MissingAriaAttributes,
                AccessibilityErrorKind::InvalidAriaValue,
                AccessibilityErrorKind::MissingAltText,
                AccessibilityErrorKind::HeadingStructure,
                AccessibilityErrorKind::MissingFormLabels,
                AccessibilityErrorKind::Other,
            ];
            for kind in kinds {
                assert!(!kind.to_string().is_empty());
            }
        }
    }

    // Input/Output Error Tests
    mod io_errors {
        use super::*;

        #[test]
        fn test_io_error_kinds() {
            let error_kinds = [
                io::ErrorKind::NotFound,
                io::ErrorKind::PermissionDenied,
                io::ErrorKind::ConnectionRefused,
                io::ErrorKind::ConnectionReset,
                io::ErrorKind::ConnectionAborted,
                io::ErrorKind::NotConnected,
                io::ErrorKind::AddrInUse,
                io::ErrorKind::AddrNotAvailable,
                io::ErrorKind::BrokenPipe,
                io::ErrorKind::AlreadyExists,
                io::ErrorKind::WouldBlock,
                io::ErrorKind::InvalidInput,
                io::ErrorKind::InvalidData,
                io::ErrorKind::TimedOut,
                io::ErrorKind::WriteZero,
                io::ErrorKind::Interrupted,
                io::ErrorKind::Unsupported,
                io::ErrorKind::UnexpectedEof,
                io::ErrorKind::OutOfMemory,
                io::ErrorKind::Other,
            ];

            for kind in error_kinds {
                let io_error = io::Error::new(kind, "test error");
                let html_error: HtmlError = io_error.into();
                assert!(matches!(html_error, HtmlError::Io(_)));
            }
        }
    }

    // Helper Method Tests
    mod helper_methods {
        use super::*;

        #[test]
        fn test_invalid_input_with_content() {
            let error = HtmlError::invalid_input(
                "Bad input",
                Some("problematic content".to_string()),
            );
            assert!(error.to_string().contains("Invalid input"));
        }

        #[test]
        fn test_input_too_large() {
            let error = HtmlError::input_too_large(1024);
            assert!(error.to_string().contains("1024 bytes"));
        }

        #[test]
        fn test_template_rendering_error() {
            let source_error = Box::new(io::Error::new(
                io::ErrorKind::Other,
                "render failed",
            ));
            let error = HtmlError::TemplateRendering {
                message: "Template error".to_string(),
                source: source_error,
            };
            assert!(error
                .to_string()
                .contains("Template rendering failed"));
        }
    }

    // Miscellaneous Error Tests
    mod misc_errors {
        use super::*;

        #[test]
        fn test_missing_html_element() {
            let error =
                HtmlError::MissingHtmlElement("title".to_string());
            assert!(error
                .to_string()
                .contains("Missing required HTML element"));
        }

        #[test]
        fn test_invalid_structured_data() {
            let error = HtmlError::InvalidStructuredData(
                "Invalid JSON-LD".to_string(),
            );
            assert!(error
                .to_string()
                .contains("Invalid structured data"));
        }

        #[test]
        fn test_invalid_front_matter_format() {
            let error = HtmlError::InvalidFrontMatterFormat(
                "Missing closing delimiter".to_string(),
            );
            assert!(error
                .to_string()
                .contains("Invalid front matter format"));
        }

        #[test]
        fn test_parsing_error() {
            let error =
                HtmlError::ParsingError("Unexpected token".to_string());
            assert!(error.to_string().contains("Parsing error"));
        }

        #[test]
        fn test_validation_error() {
            let error = HtmlError::ValidationError(
                "Schema validation failed".to_string(),
            );
            assert!(error.to_string().contains("Validation error"));
        }

        #[test]
        fn test_unexpected_error() {
            let error = HtmlError::UnexpectedError(
                "Something went wrong".to_string(),
            );
            assert!(error.to_string().contains("Unexpected error"));
        }
    }
}

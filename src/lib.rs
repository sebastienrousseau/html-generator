// src/lib.rs

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/html-generator/images/favicon.ico",
    html_logo_url = "https://kura.pro/html-generator/images/logos/html-generator.svg",
    html_root_url = "https://docs.rs/html-generator"
)]
#![crate_name = "html_generator"]
#![crate_type = "lib"]

/// The `accessibility` module contains functions for improving accessibility.
pub mod accessibility;

/// The `error` module contains error types for HTML generation.
pub mod error;

/// The `generator` module contains functions for generating HTML content.
pub mod generator;

/// The `performance` module contains functions for optimizing performance.
pub mod performance;

/// The `seo` module contains functions for optimizing SEO.
pub mod seo;

/// The `utils` module contains utility functions.
pub mod utils;

pub use accessibility::{add_aria_attributes, validate_wcag};
pub use generator::generate_html;
pub use performance::{async_generate_html, minify_html};
pub use seo::{generate_meta_tags, generate_structured_data};
pub use utils::{extract_front_matter, format_header_with_id_class};

use thiserror::Error;

/// Configuration options for HTML generation
#[derive(Debug, Copy, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,
    /// Minify the generated HTML output
    pub minify_output: bool,
    /// Automatically add ARIA attributes for accessibility
    pub add_aria_attributes: bool,
    /// Generate structured data (JSON-LD) based on content
    pub generate_structured_data: bool,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        HtmlConfig {
            enable_syntax_highlighting: true,
            minify_output: false,
            add_aria_attributes: true,
            generate_structured_data: false,
        }
    }
}

/// Error type for HTML generation
#[derive(Debug, Error)]
pub enum HtmlError {
    /// Error occurred during Markdown conversion
    #[error("Markdown conversion error: {0}")]
    MarkdownConversionError(String),
    /// Error occurred during template rendering
    #[error("Template rendering error: {0}")]
    TemplateRenderingError(String),
    /// Error occurred during HTML minification
    #[error("Minification error: {0}")]
    MinificationError(String),
    /// Error occurred during SEO optimization
    #[error("SEO optimization error: {0}")]
    SeoError(String),
    /// Error occurred during accessibility enhancements
    #[error("Accessibility error: {0}")]
    AccessibilityError(String),
}

/// Result type for HTML generation
pub type Result<T> = std::result::Result<T, HtmlError>;

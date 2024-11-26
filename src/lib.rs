// src/lib.rs

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/html-generator/images/favicon.ico",
    html_logo_url = "https://kura.pro/html-generator/images/logos/html-generator.svg",
    html_root_url = "https://docs.rs/html-generator"
)]
#![crate_name = "html_generator"]
#![crate_type = "lib"]

//! HTML Generator: A modern HTML generation and optimization library
//!
//! This crate provides a comprehensive suite of tools for generating, optimizing,
//! and managing HTML content with a focus on accessibility, SEO, and performance.
//!
//! # Features
//!
//! - **HTML Generation**: Convert Markdown to HTML with customizable options
//! - **Accessibility**: Automated ARIA attributes and WCAG compliance checking
//! - **SEO Optimization**: Meta tag generation and structured data support
//! - **Performance**: HTML minification and async generation capabilities
//!
//! # Example
//!
//! ```rust
//! use html_generator::{generate_html, HtmlConfig};
//!
//! let markdown = "# Hello World\n\nWelcome to HTML Generator.";
//! let config = HtmlConfig::default();
//!
//! match generate_html(markdown, &config) {
//!     Ok(html) => println!("Generated HTML: {}", html),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

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

pub use crate::error::HtmlError;
/// Public API for the HTML Generator library
pub use accessibility::{add_aria_attributes, validate_wcag};
pub use generator::generate_html;
pub use performance::{async_generate_html, minify_html};
pub use seo::{generate_meta_tags, generate_structured_data};
pub use utils::{extract_front_matter, format_header_with_id_class};

/// Common constants used throughout the library
pub mod constants {
    /// Default maximum input size (5MB)
    pub const DEFAULT_MAX_INPUT_SIZE: usize = 5 * 1024 * 1024;

    /// Default language
    pub const DEFAULT_LANGUAGE: &str = "en-GB";

    /// Default syntax theme
    pub const DEFAULT_SYNTAX_THEME: &str = "github";
}

/// Configuration options for HTML generation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks.
    ///
    /// When enabled, code blocks in Markdown will be highlighted using the
    /// specified theme.
    pub enable_syntax_highlighting: bool,

    /// Theme to use for syntax highlighting.
    ///
    /// Only applicable when `enable_syntax_highlighting` is true.
    pub syntax_theme: Option<String>,

    /// Minify the generated HTML output.
    ///
    /// When enabled, removes unnecessary whitespace and comments to reduce
    /// file size.
    pub minify_output: bool,

    /// Automatically add ARIA attributes for accessibility.
    pub add_aria_attributes: bool,

    /// Generate structured data (JSON-LD) based on content.
    pub generate_structured_data: bool,

    /// Maximum size (in bytes) for input content.
    ///
    /// Defaults to 5MB to prevent memory issues with large inputs.
    pub max_input_size: usize,

    /// Language for generated content.
    ///
    /// Used for lang attributes and meta tags.
    pub language: String,

    /// Enable table of contents generation.
    pub generate_toc: bool,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            syntax_theme: Some("github".to_string()),
            minify_output: false,
            add_aria_attributes: true,
            generate_structured_data: false,
            max_input_size: 5 * 1024 * 1024, // 5MB
            language: String::from("en-GB"),
            generate_toc: false,
        }
    }
}

/// Get the current version of the library
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the minimum supported Rust version
pub fn min_rust_version() -> &'static str {
    env!("CARGO_PKG_RUST_VERSION")
}

/// Result type for HTML generation
pub type Result<T> = std::result::Result<T, HtmlError>;

#[derive(Default)]
/// Builder for `HtmlConfig` to customize HTML generation options.
#[derive(Debug)]
pub struct HtmlConfigBuilder {
    config: HtmlConfig,
}

impl HtmlConfigBuilder {
    /// Create a new `HtmlConfigBuilder` with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable syntax highlighting for code blocks.
    /// If enabled but no theme is provided, defaults to "github" theme.
    pub fn with_syntax_highlighting(
        mut self,
        enable: bool,
        theme: Option<String>,
    ) -> Self {
        self.config.enable_syntax_highlighting = enable;
        self.config.syntax_theme = if enable {
            theme.or_else(|| Some("github".to_string()))
        } else {
            None
        };
        self
    }

    /// Set the language for generated content.
    /// Only accepts valid language codes (e.g., "en-GB", "fr-FR").
    pub fn with_language(
        mut self,
        language: impl Into<String>,
    ) -> Self {
        let lang = language.into();
        if lang.contains('-') && lang.len() >= 4 {
            self.config.language = lang;
        }
        self
    }

    /// Enable or disable minification of the generated HTML output.
    pub fn build(self) -> HtmlConfig {
        self.config
    }

    /// Enable or disable minification of the generated HTML output.
    pub fn with_minification(mut self, enable: bool) -> Self {
        self.config.minify_output = enable;
        self
    }

    /// Enable or disable automatic addition of ARIA attributes for accessibility.
    pub fn with_aria_attributes(mut self, enable: bool) -> Self {
        self.config.add_aria_attributes = enable;
        self
    }

    /// Enable or disable generation of structured data (JSON-LD).
    pub fn with_structured_data(mut self, enable: bool) -> Self {
        self.config.generate_structured_data = enable;
        self
    }

    /// Set the maximum size (in bytes) for input content.
    /// Enforces a minimum size of 1KB.
    pub fn with_max_input_size(mut self, size: usize) -> Self {
        self.config.max_input_size = size.max(1024); // Minimum 1KB
        self
    }

    /// Enable or disable generation of table of contents.
    pub fn with_toc(mut self, enable: bool) -> Self {
        self.config.generate_toc = enable;
        self
    }
}

impl HtmlConfig {
    /// Create a new `HtmlConfig` with default options.
    pub fn builder() -> HtmlConfigBuilder {
        HtmlConfigBuilder::default()
    }

    /// Check if syntax highlighting is enabled for code blocks.
    ///
    /// When enabled, code blocks will be syntax highlighted using the configured theme.
    pub fn is_syntax_highlighting_enabled(&self) -> bool {
        self.enable_syntax_highlighting
    }

    /// Get the configured syntax highlighting theme.
    ///
    /// Returns the theme name if syntax highlighting is enabled, None otherwise.
    pub fn get_syntax_theme(&self) -> Option<&str> {
        self.syntax_theme.as_deref()
    }

    /// Check if HTML minification is enabled.
    ///
    /// When enabled, unnecessary whitespace and comments will be removed from the output HTML.
    pub fn is_minification_enabled(&self) -> bool {
        self.minify_output
    }

    /// Check if ARIA attributes generation is enabled.
    ///
    /// When enabled, appropriate ARIA attributes will be automatically added to HTML elements
    /// to improve accessibility.
    pub fn are_aria_attributes_enabled(&self) -> bool {
        self.add_aria_attributes
    }

    /// Check if structured data (JSON-LD) generation is enabled.
    ///
    /// When enabled, structured data will be generated in JSON-LD format
    /// to improve SEO.
    pub fn is_structured_data_enabled(&self) -> bool {
        self.generate_structured_data
    }

    /// Check if table of contents generation is enabled.
    ///
    /// When enabled, a table of contents will be generated from the document headings.
    pub fn is_toc_enabled(&self) -> bool {
        self.generate_toc
    }

    /// Get the configured language for content generation.
    ///
    /// Returns the language code (e.g., "en-GB", "fr-FR") that will be used
    /// in lang attributes and meta tags.
    pub fn get_language(&self) -> &str {
        &self.language
    }

    /// Get the configured maximum input size in bytes.
    ///
    /// Returns the maximum allowed size for input content. Default is 5MB.
    pub fn get_max_input_size(&self) -> usize {
        self.max_input_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // HtmlConfig Tests
    mod config_tests {
        use super::*;
        use crate::constants::*;

        #[test]
        fn test_default_config() {
            let config = HtmlConfig::default();
            assert!(config.enable_syntax_highlighting);
            assert_eq!(config.syntax_theme, Some("github".to_string()));
            assert!(!config.minify_output);
            assert!(config.add_aria_attributes);
            assert!(!config.generate_structured_data);
            assert_eq!(config.max_input_size, DEFAULT_MAX_INPUT_SIZE);
            assert_eq!(config.language, DEFAULT_LANGUAGE);
            assert!(!config.generate_toc);
        }

        #[test]
        fn test_config_equality() {
            let config1 = HtmlConfig::default();
            let config2 = HtmlConfig::default();
            assert_eq!(config1, config2);
        }

        #[test]
        fn test_config_clone() {
            let config1 = HtmlConfig::default();
            let config2 = config1.clone();
            assert_eq!(config1, config2);
        }

        #[test]
        fn test_config_debug() {
            let config = HtmlConfig::default();
            let debug_string = format!("{:?}", config);
            assert!(debug_string.contains("enable_syntax_highlighting"));
            assert!(debug_string.contains("syntax_theme"));
            assert!(debug_string.contains("minify_output"));
        }
    }

    // HtmlConfigBuilder Tests
    mod builder_tests {
        use super::*;

        #[test]
        fn test_builder_new() {
            let builder = HtmlConfigBuilder::new();
            let config = builder.build();
            assert_eq!(config, HtmlConfig::default());
        }

        #[test]
        fn test_builder_with_syntax_highlighting() {
            let config = HtmlConfigBuilder::new()
                .with_syntax_highlighting(false, None)
                .build();
            assert!(!config.enable_syntax_highlighting);
            assert_eq!(config.syntax_theme, None);
        }

        #[test]
        fn test_builder_with_custom_theme() {
            let config = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("dracula".to_string()),
                )
                .build();
            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("dracula".to_string())
            );
        }

        #[test]
        fn test_builder_with_language() {
            let config =
                HtmlConfigBuilder::new().with_language("fr-FR").build();
            assert_eq!(config.language, "fr-FR");
        }

        #[test]
        fn test_builder_with_valid_languages() {
            let valid_langs = ["en-GB", "fr-FR", "de-DE", "zh-CN"];
            for lang in valid_langs {
                let config = HtmlConfigBuilder::new()
                    .with_language(lang)
                    .build();
                assert_eq!(config.language, lang);
            }
        }

        #[test]
        fn test_builder_with_more_invalid_languages() {
            let invalid_langs = ["en", "f", "", "fr_FR"];
            for lang in invalid_langs {
                let config = HtmlConfigBuilder::new()
                    .with_language(lang)
                    .build();
                assert_eq!(config.language, "en-GB"); // should keep default
            }
        }

        #[test]
        fn test_builder_chaining() {
            let config = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_language("es-ES")
                .build();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("monokai".to_string())
            );
            assert_eq!(config.language, "es-ES");
        }

        #[test]
        fn test_builder_debug() {
            let builder = HtmlConfigBuilder::new();
            let debug_string = format!("{:?}", builder);
            assert!(debug_string.contains("HtmlConfigBuilder"));
        }

        #[test]
        fn test_builder_with_invalid_language() {
            let config = HtmlConfigBuilder::new()
                .with_language("fr") // too short
                .build();
            assert_eq!(config.language, "en-GB"); // should keep default
        }

        #[test]
        fn test_builder_with_small_input_size() {
            let config = HtmlConfigBuilder::new()
                .with_max_input_size(100) // less than minimum
                .build();
            assert_eq!(config.max_input_size, 1024); // should use minimum
        }

        #[test]
        fn test_builder_all_options() {
            let config = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_minification(true)
                .with_aria_attributes(false)
                .with_structured_data(true)
                .with_max_input_size(1024 * 1024)
                .with_language("fr-FR")
                .with_toc(true)
                .build();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("monokai".to_string())
            );
            assert!(config.minify_output);
            assert!(!config.add_aria_attributes);
            assert!(config.generate_structured_data);
            assert_eq!(config.max_input_size, 1024 * 1024);
            assert_eq!(config.language, "fr-FR");
            assert!(config.generate_toc);
        }

        #[test]
        fn test_all_config_getters() {
            let config = HtmlConfig::default();
            assert!(!config.is_minification_enabled());
            assert!(config.are_aria_attributes_enabled());
            assert!(!config.is_structured_data_enabled());
            assert!(!config.is_toc_enabled());
            assert_eq!(config.get_language(), "en-GB");
            assert_eq!(config.get_max_input_size(), 5 * 1024 * 1024);
        }
    }

    // Constants Tests
    mod constants_tests {
        use super::*;

        #[test]
        fn test_default_max_input_size() {
            assert_eq!(
                constants::DEFAULT_MAX_INPUT_SIZE,
                5 * 1024 * 1024
            );
        }

        #[test]
        fn test_default_language() {
            assert_eq!(constants::DEFAULT_LANGUAGE, "en-GB");
        }

        #[test]
        fn test_default_syntax_theme() {
            assert_eq!(constants::DEFAULT_SYNTAX_THEME, "github");
        }
    }

    // Version Information Tests
    mod version_tests {
        use super::*;

        #[test]
        fn test_version() {
            let v = version();
            assert!(!v.is_empty());
            assert!(v.split('.').count() >= 2);
        }

        #[test]
        fn test_min_rust_version() {
            let v = min_rust_version();
            assert!(!v.is_empty());
            assert!(v.split('.').count() >= 2);
        }
    }

    // Config Factory Method Tests
    mod config_factory_tests {
        use super::*;

        #[test]
        fn test_config_builder_factory() {
            let config = HtmlConfig::builder().build();
            assert_eq!(config, HtmlConfig::default());
        }

        #[test]
        fn test_config_custom_build() {
            let config = HtmlConfig::builder()
                .with_syntax_highlighting(
                    true,
                    Some("tomorrow".to_string()),
                )
                .with_language("de-DE")
                .build();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("tomorrow".to_string())
            );
            assert_eq!(config.language, "de-DE");
        }
    }

    // Result Type Tests
    mod result_tests {
        use super::*;

        #[test]
        fn test_result_ok() {
            let result: Result<i32> = Ok(42);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        #[test]
        fn test_result_err() {
            let error =
                HtmlError::InvalidInput("test error".to_string());
            let result: Result<i32> = Err(error);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                HtmlError::InvalidInput(_)
            ));
        }
    }

    // Module Re-exports Tests
    mod reexport_tests {
        use super::*;

        #[test]
        fn test_accessibility_reexports() {
            // Verify that the re-exported functions exist
            // We don't need to test their functionality here
            let _add_aria = add_aria_attributes;
            let _validate = validate_wcag;
        }

        #[test]
        fn test_generator_reexports() {
            let _gen_html = generate_html;
        }

        #[test]
        fn test_performance_reexports() {
            let _async_gen = async_generate_html;
            let _minify = minify_html;
        }

        #[test]
        fn test_seo_reexports() {
            let _gen_meta = generate_meta_tags;
            let _gen_struct = generate_structured_data;
        }

        #[test]
        fn test_utils_reexports() {
            let _extract = extract_front_matter;
            let _format = format_header_with_id_class;
        }
    }
}

#![forbid(unsafe_code)]
// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://cloudcdn.pro/html-generator/v1/favicon.ico",
    html_logo_url = "https://cloudcdn.pro/html-generator/v1/logos/html-generator.svg",
    html_root_url = "https://docs.rs/html-generator"
)]
#![crate_name = "html_generator"]
#![crate_type = "lib"]

use std::{
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Component, Path},
};

/// Maximum buffer size for reading files (16MB)
const MAX_BUFFER_SIZE: usize = 16 * 1024 * 1024;

// Re-export public modules
pub mod accessibility;
pub mod elements;
pub mod emojis;
pub mod error;
pub mod generator;
pub mod performance;
pub mod seo;
pub mod utils;

// Inlined private YAML serde implementation (upstream:
// `/Users/seb/Code/Public/Rust/yaml_safe`). Kept private so the
// crate's external surface is unchanged.
mod yaml;

// Re-export primary types and functions for convenience
pub use crate::error::HtmlError;
pub use accessibility::{add_aria_attributes, validate_wcag};
pub use emojis::load_emoji_sequences;
pub use generator::{
    generate_html, generate_html_with_diagnostics, Diagnostic,
    DiagnosticLevel, HtmlOutput,
};
#[cfg(feature = "async")]
pub use performance::async_generate_html;
pub use performance::{minify_html, minify_html_string};
pub use seo::{generate_meta_tags, generate_structured_data};
pub use utils::{
    extract_front_matter, extract_front_matter_data,
    format_header_with_id_class,
};

/// Common constants used throughout the library.
///
/// This module contains configuration values and limits that help ensure
/// secure and efficient operation of the library.
///
/// # Examples
///
/// ```
/// use html_generator::constants::{DEFAULT_LANGUAGE, DEFAULT_MAX_INPUT_SIZE};
///
/// assert_eq!(DEFAULT_LANGUAGE, "en-GB");
/// assert!(DEFAULT_MAX_INPUT_SIZE > 0);
/// ```
pub mod constants {
    /// Maximum allowed input size (5MB) to prevent denial of service attacks.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::DEFAULT_MAX_INPUT_SIZE;
    /// assert_eq!(DEFAULT_MAX_INPUT_SIZE, 5 * 1024 * 1024);
    /// ```
    pub const DEFAULT_MAX_INPUT_SIZE: usize = 5 * 1024 * 1024;

    /// Minimum required input size (1KB) for meaningful processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::MIN_INPUT_SIZE;
    /// assert_eq!(MIN_INPUT_SIZE, 1024);
    /// ```
    pub const MIN_INPUT_SIZE: usize = 1024;

    /// Default language code for HTML generation (British English).
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::DEFAULT_LANGUAGE;
    /// assert_eq!(DEFAULT_LANGUAGE, "en-GB");
    /// ```
    pub const DEFAULT_LANGUAGE: &str = "en-GB";

    /// Default syntax highlighting theme (`github`).
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::DEFAULT_SYNTAX_THEME;
    /// assert_eq!(DEFAULT_SYNTAX_THEME, "github");
    /// ```
    pub const DEFAULT_SYNTAX_THEME: &str = "github";

    /// Maximum file path length.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::MAX_PATH_LENGTH;
    /// assert_eq!(MAX_PATH_LENGTH, 4096);
    /// ```
    pub const MAX_PATH_LENGTH: usize = 4096;

    /// Regular expression pattern for validating language codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::constants::LANGUAGE_CODE_PATTERN;
    /// use regex::Regex;
    ///
    /// let re = Regex::new(LANGUAGE_CODE_PATTERN).unwrap();
    /// assert!(re.is_match("en-GB"));
    /// ```
    pub const LANGUAGE_CODE_PATTERN: &str = r"^[a-z]{2}-[A-Z]{2}$";

    /// Verify invariants at compile time
    const _: () = assert!(MIN_INPUT_SIZE <= DEFAULT_MAX_INPUT_SIZE);
    const _: () = assert!(MAX_PATH_LENGTH > 0);
}

/// Result type alias for library operations.
///
/// # Examples
///
/// ```
/// use html_generator::{error::HtmlError, Result};
///
/// fn run() -> Result<()> {
///     Err(HtmlError::InvalidInput("demo".into()))
/// }
/// assert!(run().is_err());
/// ```
pub type Result<T> = std::result::Result<T, HtmlError>;

/// Legacy configuration type — use [`HtmlConfig`] directly instead.
///
/// This type is kept for backward compatibility. The `encoding` field
/// has been moved into `HtmlConfig` itself.
#[deprecated(
    since = "0.0.4",
    note = "use HtmlConfig directly — encoding is now a field on HtmlConfig"
)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MarkdownConfig {
    /// The encoding to use for input/output (defaults to "utf-8")
    pub encoding: String,

    /// HTML generation configuration
    pub html_config: HtmlConfig,
}

#[allow(deprecated)]
impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            encoding: String::from("utf-8"),
            html_config: HtmlConfig::default(),
        }
    }
}

#[allow(deprecated)]
impl From<MarkdownConfig> for HtmlConfig {
    fn from(mc: MarkdownConfig) -> Self {
        let mut c = mc.html_config;
        c.encoding = mc.encoding;
        c
    }
}

/// Errors that can occur during configuration.
///
/// # Examples
///
/// ```
/// use html_generator::ConfigError;
///
/// let err = ConfigError::InvalidLanguageCode("xx".into());
/// assert!(err.to_string().contains("Invalid language code"));
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Error for invalid input size configuration
    #[error(
        "Invalid input size: {0} bytes is below minimum of {1} bytes"
    )]
    InvalidInputSize(usize, usize),

    /// Error for invalid language code
    #[error("Invalid language code: {0}")]
    InvalidLanguageCode(String),

    /// Error for invalid file path
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
}

/// Output destination for HTML generation.
///
/// Specifies where the generated HTML content should be written.
///
/// # Examples
///
/// Writing HTML to a file:
/// ```
/// use std::fs::File;
/// use html_generator::OutputDestination;
///
/// let output = OutputDestination::File("output.html".to_string());
/// ```
///
/// Writing HTML to an in-memory buffer:
/// ```
/// use std::io::Cursor;
/// use html_generator::OutputDestination;
///
/// let buffer = Cursor::new(Vec::new());
/// let output = OutputDestination::Writer(Box::new(buffer));
/// ```
///
/// Writing HTML to standard output:
/// ```
/// use html_generator::OutputDestination;
///
/// let output = OutputDestination::Stdout;
/// ```
#[non_exhaustive]
pub enum OutputDestination {
    /// Write output to a file at the specified path.
    ///
    /// # Example
    ///
    /// ```
    /// use html_generator::OutputDestination;
    ///
    /// let output = OutputDestination::File("output.html".to_string());
    /// ```
    File(String),

    /// Write output using a custom writer implementation.
    ///
    /// This can be used for in-memory buffers, network streams,
    /// or other custom output destinations.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use html_generator::OutputDestination;
    ///
    /// let buffer = Cursor::new(Vec::new());
    /// let output = OutputDestination::Writer(Box::new(buffer));
    /// ```
    Writer(Box<dyn Write>),

    /// Write output to standard output (default).
    ///
    /// This is useful for command-line tools and scripts.
    ///
    /// # Example
    ///
    /// ```
    /// use html_generator::OutputDestination;
    ///
    /// let output = OutputDestination::Stdout;
    /// ```
    Stdout,
}

/// Default implementation for OutputDestination.
impl Default for OutputDestination {
    fn default() -> Self {
        Self::Stdout
    }
}

/// Debug implementation for OutputDestination.
impl fmt::Debug for OutputDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => {
                f.debug_tuple("File").field(path).finish()
            }
            Self::Writer(_) => write!(f, "Writer(<dyn Write>)"),
            Self::Stdout => write!(f, "Stdout"),
        }
    }
}

/// Implements `Display` for `OutputDestination`.
impl fmt::Display for OutputDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputDestination::File(path) => {
                write!(f, "File({})", path)
            }
            OutputDestination::Writer(_) => {
                write!(f, "Writer(<dyn Write>)")
            }
            OutputDestination::Stdout => write!(f, "Stdout"),
        }
    }
}

/// Configuration options for HTML generation.
///
/// Controls various aspects of the HTML generation process including
/// syntax highlighting, accessibility features, and output formatting.
///
/// # Examples
///
/// ```
/// use html_generator::HtmlConfig;
///
/// let cfg = HtmlConfig::default();
/// assert!(cfg.add_aria_attributes);
/// assert_eq!(cfg.language, "en-GB");
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HtmlConfig {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,

    /// Theme to use for syntax highlighting
    pub syntax_theme: Option<String>,

    /// Minify the generated HTML output
    pub minify_output: bool,

    /// Automatically add ARIA attributes for accessibility
    pub add_aria_attributes: bool,

    /// Generate structured data (JSON-LD) based on content
    pub generate_structured_data: bool,

    /// Maximum size (in bytes) for input content
    pub max_input_size: usize,

    /// Language for generated content
    pub language: String,

    /// Enable table of contents generation
    pub generate_toc: bool,

    /// Allow raw HTML passthrough in Markdown conversion.
    ///
    /// When `false` (the default), raw HTML tags in Markdown input are
    /// stripped from the output, preventing XSS when processing
    /// untrusted content. Set to `true` only when the Markdown source
    /// is fully trusted.
    pub allow_unsafe_html: bool,

    /// Sanitize raw HTML using ammonia instead of stripping it.
    ///
    /// When `true` and `allow_unsafe_html` is also `true`, the library
    /// runs ammonia over the final output to strip dangerous elements
    /// (`<script>`, `onclick`, etc.) while preserving safe tags like
    /// `<div>`, `<span>`, and `<img>`. This provides a secure
    /// middle-ground for user-authored HTML.
    ///
    /// Has no effect when `allow_unsafe_html` is `false` (HTML is
    /// already stripped by the Markdown renderer).
    pub sanitize_html: bool,

    /// Wrap output in a full HTML5 document.
    ///
    /// When `true`, the pipeline wraps the generated body in:
    /// ```html
    /// <!DOCTYPE html>
    /// <html lang="{language}">
    /// <head><meta charset="utf-8"><title>…</title>{meta}{json-ld}</head>
    /// <body>{content}</body>
    /// </html>
    /// ```
    ///
    /// SEO meta tags and JSON-LD are placed in `<head>`, and the
    /// `language` field is injected as the `lang` attribute. When
    /// `false` (the default), only an HTML fragment is returned.
    pub generate_full_document: bool,

    /// Maximum buffer size for file I/O operations (default: 16MB).
    ///
    /// Controls the upper bound on buffer allocation when reading
    /// input files. Adjust this if you need to process unusually
    /// large documents or want to constrain memory usage.
    pub max_buffer_size: usize,

    /// The encoding for file I/O (defaults to "utf-8").
    ///
    /// This field is used by [`markdown_file_to_html`] when reading
    /// or writing files. In-memory functions ignore it.
    pub encoding: String,
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            syntax_theme: Some(
                constants::DEFAULT_SYNTAX_THEME.to_string(),
            ),
            minify_output: false,
            add_aria_attributes: true,
            generate_structured_data: false,
            max_input_size: constants::DEFAULT_MAX_INPUT_SIZE,
            language: String::from(constants::DEFAULT_LANGUAGE),
            generate_toc: false,
            allow_unsafe_html: false,
            sanitize_html: false,
            generate_full_document: false,
            max_buffer_size: 16 * 1024 * 1024,
            encoding: String::from("utf-8"),
        }
    }
}

impl HtmlConfig {
    /// Creates a new `HtmlConfig` using the builder pattern.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use html_generator::HtmlConfig;
    ///
    /// let config = HtmlConfig::builder()
    ///     .with_syntax_highlighting(true, Some("monokai".to_string()))
    ///     .with_language("en-GB")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> HtmlConfigBuilder {
        HtmlConfigBuilder::default()
    }

    /// Validates the configuration settings.
    ///
    /// Checks that all configuration values are within acceptable ranges
    /// and conform to required formats.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is valid, or an appropriate
    /// error if validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfig;
    ///
    /// let cfg = HtmlConfig::default();
    /// cfg.validate().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::HtmlError::InvalidInput`] if `language`
    /// is not a valid BCP 47 code or `max_input_size` is below
    /// [`constants::MIN_INPUT_SIZE`].
    pub fn validate(&self) -> Result<()> {
        if self.max_input_size < constants::MIN_INPUT_SIZE {
            return Err(HtmlError::InvalidInput(format!(
                "Input size must be at least {} bytes",
                constants::MIN_INPUT_SIZE
            )));
        }
        if !validate_language_code(&self.language) {
            return Err(HtmlError::InvalidInput(format!(
                "Invalid language code: {}",
                self.language
            )));
        }
        Ok(())
    }

    /// Validates a file path before it is opened by
    /// [`markdown_file_to_html`].
    ///
    /// Rejects paths that are empty, too long, contain a NUL byte, contain
    /// any `..` component (directory traversal), or use an extension other
    /// than `.md` or `.html`.
    ///
    /// This validator is defensive only: it does **not** decide whether a
    /// caller is authorised to read the target file. Callers that expose
    /// this API to untrusted input must enforce their own authorisation
    /// (e.g. chroot, a sandbox root directory, or an allow-list) on top
    /// of this check. Absolute paths are accepted deliberately so that
    /// CLI tools can be invoked with fully qualified filenames.
    pub(crate) fn validate_file_path(
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        if path_str.is_empty() {
            return Err(HtmlError::InvalidInput(
                "File path cannot be empty".to_string(),
            ));
        }

        if path_str.len() > constants::MAX_PATH_LENGTH {
            return Err(HtmlError::InvalidInput(format!(
                "File path exceeds maximum length of {} characters",
                constants::MAX_PATH_LENGTH
            )));
        }

        // Reject NUL bytes: on Unix, C-string path handling silently
        // truncates at the first NUL, which is a classic smuggling vector
        // (e.g. "safe.md\0/etc/passwd").
        if path_str.as_bytes().contains(&0) {
            return Err(HtmlError::InvalidInput(
                "File path must not contain NUL bytes".to_string(),
            ));
        }

        if path.components().any(|c| matches!(c, Component::ParentDir))
        {
            return Err(HtmlError::InvalidInput(
                "Directory traversal is not allowed in file paths"
                    .to_string(),
            ));
        }

        if let Some(ext) = path.extension() {
            if !matches!(ext.to_string_lossy().as_ref(), "md" | "html")
            {
                return Err(HtmlError::InvalidInput(
                    "Invalid file extension: only .md and .html files are allowed".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Builder for constructing `HtmlConfig` instances.
///
/// Provides a fluent interface for creating and customizing HTML
/// configuration options.
///
/// # Examples
///
/// ```
/// use html_generator::HtmlConfigBuilder;
///
/// let cfg = HtmlConfigBuilder::new()
///     .with_language("en-GB")
///     .with_full_document(true)
///     .build()
///     .unwrap();
/// assert!(cfg.generate_full_document);
/// ```
#[derive(Debug, Default)]
pub struct HtmlConfigBuilder {
    config: HtmlConfig,
}

impl HtmlConfigBuilder {
    /// Creates a new `HtmlConfigBuilder` with default options.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let _ = HtmlConfigBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables syntax highlighting for code blocks.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable syntax highlighting
    /// * `theme` - Optional theme name for syntax highlighting
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_syntax_highlighting(true, Some("monokai".into()))
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.syntax_theme.as_deref(), Some("monokai"));
    /// ```
    #[must_use]
    pub fn with_syntax_highlighting(
        mut self,
        enable: bool,
        theme: Option<String>,
    ) -> Self {
        self.config.enable_syntax_highlighting = enable;
        self.config.syntax_theme = if enable {
            theme.or_else(|| {
                Some(constants::DEFAULT_SYNTAX_THEME.to_string())
            })
        } else {
            None
        };
        self
    }

    /// Sets the language for generated content.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_language("fr-FR")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.language, "fr-FR");
    /// ```
    #[must_use]
    pub fn with_language(
        mut self,
        language: impl Into<String>,
    ) -> Self {
        self.config.language = language.into();
        self
    }

    /// Enables or disables HTML sanitization via ammonia.
    ///
    /// When enabled alongside `allow_unsafe_html`, dangerous elements
    /// are stripped while safe tags are preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_sanitization(true)
    ///     .build()
    ///     .unwrap();
    /// assert!(cfg.sanitize_html);
    /// ```
    #[must_use]
    pub fn with_sanitization(mut self, enable: bool) -> Self {
        self.config.sanitize_html = enable;
        self
    }

    /// Enables or disables full HTML5 document wrapping.
    ///
    /// When enabled, the output is wrapped in `<!DOCTYPE html>` with
    /// `<head>` (containing meta/JSON-LD) and `<body>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_full_document(true)
    ///     .build()
    ///     .unwrap();
    /// assert!(cfg.generate_full_document);
    /// ```
    #[must_use]
    pub fn with_full_document(mut self, enable: bool) -> Self {
        self.config.generate_full_document = enable;
        self
    }

    /// Sets the maximum buffer size for file I/O operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_max_buffer_size(8 * 1024 * 1024)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.max_buffer_size, 8 * 1024 * 1024);
    /// ```
    #[must_use]
    pub fn with_max_buffer_size(mut self, size: usize) -> Self {
        self.config.max_buffer_size = size;
        self
    }

    /// Builds the configuration, validating all settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_generator::HtmlConfigBuilder;
    ///
    /// let cfg = HtmlConfigBuilder::new()
    ///     .with_language("en-GB")
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(cfg.language, "en-GB");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the first [`crate::error::HtmlError::InvalidInput`]
    /// produced by [`HtmlConfig::validate`] (e.g. an unknown language
    /// code or a `max_input_size` below the minimum).
    pub fn build(self) -> Result<HtmlConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Converts Markdown content to HTML.
///
/// This function processes Unicode Markdown content and returns HTML output.
/// The input must be valid Unicode - if your input is encoded (e.g., UTF-8),
/// you must decode it before passing it to this function.
///
/// # Arguments
///
/// * `content` - The Markdown content as a Unicode string
/// * `config` - Optional configuration for the conversion
///
/// # Returns
///
/// Returns the generated HTML as a Unicode string wrapped in a `Result`
///
/// # Errors
///
/// Returns an error if:
/// * The input content is invalid Unicode
/// * HTML generation fails
/// * Input size exceeds configured maximum
///
/// # Examples
///
/// ```rust
/// use html_generator::{markdown_to_html, MarkdownConfig};
///
/// let markdown = "# Hello\n\nWorld";
/// let html = markdown_to_html(markdown, None)?;
/// assert!(html.contains("<h1>Hello</h1>"));
/// # Ok::<(), html_generator::error::HtmlError>(())
/// ```
#[allow(deprecated)]
pub fn markdown_to_html(
    content: &str,
    config: Option<MarkdownConfig>,
) -> Result<String> {
    let html_config: HtmlConfig =
        config.map_or_else(HtmlConfig::default, HtmlConfig::from);

    if content.is_empty() {
        return Err(HtmlError::InvalidInput(
            "Input content is empty".to_string(),
        ));
    }

    if content.len() > html_config.max_input_size {
        return Err(HtmlError::InputTooLarge(content.len()));
    }

    generate_html(content, &html_config)
}

/// Converts a Markdown file to HTML.
///
/// This function reads from a file or stdin and writes the generated HTML to
/// a specified destination. It handles encoding/decoding of content.
///
/// # Arguments
///
/// * `input` - The input source (file path or None for stdin)
/// * `output` - The output destination (defaults to stdout)
/// * `config` - Optional configuration including encoding settings
///
/// # Returns
///
/// Returns `Result<()>` indicating success or failure of the operation.
///
/// # Errors
///
/// Returns an error if:
/// * Input file is not found or cannot be read
/// * Output file cannot be written
/// * Configuration is invalid
/// * Input size exceeds configured maximum
///
/// # Examples
///
/// ```no_run
/// use html_generator::{markdown_file_to_html, OutputDestination, MarkdownConfig};
/// use std::path::{Path, PathBuf};
///
/// // Convert file to HTML and write to stdout
/// markdown_file_to_html(
///     Some(PathBuf::from("input.md")),
///     None,
///     None,
/// )?;
///
/// // Convert stdin to HTML file
/// markdown_file_to_html(
///     None::<PathBuf>,  // Explicit type annotation
///     Some(OutputDestination::File("output.html".into())),
///     Some(MarkdownConfig::default()),
/// )?;
/// # Ok::<(), html_generator::error::HtmlError>(())
/// ```
#[inline]
#[allow(deprecated)]
pub fn markdown_file_to_html(
    input: Option<impl AsRef<Path>>,
    output: Option<OutputDestination>,
    config: Option<MarkdownConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let output = output.unwrap_or_default();

    // Validate paths first
    validate_paths(&input, &output)?;

    // Read and process input
    let content = read_input(input)?;

    // Generate HTML
    let html = markdown_to_html(&content, Some(config))?;

    // Write output
    write_output(output, html.as_bytes())
}

/// Validates input and output paths
fn validate_paths(
    input: &Option<impl AsRef<Path>>,
    output: &OutputDestination,
) -> Result<()> {
    if let Some(path) = input.as_ref() {
        HtmlConfig::validate_file_path(path)?;
    }
    if let OutputDestination::File(ref path) = output {
        HtmlConfig::validate_file_path(path)?;
    }
    Ok(())
}

/// Reads the full contents of `reader` into a UTF-8 string, wrapping
/// any I/O error as `HtmlError::Io` with the given label for context
/// (e.g. `"input"` or `"stdin"`).
///
/// Extracted so the stdin path of [`read_input`] is testable against
/// an in-memory reader without needing a child process.
fn read_all_from_reader<R: Read>(
    mut reader: R,
    label: &str,
) -> Result<String> {
    let mut content = String::with_capacity(MAX_BUFFER_SIZE);
    // read_to_string returns the byte count; we only need the String.
    let _ = reader.read_to_string(&mut content).map_err(|e| {
        HtmlError::Io(io::Error::new(
            e.kind(),
            format!("Failed to read from {label}: {e}"),
        ))
    })?;
    Ok(content)
}

/// Reads content from the input source (a file path, or stdin when
/// `None`).
fn read_input(input: Option<impl AsRef<Path>>) -> Result<String> {
    match input {
        Some(path) => {
            let file = File::open(path).map_err(HtmlError::Io)?;
            let reader =
                BufReader::with_capacity(MAX_BUFFER_SIZE, file);
            read_all_from_reader(reader, "input")
        }
        None => {
            let stdin = io::stdin();
            let reader =
                BufReader::with_capacity(MAX_BUFFER_SIZE, stdin.lock());
            read_all_from_reader(reader, "stdin")
        }
    }
}

/// Writes `content` to `writer`, wrapping any I/O error as
/// `HtmlError::Io` with a label like `"file '…'"` or `"stdout"`.
///
/// Extracted so every destination in [`write_output`] shares one
/// tested implementation, and so the error paths can be exercised by
/// a failing in-memory writer.
fn write_all_to_writer<W: Write>(
    mut writer: W,
    content: &[u8],
    label: &str,
) -> Result<()> {
    writer.write_all(content).map_err(|e| {
        HtmlError::Io(io::Error::new(
            e.kind(),
            format!("Failed to write to {label}: {e}"),
        ))
    })?;
    writer.flush().map_err(|e| {
        HtmlError::Io(io::Error::new(
            e.kind(),
            format!("Failed to flush {label}: {e}"),
        ))
    })?;
    Ok(())
}

/// Writes content to the output destination.
fn write_output(
    output: OutputDestination,
    content: &[u8],
) -> Result<()> {
    match output {
        OutputDestination::File(path) => {
            let file = File::create(&path).map_err(|e| {
                HtmlError::Io(io::Error::new(
                    e.kind(),
                    format!("Failed to create file '{}': {}", path, e),
                ))
            })?;
            write_all_to_writer(
                BufWriter::new(file),
                content,
                &format!("file '{path}'"),
            )
        }
        OutputDestination::Writer(mut writer) => write_all_to_writer(
            BufWriter::new(&mut writer),
            content,
            "output",
        ),
        OutputDestination::Stdout => {
            let stdout = io::stdout();
            write_all_to_writer(
                BufWriter::new(stdout.lock()),
                content,
                "stdout",
            )
        }
    }
}

/// Validates that a language code matches the BCP 47 format (e.g., "en-GB").
///
/// This function checks if a given language code follows the BCP 47 format,
/// which requires both language and region codes.
///
/// # Arguments
///
/// * `lang` - The language code to validate
///
/// # Returns
///
/// Returns true if the language code is valid (e.g., "en-GB"), false otherwise.
///
/// # Examples
///
/// ```
/// use html_generator::validate_language_code;
///
/// assert!(validate_language_code("en-GB"));  // Valid
/// assert!(!validate_language_code("en"));    // Invalid - missing region
/// assert!(!validate_language_code("123"));   // Invalid - not a language code
/// assert!(!validate_language_code("en_GB")); // Invalid - wrong separator
/// ```
pub fn validate_language_code(lang: &str) -> bool {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static LANG_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(constants::LANGUAGE_CODE_PATTERN)
            .expect("static LANG_REGEX must compile")
    });

    LANG_REGEX.is_match(lang)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::io::Cursor;
    use tempfile::{tempdir, TempDir};

    /// A reader whose `read` call always fails — used to cover the
    /// stdin failure branch of [`read_all_from_reader`].
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("synthetic read failure"))
        }
    }

    /// A writer whose `write` + `flush` both fail — used to cover the
    /// write/flush error branches of [`write_all_to_writer`].
    struct FailingWriter {
        /// If `true`, fail on `flush` only (writes succeed), otherwise
        /// fail immediately on `write`.
        flush_only: bool,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.flush_only {
                Ok(buf.len())
            } else {
                Err(io::Error::other("synthetic write failure"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("synthetic flush failure"))
        }
    }

    #[test]
    fn test_read_all_from_reader_success() {
        let input = Cursor::new(b"hello world".to_vec());
        let s = read_all_from_reader(input, "memory").unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_read_all_from_reader_surfaces_io_error() {
        let err =
            read_all_from_reader(FailingReader, "stdin").unwrap_err();
        match err {
            HtmlError::Io(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("Failed to read from stdin"),
                    "unexpected error: {msg}"
                );
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn test_write_all_to_writer_success_covers_stdout_path() {
        let mut buf: Vec<u8> = Vec::new();
        write_all_to_writer(&mut buf, b"hi", "memory").unwrap();
        assert_eq!(buf, b"hi");
    }

    #[test]
    fn test_write_all_to_writer_surfaces_write_error() {
        let err = write_all_to_writer(
            FailingWriter { flush_only: false },
            b"x",
            "output",
        )
        .unwrap_err();
        assert!(
            matches!(err, HtmlError::Io(ref e) if e.to_string().contains("Failed to write to output"))
        );
    }

    #[test]
    fn test_write_all_to_writer_surfaces_flush_error() {
        let err = write_all_to_writer(
            FailingWriter { flush_only: true },
            b"x",
            "output",
        )
        .unwrap_err();
        assert!(
            matches!(err, HtmlError::Io(ref e) if e.to_string().contains("Failed to flush output"))
        );
    }

    /// Creates a temporary test directory for file operations.
    ///
    /// The directory and its contents are automatically cleaned up when
    /// the returned TempDir is dropped.
    fn setup_test_dir() -> TempDir {
        tempdir().expect("Failed to create temporary directory")
    }

    /// Creates a test file with the given content.
    ///
    /// # Arguments
    ///
    /// * `dir` - The temporary directory to create the file in
    /// * `content` - The content to write to the file
    ///
    /// # Returns
    ///
    /// Returns the path to the created file.
    fn create_test_file(
        dir: &TempDir,
        content: &str,
    ) -> std::path::PathBuf {
        let path = dir.path().join("test.md");
        std::fs::write(&path, content)
            .expect("Failed to write test file");
        path
    }

    mod config_tests {
        use super::*;

        #[test]
        fn test_config_validation() {
            // Test invalid input size
            let config = HtmlConfig {
                max_input_size: 100, // Too small
                ..Default::default()
            };
            assert!(config.validate().is_err());

            // Test invalid language code
            let config = HtmlConfig {
                language: "invalid".to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_err());

            // Test valid default configuration
            let config = HtmlConfig::default();
            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_config_builder() {
            let result = HtmlConfigBuilder::new()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_language("en-GB")
                .build();

            assert!(result.is_ok());
            let config = result.unwrap();
            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("monokai".to_string())
            );
            assert_eq!(config.language, "en-GB");
        }

        #[test]
        fn test_config_builder_invalid() {
            let result = HtmlConfigBuilder::new()
                .with_language("invalid")
                .build();

            assert!(matches!(
                result,
                Err(HtmlError::InvalidInput(msg)) if msg.contains("Invalid language code")
            ));
        }

        #[test]
        fn test_html_config_with_no_syntax_theme() {
            let config = HtmlConfig {
                enable_syntax_highlighting: true,
                syntax_theme: None,
                ..Default::default()
            };

            assert!(config.validate().is_ok());
        }

        #[test]
        fn test_file_conversion_with_large_output() -> Result<()> {
            let temp_dir = setup_test_dir();
            let input_path = create_test_file(
                &temp_dir,
                "# Large\n\nContent".repeat(10_000).as_str(),
            );
            let output_path = temp_dir.path().join("large_output.html");

            let result = markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                None,
            );

            assert!(result.is_ok());
            let content = std::fs::read_to_string(output_path)?;
            assert!(content.contains("<h1>Large</h1>"));

            Ok(())
        }

        #[test]
        fn test_markdown_with_broken_syntax() {
            let markdown = "# Unmatched Header\n**Bold start";
            let result = markdown_to_html(markdown, None);
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.contains("<h1>Unmatched Header</h1>"));
            assert!(html.contains("**Bold start</p>")); // Ensure content is preserved
        }

        #[test]
        fn test_language_code_with_custom_regex() {
            let custom_lang_regex =
                Regex::new(r"^[a-z]{2}-[A-Z]{2}$").unwrap();
            assert!(custom_lang_regex.is_match("en-GB"));
            assert!(!custom_lang_regex.is_match("EN-gb")); // Case-sensitive check
        }

        #[test]
        fn test_markdown_to_html_error_handling() {
            let result = markdown_to_html("", None);
            assert!(matches!(result, Err(HtmlError::InvalidInput(_))));

            let oversized_input =
                "a".repeat(constants::DEFAULT_MAX_INPUT_SIZE + 1);
            let result = markdown_to_html(&oversized_input, None);
            assert!(matches!(result, Err(HtmlError::InputTooLarge(_))));
        }

        #[test]
        fn test_performance_with_nested_lists() {
            let nested_list = "- Item\n".repeat(1000);
            let result = markdown_to_html(&nested_list, None);
            assert!(result.is_ok());
            let html = result.unwrap();
            assert!(html.matches("<li>").count() == 1000);
        }
    }

    mod file_validation_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_valid_paths() {
            let valid_paths = [
                PathBuf::from("test.md"),
                PathBuf::from("test.html"),
                PathBuf::from("subfolder/test.md"),
            ];

            for path in valid_paths {
                assert!(
                    HtmlConfig::validate_file_path(&path).is_ok(),
                    "Path should be valid: {:?}",
                    path
                );
            }
        }

        #[test]
        fn test_invalid_paths() {
            let invalid_paths = [
                PathBuf::from(""),           // Empty path
                PathBuf::from("../test.md"), // Directory traversal
                PathBuf::from("test.exe"),   // Invalid extension
                PathBuf::from(
                    "a".repeat(constants::MAX_PATH_LENGTH + 1),
                ), // Too long
            ];

            for path in invalid_paths {
                assert!(
                    HtmlConfig::validate_file_path(&path).is_err(),
                    "Path should be invalid: {:?}",
                    path
                );
            }
        }
    }

    mod markdown_conversion_tests {
        use super::*;

        #[test]
        fn test_basic_conversion() {
            let markdown = "# Test\n\nHello world";
            let result = markdown_to_html(markdown, None);
            assert!(result.is_ok());

            let html = result.unwrap();
            assert!(html.contains("<h1>Test</h1>"));
            assert!(html.contains("<p>Hello world</p>"));
        }

        #[test]
        fn test_conversion_with_config() {
            let markdown = "# Test\n```rust\nfn main() {}\n```";
            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            let result = markdown_to_html(markdown, Some(config));
            assert!(result.is_ok());
            assert!(result.unwrap().contains("language-rust"));
        }

        #[test]
        fn test_empty_content() {
            assert!(matches!(
                markdown_to_html("", None),
                Err(HtmlError::InvalidInput(_))
            ));
        }

        #[test]
        fn test_content_too_large() {
            let large_content =
                "a".repeat(constants::DEFAULT_MAX_INPUT_SIZE + 1);
            assert!(matches!(
                markdown_to_html(&large_content, None),
                Err(HtmlError::InputTooLarge(_))
            ));
        }
    }

    mod file_operation_tests {
        use super::*;

        #[test]
        fn test_file_conversion() -> Result<()> {
            let temp_dir = setup_test_dir();
            let input_path =
                create_test_file(&temp_dir, "# Test\n\nHello world");
            let output_path = temp_dir.path().join("test.html");

            markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                None::<MarkdownConfig>,
            )?;

            let content = std::fs::read_to_string(output_path)?;
            assert!(content.contains("<h1>Test</h1>"));

            Ok(())
        }

        #[test]
        fn test_writer_output() {
            let temp_dir = setup_test_dir();
            let input_path =
                create_test_file(&temp_dir, "# Test\nHello");
            let buffer = Box::new(Cursor::new(Vec::new()));

            let result = markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::Writer(buffer)),
                None,
            );

            assert!(result.is_ok());
        }

        #[test]
        fn test_writer_output_no_input() {
            let buffer = Box::new(Cursor::new(Vec::new()));

            let result = markdown_file_to_html(
                Some(Path::new("nonexistent.md")),
                Some(OutputDestination::Writer(buffer)),
                None,
            );

            assert!(result.is_err());
        }
    }

    mod language_validation_tests {
        use super::*;

        #[test]
        fn test_valid_language_codes() {
            let valid_codes =
                ["en-GB", "fr-FR", "de-DE", "es-ES", "zh-CN"];

            for code in valid_codes {
                assert!(
                    validate_language_code(code),
                    "Language code '{}' should be valid",
                    code
                );
            }
        }

        #[test]
        fn test_invalid_language_codes() {
            let invalid_codes = [
                "",        // Empty
                "en",      // Missing region
                "eng-GBR", // Wrong format
                "en_GB",   // Wrong separator
                "123-45",  // Invalid characters
                "GB-en",   // Wrong order
                "en-gb",   // Wrong case
            ];

            for code in invalid_codes {
                assert!(
                    !validate_language_code(code),
                    "Language code '{}' should be invalid",
                    code
                );
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_end_to_end_conversion() -> Result<()> {
            let temp_dir = setup_test_dir();
            let content = r#"---
title: Test Document
---

# Hello World

This is a test document with:
- A list
- And some **bold** text
"#;
            let input_path = create_test_file(&temp_dir, content);
            let output_path = temp_dir.path().join("test.html");

            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    generate_toc: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                Some(config),
            )?;

            let html = std::fs::read_to_string(&output_path)?;
            assert!(html.contains("<h1>Hello World</h1>"));
            assert!(html.contains("<strong>bold</strong>"));
            assert!(html.contains("<ul>"));

            Ok(())
        }

        #[test]
        fn test_output_destination_debug() {
            assert_eq!(
                format!(
                    "{:?}",
                    OutputDestination::File("test.html".to_string())
                ),
                r#"File("test.html")"#
            );
            assert_eq!(
                format!("{:?}", OutputDestination::Stdout),
                "Stdout"
            );

            let writer = Box::new(Cursor::new(Vec::new()));
            assert_eq!(
                format!("{:?}", OutputDestination::Writer(writer)),
                "Writer(<dyn Write>)"
            );
        }
    }

    mod markdown_config_tests {
        use super::*;

        #[test]
        fn test_markdown_config_custom_encoding() {
            let config = MarkdownConfig {
                encoding: "latin1".to_string(),
                html_config: HtmlConfig::default(),
            };
            assert_eq!(config.encoding, "latin1");
        }

        #[test]
        fn test_markdown_config_default() {
            let config = MarkdownConfig::default();
            assert_eq!(config.encoding, "utf-8");
            assert_eq!(config.html_config, HtmlConfig::default());
        }

        #[test]
        fn test_markdown_config_clone() {
            let config = MarkdownConfig::default();
            let cloned = config.clone();
            assert_eq!(config, cloned);
        }
    }

    mod config_error_tests {
        use super::*;

        #[test]
        fn test_config_error_display() {
            let error = ConfigError::InvalidInputSize(100, 1024);
            assert!(error.to_string().contains("Invalid input size"));

            let error =
                ConfigError::InvalidLanguageCode("xx".to_string());
            assert!(error
                .to_string()
                .contains("Invalid language code"));

            let error =
                ConfigError::InvalidFilePath("../bad/path".to_string());
            assert!(error.to_string().contains("Invalid file path"));
        }
    }

    mod output_destination_tests {
        use super::*;

        #[test]
        fn test_output_destination_default() {
            assert!(matches!(
                OutputDestination::default(),
                OutputDestination::Stdout
            ));
        }

        #[test]
        fn test_output_destination_file() {
            let dest = OutputDestination::File("test.html".to_string());
            assert!(matches!(dest, OutputDestination::File(_)));
        }

        #[test]
        fn test_output_destination_writer() {
            let writer = Box::new(Cursor::new(Vec::new()));
            let dest = OutputDestination::Writer(writer);
            assert!(matches!(dest, OutputDestination::Writer(_)));
        }
    }

    mod html_config_tests {
        use super::*;

        #[test]
        fn test_html_config_builder_all_options() {
            let config = HtmlConfig::builder()
                .with_syntax_highlighting(
                    true,
                    Some("dracula".to_string()),
                )
                .with_language("en-US")
                .build()
                .unwrap();

            assert!(config.enable_syntax_highlighting);
            assert_eq!(
                config.syntax_theme,
                Some("dracula".to_string())
            );
            assert_eq!(config.language, "en-US");
        }

        #[test]
        fn test_html_config_validation_edge_cases() {
            let config = HtmlConfig {
                max_input_size: constants::MIN_INPUT_SIZE,
                ..Default::default()
            };
            assert!(config.validate().is_ok());

            let config = HtmlConfig {
                max_input_size: constants::MIN_INPUT_SIZE - 1,
                ..Default::default()
            };
            assert!(config.validate().is_err());
        }
    }

    mod markdown_processing_tests {
        use super::*;

        #[test]
        fn test_markdown_to_html_with_front_matter() -> Result<()> {
            let markdown = r#"---
title: Test
author: Test Author
---
# Heading
Content"#;
            let html = markdown_to_html(markdown, None)?;
            assert!(html.contains("<h1>Heading</h1>"));
            assert!(html.contains("<p>Content</p>"));
            Ok(())
        }

        #[test]
        fn test_markdown_to_html_with_code_blocks() -> Result<()> {
            let markdown = r#"```rust
fn main() {
    println!("Hello");
}
```"#;
            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            let html = markdown_to_html(markdown, Some(config))?;
            assert!(html.contains("language-rust"));
            Ok(())
        }

        #[test]
        fn test_markdown_to_html_with_tables() -> Result<()> {
            let markdown = r#"
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
"#;
            let html = markdown_to_html(markdown, None)?;
            // First verify the HTML output to see what we're getting
            println!("Generated HTML for table: {}", html);
            // Check for common table elements - div wrapper is often used for table responsiveness
            assert!(html.contains("Header 1"));
            assert!(html.contains("Cell 1"));
            assert!(html.contains("Cell 2"));
            Ok(())
        }

        #[test]
        fn test_invalid_encoding_handling() {
            let config = MarkdownConfig {
                encoding: "unsupported-encoding".to_string(),
                html_config: HtmlConfig::default(),
            };
            // Simulate usage where encoding matters
            let result = markdown_to_html("# Test", Some(config));
            assert!(result.is_ok()); // Assuming encoding isn't directly validated during processing
        }

        #[test]
        fn test_config_error_types() {
            let error = ConfigError::InvalidInputSize(512, 1024);
            assert_eq!(format!("{}", error), "Invalid input size: 512 bytes is below minimum of 1024 bytes");
        }
    }

    mod file_processing_tests {
        use crate::constants;
        use crate::HtmlConfig;
        use crate::{
            markdown_file_to_html, HtmlError, OutputDestination,
        };
        use std::io::Cursor;
        use std::path::Path;
        use tempfile::NamedTempFile;

        #[test]
        fn test_display_file() {
            let output =
                OutputDestination::File("output.html".to_string());
            let display = format!("{}", output);
            assert_eq!(display, "File(output.html)");
        }

        #[test]
        fn test_display_stdout() {
            let output = OutputDestination::Stdout;
            let display = format!("{}", output);
            assert_eq!(display, "Stdout");
        }

        #[test]
        fn test_display_writer() {
            let buffer = Cursor::new(Vec::new());
            let output = OutputDestination::Writer(Box::new(buffer));
            let display = format!("{}", output);
            assert_eq!(display, "Writer(<dyn Write>)");
        }

        #[test]
        fn test_debug_file() {
            let output =
                OutputDestination::File("output.html".to_string());
            let debug = format!("{:?}", output);
            assert_eq!(debug, r#"File("output.html")"#);
        }

        #[test]
        fn test_debug_stdout() {
            let output = OutputDestination::Stdout;
            let debug = format!("{:?}", output);
            assert_eq!(debug, "Stdout");
        }

        #[test]
        fn test_debug_writer() {
            let buffer = Cursor::new(Vec::new());
            let output = OutputDestination::Writer(Box::new(buffer));
            let debug = format!("{:?}", output);
            assert_eq!(debug, "Writer(<dyn Write>)");
        }

        #[test]
        fn test_file_to_html_invalid_input() {
            let result = markdown_file_to_html(
                Some(Path::new("nonexistent.md")),
                None,
                None,
            );
            assert!(matches!(result, Err(HtmlError::Io(_))));
        }

        #[test]
        fn test_file_to_html_with_invalid_output_path(
        ) -> Result<(), HtmlError> {
            let input = NamedTempFile::new()?;
            std::fs::write(&input, "# Test")?;

            let result = markdown_file_to_html(
                Some(input.path()),
                Some(OutputDestination::File(
                    "/invalid/path/test.html".to_string(),
                )),
                None,
            );
            assert!(result.is_err());
            Ok(())
        }

        // Test for Default implementation of OutputDestination
        #[test]
        fn test_output_destination_default() {
            let default = OutputDestination::default();
            assert!(matches!(default, OutputDestination::Stdout));
        }

        // Test for Debug implementation of OutputDestination
        #[test]
        fn test_output_destination_debug() {
            let file_debug = format!(
                "{:?}",
                OutputDestination::File(
                    "path/to/file.html".to_string()
                )
            );
            assert_eq!(file_debug, r#"File("path/to/file.html")"#);

            let writer_debug = format!(
                "{:?}",
                OutputDestination::Writer(Box::new(Cursor::new(
                    Vec::new()
                )))
            );
            assert_eq!(writer_debug, "Writer(<dyn Write>)");

            let stdout_debug =
                format!("{:?}", OutputDestination::Stdout);
            assert_eq!(stdout_debug, "Stdout");
        }

        // Test for Display implementation of OutputDestination
        #[test]
        fn test_output_destination_display() {
            let file_display = format!(
                "{}",
                OutputDestination::File(
                    "path/to/file.html".to_string()
                )
            );
            assert_eq!(file_display, "File(path/to/file.html)");

            let writer_display = format!(
                "{}",
                OutputDestination::Writer(Box::new(Cursor::new(
                    Vec::new()
                )))
            );
            assert_eq!(writer_display, "Writer(<dyn Write>)");

            let stdout_display =
                format!("{}", OutputDestination::Stdout);
            assert_eq!(stdout_display, "Stdout");
        }

        // Test for Default implementation of HtmlConfig
        #[test]
        fn test_html_config_default() {
            let default = HtmlConfig::default();
            assert!(default.enable_syntax_highlighting);
            assert_eq!(
                default.syntax_theme,
                Some(constants::DEFAULT_SYNTAX_THEME.to_string())
            );
            assert!(!default.minify_output);
            assert!(default.add_aria_attributes);
            assert!(!default.generate_structured_data);
            assert_eq!(
                default.max_input_size,
                constants::DEFAULT_MAX_INPUT_SIZE
            );
            assert_eq!(
                default.language,
                constants::DEFAULT_LANGUAGE.to_string()
            );
            assert!(!default.generate_toc);
        }

        // Test for HtmlConfigBuilder
        #[test]
        fn test_html_config_builder() {
            let builder = HtmlConfig::builder()
                .with_syntax_highlighting(
                    true,
                    Some("monokai".to_string()),
                )
                .with_language("en-US")
                .build()
                .unwrap();

            assert!(builder.enable_syntax_highlighting);
            assert_eq!(
                builder.syntax_theme,
                Some("monokai".to_string())
            );
            assert_eq!(builder.language, "en-US");
        }

        // Test for long file path validation
        #[test]
        fn test_long_file_path_validation() {
            let long_path = "a".repeat(constants::MAX_PATH_LENGTH + 1);
            let result = HtmlConfig::validate_file_path(long_path);
            assert!(
                matches!(result, Err(HtmlError::InvalidInput(ref msg)) if msg.contains("File path exceeds maximum length"))
            );
        }

        /// Absolute paths are deliberately accepted: CLI tools invoke the
        /// library with fully qualified filenames. Authorisation is the
        /// caller's responsibility; see [`HtmlConfig::validate_file_path`]
        /// docs.
        #[test]
        fn test_absolute_path_is_accepted() {
            let result = HtmlConfig::validate_file_path(
                "/absolute/path/to/file.md",
            );
            assert!(
                result.is_ok(),
                "absolute paths must be accepted, got {result:?}"
            );
        }

        /// NUL byte smuggling must be rejected — on Unix, C-string path
        /// handling silently truncates at the first NUL.
        #[test]
        fn test_nul_byte_path_is_rejected() {
            let result = HtmlConfig::validate_file_path("safe.md\0bad");
            assert!(
                matches!(result, Err(HtmlError::InvalidInput(ref msg)) if msg.contains("NUL")),
                "NUL byte in path must be rejected, got {result:?}"
            );
        }
    }

    mod language_validation_extended_tests {
        use super::*;

        #[test]
        fn test_language_code_edge_cases() {
            // Test empty string
            assert!(!validate_language_code(""));

            // Test single character
            assert!(!validate_language_code("a"));

            // Test incorrect casing
            assert!(!validate_language_code("EN-GB"));
            assert!(!validate_language_code("en-gb"));

            // Test invalid separators
            assert!(!validate_language_code("en_GB"));
            assert!(!validate_language_code("en GB"));

            // Test too many segments
            assert!(!validate_language_code("en-GB-extra"));
        }

        #[test]
        fn test_language_code_special_cases() {
            // Test with numbers
            assert!(!validate_language_code("e1-GB"));
            assert!(!validate_language_code("en-G1"));

            // Test with special characters
            assert!(!validate_language_code("en-GB!"));
            assert!(!validate_language_code("en@GB"));

            // Test with Unicode characters
            assert!(!validate_language_code("あa-GB"));
            assert!(!validate_language_code("en-あa"));
        }
    }

    mod integration_extended_tests {
        use super::*;

        #[test]
        fn test_full_conversion_pipeline() -> Result<()> {
            // Create temporary files
            let temp_dir = tempdir()?;
            let input_path = temp_dir.path().join("test.md");
            let output_path = temp_dir.path().join("test.html");

            // Test content with various Markdown features
            let content = r#"---
title: Test Document
author: Test Author
---

# Main Heading

## Subheading

This is a paragraph with *italic* and **bold** text.

- List item 1
- List item 2
  - Nested item
  - Another nested item

```rust
fn main() {
    println!("Hello, world!");
}
```

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |

> This is a blockquote

[Link text](https://example.com)"#;

            std::fs::write(&input_path, content)?;

            // Configure with all features enabled
            let config = MarkdownConfig {
                html_config: HtmlConfig {
                    enable_syntax_highlighting: true,
                    generate_toc: true,
                    add_aria_attributes: true,
                    generate_structured_data: true,
                    minify_output: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            markdown_file_to_html(
                Some(&input_path),
                Some(OutputDestination::File(
                    output_path.to_string_lossy().into(),
                )),
                Some(config),
            )?;

            let html = std::fs::read_to_string(&output_path)?;

            // Verify all expected elements are present
            println!("Generated HTML: {}", html);
            assert!(html.contains("<h1>"));
            assert!(html.contains("<h2>"));
            assert!(html.contains("<em>"));
            assert!(html.contains("<strong>"));
            assert!(html.contains("<ul>"));
            assert!(html.contains("<li>"));
            assert!(html.contains("language-rust"));

            // Verify table content instead of specific HTML structure
            assert!(html.contains("Column 1"));
            assert!(html.contains("Column 2"));
            assert!(html.contains("Cell 1"));
            assert!(html.contains("Cell 2"));

            assert!(html.contains("<blockquote>"));
            assert!(html.contains("<a href="));

            Ok(())
        }

        #[test]
        fn test_missing_html_config_fallback() {
            let config = MarkdownConfig {
                encoding: "utf-8".to_string(),
                html_config: HtmlConfig {
                    enable_syntax_highlighting: false,
                    syntax_theme: None,
                    ..Default::default()
                },
            };
            let result = markdown_to_html("# Test", Some(config));
            assert!(result.is_ok());
        }

        #[test]
        fn test_invalid_output_destination() {
            let result = markdown_file_to_html(
                Some(Path::new("test.md")),
                Some(OutputDestination::File(
                    "/root/forbidden.html".to_string(),
                )),
                None,
            );
            assert!(result.is_err());
        }
    }

    mod performance_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_large_document_performance() -> Result<()> {
            let base_content =
                "# Heading\n\nParagraph\n\n- List item\n\n";
            let large_content = base_content.repeat(1000);

            let start = Instant::now();
            let html = markdown_to_html(&large_content, None)?;
            let duration = start.elapsed();

            // Log performance metrics
            println!("Large document conversion took: {:?}", duration);
            println!("Input size: {} bytes", large_content.len());
            println!("Output size: {} bytes", html.len());

            // Basic validation
            assert!(html.contains("<h1>"));
            assert!(html.contains("<p>"));
            assert!(html.contains("<ul>"));

            Ok(())
        }
    }
}

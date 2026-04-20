// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! HTML5 semantic element builders.
//!
//! This module provides type-safe builders for modern HTML5 semantic
//! elements with built-in ARIA attribute support and accessibility
//! validation.
//!
//! # Examples
//!
//! ```
//! use html_generator::elements::{Article, Section, Nav, Aside, Template};
//!
//! let nav = Nav::new()
//!     .aria_label("Main navigation")
//!     .id("main-nav")
//!     .child("<ul><li><a href=\"/\">Home</a></li></ul>")
//!     .build();
//! assert!(nav.contains("role=\"navigation\""));
//! assert!(nav.contains("aria-label=\"Main navigation\""));
//! ```

use crate::seo::escape_html;
use std::collections::HashMap;

/// Builder for `<article>` elements.
///
/// Represents a self-contained composition in a document, such as a
/// blog post, news article, or forum post.
#[derive(Debug, Clone, Default)]
pub struct Article {
    id: Option<String>,
    class: Option<String>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    attrs: HashMap<String, String>,
    children: Vec<String>,
}

/// Builder for `<section>` elements.
///
/// Represents a standalone section of a document, typically with a
/// heading.
#[derive(Debug, Clone, Default)]
pub struct Section {
    id: Option<String>,
    class: Option<String>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    attrs: HashMap<String, String>,
    children: Vec<String>,
}

/// Builder for `<nav>` elements.
///
/// Represents a navigation section containing links to other pages or
/// sections within the page.
#[derive(Debug, Clone, Default)]
pub struct Nav {
    id: Option<String>,
    class: Option<String>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    attrs: HashMap<String, String>,
    children: Vec<String>,
}

/// Builder for `<aside>` elements.
///
/// Represents content tangentially related to the surrounding content,
/// such as sidebars, pull quotes, or advertising.
#[derive(Debug, Clone, Default)]
pub struct Aside {
    id: Option<String>,
    class: Option<String>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    attrs: HashMap<String, String>,
    children: Vec<String>,
}

/// Template for composing multiple semantic elements into a page
/// structure.
///
/// Provides a high-level API for building accessible HTML5 document
/// layouts.
///
/// # Examples
///
/// ```
/// use html_generator::elements::Template;
///
/// let page = Template::new()
///     .nav("Main navigation", "<ul><li>Home</li></ul>")
///     .main_content("<h1>Welcome</h1><p>Content here.</p>")
///     .aside("Related", "<p>Related links</p>")
///     .build();
/// assert!(page.contains("<nav"));
/// assert!(page.contains("<main"));
/// assert!(page.contains("<aside"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Template {
    nav: Option<String>,
    header: Option<String>,
    main: Option<String>,
    sections: Vec<String>,
    aside: Option<String>,
    footer: Option<String>,
}

// ─── Shared builder trait ──────────────────────────────────────────

/// Trait implemented by all semantic element builders.
pub trait SemanticElement {
    /// Render the element to an HTML string.
    fn build(&self) -> String;
}

// ─── Macro to reduce repetition ───────────────────────────────────

macro_rules! impl_element_builder {
    ($type:ident, $tag:expr, $role:expr) => {
        impl $type {
            /// Creates a new builder with default values.
            #[must_use]
            pub fn new() -> Self {
                Self::default()
            }

            /// Sets the `id` attribute.
            #[must_use]
            pub fn id(mut self, id: &str) -> Self {
                self.id = Some(id.to_string());
                self
            }

            /// Sets the `class` attribute.
            #[must_use]
            pub fn class(mut self, class: &str) -> Self {
                self.class = Some(class.to_string());
                self
            }

            /// Sets the `aria-label` attribute.
            #[must_use]
            pub fn aria_label(mut self, label: &str) -> Self {
                self.aria_label = Some(label.to_string());
                self
            }

            /// Sets the `aria-labelledby` attribute.
            #[must_use]
            pub fn aria_labelledby(mut self, id: &str) -> Self {
                self.aria_labelledby = Some(id.to_string());
                self
            }

            /// Adds a custom attribute.
            #[must_use]
            pub fn attr(mut self, key: &str, value: &str) -> Self {
                let _ = self
                    .attrs
                    .insert(key.to_string(), value.to_string());
                self
            }

            /// Appends child HTML content.
            #[must_use]
            pub fn child(mut self, html: &str) -> Self {
                self.children.push(html.to_string());
                self
            }

            /// Appends multiple child HTML content strings.
            #[must_use]
            pub fn children(mut self, items: &[&str]) -> Self {
                for item in items {
                    self.children.push((*item).to_string());
                }
                self
            }

            /// Renders the element to an HTML string.
            #[must_use]
            pub fn build(&self) -> String {
                let mut parts = Vec::new();
                parts.push(format!("<{}", $tag));

                if let Some(ref role) = Some($role) {
                    if !role.is_empty() {
                        parts.push(format!(" role=\"{}\"", role));
                    }
                }

                if let Some(ref id) = self.id {
                    parts.push(format!(" id=\"{}\"", escape_html(id)));
                }
                if let Some(ref class) = self.class {
                    parts.push(format!(
                        " class=\"{}\"",
                        escape_html(class)
                    ));
                }
                if let Some(ref label) = self.aria_label {
                    parts.push(format!(
                        " aria-label=\"{}\"",
                        escape_html(label)
                    ));
                }
                if let Some(ref id) = self.aria_labelledby {
                    parts.push(format!(
                        " aria-labelledby=\"{}\"",
                        escape_html(id)
                    ));
                }

                for (key, value) in &self.attrs {
                    parts.push(format!(
                        " {}=\"{}\"",
                        escape_html(key),
                        escape_html(value)
                    ));
                }

                parts.push(">".to_string());

                for child in &self.children {
                    parts.push(child.clone());
                }

                parts.push(format!("</{}>", $tag));
                parts.concat()
            }
        }

        impl SemanticElement for $type {
            fn build(&self) -> String {
                $type::build(self)
            }
        }
    };
}

impl_element_builder!(Article, "article", "article");
impl_element_builder!(Section, "section", "region");
impl_element_builder!(Nav, "nav", "navigation");
impl_element_builder!(Aside, "aside", "complementary");

// ─── Template implementation ──────────────────────────────────────

impl Template {
    /// Creates a new empty template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a navigation section.
    #[must_use]
    pub fn nav(mut self, label: &str, content: &str) -> Self {
        self.nav =
            Some(Nav::new().aria_label(label).child(content).build());
        self
    }

    /// Adds a header section.
    #[must_use]
    pub fn header(mut self, content: &str) -> Self {
        self.header = Some(format!("<header>{content}</header>"));
        self
    }

    /// Adds the main content area.
    #[must_use]
    pub fn main_content(mut self, content: &str) -> Self {
        self.main =
            Some(format!("<main role=\"main\">{content}</main>"));
        self
    }

    /// Adds a section to the template.
    #[must_use]
    pub fn section(mut self, label: &str, content: &str) -> Self {
        self.sections.push(
            Section::new().aria_label(label).child(content).build(),
        );
        self
    }

    /// Adds an aside section.
    #[must_use]
    pub fn aside(mut self, label: &str, content: &str) -> Self {
        self.aside =
            Some(Aside::new().aria_label(label).child(content).build());
        self
    }

    /// Adds a footer section.
    #[must_use]
    pub fn footer(mut self, content: &str) -> Self {
        self.footer = Some(format!(
            "<footer role=\"contentinfo\">{content}</footer>"
        ));
        self
    }

    /// Renders the full template to an HTML string.
    #[must_use]
    pub fn build(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref nav) = self.nav {
            parts.push(nav.as_str());
        }
        if let Some(ref header) = self.header {
            parts.push(header.as_str());
        }
        if let Some(ref main) = self.main {
            parts.push(main.as_str());
        }
        for section in &self.sections {
            parts.push(section.as_str());
        }
        if let Some(ref aside) = self.aside {
            parts.push(aside.as_str());
        }
        if let Some(ref footer) = self.footer {
            parts.push(footer.as_str());
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_builder() {
        let html = Article::new()
            .id("post-1")
            .class("blog-post")
            .aria_label("Blog post about Rust")
            .child("<h2>Learning Rust</h2>")
            .child("<p>Rust is great.</p>")
            .build();

        assert!(html.contains("<article"));
        assert!(html.contains("role=\"article\""));
        assert!(html.contains("id=\"post-1\""));
        assert!(html.contains("class=\"blog-post\""));
        assert!(html.contains("aria-label=\"Blog post about Rust\""));
        assert!(html.contains("<h2>Learning Rust</h2>"));
        assert!(html.contains("</article>"));
    }

    #[test]
    fn test_section_builder() {
        let html = Section::new()
            .aria_label("Introduction")
            .child("<h2>Intro</h2>")
            .build();

        assert!(html.contains("<section"));
        assert!(html.contains("role=\"region\""));
        assert!(html.contains("aria-label=\"Introduction\""));
        assert!(html.contains("</section>"));
    }

    #[test]
    fn test_nav_builder() {
        let html = Nav::new()
            .id("main-nav")
            .aria_label("Main navigation")
            .child("<ul><li>Home</li></ul>")
            .build();

        assert!(html.contains("<nav"));
        assert!(html.contains("role=\"navigation\""));
        assert!(html.contains("aria-label=\"Main navigation\""));
        assert!(html.contains("id=\"main-nav\""));
        assert!(html.contains("</nav>"));
    }

    #[test]
    fn test_aside_builder() {
        let html = Aside::new()
            .aria_label("Related links")
            .child("<p>See also...</p>")
            .build();

        assert!(html.contains("<aside"));
        assert!(html.contains("role=\"complementary\""));
        assert!(html.contains("</aside>"));
    }

    #[test]
    fn test_template_composition() {
        let page = Template::new()
            .nav("Site navigation", "<ul><li>Home</li></ul>")
            .header("<h1>My Site</h1>")
            .main_content("<p>Welcome!</p>")
            .section("About", "<p>About us</p>")
            .aside("Sidebar", "<p>Links</p>")
            .footer("<p>Copyright 2025</p>")
            .build();

        assert!(page.contains("<nav"));
        assert!(page.contains("<header>"));
        assert!(page.contains("<main role=\"main\">"));
        assert!(page.contains("<section"));
        assert!(page.contains("<aside"));
        assert!(page.contains("<footer"));
    }

    #[test]
    fn test_escapes_attributes() {
        let html = Nav::new()
            .aria_label("<script>alert('xss')</script>")
            .build();

        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_custom_attrs() {
        let html = Article::new()
            .attr("data-post-id", "42")
            .attr("itemscope", "")
            .child("<p>Content</p>")
            .build();

        assert!(html.contains("data-post-id=\"42\""));
    }
}

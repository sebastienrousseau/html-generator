//! src/examples/aria_elements_example.rs
#![allow(missing_docs)]

use html_generator::accessibility::add_aria_attributes;
use html_generator::error::HtmlError;

/// Custom result type for our examples
type Result<T> = std::result::Result<T, HtmlError>;

/// Convert accessibility errors to HtmlError
fn convert_error(e: html_generator::accessibility::Error) -> HtmlError {
    HtmlError::accessibility(
        html_generator::error::ErrorKind::Other,
        e.to_string(),
        None,
    )
}

/// Entry point for the ARIA elements generation examples.
fn main() -> Result<()> {
    println!("\n🧪 ARIA Elements Generation Examples\n");

    button_examples()?;
    navigation_examples()?;
    form_examples()?;
    landmark_examples()?;
    interactive_elements_examples()?;
    modal_dialog_examples()?;
    table_examples()?;
    live_region_examples()?;

    // New: Additional emoji-based tests
    extra_emoji_examples()?;

    println!("\n🎉 All ARIA element examples completed successfully!");
    Ok(())
}

/// Demonstrates ARIA attributes for **button elements**.
fn button_examples() -> Result<()> {
    println!("🦀 Button ARIA Examples");
    println!("---------------------------------------------");

    // Flattened array of (html, description) tuples
    let examples = [
        // Basic button
        ("<button>Click me</button>", "Basic button"),
        // Icon button examples
        (
            r#"<button><span class="icon">🔍</span></button>"#,
            "Icon button - magnifying glass",
        ),
        (
            r#"<button><span class="icon">🔎</span></button>"#,
            "Icon button - zoom in",
        ),
        (
            r#"<button><span class="icon">🔧</span></button>"#,
            "Icon button - wrench",
        ),
        (
            r#"<button><span class="icon">⚙️</span></button>"#,
            "Icon button - gear",
        ),
        (
            r#"<button><span class="icon">✏️</span></button>"#,
            "Icon button - pencil",
        ),
        (
            r#"<button><span class="icon">🗑️</span></button>"#,
            "Icon button - trash",
        ),
        // Toggle button
        (r#"<button type="button">Menu</button>"#, "Toggle button"),
        // Disabled button
        (r#"<button disabled>Submit</button>"#, "Disabled button"),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **navigation elements**.
fn navigation_examples() -> Result<()> {
    println!("\n🦀 Navigation ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Main navigation
        ("<nav><ul><li>Home</li></ul></nav>", "Main navigation"),
        // Breadcrumb navigation
        (
            r#"<nav class="breadcrumb"><ol><li>Home</li><li>Products</li></ol></nav>"#,
            "Breadcrumb",
        ),
        // Secondary navigation
        (
            r#"<nav class="sidebar"><ul><li>Settings</li></ul></nav>"#,
            "Secondary navigation",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **form elements**.
fn form_examples() -> Result<()> {
    println!("\n🦀 Form ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Text input
        (
            r#"<input type="text" placeholder="Enter name">"#,
            "Text input",
        ),
        // Search input
        (
            r#"<input type="search" placeholder="Search...">"#,
            "Search input",
        ),
        // Checkbox
        (r#"<input type="checkbox" id="terms">"#, "Checkbox"),
        // Radio button group
        (
            r#"<div><input type="radio" name="option" value="1"><input type="radio" name="option" value="2"></div>"#,
            "Radio group",
        ),
        // Select dropdown
        (
            r#"<select><option>Choose...</option></select>"#,
            "Select dropdown",
        ),
        // Textarea
        (
            r#"<textarea placeholder="Enter description"></textarea>"#,
            "Textarea",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **landmark regions**.
fn landmark_examples() -> Result<()> {
    println!("\n🦀 Landmark ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Header
        (r#"<header><h1>Site Title</h1></header>"#, "Header landmark"),
        // Main content
        (
            r#"<main><article>Content</article></main>"#,
            "Main content landmark",
        ),
        // Complementary sidebar
        (
            r#"<aside><div>Related content</div></aside>"#,
            "Complementary landmark",
        ),
        // Footer
        (
            r#"<footer><p>Copyright 2024</p></footer>"#,
            "Footer landmark",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **interactive elements** (tabs, accordions, tooltips, menus).
fn interactive_elements_examples() -> Result<()> {
    println!("\n🦀 Interactive Elements ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Tabs
        (
            r#"<div role="tablist"><button>Tab 1</button><button>Tab 2</button></div>"#,
            "Tabs",
        ),
        // Accordion
        (
            r#"<div class="accordion"><button>Section 1</button><div>Content</div></div>"#,
            "Accordion",
        ),
        // Tooltip
        (r#"<button title="More information">?"#, "Tooltip"),
        // Menu
        (
            r#"<div class="menu"><button>Menu</button><ul><li>Item 1</li></ul></div>"#,
            "Menu",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **modal dialogs** (basic, alert, confirmation).
fn modal_dialog_examples() -> Result<()> {
    println!("\n🦀 Modal Dialog ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Basic modal
        (
            r#"<div class="modal"><div class="modal-content"><button>Close</button></div></div>"#,
            "Basic modal",
        ),
        // Alert dialog
        (
            r#"<div class="modal alert"><div class="modal-content"><h2>Warning</h2><button>OK</button></div></div>"#,
            "Alert dialog",
        ),
        // Confirmation dialog
        (
            r#"<div class="modal"><div class="modal-content"><h2>Confirm</h2><button>Yes</button><button>No</button></div></div>"#,
            "Confirmation dialog",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **tables** (basic, sortable, complex).
fn table_examples() -> Result<()> {
    println!("\n🦀 Table ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Basic table
        (
            r#"<table><tr><th>Header</th></tr><tr><td>Data</td></tr></table>"#,
            "Basic table",
        ),
        // Sortable table
        (
            r#"<table><tr><th class="sortable">Name</th></tr><tr><td>John</td></tr></table>"#,
            "Sortable table",
        ),
        // Complex table with spanning cells
        (
            r#"<table><tr><th colspan="2">Header</th></tr><tr><td>Data 1</td><td>Data 2</td></tr></table>"#,
            "Complex table",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// Demonstrates ARIA attributes for **live regions** (alert, status, etc.).
fn live_region_examples() -> Result<()> {
    println!("\n🦀 Live Region ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // Alert
        (
            r#"<div class="alert">Error occurred!</div>"#,
            "Alert region",
        ),
        // Status
        (r#"<div class="status">Loading...</div>"#, "Status region"),
        // Live region
        (
            r#"<div class="updates">New message received</div>"#,
            "Live region",
        ),
        // Timer
        (
            r#"<div class="timer">Time remaining: 5:00</div>"#,
            "Timer region",
        ),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

/// **New**: Demonstrates additional examples with various emojis
/// to ensure your emoji → label mapping is fully tested.
fn extra_emoji_examples() -> Result<()> {
    println!("\n🦀 Extra Emoji ARIA Examples");
    println!("---------------------------------------------");

    let examples = [
        // A button containing multiple emojis
        (
            r#"<button>🫷 🫸</button>"#,
            "Left-right pushing hand buttons",
        ),
        // A container with an emoji that might map to "edit"
        (r#"<div><span>📝</span> Note</div>"#, "Edit pencil icon"),
        // Another container with "close" X
        (r#"<div class="close-box">✖</div>"#, "Close box"),
        // Icon with variation selector
        (r#"<button><span>⚡️</span></button>"#, "Lightning icon"),
        // A random single-line usage
        (r#"<span>❓ Info needed</span>"#, "Question mark usage"),
        // Multiple different emojis
        (r#"<p>Play ▶ Pause ⏸ Stop ⏹</p>"#, "Media control emojis"),
    ];

    for (html, description) in examples {
        println!("\nTesting {}", description);
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        println!("Original:  {}", html);
        println!("Enhanced:  {}", enhanced);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_aria_attributes() -> Result<()> {
        let html = "<button>Click me</button>";
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        // Check for basic ARIA attributes on a button
        assert!(enhanced.contains("aria-label"));
        assert!(enhanced.contains(r#"role="button""#));
        Ok(())
    }

    #[test]
    fn test_navigation_aria_attributes() -> Result<()> {
        let html = "<nav><ul><li>Home</li></ul></nav>";
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        // Check for a role of navigation
        assert!(enhanced.contains("aria-label"));
        assert!(enhanced.contains(r#"role="navigation""#));
        Ok(())
    }

    #[test]
    fn test_form_aria_attributes() -> Result<()> {
        let html = r#"<input type="text" placeholder="Enter name">"#;
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        // Expect some ARIA-based labeling
        assert!(enhanced.contains("aria-label"));
        Ok(())
    }

    #[test]
    fn test_landmark_aria_attributes() -> Result<()> {
        let html = "<main><article>Content</article></main>";
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;
        // Typically, <main> is turned into role="main"
        assert!(enhanced.contains(r#"role="main""#));
        Ok(())
    }

    #[test]
    fn test_extra_emoji_examples() -> Result<()> {
        // Quick check on one of the new examples
        let html = r#"<button>🫷 🫸</button>"#;
        let enhanced =
            add_aria_attributes(html, None).map_err(convert_error)?;

        // If your loader maps "🫷" -> "leftwards" and "🫸" -> "rightwards",
        // you might get aria-label="leftwards-rightwards" or similar.
        // We'll just check that the aria-label is non-empty.
        assert!(enhanced.contains("aria-label"));
        Ok(())
    }
}

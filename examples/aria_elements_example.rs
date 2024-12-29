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

    // Existing examples
    button_examples()?;
    navigation_examples()?;
    form_examples()?;
    landmark_examples()?;
    interactive_elements_examples()?;
    modal_dialog_examples()?;
    table_examples()?;
    extra_emoji_examples()?;

    // New: Extended ARIA coverage and dynamic content examples
    dynamic_content_examples()?;
    nested_interactive_examples()?;
    complex_table_examples()?;
    live_region_examples()?;

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
        // Button with aria-haspopup for a dropdown menu
        (
            r#"<button aria-haspopup="true">Profile</button>"#,
            "Button with dropdown (aria-haspopup)",
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
        // Input with aria-describedby for additional help text
        (
            r#"<div>
                 <label for="username">Username</label>
                 <input type="text" id="username" aria-describedby="usernameHelp">
                 <small id="usernameHelp">No spaces allowed</small>
               </div>"#,
            "Input with aria-describedby",
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
        println!("This element has implicit ARIA roles, meaning it inherently conveys its landmark purpose to assistive technologies without needing explicit ARIA attributes. No major changes expected.");
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
        (r#"<button title="More information">?</button>"#, "Tooltip"),
        // Menu with aria-haspopup
        (
            r#"<button aria-haspopup="true">Open Menu</button><ul class="menu"><li>Item 1</li><li>Item 2</li></ul>"#,
            "Button + Menu",
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
            r#"<div class="modal" aria-hidden="true"><div class="modal-content"><button>Close</button></div></div>"#,
            "Basic modal",
        ),
        // Alert dialog
        (
            r#"<div class="modal alert" role="alertdialog"><div class="modal-content"><h2>Warning</h2><button>OK</button></div></div>"#,
            "Alert dialog",
        ),
        // Confirmation dialog
        (
            r#"<div class="modal" aria-modal="true"><div class="modal-content"><h2>Confirm</h2><button>Yes</button><button>No</button></div></div>"#,
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
            r#"<table>
                 <tr><th colspan="2">Header</th></tr>
                 <tr><td>Data 1</td><td>Data 2</td></tr>
               </table>"#,
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

/* ---------------------------------------------------------------------------
   NEW SECTIONS: Demonstrating dynamic content, nested elements, complex tables,
   and live regions, with additional context for each example.
---------------------------------------------------------------------------- */

/// **New**: Demonstrates how to manage ARIA attributes dynamically
/// (e.g., expanding/collapsing accordions, opening/closing modals).
fn dynamic_content_examples() -> Result<()> {
    println!("\n🦀 Dynamic Content ARIA Examples");
    println!("---------------------------------------------");
    println!("These examples illustrate how ARIA attributes can be updated at runtime.");

    // Example of an accordion with aria-expanded toggled dynamically
    let accordion_html = r#"
    <div class="accordion">
      <button aria-expanded="false" aria-controls="section1-content">
        Section 1
      </button>
      <div id="section1-content" hidden>
        <p>Content for section 1</p>
      </div>
    </div>
    "#;

    println!("\nTesting Accordion with aria-expanded toggle");
    println!("Explanation: The button uses `aria-expanded` to indicate the accordion state, while `aria-controls` associates it with the collapsible content.");
    let enhanced_accordion = add_aria_attributes(accordion_html, None)
        .map_err(convert_error)?;
    println!("Original:  {}", accordion_html);
    println!("Enhanced:  {}", enhanced_accordion);

    // Example of a modal that gets aria-hidden toggled
    let modal_html = r#"
    <div class="modal" aria-hidden="true">
      <div class="modal-content" role="dialog">
        <h2>Welcome</h2>
        <button>Close</button>
      </div>
    </div>
    "#;
    println!("\nTesting Modal with aria-hidden toggle");
    println!("Explanation: When the modal opens, `aria-hidden` should be set to `false`, and focus is trapped inside the dialog for screen reader clarity.");
    let enhanced_modal =
        add_aria_attributes(modal_html, None).map_err(convert_error)?;
    println!("Original:  {}", modal_html);
    println!("Enhanced:  {}", enhanced_modal);

    Ok(())
}

/// **New**: Demonstrates nested interactive elements (e.g., a menu within a modal dialog).
fn nested_interactive_examples() -> Result<()> {
    println!("\n🦀 Nested Interactive Elements ARIA Examples");
    println!("---------------------------------------------");
    println!("These scenarios illustrate layered UI components with proper ARIA relationships.");

    let nested_html = r##"
<div class="modal" aria-modal="true" role="dialog">
  <div class="modal-content">
    <h2>Settings</h2>
    <button aria-haspopup="true" aria-controls="settings-menu">Preferences</button>
    <ul id="settings-menu" class="menu" hidden>
      <li><a href="#profile">Profile</a></li>
      <li><a href="#privacy">Privacy</a></li>
    </ul>
    <button>Close</button>
  </div>
</div>
"##;

    println!("\nTesting Modal with nested menu");
    println!("Explanation: `role=\"dialog\"` identifies the modal. The `Preferences` button indicates a submenu with `aria-haspopup` and `aria-controls` linking to the hidden menu.");
    let enhanced_nested = add_aria_attributes(nested_html, None)
        .map_err(convert_error)?;
    println!("Original:  {}", nested_html);
    println!("Enhanced:  {}", enhanced_nested);

    Ok(())
}

/// **New**: Demonstrates more complex tables with row/column headers, summaries, and multi-level headers.
fn complex_table_examples() -> Result<()> {
    println!("\n🦀 Complex Table ARIA Examples");
    println!("---------------------------------------------");
    println!("Tables with row groups, column groups, and summary text for screen readers.");

    let complex_table_html = r#"
    <table aria-describedby="table-summary">
      <caption>Quarterly Financial Report</caption>
      <thead>
        <tr>
          <th rowspan="2">Region</th>
          <th colspan="2">Q1</th>
          <th colspan="2">Q2</th>
        </tr>
        <tr>
          <th>Revenue</th>
          <th>Expenses</th>
          <th>Revenue</th>
          <th>Expenses</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <th scope="row">North</th>
          <td>$120k</td>
          <td>$90k</td>
          <td>$150k</td>
          <td>$100k</td>
        </tr>
        <tr>
          <th scope="row">South</th>
          <td>$100k</td>
          <td>$70k</td>
          <td>$110k</td>
          <td>$80k</td>
        </tr>
      </tbody>
    </table>
    <div id="table-summary" hidden>
      This table shows quarterly revenue and expenses for different regions.
    </div>
    "#;

    println!("\nTesting complex table with row/column headers");
    println!("Explanation: Using `rowspan` and `colspan` properly, plus `aria-describedby` referencing a hidden summary for screen readers.");
    let enhanced_table = add_aria_attributes(complex_table_html, None)
        .map_err(convert_error)?;
    println!("Original:  {}", complex_table_html);
    println!("Enhanced:  {}", enhanced_table);

    Ok(())
}

/// **New**: Demonstrates ARIA live region usage for dynamic content updates.
fn live_region_examples() -> Result<()> {
    println!("\n🦀 Live Region ARIA Examples");
    println!("---------------------------------------------");
    println!("ARIA live regions help assistive technologies announce updates.");

    let live_region_html = r#"
    <div>
      <button id="notify-btn">Notify</button>
      <!--
        The aria-live attribute here informs screen readers that any text change
        within this container should be announced automatically.
      -->
      <div id="notification-area" aria-live="polite"></div>
    </div>
    "#;

    println!("\nTesting live region usage");
    println!("Explanation: `aria-live=\"polite\"` ensures new text in this region is read by screen readers without interrupting the user immediately.");
    let enhanced_live_region =
        add_aria_attributes(live_region_html, None)
            .map_err(convert_error)?;
    println!("Original:  {}", live_region_html);
    println!("Enhanced:  {}", enhanced_live_region);

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

    #[test]
    fn test_dynamic_content_examples() -> Result<()> {
        let accordion_html = r#"
        <div class="accordion">
          <button aria-expanded="false" aria-controls="section1-content">
            Section 1
          </button>
          <div id="section1-content" hidden>
            <p>Content for section 1</p>
          </div>
        </div>
        "#;

        let enhanced_accordion =
            add_aria_attributes(accordion_html, None)
                .map_err(convert_error)?;
        // Check for aria-expanded, aria-controls presence or correct usage
        assert!(enhanced_accordion.contains("aria-expanded"));
        assert!(enhanced_accordion.contains("aria-controls"));
        Ok(())
    }

    #[test]
    fn test_live_region_examples() -> Result<()> {
        let live_region_html = r#"
        <div>
          <button id="notify-btn">Notify</button>
          <div id="notification-area" aria-live="polite"></div>
        </div>
        "#;

        let enhanced_live_region =
            add_aria_attributes(live_region_html, None)
                .map_err(convert_error)?;
        // Check that aria-live attribute remains or is set
        assert!(enhanced_live_region.contains(r#"aria-live="polite""#));
        Ok(())
    }
}

//! src/examples/style_example.rs
#![allow(missing_docs)]

use html_generator::error::HtmlError;
use html_generator::generator::markdown_to_html_with_extensions;

/// A simple result type for our examples.
type Result<T> = std::result::Result<T, HtmlError>;

fn main() -> Result<()> {
    println!("\n🖌️  Markdown Style Examples\n");

    // 1) Demonstrate a single note block
    note_block_example()?;

    // 2) Demonstrate a warning block with multiline text
    warning_block_example()?;

    // 3) Demonstrate an image with .class="..."
    image_class_example()?;

    // 4) Demonstrate a short "long-form" snippet (heading, image, reference)
    long_form_example()?;

    // 5) Demonstrate a short Markdown table
    table_example()?;

    // 6) Demonstrate bullet & nested lists
    bullet_list_example()?;

    // 7) Demonstrate a blockquote with optional citation
    blockquote_example()?;

    // 8) Demonstrate a fenced code block
    code_block_example()?;

    println!(
        "\n🎉 All style/Markdown examples completed successfully!"
    );
    Ok(())
}

/// 1) Example: a note block (`:::note`) → <div class="alert alert-info"...>
fn note_block_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Custom Block Example: `:::note`");
    println!("────────────────────────────────────────────\n");

    let markdown = r":::note
This is a note with a custom class.
:::";

    println!("Testing `:::note` block...\n");

    // Show Markdown snippet, indented
    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!(); // blank line

    // Attempt to convert
    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully parsed `:::note` block.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 2) Example: a warning block (`:::warning`) with multiline text.
fn warning_block_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Custom Block Example: `:::warning`");
    println!("────────────────────────────────────────────\n");

    let markdown = r":::warning
**Caution:** This operation is sensitive and might lead to unexpected results.
Proceed carefully and confirm your backups are in place.
:::";

    println!("Testing `:::warning` block with multiline text...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!();

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully parsed `:::warning` block.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 3) Example: an inline image with `.class="img-fluid"`.
fn image_class_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Image + Class Example: `.class=\"img-fluid\"`");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"![A very tall building](https://example.com/image.webp).class="img-fluid""#;

    println!(
        "Testing an inline image with `.class=\"img-fluid\"`...\n"
    );

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!(); // blank line

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully parsed image with `.class=\"img-fluid\"`.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 4) A short "long-form" snippet with a heading, image, and reference link.
fn long_form_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Long-Form Snippet Example");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"## A Short News Heading

![Awesome Network](https://example.com/network.webp).class="fade-in w-25 p-3 float-start"

This article explores advanced technology in quantum computing.
[**Read more ❯**][01]

[01]: https://example.com/quantum-news "Quantum News"
"#;

    println!("Testing a small long-form snippet...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!(); // blank line

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!(
                "\n✅ Successfully handled small long-form snippet.\n"
            );
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 5) Example: A short Markdown table
fn table_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Markdown Table Example");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"| Project    | Language | Status  |
|------------|----------|---------|
| Shokunin   | Rust     | Active  |
| Pain001    | Rust     | Beta    |
| AudioTool  | Python   | Alpha   |
"#;

    println!("Testing a short table snippet...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!();

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully rendered a Markdown table.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 6) Bullet list & nested items
fn bullet_list_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Bullet List + Nested Items Example");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"* Item A
* Item B
  * Sub-item B1
  * Sub-item B2
* Item C
"#;

    println!("Testing a bullet list with nested items...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!();

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully rendered bullet list.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 7) Blockquote with an optional citation.
fn blockquote_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Blockquote + Citation Example");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"> “Imagination is more important than knowledge.”
> — *Albert Einstein*
"#;

    println!("Testing a blockquote with attribution...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!();

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!(
                "\n✅ Successfully rendered blockquote + citation.\n"
            );
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

/// 8) Fenced code block with syntax highlighting
fn code_block_example() -> Result<()> {
    println!("────────────────────────────────────────────");
    println!("🦀 Code Block Example");
    println!("────────────────────────────────────────────\n");

    let markdown = r#"```rust
fn main() {
    println!("Hello, world!");
}
```"#;

    println!("Testing a fenced code block with a Rust snippet...\n");

    println!("📄 Markdown Snippet:\n");
    for line in markdown.lines() {
        println!("   {line}");
    }
    println!();

    match markdown_to_html_with_extensions(markdown) {
        Ok(html) => {
            println!("🖥️ HTML Output:\n");
            for line in html.lines() {
                println!("   {line}");
            }
            println!("\n✅ Successfully rendered fenced code block.\n");
        }
        Err(e) => {
            println!("\n❌ Unexpected failure: {e}\n");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that headings, images, and references in the long snippet are correct.
    #[test]
    fn test_long_news_articles_example() -> Result<()> {
        let partial_markdown = r#"## All News Stories

![Alt text](https://example.com/image.webp).class="fade-in w-25"
"#;

        match markdown_to_html_with_extensions(partial_markdown) {
            Ok(html) => {
                // Expect success
                assert!(html.contains("<h2>All News Stories</h2>"));
                assert!(html.contains(r#"class="fade-in w-25""#));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

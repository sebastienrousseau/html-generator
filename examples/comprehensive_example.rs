//! # Batch Processing Markdown to HTML Example
//!
//! This example showcases the functionality of the `html-generator` library by
//! converting Markdown content from multiple sources into HTML.
//!
//! ## Features Highlighted
//! - Processes Markdown from a variety of sources, including basic and extended features.
//! - Displays the HTML output for each source.

use html_generator::{generate_html, HtmlConfig};
use std::collections::HashMap;

/// Entry point for the batch processing Markdown to HTML example.
///
/// Demonstrates conversion of Markdown from multiple sources into HTML.
///
/// # Errors
/// Returns an error if any Markdown to HTML conversion fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🦀 Welcome to the Batch Processing Markdown to HTML Example!");
    println!("================================================================");

    // Markdown sources from the `./basic` folder
    let basic_sources: HashMap<&str, &str> = vec![
        (
            "📝 Amps and Angle Encoding",
            include_str!("./basic/amps-and-angle-encoding.txt"),
        ),
        (
            "🔗 Angle Links and Images",
            include_str!("./basic/angle-links-and-img.txt"),
        ),
        ("🌐 Auto Links", include_str!("./basic/auto-links.txt")),
        (
            "🎯 Backlash Escapes",
            include_str!("./basic/backlash-escapes.txt"),
        ),
        (
            "📜 Blockquotes with Code Blocks",
            include_str!("./basic/blockquotes-with-code-blocks.txt"),
        ),
        (
            "💡 Code Syntax Highlighting",
            include_str!("./basic/code_syntax_highlighting.txt"),
        ),
        (
            "🔢 Code Block in List",
            include_str!("./basic/codeblock-in-list.txt"),
        ),
        (
            "📦 Custom Containers",
            include_str!("./basic/custom_containers.txt"),
        ),
        ("🕵️‍♂️ Edge Cases", include_str!("./basic/edge_cases.txt")),
        (
            "😀 Emoji Content",
            include_str!("./basic/emoji_content.txt"),
        ),
        (
            "🚩 Escaped Characters",
            include_str!("./basic/escaped_characters.txt"),
        ),
        (
            "⏩ Hard Wrapped Lines",
            include_str!("./basic/hard-wrapped.txt"),
        ),
        (
            "⏤ Horizontal Rules",
            include_str!("./basic/horizontal-rules.txt"),
        ),
        (
            "📚 Large Markdown File",
            include_str!("./basic/large_markdown.txt"),
        ),
        ("🔗 Inline Links", include_str!("./basic/links-inline.txt")),
        (
            "📖 Reference Links",
            include_str!("./basic/links-reference.txt"),
        ),
        (
            "🗨️ Literal Quotes",
            include_str!("./basic/literal-quotes.txt"),
        ),
        (
            "📚 Markdown Basics",
            include_str!("./basic/markdown-documentation-basics.txt"),
        ),
        (
            "📘 Markdown Syntax",
            include_str!("./basic/markdown-syntax.txt"),
        ),
        (
            "🗨️ Nested Blockquotes",
            include_str!("./basic/nested-blockquotes.txt"),
        ),
        (
            "🔢 Ordered and Unordered Lists",
            include_str!("./basic/ordered-and-unordered-list.txt"),
        ),
        (
            "💪 Strong and Emphasis Together",
            include_str!("./basic/strong-and-em-together.txt"),
        ),
        ("🗂️ Tabs", include_str!("./basic/tabs.txt")),
        ("🧹 Tidiness", include_str!("./basic/tidyness.txt")),
    ]
    .into_iter()
    .collect();

    // Markdown sources from the `./extensions` folder
    let extensions_sources: HashMap<&str, &str> = vec![
        ("📘 Admonition", include_str!("./extensions/admonition.txt")),
        (
            "⚙️ Attribute List",
            include_str!("./extensions/attr_list.txt"),
        ),
        ("✨ Codehilite", include_str!("./extensions/codehilite.txt")),
        (
            "🐙 GitHub Flavored Markdown",
            include_str!("./extensions/github_flavored.txt"),
        ),
        (
            "🌟 NL2BR with Attribute List",
            include_str!("./extensions/nl2br_w_attr_list.txt"),
        ),
        ("📋 Sane Lists", include_str!("./extensions/sane_lists.txt")),
        (
            "🗂️ Table of Contents (TOC)",
            include_str!("./extensions/toc.txt"),
        ),
        (
            "🚨 TOC Invalid",
            include_str!("./extensions/toc_invalid.txt"),
        ),
        (
            "📄 TOC Nested List",
            include_str!("./extensions/toc_nested_list.txt"),
        ),
        ("📂 TOC Nested", include_str!("./extensions/toc_nested.txt")),
    ]
    .into_iter()
    .collect();

    // Markdown sources from the `./misc` folder
    let misc_sources: HashMap<&str, &str> = vec![
        (
            "⚙️ CRLF Line Ends",
            include_str!("./misc/CRLF_line_ends.txt"),
        ),
        (
            "🔗 Adjacent Headers",
            include_str!("./misc/adjacent-headers.txt"),
        ),
        ("🌍 Arabic", include_str!("./misc/arabic.txt")),
        (
            "🔗 Autolinks with Asterisks",
            include_str!("./misc/autolinks_with_asterisks.txt"),
        ),
        (
            "🇷🇺 Autolinks with Asterisks (Russian)",
            include_str!("./misc/autolinks_with_asterisks_russian.txt"),
        ),
        (
            "🏷️ Backtick Escape",
            include_str!("./misc/backtick-escape.txt"),
        ),
        ("🔄 Bidi", include_str!("./misc/bidi.txt")),
        (
            "📜 Blank Block Quote",
            include_str!("./misc/blank-block-quote.txt"),
        ),
        (
            "🔲 Blank Lines in Codeblocks",
            include_str!("./misc/blank_lines_in_codeblocks.txt"),
        ),
        (
            "🖋️ Blockquote Below Paragraph",
            include_str!("./misc/blockquote-below-paragraph.txt"),
        ),
        (
            "⏤ Blockquote Horizontal Rule",
            include_str!("./misc/blockquote-hr.txt"),
        ),
        ("🗨️ Blockquote", include_str!("./misc/blockquote.txt")),
        ("🔗 Bold Links", include_str!("./misc/bold_links.txt")),
        ("⏎ Line Break", include_str!("./misc/br.txt")),
        (
            "🔎 Bracket Regular Expression",
            include_str!("./misc/bracket_re.txt"),
        ),
        (
            "🖼️ Brackets in Image Title",
            include_str!("./misc/brackets-in-img-title.txt"),
        ),
        (
            "🖋️ Code First Line",
            include_str!("./misc/code-first-line.txt"),
        ),
        (
            "🔗 Emphasis Around Links",
            include_str!("./misc/em-around-links.txt"),
        ),
        (
            "💪 Emphasis and Strong",
            include_str!("./misc/em_strong.txt"),
        ),
        (
            "💡 Complex Emphasis and Strong",
            include_str!("./misc/em_strong_complex.txt"),
        ),
        ("📧 Email", include_str!("./misc/email.txt")),
        ("🔗 Escaped Links", include_str!("./misc/escaped_links.txt")),
        ("📋 Funky List", include_str!("./misc/funky-list.txt")),
        ("#️⃣ H1", include_str!("./misc/h1.txt")),
        ("#️⃣ Hash", include_str!("./misc/hash.txt")),
        (
            "🗂️ Header in Lists",
            include_str!("./misc/header-in-lists.txt"),
        ),
        ("#️⃣ Headers", include_str!("./misc/headers.txt")),
        ("⏤ Horizontal Line", include_str!("./misc/hline.txt")),
        ("🖼️ Image 2", include_str!("./misc/image-2.txt")),
        (
            "🔗 Image in Links",
            include_str!("./misc/image_in_links.txt"),
        ),
        (
            "✏️ Insert at Start of Paragraph",
            include_str!("./misc/ins-at-start-of-paragraph.txt"),
        ),
        ("📄 Inside HTML", include_str!("./misc/inside_html.txt")),
        ("🇯🇵 Japanese", include_str!("./misc/japanese.txt")),
        (
            "🗨️ Lazy Blockquote",
            include_str!("./misc/lazy-block-quote.txt"),
        ),
        (
            "🔗 Link with Parenthesis",
            include_str!("./misc/link-with-parenthesis.txt"),
        ),
        ("🗂️ Lists", include_str!("./misc/lists.txt")),
        ("🗂️ Lists 2", include_str!("./misc/lists2.txt")),
        ("🗂️ Lists 3", include_str!("./misc/lists3.txt")),
        ("🗂️ Lists 4", include_str!("./misc/lists4.txt")),
        ("🗂️ Lists 5", include_str!("./misc/lists5.txt")),
        ("🗂️ Lists 6", include_str!("./misc/lists6.txt")),
        ("🗂️ Lists 7", include_str!("./misc/lists7.txt")),
        ("🗂️ Lists 8", include_str!("./misc/lists8.txt")),
        (
            "🔗 Missing Link Definition",
            include_str!("./misc/missing-link-def.txt"),
        ),
        (
            "🗨️ Multi-paragraph Blockquote",
            include_str!("./misc/multi-paragraph-block-quote.txt"),
        ),
        ("🧪 Multi Test", include_str!("./misc/multi-test.txt")),
        ("🗂️ Nested Lists", include_str!("./misc/nested-lists.txt")),
        (
            "🔍 Nested Patterns",
            include_str!("./misc/nested-patterns.txt"),
        ),
        ("🛠️ Normalize", include_str!("./misc/normalize.txt")),
        (
            "#️⃣ Numeric Entity",
            include_str!("./misc/numeric-entity.txt"),
        ),
        (
            "🖋️ Paragraph with Horizontal Rule",
            include_str!("./misc/para-with-hr.txt"),
        ),
        ("🇷🇺 Russian", include_str!("./misc/russian.txt")),
        ("💡 Smart Emphasis", include_str!("./misc/smart_em.txt")),
        ("🧪 Some Test", include_str!("./misc/some-test.txt")),
        ("🖋️ Span", include_str!("./misc/span.txt")),
        (
            "💪 Strong with Underscores",
            include_str!("./misc/strong-with-underscores.txt"),
        ),
        ("💪 Strong in Tags", include_str!("./misc/stronintags.txt")),
        ("🔢 Tabs in Lists", include_str!("./misc/tabs-in-lists.txt")),
        ("⏩ Two Spaces", include_str!("./misc/two-spaces.txt")),
        ("💡 Uche", include_str!("./misc/uche.txt")),
        ("🔗 Underscores", include_str!("./misc/underscores.txt")),
        ("🌐 URL with Spaces", include_str!("./misc/url_spaces.txt")),
    ]
    .into_iter()
    .collect();

    // Process each group of sources
    println!("\n📂 Processing Markdown from the `./basic` folder");
    process_sources("📄 Basic Features", basic_sources)?;

    println!("\n📂 Processing Markdown from the `./extensions` folder");
    process_sources("🧩 Extended Features", extensions_sources)?;

    println!("\n📂 Processing Markdown from the `./misc` folder");
    process_sources("🔍 Miscellaneous Features", misc_sources)?;

    println!("\n🎉 Batch processing example completed successfully!");

    Ok(())
}

/// Processes a group of Markdown sources and generates HTML for each.
fn process_sources(
    group_name: &str,
    sources: HashMap<&str, &str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗂️ Group: {group_name}");
    println!(
        "----------------------------------------------------------"
    );

    let config = HtmlConfig::default();

    for (title, markdown) in sources {
        println!("\n📝 Processing: {title}");
        println!("----------------------------------------------------------");

        // Generate HTML from Markdown
        match generate_html(markdown, &config) {
            Ok(html) => {
                println!(
                    "    ✅ Successfully generated HTML:\n{}",
                    html
                );
            }
            Err(e) => {
                println!("    ❌ Failed to generate HTML: {}", e);
            }
        }
    }

    Ok(())
}

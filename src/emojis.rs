// Copyright © 2025 HTML Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Emoji Sequences Loader
//!
//! Emoji data copyright (c) 2024 Unicode, Inc.
//! License: <http://www.unicode.org/copyright.html>
//! For terms of use, see <http://www.unicode.org/terms_of_use.html>
//!
//! This module provides functions to load and parse emoji sequences
//! from a simple text file. Each line in the file typically consists
//! of three fields separated by semicolons, for example:
//!
//! ```text
//! 2B06 FE0F ; Basic_Emoji ; up
//! ```
//!
//! ### Field Breakdown:
//! 1. `2B06 FE0F`: The hexadecimal code points for the emoji sequence.
//! 2. `Basic_Emoji`: A type field (often unused in this context).
//! 3. `up`: The user-friendly label or description for the emoji sequence.
//!
//! ### Notes:
//! - Lines that start with `#` or are blank are treated as comments.
//! - Trailing comments in the file are ignored or processed to derive the emoji's descriptive label.
//!
//! ### Example Comment Parsing:
//! ```text
//! 26A1 ; emoji ; L1 ; none ; a j # V4.0 (⚡) HIGH VOLTAGE SIGN
//! ```
//! The descriptive label derived would be: `"high-voltage-sign"`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Loads emoji sequences and their descriptive labels from a file.
///
/// This function processes files formatted with semicolon-separated fields.
/// For example, a line in the file might look like:
/// ```text
/// 2B06 FE0F ; Basic_Emoji ; up
/// ```
///
/// The mapping constructed will use the UTF-8 emoji sequence as the key
/// and a normalized, human-readable label as the value. For instance:
/// - `"⚡"` → `"high-voltage-sign"`
///
/// Lines starting with `#` or empty lines are ignored. Comments after a `#`
/// are parsed to extract descriptive labels.
///
/// # Arguments
///
/// * `filepath` - A path-like reference to the input file, such as `"emoji-data.txt"`.
///
/// # Returns
///
/// A [`HashMap<String, String>`] where:
/// - Keys are emoji strings (e.g., `"⚡"`).
/// - Values are normalized, lowercase, dash-separated labels (e.g., `"high-voltage-sign"`).
///
/// # Errors
///
/// Returns a [`Result`] indicating success or failure to read the file.
pub fn load_emoji_sequences<P: AsRef<Path>>(
    filepath: P,
) -> Result<HashMap<String, String>, std::io::Error> {
    let contents = fs::read_to_string(filepath)?;

    let mut map = HashMap::new();

    for raw_line in contents.lines() {
        let line = raw_line.trim();

        // Skip empty lines or comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Separate the data portion from the comment portion (if any)
        let (data_part, comment_part) = match line.split_once('#') {
            Some((before, after)) => (before.trim(), after.trim()),
            None => (line, ""),
        };

        // Extract the label from the comment portion
        let raw_label_after_paren =
            if let Some(close_paren_idx) = comment_part.find(')') {
                &comment_part[close_paren_idx + 1..]
            } else {
                comment_part
            };

        // Normalize the label
        let short_label = raw_label_after_paren
            .trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-");

        // Parse data fields
        let data_fields: Vec<&str> =
            data_part.split(';').map(|s| s.trim()).collect();
        if data_fields.is_empty() {
            continue;
        }

        // Extract the hexadecimal code points
        let hex_seq = data_fields[0];

        // Convert hex code points into a UTF-8 emoji string
        let emoji_string: String = hex_seq
            .split_whitespace()
            .filter_map(|hex| u32::from_str_radix(hex, 16).ok())
            .flat_map(char::from_u32)
            .collect();

        if emoji_string.is_empty() {
            continue; // Skip invalid sequences
        }

        // Insert the emoji string and its label into the map
        let _ = map.insert(emoji_string, short_label);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper function to write test data to a temporary file and return the path.
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new()
            .expect("Failed to create temporary file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temporary file");
        file
    }

    #[test]
    fn test_load_emoji_sequences_basic() {
        let test_data = r#"
            26A1 ; emoji ; L1 ; none ; a j # V4.0 (⚡) HIGH VOLTAGE SIGN
            1F600 ; emoji ; L1 ; none ; j     # V6.0 (😀) GRINNING FACE
        "#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path()).unwrap();

        let mut expected = HashMap::new();
        let _ = expected
            .insert("⚡".to_string(), "high-voltage-sign".to_string());
        let _ = expected
            .insert("😀".to_string(), "grinning-face".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_empty_file() {
        let test_data = "";

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_emoji_sequences_with_comments_and_blanks() {
        let test_data = r#"
    # This is a comment

    1F44D ; emoji ; L1 ; none ; j # V6.0 (👍) THUMBS UP SIGN

    # Another comment here

"#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected
            .insert("👍".to_string(), "thumbs-up-sign".to_string());

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_no_comment_label() {
        let test_data = r#"
    1F4AF ; emoji ; L1 ; none ; j # V6.0 (💯) HUNDRED POINTS SYMBOL
    1F602 ; emoji ; L1 ; none ; j
"#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected.insert(
            "💯".to_string(),
            "hundred-points-symbol".to_string(),
        );
        let _ = expected.insert("😂".to_string(), "".to_string()); // No comment means empty label

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_invalid_hex_code() {
        let test_data = r#"
    26A1 ; emoji ; L1 ; none ; a j # V4.0 (⚡) HIGH VOLTAGE SIGN
    INVALID_HEX ; emoji ; L1 ; none ; j # Invalid hex code
"#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected
            .insert("⚡".to_string(), "high-voltage-sign".to_string());

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_multi_codepoint() {
        let test_data = r#"
    1F1E6 1F1FA ; emoji ; L1 ; none ; j # V6.0 (🇦🇺) FLAG FOR AUSTRALIA
"#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected
            .insert("🇦🇺".to_string(), "flag-for-australia".to_string());

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_missing_label() {
        let test_data = r#"
    1F44D ; emoji ; L1 ; none ; j # V6.0 (👍) THUMBS UP SIGN
    1F602 ; emoji ; L1 ; none ; j
    1F600 ; emoji ; L1 ; none ; j #
"#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected
            .insert("👍".to_string(), "thumbs-up-sign".to_string());
        let _ = expected.insert("😂".to_string(), "".to_string()); // Missing label
        let _ = expected.insert("😀".to_string(), "".to_string()); // Empty comment after '#'

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_handles_empty_and_whitespace() {
        let test_data = r#"

    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY

    "#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_handles_trailing_whitespace() {
        let test_data = r#"
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY
    "#;

        let file = create_temp_file(test_data);

        let result = load_emoji_sequences(file.path());

        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_load_emoji_sequences_skip_invalid_lines() {
        let test_data = r#"
    # Comment line
    ; invalid line ; no hex code ; # Just semicolons
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        // Only the valid emoji line should be processed
        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_split_behavior() {
        let test_data = r#"
    26A1;emoji;L1;none;a j# V4.0 (⚡) HIGH VOLTAGE SIGN
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY
    26A1  ;  emoji  ;  L1  ;  none  ;  a j  # V4.0 (⚡) HIGH VOLTAGE SIGN
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        let mut expected = HashMap::new();
        let _ = expected
            .insert("⚡".to_string(), "high-voltage-sign".to_string());
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_parenthesis_variations() {
        let test_data = r#"
    26A1 ; emoji ; L1 ; none ; a j # (⚡) HIGH VOLTAGE
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS
    1F603 ; emoji ; L1 ; none ; j # V6.0 (😃) SMILEY FACE
    1F604 ; emoji ; L1 ; none ; j # V6.0 (😄) GRINNING FACE
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        let mut expected = HashMap::new();
        let _ = expected
            .insert("⚡".to_string(), "high-voltage".to_string());
        let _ = expected
            .insert("😂".to_string(), "face-with-tears".to_string());
        let _ = expected
            .insert("😃".to_string(), "smiley-face".to_string());
        let _ = expected
            .insert("😄".to_string(), "grinning-face".to_string());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_unparseable_sequences() {
        let test_data = r#"
    110000 ; emoji ; L1 ; none ; j # Above Unicode range INVALID
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY
    D800 ; emoji ; L1 ; none ; j # Surrogate code point
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        // Only the valid emoji should be included
        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_empty_fields() {
        let test_data = r#"
    ; ; ; ; ; # Empty fields should be skipped
    1F602 ; emoji ; L1 ; none ; j # V6.0 (😂) FACE WITH TEARS OF JOY
    #
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_emoji_sequences_whitespace_variations() {
        let test_data = r#"
    1F602;emoji;L1;none;j# V6.0 (😂) FACE WITH TEARS OF JOY
    1F603  ;  emoji  ;  L1  ;  none  ;  j  # V6.0 (😃) SMILEY FACE
    "#;

        let file = create_temp_file(test_data);
        let result = load_emoji_sequences(file.path()).unwrap();

        let mut expected = HashMap::new();
        let _ = expected.insert(
            "😂".to_string(),
            "face-with-tears-of-joy".to_string(),
        );
        let _ = expected
            .insert("😃".to_string(), "smiley-face".to_string());
        assert_eq!(result, expected);
    }
}

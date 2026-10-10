// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 HTML Generator. All rights reserved.

//! Resource budgets on YAML front matter.
//!
//! `extract_front_matter_data` deserializes YAML straight into a
//! `serde_json::Value`, which is noyalib's typed (streaming) path.
//! Before noyalib 0.0.53 that path skipped the alias-amplification
//! budgets (RUSTSEC-2026-0333), so a few hundred bytes of front matter
//! could expand into megabytes of JSON. These inputs must be refused.

use html_generator::{
    error::HtmlError, utils::extract_front_matter_data,
};

/// Front matter of `levels` anchored flow sequences, each holding
/// `width` aliases of the one before: `width^levels` scalars once
/// expanded, from a document of a few hundred bytes.
fn alias_bomb(levels: usize, width: usize) -> String {
    let mut doc = String::from("---\n");
    for level in 0..levels {
        let item = if level == 0 {
            "x".to_string()
        } else {
            format!("*l{}", level - 1)
        };
        let items = vec![item; width].join(", ");
        doc.push_str(&format!("l{level}: &l{level} [{items}]\n"));
    }
    doc.push_str("---\n# Body\n");
    doc
}

fn assert_refused(content: &str) {
    match extract_front_matter_data(content) {
        Err(HtmlError::InvalidFrontMatterFormat(_)) => {}
        Err(other) => panic!("wrong error variant: {other:?}"),
        Ok((data, _)) => panic!(
            "{}-byte front matter was accepted and expanded to {} bytes of JSON",
            content.len(),
            data.to_string().len()
        ),
    }
}

#[test]
fn wide_alias_fan_out_is_refused() {
    // 30^4 = 810,000 scalars from about 590 bytes.
    assert_refused(&alias_bomb(4, 30));
}

#[test]
fn deep_alias_chain_is_refused() {
    // 10^6 = 1,000,000 scalars from about 350 bytes.
    assert_refused(&alias_bomb(6, 10));
}

#[test]
fn modest_alias_use_still_parses() {
    // Control: anchors and aliases within the budgets keep working, so
    // the two tests above fail for the amplification, not for aliases.
    let content = "---\nbase: &b [a, b]\ncopy: *b\n---\n# Body";
    let (data, body) = extract_front_matter_data(content).unwrap();
    assert_eq!(data["copy"], data["base"]);
    assert_eq!(body, "# Body");
}

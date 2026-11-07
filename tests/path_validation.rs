// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 HTML Generator. All rights reserved.

//! Characterisation of the path check in `markdown_file_to_html`.
//!
//! The check is not containment under a root: it rejects any `..`
//! component, even one that stays inside the starting directory, and
//! it accepts absolute paths so CLI tools can pass fully qualified
//! filenames. A root-relative join (such as `safe_join(root, path)`)
//! would accept `a/../b.md` and reject `/abs/x.md`, so it is not a
//! drop-in replacement. These tests pin the current behaviour.

use html_generator::{
    error::HtmlError, markdown_file_to_html, OutputDestination,
};
use std::{io::ErrorKind, path::PathBuf};

const TRAVERSAL: &str = "Directory traversal is not allowed";

fn convert(input: &str) -> Result<(), HtmlError> {
    markdown_file_to_html(Some(PathBuf::from(input)), None, None)
}

fn assert_traversal_rejected(
    result: Result<(), HtmlError>,
    path: &str,
) {
    assert!(
        matches!(&result, Err(HtmlError::InvalidInput(msg)) if msg.contains(TRAVERSAL)),
        "{path:?} must be rejected as traversal, got {result:?}"
    );
}

/// Accepted paths get past validation and fail only at `File::open`.
fn assert_passes_validation(path: &str) {
    let result = convert(path);
    assert!(
        matches!(&result, Err(HtmlError::Io(e)) if e.kind() == ErrorKind::NotFound),
        "{path:?} must pass validation and fail on open, got {result:?}"
    );
}

#[test]
fn any_parent_dir_component_is_rejected() {
    for path in [
        "../x.md",
        "a/../b.md",
        "a/b/..",
        "./../x.md",
        "/abs/../x.md",
    ] {
        assert_traversal_rejected(convert(path), path);
    }
}

#[test]
fn parent_dir_in_output_path_is_rejected_before_reading_input() {
    let result = markdown_file_to_html(
        None::<PathBuf>,
        Some(OutputDestination::File("../out.html".into())),
        None,
    );
    assert_traversal_rejected(result, "../out.html");
}

#[test]
fn dots_inside_a_file_name_are_not_traversal() {
    assert_passes_validation("missing-dir/a..b.md");
    assert_passes_validation("missing-dir/..md");
}

#[test]
fn current_dir_components_are_accepted() {
    assert_passes_validation("./missing-dir/./x.md");
}

#[test]
fn absolute_paths_are_accepted() {
    let abs = std::env::temp_dir()
        .join("html-generator-path-validation-missing")
        .join("x.md");
    assert!(abs.is_absolute());
    assert_passes_validation(abs.to_str().expect("UTF-8 temp dir"));
}

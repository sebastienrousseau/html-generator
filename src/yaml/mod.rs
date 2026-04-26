// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Pure-Rust YAML serde implementation, inlined into html-generator.
//!
//! Mirrors a subset of the `serde_yml` API surface: no `unsafe`, no
//! `libyml` C dependency, no FFI. The implementation is a snapshot of
//! `yaml_safe@0.1.0` (upstream at `/Users/seb/Code/Public/Rust/yaml_safe`)
//! taken at commit time.
//!
//! # Why inlined
//!
//! `html-generator` needs YAML parsing for front-matter extraction and
//! deliberately avoids both `serde_yaml` (unmaintained) and `serde_yml`
//! (RUSTSEC-2025-0068: unsound, links the unsafe `libyml` C library).
//! Vendoring as a separate path crate worked but blocked
//! `cargo publish --dry-run` because path deps must carry a registry
//! version. Inlining as a private module sidesteps that without
//! adding an unsound transitive dependency.
//!
//! Fixes flow upstream first; this tree is refreshed by re-vendoring,
//! not by editing in place. The only call site in the parent crate is
//! `parse_yaml_to_map` in `src/utils.rs`.

// Vendored verbatim — relax the parent crate's stricter lints across
// this subtree so the YAML library compiles unmodified. The original
// upstream still has `#![forbid(unsafe_code)]`, which the parent crate
// also enforces; the rest of the parent's lint policy doesn't apply
// to a third-party tree we don't author.
#![allow(
    dead_code,
    unused_imports,
    unused_results,
    unused_qualifications,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

pub(crate) mod de;
pub(crate) mod error;
pub(crate) mod mapping;
pub(crate) mod number;
pub(crate) mod ser;
pub(crate) mod value;
pub(crate) mod with;

pub(crate) use de::{
    from_reader, from_slice, from_str, Deserializer, DocumentIter,
};
pub(crate) use error::{Error, Location, Result};
pub(crate) use mapping::Mapping;
pub(crate) use number::Number;
pub(crate) use ser::{to_string, to_writer, Serializer, State};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use value::tagged::{Tag, TaggedValue};
pub(crate) use value::{from_value, to_value, Sequence, Value};

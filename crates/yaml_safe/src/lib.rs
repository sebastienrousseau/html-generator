// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A safe, local YAML serialization/deserialization library
//! compatible with the `serde_yml` API.
//!
//! Pure-Rust drop-in replacement: no `unsafe`, no `libyml` C
//! dependency, no FFI.
//!
//! # Why this crate is vendored here
//!
//! `html-generator` needs YAML parsing for front-matter extraction
//! and avoids both the upstream `serde_yaml` (unmaintained) and
//! `serde_yml` (RUSTSEC-2025-0068: unsound, depends on the unsafe
//! `libyml`) crates from crates.io.
//!
//! This directory is a vendored snapshot of `yaml_safe`, mirrored
//! from `/Users/seb/Code/Public/Rust/yaml_safe` (`yaml_safe@0.1.0`)
//! at commit time. It exists as a path dependency, not a workspace
//! member, and carries `publish = false`. The standalone `yaml_safe`
//! repo is the upstream of record — fixes flow there first.
//!
//! When `yaml_safe` is published to crates.io, the path dependency
//! in the parent `Cargo.toml` becomes a registry dependency
//! (`yaml_safe = "0.1"`) and this directory is deleted in a single
//! commit. The only call site in the parent crate is
//! `parse_yaml_to_map` in `src/utils.rs`.

#![forbid(unsafe_code)]

pub mod de;
pub mod error;
pub mod mapping;
pub mod number;
pub mod ser;
pub mod value;
pub mod with;

pub use de::{
    from_reader, from_slice, from_str, Deserializer, DocumentIter,
};
pub use error::{Error, Location, Result};
pub use mapping::Mapping;
pub use number::Number;
pub use ser::{to_string, to_writer, Serializer, State};
pub use serde::{Deserialize, Serialize};
pub use value::tagged::{Tag, TaggedValue};
pub use value::{from_value, to_value, Sequence, Value};

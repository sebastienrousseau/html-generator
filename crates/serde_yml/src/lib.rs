// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A safe, local YAML serialization/deserialization library
//! compatible with the `serde_yml` API.
//!
//! # Why this crate is vendored
//!
//! `html-generator` needs YAML parsing for front-matter extraction but
//! avoids the upstream `serde_yaml` (unmaintained) and `serde_yml`
//! (depends on the unsafe `libyml` C library) crates. This vendored
//! implementation:
//!
//! - Is `#![forbid(unsafe_code)]` — no `unsafe` blocks, no FFI.
//! - Exposes a subset of the upstream `serde_yml` API sufficient for
//!   our front-matter path: `from_str`, `to_string`, `Value`, etc.
//! - Is `publish = false`: it is **not** a public library and has no
//!   stability guarantees outside this workspace.
//! - Is pinned at a single version and updated only when the parent
//!   crate needs new behaviour.
//!
//! If a maintained, pure-Rust YAML crate with an equivalent API
//! appears upstream (e.g. a successor to `yaml-rust2`), we intend to
//! delete this directory and depend on it directly.

#![forbid(unsafe_code)]

pub mod de;
pub mod error;
pub mod mapping;
pub mod number;
pub mod ser;
pub mod value;

pub use de::{from_reader, from_slice, from_str, Deserializer};
pub use error::{Error, Location, Result};
pub use mapping::Mapping;
pub use number::Number;
pub use ser::{to_string, to_writer, Serializer, State};
pub use value::tagged::{Tag, TaggedValue};
pub use value::{from_value, to_value, Sequence, Value};

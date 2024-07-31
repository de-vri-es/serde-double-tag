//! This crates provides derive macros for a double tagged enum representation for [`serde`].
//! It is basically a combination of externally and adjecently tagged enums.
//!
//! If you enable the `schemars` feature,
//! the crate also exposes a derive macro for the [`schemars::JsonSchema`] trait.
//!
//! For example, consider this enum:
//! ```
//! #[derive(serde_double_tag::Deserialize, serde_double_tag::Serialize)]
//! #[serde(tag = "species")]
//! #[serde(rename_all = "snake_case")]
//! enum Friend {
//!   Human {
//!     name: String,
//!     hobbies: Vec<String>,
//!   },
//!   Dog {
//!     name: String,
//!     color: String,
//!   }
//! }
//! ```
//!
//! A `Friend::Human` will be serialized as:
//! ```json
//! {
//!   "species": "human",
//!   "human": {
//!     "name": "Zohan",
//!     "hobbies": ["hair dressing"],
//!   }
//! }
//! ```
//!
//! Similarly, a `Friend::Dog` will be serialized as:
//! ```json
//! {
//!   "species": "dog",
//!   "dog": {
//!     "name": "Scrappy",
//!     "color": "white and gray",
//!   }
//! }
//! ```
//!
//! This enum representation could be useful if you want data for the different variants to co-exist in a single file or in your database.
//! Since each variant uses a different field name, they will never conflict.
//! And since there is still a separate field for the enum tag, you can still known which variant is actually active.
//!
//! Currently supported `serde` attributes:
//! * `#[serde(rename = "...")]
//! * `#[serde(rename_all = "...")]
//! * `#[serde(rename_all_fields = "...")]
//! * `#[serde(deny_unknown_fields = "...")]
//!
//! # `schemars`

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub mod internal__;

/// Derive [`serde::Serialize`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
pub use serde_double_tag_derive::Serialize;

/// Derive [`serde::Deserialize`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
pub use serde_double_tag_derive::Deserialize;

/// Derive [`schemars::JsonSchema`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
#[cfg(feature = "schemars")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "schemars")))]
pub use serde_double_tag_derive::JsonSchema;

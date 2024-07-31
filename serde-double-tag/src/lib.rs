#[doc(hidden)]
pub mod internal__;

/// Derive [`serde::Serialize`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
#[cfg(feature = "derive")]
pub use serde_double_tag_derive::Serialize;

/// Derive [`serde::Deserialize`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
#[cfg(feature = "derive")]
pub use serde_double_tag_derive::Deserialize;

/// Derive [`schemars::JsonSchema`] for an enum using the double-tagged enum representation.
///
/// See the module documentation for details on the enum representation.
#[cfg(all(feature = "derive", feature = "schemars"))]
pub use serde_double_tag_derive::JsonSchema;

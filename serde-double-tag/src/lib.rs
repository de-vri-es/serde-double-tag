#[doc(hidden)]
pub mod internal__;

/// Derive [`serde::Serialize`] for an enum using the doubly-tagged format.
///
/// See the module documentation for details on the serialization format.
#[cfg(feature = "derive")]
pub use serde_double_tag_derive::Serialize;

/// Derive [`serde::Serialize`] for an enum using the doubly-tagged format.
///
/// See the module documentation for details on the serialization format.
#[cfg(feature = "derive")]
pub use serde_double_tag_derive::Deserialize;

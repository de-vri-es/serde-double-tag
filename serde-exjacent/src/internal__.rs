pub use serde;

#[cfg(feature = "schemars")]
pub use schemars;

/// Marker type to indicate something does not implement [`serde::Deserialize`].
pub struct DoesNotImplementDeserialize;

/// Marker type to indicate something does implement [`serde::Deserialize`].
pub struct ImplementsDeserialize;

/// Marker type to indicate something does not implement [`serde::Serialize`].
pub struct DoesNotImplementSerialize;

/// Marker type to indicate something does implement [`serde::Serialize`].
pub struct ImplementsSerialize;

/// Marker to represent a type without having a value.
///
/// This will be used for auto-deref specialization to query type properties in a proc macro.
pub struct Type<T: ?Sized>(core::marker::PhantomData<*const T>);

impl<T: ?Sized> Type<T> {
    /// Create a new type marker.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

/// Helper for proc macros to turn a [`&str`] into a [`String`].
///
/// Used to not rely on the prelude, and to avoid ambiguous function calls.
/// Who knows what weird things the user imported into the call site.
#[inline]
pub fn string(input: &str) -> String {
    input.into()
}

/// Helper for proc macros to turn something into a [`serde_json::Value`].
///
/// Used to not rely on the prelude, and to avoid ambiguous function calls.
/// Who knows what weird things the user imported into the call site.
pub fn json_value<T: Into<serde_json::Value>>(input: T) -> serde_json::Value {
    input.into()
}

/// Trait that is implemented on `&Type<T>` if `T` implements [`serde::Deserialize`].
///
/// Note that this trait should be implemented on `&Type<T>`, not on `Type<T>`.
/// This way, the impl will be used before any impl on `Type<T>` by auto-deref.
pub trait ImplsDeserialize {
    fn does_it_implement_deserialize(&self) -> ImplementsDeserialize {
        ImplementsDeserialize
    }
}

impl<'de, T: ?Sized + serde::Deserialize<'de>> ImplsDeserialize for &Type<T> {}

/// Trait that is implemented on `&Type<T>` if `T` implements [`serde::Serialize`].
///
/// Note that this trait should be implemented on `&Type<T>`, not on `Type<T>`.
/// This way, the impl will be used before any impl on `Type<T>` by auto-deref.
pub trait ImplsSerialize {
    fn does_it_implement_serialize(&self) -> ImplementsSerialize {
        ImplementsSerialize
    }
}

impl<T: ?Sized + serde::Serialize> ImplsSerialize for &Type<T> {}

/// Trait that is implemented on `Type<T>` for every `T`.
///
/// Acts as a fallback for more specific implementations on `&Type<T>`.
/// Will be chosen last by the rules of auto-deref.
pub trait Everything {
    fn does_it_implement_deserialize(&self) -> DoesNotImplementDeserialize {
        DoesNotImplementDeserialize
    }

    fn does_it_implement_serialize(&self) -> DoesNotImplementSerialize {
        DoesNotImplementSerialize
    }
}

impl<T: ?Sized> Everything for Type<T> {}

/// Function to assert the result of `(&&Type::<T>::new()).does_it_implement_deserialize()`.
///
/// This only works in a proc macro and not in generic context.
pub fn does_not_implement_deserialize(_: DoesNotImplementDeserialize) {}

/// Function to assert the result of `(&&Type::<T>::new()).does_it_implement_serialize()`.
///
/// This only works in a proc macro and not in generic context.
pub fn does_not_implement_serialize(_: DoesNotImplementSerialize) {}

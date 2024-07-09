#[doc(hidden)]
pub mod internal__;

/// Derive [`serde::Serialize`] for an enum using the doubly-tagged format.
///
/// See the module documentation for details on the serialization format.
#[cfg(feature = "derive")]
pub use serde_exjacent_derive::Serialize;

/// Derive [`serde::Serialize`] for an enum using the doubly-tagged format.
///
/// See the module documentation for details on the serialization format.
#[cfg(feature = "derive")]
pub use serde_exjacent_derive::Deserialize;

///// Derive the [`ConfigEnum`] trait and the required super traits.
/////
///// This also derives [`DeserializePartial`], [`SerializePartial`] and [`schemars::JsonSchema`].
/////
///// A config enum is serialized with a separate `type` field to indicate which variant is active.
///// Additionally, each variant is stored in a field named after the variant.
///// The value for the `type` field and the name of the variant field are the same:
///// the name of the variant in lower case with underscores separating the words (`snake_case`).
/////
///// A [`ConfigEnum`] must be wrapped in [`PreserveUnknownFields`] when used in a larger configuration struct.
///// Doing this will ensure that the inactive variants are preserved when deserialized and re-serialized.
///// To ensure you can not forget this, the type is forbidden at compile time to also implement [`serde::Deserialize`] or [`serde::Serialize`].
/////
///// The enum variants may either be a unit variant, a struct variant or a tuple variant with exactly zero fields or one field.
///// In particular, tuple variants with more than one field are forbidden because the user interface can not show them properly.
/////
///// # Example
///// ```
///// #[derive(serde_exjacent::ConfigEnum)]
///// enum DeviceConfig {
/////   Fake,
/////   Can(CanConfig),
/////   Io {
/////     pin: u8,
/////     invert_polarity: bool,
/////   },
///// }
/////
///// #[derive(schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
///// struct CanConfig {
/////   interface: String,
/////   priority: u8,
///// }
///// ```
/////
///// The `DeviceConfig` be serialized as:
/////
///// ```json
///// {"type": "fake"}
///// ```
/////
///// Or
/////
///// ```json
///// {
/////   "type": "can",
/////   "can": {
/////     "interface": "can0",
/////     "priority": 8,
/////   }
///// }
///// ```
/////
///// Or:
///// ```json
///// {
/////   "type": "io",
/////   "io": {
/////     "pin": 3,
/////     "invert_polarity": false,
/////   }
///// }
///// ```
//#[cfg(feature = "derive")]
//pub use serde_exjacent::ConfigEnum;

/// Simple wrapper struct for objects to preserve unknown fields.
///
/// The wrapped object must implement [`DeserializePartial`] and [`SerializePartial`].
#[derive(Debug, Clone)]
pub struct PreserveUnknownFields<T> {
    /// The wrapped value.
    pub value: T,

    /// The unknown fields.
    pub unknown_fields: serde_json::Map<String, serde_json::Value>,
}

pub trait DeserializePartial: Sized {
    /// Deserialize self from the object.
    ///
    /// All recognized fields are removed from the object.
    /// All unrecognized fields are left in the object.
    fn deserialize_partial(value: &mut serde_json::Map<String, serde_json::Value>) -> Result<Self, serde_json::Error>;
}

pub trait SerializePartial: Sized {
    /// Serialize self into an existing object.
    ///
    /// Pre-existing keys should be considered more important and must not be overwritten.
    fn serialize_partial(&self, output: &mut serde_json::Map<String, serde_json::Value>) -> Result<(), serde_json::Error>;
}

impl<'de, T: DeserializePartial> serde::Deserialize<'de> for PreserveUnknownFields<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let mut object = serde::Deserialize::deserialize(deserializer)?;
        let value = T::deserialize_partial(&mut object).map_err(D::Error::custom)?;
        Ok(Self {
            value,
            unknown_fields: object,
        })
    }
}

impl<T: SerializePartial> serde::Serialize for PreserveUnknownFields<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;

        let mut output = serde_json::Map::<String, serde_json::Value>::new();
        self.value
            .serialize_partial(&mut output)
            .map_err(S::Error::custom)?;
        for (key, value) in &self.unknown_fields {
            output.entry(key).or_insert_with(|| value.clone());
        }

        output.serialize(serializer)
    }
}

/// Trait for structured configuration enums.
pub trait ConfigEnum: SerializePartial + DeserializePartial {}

#[cfg(feature = "schemars")]
impl<T: schemars::JsonSchema> schemars::JsonSchema for PreserveUnknownFields<T> {
    fn is_referenceable() -> bool {
        T::is_referenceable()
    }

    fn schema_name() -> String {
        T::schema_name()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        T::schema_id()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        T::json_schema(gen)
    }

    fn _schemars_private_non_optional_json_schema(
        gen: &mut schemars::gen::SchemaGenerator,
    ) -> schemars::schema::Schema {
        T::_schemars_private_non_optional_json_schema(gen)
    }

    fn _schemars_private_is_option() -> bool {
        T::_schemars_private_is_option()
    }
}

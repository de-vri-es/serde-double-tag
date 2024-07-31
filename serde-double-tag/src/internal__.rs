//! Internal utilities for use by the derive macros.
//!
//! Do not call these directly, as they may be changed without matching semver bump.

#![deny(missing_docs)]

pub use ::serde;

#[cfg(feature = "schemars")]
pub use ::schemars;

/// Convert a value into a [`String`].
pub fn string(input: impl Into<String>) -> String {
	input.into()
}

/// Convert a value into a [`serde_json::Value`].
#[inline]
#[cfg(feature = "schemars")]
pub fn json_value(input: impl Into<serde_json::Value>) -> serde_json::Value {
	input.into()
}

/// Create a schema for an object with the given properties.
///
/// All properties will be required.
#[inline]
#[cfg(feature = "schemars")]
pub fn object_schema(properties: schemars::Map<String, schemars::schema::Schema>) -> schemars::schema::Schema {
	let required = properties.keys().cloned().collect();
	schemars::schema::SchemaObject {
		instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(schemars::schema::InstanceType::Object))),
		object: Some(Box::new(schemars::schema::ObjectValidation {
			properties,
			required,
			..Default::default()
		})),
		..Default::default()
	}.into()
}

/// Create a schema for a constant string value.
#[inline]
#[cfg(feature = "schemars")]
pub fn const_string_value(value: &str) -> schemars::schema::Schema {
	schemars::schema::SchemaObject {
		instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(schemars::schema::InstanceType::String))),
		const_value: Some(value.into()),
		..Default::default()
	}.into()
}

/// Create a subschema for a variant.
#[inline]
#[cfg(feature = "schemars")]
pub fn variant_subschema(
	tag_field_name: &'static str,
	variant_name: &str,
	variant_subschema: schemars::schema::Schema,
) -> schemars::schema::SubschemaValidation {
	let mut if_properties = schemars::Map::with_capacity(1);
	if_properties.insert(tag_field_name.into(), const_string_value(variant_name));

	let mut then_properties = schemars::Map::with_capacity(1);
	then_properties.insert(variant_name.to_string(), variant_subschema);

	::schemars::schema::SubschemaValidation {
		if_schema: Some(Box::new(object_schema(if_properties))),
		then_schema: Some(Box::new(object_schema(then_properties))),
		..Default::default()
	}
}

/// Create a schema with the given subschema.
#[inline]
#[cfg(feature = "schemars")]
pub fn subschema_to_schema(subschema: schemars::schema::SubschemaValidation) -> schemars::schema::Schema {
	schemars::schema::SchemaObject {
		subschemas: Some(Box::new(subschema)),
		..Default::default()
	}.into()
}

/// Create a schema for a unit value.
#[inline]
#[cfg(feature = "schemars")]
pub fn unit_schema() -> schemars::schema::Schema {
	schemars::schema::SchemaObject {
		instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(schemars::schema::InstanceType::Null))),
		..Default::default()
	}.into()
}

/// Names of the tag and content fields for an enum variant.
#[repr(C)]
pub struct FieldNames {
	/// The name of the tag field.
	pub tag: &'static str,

	/// The name of the content field for this variant.
	pub content: &'static str,
}

impl FieldNames {
	fn as_slice(&'static self) -> &'static [&'static str] {
		unsafe {
			core::slice::from_raw_parts(&self.tag, 2)
		}
	}
}

/// Deserialize the tag field from a `MapAccess`.
pub fn deserialize_tag<'de, Tag, M>(tag_field_name: &'static str, map: &mut M) -> Result<Tag, M::Error>
where
	Tag: serde::de::Deserialize<'de>,
	M: serde::de::MapAccess<'de>,
{
	struct TagKeySeed(pub &'static str);

	impl<'de> serde::de::DeserializeSeed<'de> for TagKeySeed {
		type Value = ();

		fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
			struct Visitor {
				field_name: &'static str,
			}

			impl<'de> serde::de::Visitor<'de> for Visitor {
				type Value = ();

				fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
					write!(formatter, "a field with name {:?}", self.field_name)
				}

				fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
					if value == self.field_name {
						Ok(())
					} else {
						Err(E::custom(format_args!("expected a field with name {:?}, but got {value:?}", self.field_name)))
					}
				}
			}

			deserializer.deserialize_identifier(Visitor {
				field_name: self.0,
			})
		}
	}

	let key_seed = TagKeySeed(tag_field_name);
	let value_seed = core::marker::PhantomData::<Tag>;
	let ((), tag) = map.next_entry_seed(key_seed, value_seed)?
		.ok_or_else(|| serde::de::Error::missing_field(tag_field_name))?;
	Ok(tag)
}

/// Deserialize the variant fields from a `MapAccess`.
pub fn deserialize_variant_required<'de, T, M>(fields: &'static FieldNames, mut map: M, deny_unknown_fields: bool) -> Result<T, M::Error>
where
	T: serde::de::Deserialize<'de>,
	M: serde::de::MapAccess<'de>,
{
	use serde::de::IgnoredAny;

	let key_seed = VariantKeySeed(fields.content);
	let variant = loop {
		match map.next_key_seed(key_seed)? {
			None => break None,
			Some(true) => break Some(map.next_value()?),
			Some(false) => {
				let _: IgnoredAny = map.next_value()?;
			}
		}
	};
	if let Some(variant) = variant {
		if deny_unknown_fields {
			map.next_key_seed(UnknownFieldKeySeed {
				known_fields: fields.as_slice(),
			})?;
		} else {
			while let Some((_, _)) = map.next_entry::<IgnoredAny, IgnoredAny>()? {
				// Ignore the remaining data in the map.
			}
		}
		Ok(variant)
	} else {
		Err(serde::de::Error::missing_field(fields.content))
	}
}

/// Deserialize the fields of a variant, substituting the default value if the field is not present.
pub fn deserialize_variant_optional<'de, T, M>(fields: &'static FieldNames, mut map: M, deny_unknown_fields: bool) -> Result<T, M::Error>
where
	T: serde::de::Deserialize<'de> + Default,
	M: serde::de::MapAccess<'de>,
{
	use serde::de::IgnoredAny;

	let key_seed = VariantKeySeed(fields.content);
	let variant = loop {
		match map.next_key_seed(key_seed)? {
			None => break None,
			Some(true) => break Some(map.next_value()?),
			Some(false) => {
				let _: serde::de::IgnoredAny = map.next_value()?;
			}
		}
	};
	if let Some(variant) = variant {
		if deny_unknown_fields {
			map.next_key_seed(UnknownFieldKeySeed {
				known_fields: fields.as_slice(),
			})?;
		} else {
			while let Some((_, _)) = map.next_entry::<IgnoredAny, IgnoredAny>()? {
				// Ignore the remaining data in the map.
			}
		}
		Ok(variant)
	} else {
		Ok(T::default())
	}
}

/// A deserialize seed for the variant data field key.
#[derive(Copy, Clone)]
struct VariantKeySeed(&'static str);

impl<'de> serde::de::DeserializeSeed<'de> for VariantKeySeed {
	type Value = bool;

	fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		struct Visitor {
			field_name: &'static str,
		}

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = bool;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "a field with name {:?}", self.field_name)
			}

			fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
				Ok(value == self.field_name)
			}
		}

		deserializer.deserialize_identifier(Visitor {
			field_name: self.0,
		})
	}
}

/// A deserialize seed for disallowed unknown fields.
///
/// Will always produce an error when deserializing a value.
#[derive(Copy, Clone)]
struct UnknownFieldKeySeed {
	known_fields: &'static [&'static str],
}

impl<'de> serde::de::DeserializeSeed<'de> for UnknownFieldKeySeed {
	type Value = ();

	fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		struct Visitor {
			known_fields: &'static [&'static str],
		}

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = ();

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("the end of the map")
			}

			fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
				if let Some(known) = self.known_fields.iter().find(|&&x| x == v) {
					Err(E::duplicate_field(known))
				} else {
					Err(E::unknown_field(v, self.known_fields))
				}
			}
		}

		deserializer.deserialize_str(Visitor {
			known_fields: self.known_fields,
		})
	}
}

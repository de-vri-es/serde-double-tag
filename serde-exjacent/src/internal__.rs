pub use serde;

#[cfg(feature = "schemars")]
pub use schemars;
use serde::de::IgnoredAny;

pub struct TagKeySeed(pub &'static str);

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

pub fn deserialize_variant<'de, T, M>(field_name: &'static str, mut map: M) -> Result<T, M::Error>
where
    T: serde::de::Deserialize<'de>,
    M: serde::de::MapAccess<'de>,
{
    let key_seed = VariantKeySeed(field_name);
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
        while let Some((_, _)) = map.next_entry::<IgnoredAny, IgnoredAny>()? {
            // Ignore the remaining data in the map.
        }
        Ok(variant)
    } else {
        Err(serde::de::Error::missing_field(field_name))
    }
}

pub fn deserialize_variant_optional<'de, T, M>(field_name: &'static str, mut map: M) -> Result<T, M::Error>
where
    T: serde::de::Deserialize<'de> + Default,
    M: serde::de::MapAccess<'de>,
{
    let key_seed = VariantKeySeed(field_name);
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
        while let Some((_, _)) = map.next_entry::<IgnoredAny, IgnoredAny>()? {
            // Ignore the remaining data in the map.
        }
        Ok(variant)
    } else {
        Ok(T::default())
    }
}

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

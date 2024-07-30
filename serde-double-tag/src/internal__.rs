pub use serde;

#[cfg(feature = "schemars")]
pub use schemars;
use serde::de::IgnoredAny;

#[repr(C)]
pub struct FieldNames {
    pub tag: &'static str,
    pub content: &'static str,
}

impl FieldNames {
    fn as_slice(&'static self) -> &'static [&'static str] {
        unsafe {
            core::slice::from_raw_parts(&self.tag, 2)
        }
    }
}

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

pub fn deserialize_variant_required<'de, T, M>(fields: &'static FieldNames, mut map: M, deny_unknown_fields: bool) -> Result<T, M::Error>
where
    T: serde::de::Deserialize<'de>,
    M: serde::de::MapAccess<'de>,
{
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
        Err(serde::de::Error::missing_field(fields.content))
    }
}

pub fn deserialize_variant_optional<'de, T, M>(fields: &'static FieldNames, mut map: M, deny_unknown_fields: bool) -> Result<T, M::Error>
where
    T: serde::de::Deserialize<'de> + Default,
    M: serde::de::MapAccess<'de>,
{
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

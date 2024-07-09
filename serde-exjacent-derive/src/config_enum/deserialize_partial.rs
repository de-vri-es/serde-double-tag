use proc_macro2::TokenStream;
use quote::quote;

use crate::util;

/// Generate code that implements the `DeserializePartial` trait for an enum.
pub fn impl_deserialize_partial_enum(crate_name: &syn::Path, item: &syn::ItemEnum) -> TokenStream {
    let tag_value: Vec<_> = item
        .variants
        .iter()
        .map(|variant| util::to_snake_case(&variant.ident.to_string()))
        .collect();

    let deserialize_variant = item
        .variants
        .iter()
        .zip(&tag_value)
        .map(|(variant, tag_value)| deserialize_partial_variant(crate_name, variant, tag_value));

    // Add a compile time check that the type does not implement `Deserialize` (which would behave different from our `DeserializePartial` implementation).
    let forbid_deserialize_impl = quote::quote_spanned!( item.ident.span() => {
        #[allow(clippy::all)] // Yes, the code is weird, we know.
        {
            #[allow(unused_imports)]
            use #crate_name::internal__::{ImplsDeserialize, Everything, Type, does_not_implement_deserialize};
            does_not_implement_deserialize((&&Type::<Self>::new()).does_it_implement_deserialize());
        }
    });

    let mut expected_tag = String::from("one of: ");
    for (i, tag_value) in tag_value.iter().enumerate() {
        use std::fmt::Write;
        if i == 0 {
            write!(expected_tag, "{tag_value:?}").unwrap();
        } else if i + 1 == item.variants.len() {
            write!(expected_tag, " or {tag_value:?}").unwrap();
        } else {
            write!(expected_tag, ", {tag_value:?}").unwrap();
        }
    }

    let ty = &item.ident;
    quote! {
        impl #crate_name::DeserializePartial for #ty {
            fn deserialize_partial(object: &mut #crate_name::Object) -> ::core::result::Result<Self, #crate_name::internal__::serde_json::Error> {
                #forbid_deserialize_impl

                let tag = match object.shift_remove("type") {
                    ::core::option::Option::Some(x) => x,
                    ::core::option::Option::None => {
                        return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field("type"));
                    }
                };
                match tag {
                    #crate_name::internal__::serde_json::Value::String(tag) => match tag.as_str() {
                        #(#tag_value => { #deserialize_variant },)*
                        tag => ::core::result::Result::Err(
                            #crate_name::internal__::serde::de::Error::invalid_value(
                                #crate_name::internal__::serde::de::Unexpected::Str(tag),
                                &#expected_tag,
                            )
                        ),
                    },
                    #crate_name::internal__::serde_json::Value::Null => ::core::result::Result::Err(
                        #crate_name::internal__::serde::de::Error::missing_field("type")
                    ),
                    #crate_name::internal__::serde_json::Value::Array(_) => ::core::result::Result::Err(
                        #crate_name::internal__::serde::de::Error::invalid_type(
                            #crate_name::internal__::serde::de::Unexpected::Seq,
                            &#expected_tag,
                        )
                    ),
                    #crate_name::internal__::serde_json::Value::Object(_) => ::core::result::Result::Err(
                        #crate_name::internal__::serde::de::Error::invalid_type(
                            #crate_name::internal__::serde::de::Unexpected::Map,
                            &#expected_tag,
                        )
                    ),
                    #crate_name::internal__::serde_json::Value::Bool(value) => ::core::result::Result::Err(
                        #crate_name::internal__::serde::de::Error::invalid_type(
                            #crate_name::internal__::serde::de::Unexpected::Bool(value),
                            &#expected_tag,
                        )
                    ),
                    #crate_name::internal__::serde_json::Value::Number(value) => ::core::result::Result::Err(
                        if let ::core::option::Option::Some(value) = value.as_u64() {
                            #crate_name::internal__::serde::de::Error::invalid_type(
                                #crate_name::internal__::serde::de::Unexpected::Unsigned(value),
                                &#expected_tag,
                            )
                        } else if let ::core::option::Option::Some(value) = value.as_i64() {
                            #crate_name::internal__::serde::de::Error::invalid_type(
                                #crate_name::internal__::serde::de::Unexpected::Signed(value),
                                &#expected_tag,
                            )
                        } else if let ::core::option::Option::Some(value) = value.as_f64() {
                            #crate_name::internal__::serde::de::Error::invalid_type(
                                #crate_name::internal__::serde::de::Unexpected::Float(value),
                                &#expected_tag,
                            )
                        } else {
                            #crate_name::internal__::serde::de::Error::invalid_type(
                                #crate_name::internal__::serde::de::Unexpected::Other("number"),
                                &#expected_tag,
                            )
                        }
                    )
                }
            }
        }
    }
}

/// Generate code to deserialize one enum variant.
fn deserialize_partial_variant(
    crate_name: &syn::Path,
    variant: &syn::Variant,
    tag_value: &str,
) -> TokenStream {
    let name = &variant.ident;
    match &variant.fields {
        syn::Fields::Unit => quote! {
            ::core::result::Result::Ok(Self::#name)
        },
        syn::Fields::Named(fields) => {
            let field_name: Vec<_> = fields
                .named
                .iter()
                .filter_map(|x| x.ident.as_ref())
                .collect();
            let field_name_str = field_name.iter().map(|x| x.to_string());
            quote! {
                let value = match object.shift_remove(#tag_value) {
                    ::core::option::Option::Some(x) => x,
                    ::core::option::Option::None => {
                        return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field(#tag_value));
                    }
                };
                let mut value: #crate_name::Object = #crate_name::internal__::serde::Deserialize::deserialize(value)?;

                ::core::result::Result::Ok(Self::#name {
                    #(
                        #field_name: match value.shift_remove(#field_name_str) {
                            ::core::option::Option::Some(x) => #crate_name::internal__::serde::Deserialize::deserialize(x)?,
                            ::core::option::Option::None => {
                                return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field(#field_name_str));
                            }
                        },
                    )*
                })
            }
        }
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.is_empty() {
                quote! {
                    ::core::result::Result::Ok(Self::#name())
                }
            } else if fields.unnamed.len() == 1 {
                quote! {
                    let value = match object.shift_remove(#tag_value) {
                        ::core::option::Option::Some(x) => x,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field(#tag_value));
                        }
                    };
                    ::core::result::Result::Ok(Self::#name(#crate_name::internal__::serde::Deserialize::deserialize(value)?))
                }
            } else {
                let index = (0..fields.unnamed.len()).map(|x| x.to_string());
                quote! {
                    let value = match object.shift_remove(#tag_value) {
                        ::core::option::Option::Some(x) => x,
                        ::core::option::Option::None => {
                            return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field(#tag_value));
                        }
                    };
                    let value: ::std::vec::Vec<#crate_name::internal__::serde_json::Value> = #crate_name::internal__::serde::Deserialize::deserialize(value)?;
                    let mut value = ::core::iter::IntoIterator::into_iter(value);

                    ::core::result::Result::Ok(Self::#name(
                        #(
                            match ::core::iter::Iterator::next(&mut value) {
                                ::core::option::Option::Some(x) => #crate_name::internal__::serde::Deserialize::deserialize(x)?,
                                ::core::option::Option::None => {
                                    return ::core::result::Result::Err(#crate_name::internal__::serde::de::Error::missing_field(#index));
                                }
                            }
                        ),*
                    ))
                }
            }
        }
    }
}

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::util;

/// Generate code that implement the `SerializePartial` trait for an enum.
pub fn impl_serialize_partial_enum(crate_name: &syn::Path, item: &syn::ItemEnum) -> TokenStream {
    let variants = item.variants.iter().map(|variant| {
        let name = &variant.ident;
        let snake_case = util::to_snake_case(&name.to_string());
        match &variant.fields {
            syn::Fields::Unit => quote! {
                Self::#name => {
                    object.insert(#crate_name::internal__::string("type"), #crate_name::internal__::json_value(#snake_case));
                },
            },
            syn::Fields::Named(fields) => {
                let field_name: Vec<_> = fields.named.iter().filter_map(|x| x.ident.as_ref()).collect();
                let field_name_str: Vec<_> = field_name.iter().map(|x| x.to_string()).collect();
                let mapped_field_name: Vec<_> = field_name.iter().map(|name| syn::Ident::new(&format!("field_{name}"), name.span())).collect();
                quote! {
                    Self::#name { #(#field_name: #mapped_field_name),* } => {
                    object.insert(#crate_name::internal__::string("type"), #crate_name::internal__::json_value(#snake_case));
                        let mut variant = #crate_name::Object::new();
                        #(
                            variant.insert(#crate_name::internal__::string(#field_name_str), #crate_name::internal__::serde_json::to_value(&#mapped_field_name)?);
                        )*
                        object.insert(#crate_name::internal__::string(#snake_case), #crate_name::internal__::json_value(variant));
                    }
                }
            },
            syn::Fields::Unnamed(fields) => {
                match fields.unnamed.len() {
                    0 => quote! {
                        Self::#name() => {
                            object.insert(#crate_name::internal__::string("type"), #crate_name::internal__::json_value(#snake_case));
                        }
                    },
                    1 => quote! {
                        Self::#name(value) => {
                            object.insert(#crate_name::internal__::string("type"), #crate_name::internal__::json_value(#snake_case));
                            object.insert(#crate_name::internal__::string(#snake_case), #crate_name::internal__::serde_json::to_value(&value)?);
                        }
                    },
                    _ => {
                        let mapped_field_name: Vec<_> = fields.unnamed.iter()
                            .enumerate()
                            .map(|(i, field)| syn::Ident::new(&format!("field_{i}"), field.ty.span()))
                            .collect();
                        quote! {
                            Self::#name ( #(#mapped_field_name),* ) => {
                                object.insert(#crate_name::internal__::string("type"), #crate_name::internal__::json_value(#snake_case));
                                let mut variant = ::std::vec::Vec::new();
                                #(
                                    variant.push(#crate_name::internal__::serde_json::to_value(&#mapped_field_name)?);
                                )*

                                object.insert(#crate_name::internal__::string(#snake_case), #crate_name::internal__::json_value(variant));
                            }
                        }
                    }
                }
            },
        }
    });

    let ty = &item.ident;

    // Add a compile time check that the type does not implement `Serialize` (which would behave different from our `SerializePartial` implementation).
    let forbid_serialize_impl = quote::quote_spanned!( ty.span() => {
        #[allow(clippy::all)] // Yes, the code is weird, we know.
        {
            #[allow(unused_imports)]
            use #crate_name::internal__::{ImplsSerialize, Everything, Type, does_not_implement_serialize};
            does_not_implement_serialize((&&Type::<Self>::new()).does_it_implement_serialize());
        }
    });

    quote! {
        impl #crate_name::SerializePartial for #ty {
            fn serialize_partial(&self, object: &mut #crate_name::Object) -> ::core::result::Result<(), #crate_name::internal__::serde_json::Error> {
                #forbid_serialize_impl
                match self {
                    #(#variants)*,
                }
                ::core::result::Result::Ok(())
            }
        }
    }
}

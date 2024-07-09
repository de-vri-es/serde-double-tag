use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::{util, Anchors};

/// Generate code that implement the serde `Serialize` trait for an enum using the double-tag format.
pub fn impl_deserialize_enum(anchors: &Anchors, tokens: TokenStream) -> TokenStream {
	let item = match util::parse_enum_item(tokens) {
		Ok(x) => x,
		Err(e) => return e.into_compile_error(),
	};

	let internal = &anchors.internal;
	let serde = &anchors.serde;

	let enum_name = &item.ident;

	let variant_name: Vec<_> = item.variants.iter()
		.map(|x| &x.ident).collect();
	let variant_tag: Vec<_> = item.variants.iter()
		.map(|x| util::to_snake_case(&x.ident.to_string())).collect();
	let variant_deserialize = item.variants.iter()
		.zip(&variant_tag)
		.map(|(variant, tag_value)| deserialize_fields(anchors, &item, variant, tag_value));

	let (_impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let (de_generics, de_lifetime, error) = util::add_lifetime(&item.generics, "de");
	let where_clause = make_where_clause(anchors, &item, &de_lifetime);
	let (impl_generics, _type_generics, _where_clause) = de_generics.split_for_impl();

	let tag_name = "type";
	let tag_enum = make_tag_enum(anchors, &item);

	quote! {
		#error
		impl #impl_generics  #serde::Deserialize<#de_lifetime> for #enum_name #type_generics #where_clause {
			fn deserialize<D: #serde::Deserializer<#de_lifetime>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
				struct Visitor #type_generics {
					_phantom: ::core::marker::PhantomData<fn() -> #enum_name #type_generics>,
				};
				impl #impl_generics #serde::de::Visitor<#de_lifetime> for Visitor #type_generics #where_clause {
					type Value = #enum_name #type_generics;

					fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
						f.write_str("map with `type` and data fields")
					}

					fn visit_map<A: #serde::de::MapAccess<#de_lifetime>>(self, mut map: A) -> ::core::result::Result<Self::Value, A::Error> {
						#tag_enum

						let key_seed = #internal::TagKeySeed(#tag_name);
						let value_seed = ::core::marker::PhantomData::<Tag>;
						let variant = match map.next_entry_seed(key_seed, value_seed)? {
							Some(((), variant)) => variant,
							None => {
								return ::core::result::Result::Err(#serde::de::Error::missing_field(#tag_name));
							}
						};
						match variant {
							#(
								Tag::#variant_name => {
									#variant_deserialize
								},
							)*
						}
					}
				}

				deserializer.deserialize_map(Visitor {
					_phantom: ::core::marker::PhantomData
				})
			}
		}
	}
}

fn make_tag_enum(anchors: &Anchors, item: &syn::ItemEnum) -> TokenStream {
	let serde = &anchors.serde;

	let variant_name: Vec<_> = item.variants.iter().map(|x| &x.ident).collect();
	let tag_value: Vec<_> = item.variants.iter().map(|x| util::to_snake_case(&x.ident.to_string())).collect();

	let tag_expecting = make_expecting("the string", &tag_value);

	quote! {
		enum Tag {
			#(#variant_name,)*
		}

		impl<'de> #serde::Deserialize<'de> for Tag {
			fn deserialize<D: #serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
				struct Visitor;

				impl<'de> #serde::de::Visitor<'de> for Visitor {
					type Value = Tag;

					fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
						f.write_str(#tag_expecting)
					}

					fn visit_str<E: #serde::de::Error>(self, value: &::core::primitive::str) -> ::core::result::Result<Self::Value, E> {
						match value {
							#(
								#tag_value => ::core::result::Result::Ok(Tag::#variant_name),
							)*
							value => ::core::result::Result::Err(E::invalid_value(
								#serde::de::Unexpected::Str(value),
								&#tag_expecting
							))
						}
					}
				}

				deserializer.deserialize_str(Visitor)
			}
		}
	}
}

fn deserialize_fields(anchors: &Anchors, item: &syn::ItemEnum, variant: &syn::Variant, variant_tag: &str) -> TokenStream {
	match &variant.fields {
		syn::Fields::Unit => deserialize_unit_variant(anchors, item, &variant.ident, variant_tag),
		syn::Fields::Unnamed(fields) => deserialize_tuple_variant(anchors, item, &variant.ident, variant_tag, fields),
		syn::Fields::Named(fields) => deserialize_struct_variant(anchors, item, &variant.ident, variant_tag, fields),
	}
}

fn deserialize_unit_variant(anchors: &Anchors, item: &syn::ItemEnum, variant_name: &syn::Ident, variant_tag: &str) -> TokenStream {
	let internal = &anchors.internal;
	let enum_name = &item.ident;
	quote! {
		let _value: () = #internal::deserialize_variant_optional(#variant_tag, map)?;
		::core::result::Result::Ok(#enum_name::#variant_name)
	}
}

fn deserialize_tuple_variant(anchors: &Anchors, item: &syn::ItemEnum, variant_name: &syn::Ident, variant_tag: &str, fields: &syn::FieldsUnnamed) -> TokenStream {
	let internal = &anchors.internal;
	let serde = &anchors.serde;

	let enum_name = &item.ident;

	match fields.unnamed.len() {
		0 => {
			quote! {
				let _value: () = #internal::deserialize_variant_optional(#variant_tag, map)?;
				::core::result::Result::Ok(#enum_name::#variant_name())
			}
		},
		1 => {
			quote! {
				let value = #internal::deserialize_variant(#variant_tag, map)?;
				::core::result::Result::Ok(#enum_name::#variant_name(value))
			}
		},
		n => {
			let variant_name_str = variant_name.to_string();
			let mapped_field_name: Vec<_> = fields.unnamed
				.iter()
				.enumerate()
				.map(|(i, field)| syn::Ident::new(&format!("field_{i}"), field.span()))
				.collect();
			let (_, enum_type_generics, enum_where_clause) = item.generics.split_for_impl();
			let enum_ty = quote! { #enum_name #enum_type_generics };

			let (de_generics, de_lifetime, error) = util::add_lifetime(&item.generics, "de");
			let (de_impl_generics, _type_generics, _where_clause) = de_generics.split_for_impl();
			let de_where_clause = make_where_clause(anchors, item, &de_lifetime);

			let expecting = format!("a tuple of {n} elements");

			let index_str = (0..n).map(|x| x.to_string());

			quote! {
				#error
				struct Wrap #enum_type_generics (#enum_ty) #enum_where_clause;
				impl #de_impl_generics #serde::Deserialize<#de_lifetime> for Wrap #enum_type_generics #de_where_clause {
					fn deserialize<D: #serde::Deserializer<#de_lifetime>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
						struct Visit #enum_type_generics(::core::marker::PhantomData<fn() -> #enum_ty>);

						impl #de_impl_generics #serde::de::Visitor<#de_lifetime> for Visit #enum_type_generics #de_where_clause {
							type Value = #enum_ty;

							fn expecting(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
								f.write_str(#expecting)
							}

							fn visit_seq<A: #serde::de::SeqAccess<#de_lifetime>>(self, mut seq: A) -> ::core::result::Result<Self::Value, A::Error> {
								#(
									let #mapped_field_name = seq.next_element()?
										.ok_or_else(|| serde::de::Error::missing_field(#index_str))?;
								)*
								let value = #enum_name::#variant_name(#( #mapped_field_name, )*);
								::core::result::Result::Ok(value)
							}
						}

						let value = deserializer.deserialize_tuple_struct(#variant_name_str, #n, Visit(::core::marker::PhantomData))?;
						::core::result::Result::Ok(Wrap(value))
					}
				}

				let value: Wrap #enum_type_generics = #internal::deserialize_variant(#variant_tag, map)?;
				::core::result::Result::Ok(value.0)
			}
		}
	}
}

fn deserialize_struct_variant(anchors: &Anchors, item: &syn::ItemEnum, variant_name: &syn::Ident, variant_tag: &str, fields: &syn::FieldsNamed) -> TokenStream {
	syn::Error::new_spanned(variant_name, "deserialize for struct variants not implemented yet").to_compile_error()
}

fn make_where_clause(anchors: &Anchors, item: &syn::ItemEnum, de_lifetime: &syn::Lifetime) -> Option<syn::WhereClause> {
	let serde = &anchors.serde;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for field in &variant.fields {
			if util::type_uses_generic(&field.ty, &item.generics) {
				let ty = &field.ty;
				predicates.push(syn::parse_quote! {
					#ty: #serde::Deserialize<#de_lifetime>
				})
			}
		}
	}

	match &item.generics.where_clause {
		Some(clause) => {
			let mut clause = clause.clone();
			clause.predicates.extend(predicates);
			Some(clause)
		},
		None => {
			if predicates.is_empty() {
				None
			} else {
				Some(syn::parse_quote!(where #(#predicates,)*))
			}
		}
	}
}

fn make_expecting(prefix: &str, values: &[impl AsRef<str>]) -> String {
	match values {
		[] => String::from("nothing"),
		[tag] => format!("{prefix} {:?}", tag.as_ref()),
		values => {
			use core::fmt::Write;
			let mut expecting = String::from(prefix);
			for (i, value) in values.iter().enumerate() {
				let value = value.as_ref();
				if i == 0 {
					write!(expecting, "{value:?}").unwrap();
				} else if i == values.len() - 1 {
					write!(expecting, " or {value:?}").unwrap();
				} else {
					write!(expecting, ", {value:?}").unwrap();
				}
			}
			expecting
		}
	}
}

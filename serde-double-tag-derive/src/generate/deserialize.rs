use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::{util, Context};

/// Generate code that implement the serde `Deserialize` trait for an enum using the double-tag format.
pub fn impl_deserialize_enum(context: &mut Context, item: crate::input::Enum) -> TokenStream {
	let internal = &context.internal;
	let serde = &context.serde;

	let enum_name = &item.ident;

	let variant_name: Vec<_> = item.variants.iter()
		.map(|x| &x.ident).collect();
	let variant_tag: Vec<_> = item.variants.iter()
		.map(|x| util::to_snake_case(&x.ident.to_string())).collect();
	let variant_deserialize = item.variants.iter()
		.zip(&variant_tag)
		.map(|(variant, tag_value)| deserialize_fields(context, &item, variant, tag_value));

	let (_impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let (de_generics, de_lifetime, error) = util::add_lifetime(&item.generics, "de");
	let where_clause = make_where_clause(context, &item, &de_lifetime);
	let (impl_generics, _type_generics, _where_clause) = de_generics.split_for_impl();

	let tag_name = "type";
	let tag_enum = make_tag_enum(context, &item);

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

fn make_tag_enum(context: &Context, item: &crate::input::Enum) -> TokenStream {
	let serde = &context.serde;
	let serde_str = serde.to_token_stream().to_string();
	let variant_name: Vec<_> = item.variants.iter().map(|x| &x.ident).collect();
	let rename_all = &item.attr.rename_all;

	quote! {
		#[derive(#serde::Deserialize)]
		#[serde(crate = #serde_str)]
		#rename_all
		enum Tag {
			#(#variant_name,)*
		}
	}
}

fn deserialize_fields(context: &Context, item: &crate::input::Enum, variant: &crate::input::Variant, variant_tag: &str) -> TokenStream {
	match &variant.fields {
		crate::input::Fields::Unit => deserialize_unit_variant(context, item, &variant.ident, variant_tag),
		crate::input::Fields::Tuple(fields) => deserialize_tuple_variant(context, item, &variant.ident, variant_tag, fields),
		crate::input::Fields::Struct(fields) => deserialize_struct_variant(context, item, &variant, variant_tag, fields),
	}
}

fn deserialize_unit_variant(context: &Context, item: &crate::input::Enum, variant_name: &syn::Ident, variant_tag: &str) -> TokenStream {
	let internal = &context.internal;
	let enum_name = &item.ident;
	quote! {
		let _value: () = #internal::deserialize_variant_optional(#variant_tag, map)?;
		::core::result::Result::Ok(#enum_name::#variant_name)
	}
}

fn deserialize_tuple_variant(context: &Context, item: &crate::input::Enum, variant_name: &syn::Ident, variant_tag: &str, fields: &crate::input::TupleFields) -> TokenStream {
	let internal = &context.internal;
	let serde = &context.serde;

	let enum_name = &item.ident;

	match fields.len() {
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
			let mapped_field_name: Vec<_> = fields.fields
				.iter()
				.map(|field| syn::Ident::new(&format!("field_{}", field.index), field.ty.span()))
				.collect();
			let (_, enum_type_generics, enum_where_clause) = item.generics.split_for_impl();
			let enum_ty = quote! { #enum_name #enum_type_generics };

			let (de_generics, de_lifetime, error) = util::add_lifetime(&item.generics, "de");
			let (de_impl_generics, _type_generics, _where_clause) = de_generics.split_for_impl();
			let de_where_clause = make_where_clause(context, item, &de_lifetime);

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

fn deserialize_struct_variant(context: &Context, item: &crate::input::Enum, variant: &crate::input::Variant, variant_tag: &str, fields: &crate::input::StructFields) -> TokenStream {
	let serde = &context.serde;
	let serde_str = quote::ToTokens::to_token_stream(serde).to_string();
	let internal = &context.internal;

	let enum_name = &item.ident;
	let variant_name = &variant.ident;
	let field_declarations = &fields.fields;
	let field_name: Vec<_> = fields.fields.iter().map(|x| &x.ident).collect();
	let mapped_field_name: Vec<_> = field_name.iter().map(|x| syn::Ident::new(&format!("field_{x}"), x.span())).collect();
	let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

	let mut serde_attributes = variant.attr.clone();
	serde_attributes.rename = None;
	let rename = variant.ident.to_string();
	let rename_all = match &variant.attr.rename_all {
		Some(x) => Some(x.to_token_stream()),
		None => item.attr.rename_all_fields.as_ref()
			.map(|x| {
				let case_convert = x.value;
				quote!(#[serde(rename_all = #case_convert)])
			}),
	};
	let bound = variant.attr.bound.as_ref()
		.or(item.attr.bound.as_ref());
	let deny_unknown_fields = &item.attr.deny_unknown_fields;
	let expecting = &item.attr.expecting;

	quote! {
		#[derive(#serde::Deserialize)]
		#[serde(crate = #serde_str)]
		#[serde(rename = #rename)]
		#rename_all
		#deny_unknown_fields
		#bound
		#expecting
		struct Fields #impl_generics #where_clause {
			#field_declarations

			#[serde(skip, default)]
			__serde_double_tag_phantom_enum: ::core::marker::PhantomData<fn() -> #enum_name #type_generics>,
		}
		let value: Fields #type_generics = #internal::deserialize_variant(#variant_tag, map)?;
		let Fields {
			#( #field_name: #mapped_field_name , )*
			__serde_double_tag_phantom_enum: _,
		} = value;
		::core::result::Result::Ok(#enum_name::#variant_name {
			#( #field_name: #mapped_field_name , )*
		})
	}
}

fn make_where_clause(context: &Context, item: &crate::input::Enum, de_lifetime: &syn::Lifetime) -> Option<syn::WhereClause> {
	let serde = &context.serde;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for ty in variant.fields.iter_types() {
			if util::type_uses_generic(ty, &item.generics) {
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

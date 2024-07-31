use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{util, Context};

/// Generate code that implement the serde `Deserialize` trait for an enum using the double-tag format.
pub fn impl_deserialize_enum(context: &mut Context, item: crate::input::Enum) -> TokenStream {
	let enum_name = &item.ident;
	let tag_field_name = super::tag_field_name(context, &item);

	let variant_name: Vec<_> = item.variants.iter()
		.map(|variant| &variant.ident).collect();
	let variant_deserialize: Vec<_> = item.variants.iter()
		.map(|variant| {
			let variant_name =  &variant.ident;
			let variant_tag_value = super::variant_tag_value(&item, variant);
			let data = make_data_struct(context, &item, variant);
			let fields = super::fields_expression(&variant.fields);
			let deny_unknown_fields = item.attr.deny_unknown_fields.is_some();

			let function = match variant.fields {
				crate::input::Fields::Unit => proc_macro2::Ident::new("deserialize_variant_optional", Span::call_site()),
				crate::input::Fields::Tuple(_) => proc_macro2::Ident::new("deserialize_variant_required", Span::call_site()),
				crate::input::Fields::Struct(_) => proc_macro2::Ident::new("deserialize_variant_required", Span::call_site()),
			};

			let internal = &context.internal;

			quote! {
				#data
				const FIELD_NAMES: #internal::FieldNames = #internal::FieldNames {
					tag: #tag_field_name,
					content: #variant_tag_value,
				};
				let Data #fields = #internal::#function(&FIELD_NAMES, map, #deny_unknown_fields)?;
				Ok(Self::Value::#variant_name #fields)
			}
		})
		.collect();

	let (_impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let (de_generics, de_lifetime) = util::add_lifetime(context, &item.generics, "de");
	let (impl_generics, _type_generics, _where_clause) = de_generics.split_for_impl();
	let where_clause = make_where_clause(context, &item, &de_lifetime);

	let tag_enum = make_tag_enum(context, &item);

	let internal = &context.internal;
	let serde = &context.serde;

	quote! {
		#[automatically_derived]
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

						let tag: Tag = #internal::deserialize_tag(#tag_field_name, &mut map)?;
						match tag {
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

fn make_where_clause(context: &Context, item: &crate::input::Enum, de_lifetime: &syn::Lifetime) -> Option<syn::WhereClause> {
	let serde = &context.serde;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for ty in variant.fields.iter_types() {
			let ty = util::strip_type_wrappers(ty);
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

fn make_tag_enum(context: &Context, item: &crate::input::Enum) -> TokenStream {
	let variant_name: Vec<_> = item.variants.iter().map(|x| &x.ident).collect();
	let rename_all = &item.attr.rename_all;
	let rename = super::tag_struct_name(item);

	let serde = &context.serde;
	let serde_str = serde.to_token_stream().to_string();

	quote! {
		#[derive(#serde::Deserialize)]
		#[serde(crate = #serde_str)]
		#[serde(rename = #rename)]
		#rename_all
		enum Tag {
			#(#variant_name,)*
		}
	}
}

fn make_data_struct(context: &mut Context, item: &crate::input::Enum, variant: &crate::input::Variant) -> TokenStream {
	// Remove generic parameters not needed for the fields of this variant.
	let fields = &variant.fields;
	let generics = util::prune_generics(&item.generics, fields.iter_types());
	let (impl_generics, _type_generics, where_clause) = generics.split_for_impl();

	// Prepare attributes for the `Data` struct.
	let data_rename_all = variant.rename_all_rule(item);
	let data_rename = &variant.attr.rename;
	let data_deny_unknown_fields = &item.attr.deny_unknown_fields;

	let serde = &context.serde;
	let serde_str = serde.to_token_stream().to_string();

	let mut tokens = quote! {
		#[derive(#serde::Deserialize)]
		#[serde(crate = #serde_str)]
		#data_rename_all
		#data_rename
		#data_deny_unknown_fields
		struct Data #impl_generics #where_clause #fields;
	};

	// If this is a unit variant, add a `Default` implementation for the data struct.
	if fields.is_unit() {
		tokens.extend(quote! {
			impl ::core::default::Default for Data {
				fn default() -> Self {
					Self
				}
			}
		})
	}

	tokens
}

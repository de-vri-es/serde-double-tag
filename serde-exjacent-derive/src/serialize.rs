use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::{util, Anchors};

/// Generate code that implement the serde `Serialize` trait for an enum using the double-tag format.
pub fn impl_serialize_enum(anchors: &Anchors, tokens: TokenStream) -> TokenStream {
	let item = match util::parse_enum_item(tokens) {
		Ok(x) => x,
		Err(e) => return e.into_compile_error(),
	};

	let serde = &anchors.serde;
	let enum_name = &item.ident;

	let match_arms = item.variants.iter().map(|variant| {
		let variant_name = &variant.ident;
		let snake_case = util::to_snake_case(&variant_name.to_string());
		match &variant.fields {
			syn::Fields::Unit => quote! {
				Self::#variant_name => {
					let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(1))?;
					#serde::ser::SerializeMap::serialize_key(&mut map, "type")?;
					#serde::ser::SerializeMap::serialize_value(&mut map, #snake_case)?;
					#serde::ser::SerializeMap::end(map)
				},
			},
			syn::Fields::Named(fields) => {
				let field_name: Vec<_> = fields.named.iter().map(|x| x.ident.as_ref().unwrap()).collect();
				let field_name_str: Vec<_> = field_name.iter().map(|x| x.to_string()).collect();
				let mapped_field_name: Vec<_> = field_name.iter().map(|name| syn::Ident::new(&format!("field_{name}"), name.span())).collect();
				let field_count = fields.named.len();

				let (field_generics, field_lifetime, error) = util::add_lifetime(&item.generics, "a");
				let (_impl_generics, enum_type_generics, enum_where_clause) = item.generics.split_for_impl();
				let (fields_impl_generics, fields_type_generics, _where_clause) = field_generics.split_for_impl();
				let fields_where_clause = make_where_clause(anchors, &item);

				quote! {
					Self::#variant_name { .. } => {
						#error
						struct Fields #fields_type_generics #enum_where_clause (&#field_lifetime #enum_name #enum_type_generics);
						impl #fields_impl_generics #serde::ser::Serialize for Fields #fields_type_generics #fields_where_clause {
							fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
							where
								S: #serde::Serializer,
							{
								match &self.0 {
									#enum_name::#variant_name { #(#field_name: #mapped_field_name),* } => {
										let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(#field_count))?;
										#(
											#serde::ser::SerializeMap::serialize_key(&mut map, #field_name_str)?;
											#serde::ser::SerializeMap::serialize_value(&mut map, #mapped_field_name)?;
										)*
										#serde::ser::SerializeMap::end(map)
									},
									// We already know `self` is the right variant, so we can not get here.
									_ => ::core::unreachable!(),
								}
							}
						}

						let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(2))?;
						#serde::ser::SerializeMap::serialize_key(&mut map, "type")?;
						#serde::ser::SerializeMap::serialize_value(&mut map, #snake_case)?;
						#serde::ser::SerializeMap::serialize_key(&mut map, #snake_case)?;
						#serde::ser::SerializeMap::serialize_value(&mut map, &Fields(self))?;
						#serde::ser::SerializeMap::end(map)
					}
				}
			},
			syn::Fields::Unnamed(fields) => match fields.unnamed.len() {
				0 => quote! {
					Self::#variant_name() => {
						let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(1))?;
						#serde::ser::SerializeMap::serialize_key(&mut map, "type")?;
						#serde::ser::SerializeMap::serialize_value(&mut map, #snake_case)?;
						#serde::ser::SerializeMap::end(map)
					},
				},
				1 => quote! {
					Self::#variant_name(value) => {
						let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(2))?;
						#serde::ser::SerializeMap::serialize_key(&mut map, "type")?;
						#serde::ser::SerializeMap::serialize_value(&mut map, #snake_case)?;
						#serde::ser::SerializeMap::serialize_key(&mut map, #snake_case)?;
						#serde::ser::SerializeMap::serialize_value(&mut map, value)?;
						#serde::ser::SerializeMap::end(map)
					},
				},
				_ => {
					let mapped_field_name: Vec<_> = fields.unnamed.iter()
						.enumerate()
						.map(|(i, field)| syn::Ident::new(&format!("field_{i}"), field.ty.span()))
						.collect();
					let field_count = fields.unnamed.len();
					let (field_generics, field_lifetime, error) = util::add_lifetime(&item.generics, "a");
					let (_impl_generics, enum_type_generics, enum_where_clause) = item.generics.split_for_impl();
					let (fields_impl_generics, fields_type_generics, _where_clause) = field_generics.split_for_impl();
					let fields_where_clause = make_where_clause(anchors, &item);

					quote! {
						Self::#variant_name ( .. ) => {
							#error
							struct Fields #fields_type_generics #enum_where_clause (&#field_lifetime #enum_name #enum_type_generics);
							impl #fields_impl_generics  #serde::ser::Serialize for Fields #fields_type_generics #fields_where_clause {
								fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
								where
									S: #serde::Serializer,
								{
									match &self.0 {
										#enum_name::#variant_name ( #(#mapped_field_name),* ) => {
											let mut seq = #serde::ser::Serializer::serialize_seq(serializer, Some(#field_count))?;
											#(
												#serde::ser::SerializeSeq::serialize_element(&mut seq, #mapped_field_name)?;
											)*
											#serde::ser::SerializeSeq::end(seq)
										},
										// We already know `self` is the right variant, so we can not get here.
										_ => ::core::unreachable!(),
									}
								}
							}

							let mut map = #serde::ser::Serializer::serialize_map(serializer, Some(2))?;
							#serde::ser::SerializeMap::serialize_key(&mut map, "type")?;
							#serde::ser::SerializeMap::serialize_value(&mut map, #snake_case)?;
							#serde::ser::SerializeMap::serialize_key(&mut map, #snake_case)?;
							#serde::ser::SerializeMap::serialize_value(&mut map, &Fields(self))?;
							#serde::ser::SerializeMap::end(map)
						}
					}
				},
			},
		}
	});

	let (impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let where_clause = make_where_clause(anchors, &item);

	quote! {
		impl #impl_generics  #serde::Serialize for #enum_name #type_generics #where_clause {
			fn serialize<S: #serde::ser::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
				match self {
					#(#match_arms)*,
				}
			}
		}
	}
}

fn make_where_clause(anchors: &Anchors, item: &syn::ItemEnum) -> Option<syn::WhereClause> {
	let serde = &anchors.serde;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for field in &variant.fields {
			if util::type_uses_generic(&field.ty, &item.generics) {
				let ty = &field.ty;
				predicates.push(syn::parse_quote! {
					#ty: #serde::Serialize
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

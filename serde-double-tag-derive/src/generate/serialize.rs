use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{util, Context};

/// Generate code that implement the serde `Serialize` trait for an enum using the double-tag format.
pub fn impl_serialize_enum(context: &mut Context, item: crate::input::Enum) -> TokenStream {
	let enum_name = &item.ident;

	let match_arms: Vec<_> = item.variants.iter()
		.map(|variant| {
			let repr = make_repr_struct(context, &item, variant);
			let variant_name = &variant.ident;
			let fields = fields_expression(&variant.fields);
			let serde = &context.serde;

			quote! {
				Self::#variant_name #fields => {
					#repr
					let repr = Repr {
						tag: Tag::#variant_name,
						data: Data #fields,
					};
					#serde::Serialize::serialize(&repr, serializer)
				},
			}
		})
		.collect();

	let (impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let where_clause = make_where_clause(context, &item);

	let serde = &context.serde;
	quote! {
		impl #impl_generics  #serde::Serialize for #enum_name #type_generics #where_clause {
			fn serialize<S: #serde::ser::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
				match self {
					#(#match_arms)*
				}
			}
		}
	}
}

fn make_where_clause(context: &Context, item: &crate::input::Enum) -> Option<syn::WhereClause> {
	let serde = &context.serde;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for ty in variant.fields.iter_types() {
			let ty = util::strip_type_wrappers(ty);
			if util::type_uses_generic(ty, &item.generics) {
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

fn make_repr_struct(context: &mut Context, item: &crate::input::Enum, variant: &crate::input::Variant) -> TokenStream {
	// Make a struct with borrowed fields.
	let (generics, lifetime) = util::add_lifetime(context, &item.generics, "serde_double_tag");
	let borrowed_fields = variant.fields.add_lifetime(&lifetime);

	// Remove generic parameters not needed for the fields of this variant.
	let generics = util::prune_generics(&generics, borrowed_fields.iter_types());
	let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

	let tag_field_name = super::tag_field_name(context, item);
	let data_field_name = super::variant_tag_value(item, variant);

	// Prepare attributes for the `Repr` struct.
	let repr_rename = item.attr.rename.clone()
		.unwrap_or_else(|| crate::input::attributes::KeyValueArg::new_call_site("serde", syn::LitStr::new(&item.ident.to_string(), item.ident.span())));
	let data_field_skip = match variant.fields {
		crate::input::Fields::Unit => Some(quote!(#[serde(skip)])),
		crate::input::Fields::Tuple(_) => None,
		crate::input::Fields::Struct(_) => None,
	};

	// Prepare attributes for the `Tag` enum.
	let tag_rename_all = &item.attr.rename_all;
	let tag_rename = match &item.attr.rename {
		None => format!("{}Tag", item.ident),
		Some(x) => format!("{}Tag", x.value.value()),
	};
	let tag_rename = quote!(#[serde(rename = #tag_rename)]);
	let variant_rename = &variant.attr.rename;

	// Prepare attributes for the `Data` struct.
	let data_rename_all = variant.rename_all_rule(item);
	let data_rename = &variant.attr.rename;

	let serde = &context.serde;
	let serde_str = serde.to_token_stream().to_string();
	let variant_name = &variant.ident;

	quote! {
		#[derive(#serde::Serialize)]
		#[serde(crate = #serde_str)]
		#repr_rename
		struct Repr #impl_generics #where_clause {
			#[serde(rename = #tag_field_name)]
			tag: Tag,

			#[serde(rename = #data_field_name)]
			#data_field_skip
			data: Data #type_generics,
		}

		#[derive(#serde::Serialize)]
		#[serde(crate = #serde_str)]
		#tag_rename_all
		#tag_rename
		enum Tag {
			#variant_rename
			#variant_name
		}

		#[derive(#serde::Serialize)]
		#[serde(crate = #serde_str)]
		#data_rename_all
		#data_rename
		struct Data #impl_generics #where_clause #borrowed_fields;
	}
}

fn fields_expression(fields: &crate::input::Fields) -> TokenStream {
	match fields {
		crate::input::Fields::Unit => TokenStream::new(),
		crate::input::Fields::Tuple(fields) => {
			let mapped_field_name = fields.fields.iter().map(|x| quote::format_ident!("field_{}", x.index));
			quote! {
				(#(#mapped_field_name),*)
			}
		},
		crate::input::Fields::Struct(fields) => {
			let field_name = fields.fields.iter().map(|x| &x.ident);
			let mapped_field_name = fields.fields.iter().map(|x| quote::format_ident!("field_{}", x.ident));
			quote! {
				{ #(#field_name: #mapped_field_name),* }
			}
		}
	}
}

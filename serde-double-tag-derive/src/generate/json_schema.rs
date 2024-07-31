use proc_macro2::TokenStream;
use quote::quote;

use crate::{util, Context};

/// Generate code that implement the serde `Deserialize` trait for an enum using the double-tag format.
pub fn impl_json_schema(context: &mut Context, item: crate::input::Enum) -> TokenStream {
	let tag_field_name = super::tag_field_name(context, &item);

	let ty = &item.ident;
	let tag_values: Vec<_> = item
		.variants
		.iter()
		.map(|variant| super::variant_tag_value(&item, variant))
		.collect();
	let variant_count = item.variants.len();
	let tag_schema = make_tag_schema(context, &tag_values);
	let subschemas = make_variant_subschemas(context, &item, &tag_field_name, &tag_values);

	let internal = &context.internal;
	let schemars = &context.schemars;

	let (impl_generics, type_generics, _where_clause) = item.generics.split_for_impl();
	let where_clause = make_where_clause(context, &item);

	quote! {
		#[automatically_derived]
		impl #impl_generics #schemars::JsonSchema for #ty #type_generics #where_clause {
			fn schema_name() -> ::std::string::String {
				#internal::string(::core::any::type_name::<Self>())
			}

			fn schema_id() -> ::std::borrow::Cow<'static, ::core::primitive::str> {
				::std::borrow::Cow::Borrowed(::core::any::type_name::<Self>())
			}

			fn json_schema(generator: &mut #schemars::gen::SchemaGenerator) -> #schemars::schema::Schema {
				let mut type_values = ::std::vec::Vec::with_capacity(#variant_count);
				#(
					type_values.push(#internal::string(#tag_values));
				)*
				let mut properties = #schemars::Map::with_capacity(1);
				let mut required = ::std::collections::BTreeSet::new();

				properties.insert(#internal::string(#tag_field_name), #tag_schema);
				required.insert(#internal::string(#tag_field_name));

				#schemars::schema::Schema::Object(
					#schemars::schema::SchemaObject {
						instance_type: ::core::option::Option::Some(
							#schemars::schema::SingleOrVec::Single(
								::std::boxed::Box::new(
									#schemars::schema::InstanceType::Object
								)
							)
						),
						object: ::core::option::Option::Some(::std::boxed::Box::new(
							#schemars::schema::ObjectValidation {
								properties,
								required,
								.. ::core::default::Default::default()
							}
						)),
						subschemas: #subschemas,
						.. ::core::default::Default::default()
					}
				)
			}
		}
	}
}

/// Generate code that creates a `schemars::schema::Schema` for an enum tag.
fn make_tag_schema(context: &Context, tag_values: &[String]) -> TokenStream {
	let count = tag_values.len();
	let internal = &context.internal;
	let schemars = &context.schemars;
	quote! {{
		#[allow(unused_mut)]
		let mut tag_values = ::std::vec::Vec::with_capacity(#count);
		#(
			tag_values.push(#internal::json_value(#tag_values));
		)*

		#schemars::schema::Schema::Object(
			#schemars::schema::SchemaObject {
				instance_type: ::core::option::Option::Some(
					#schemars::schema::SingleOrVec::Single(
						::std::boxed::Box::new(
							#schemars::schema::InstanceType::String
						)
					)
				),
				enum_values: ::core::option::Option::Some(tag_values),
				.. ::core::default::Default::default()
			}
		)
	}}
}

/// Generate code that returns a `schemars::schema::Schema` for the given [`Fields`].
fn make_schema_for_fields(context: &mut Context, item: &crate::input::Enum, variant: &crate::input::Variant) -> TokenStream {
	match &variant.fields {
		crate::input::Fields::Unit => make_schema_for_unit_value(context),
		crate::input::Fields::Tuple(fields) => make_schema_for_tuple_fields(context, fields),
		crate::input::Fields::Struct(fields) => make_schema_for_struct_fields(context, item, variant, fields),
	}
}

/// Generate code that returns a `schemars::schema::Schema` for a unit value.
fn make_schema_for_unit_value(context: &Context) -> TokenStream {
	let internal = &context.internal;
	quote!(#internal::unit_schema())
}

/// Generate code that returns the `schemars::schema::SubschemaValidation` for all variants of an enum (wrapped in `Option<Box<T>>`).
///
/// The subschemas are a bunch of `if/then` schemas that match on the enum tag,
/// and extend the object with a required property for the variant.
fn make_variant_subschemas(
	context: &mut Context,
	item: &crate::input::Enum,
	tag_field_name: &str,
	tag_values: &[String],
) -> TokenStream {
	// Generate the code for the subschema validation for each variant.
	let mut subschemas = Vec::with_capacity(item.variants.len());
	for (variant, tag_value) in item.variants.iter().zip(tag_values) {
		if variant.fields.is_unit() {
			continue;
		}
		let fields_schema = make_schema_for_fields(context, item, variant);
		let internal = &context.internal;
		subschemas.push(quote!(#internal::variant_subschema(#tag_field_name, #tag_value, #fields_schema)));
	}

	// Combine the subschemas into a single `Option<SubschemaValidation>` object.
	// Zero subschemas becomes `None`,
	// One subschema becomes `Some(subschema)`,
	// Multiple subschemas becomes `Some(AllOf(subschemas))`.
	let internal = &context.internal;
	let schemars = &context.schemars;
	match subschemas.len() {
		0 => quote!(::core::option::Option::None),
		1 => {
			let subschema = subschemas.remove(0);
			quote!( ::core::option::Option::Some(::std::boxed::Box::new(#subschema)) )
		}
		count => quote! {
			::core::option::Option::Some(::std::boxed::Box::new(
				#schemars::schema::SubschemaValidation {
					all_of: {
						let mut all_of = ::std::vec::Vec::with_capacity(#count);
						#(
							all_of.push(#internal::subschema_to_schema(#subschemas));
						)*
						::core::option::Option::Some(all_of)
					},
					.. ::core::default::Default::default()
				}
			))
		},
	}
}

/// Generate code that returns a `schemars::schema::Schema` for struct fields.
fn make_schema_for_struct_fields(
	context: &Context,
	item: &crate::input::Enum,
	variant: &crate::input::Variant,
	fields: &crate::input::StructFields,
) -> TokenStream {

	let field_name: Vec<_> = fields
		.fields
		.iter()
		.map(|field| super::field_name(item, variant, field))
		.collect();
	let field_type: Vec<_> = fields.fields.iter().map(|x| &x.ty).collect();
	let field_count = fields.fields.len();

	let schemars = &context.schemars;
	let internal = &context.internal;
	quote! {{
		#[allow(unused_mut)]
		let mut properties = #schemars::Map::with_capacity(#field_count);
		#[allow(unused_mut)]
		let mut required = ::std::collections::BTreeSet::new();
		#(
			properties.insert(#internal::string(#field_name), generator.subschema_for::<#field_type>());
			if <#field_type as #schemars::JsonSchema>::_schemars_private_is_option() == false {
				required.insert(#internal::string(#field_name));
			}
		)*
		#schemars::schema::Schema::Object(
			#schemars::schema::SchemaObject {
				instance_type: ::core::option::Option::Some(
					#schemars::schema::SingleOrVec::Single(
						::std::boxed::Box::new(
							#schemars::schema::InstanceType::Object
						)
					)
				),
				object: ::core::option::Option::Some(::std::boxed::Box::new(
						#schemars::schema::ObjectValidation {
							properties,
							required,
							..::core::default::Default::default()
						}
				)),
				..::core::default::Default::default()
			}
		)
    }}
}

/// Generate code that returns a `schemars::schema::Schema` for tuple fields.
fn make_schema_for_tuple_fields(
	context: &mut Context,
	fields: &crate::input::TupleFields,
) -> TokenStream {
	match &fields.fields.len() {
		// Treat single-field tuple variants as the inner type.
		1 => {
			let field_type = &fields.fields[0].ty;
			quote! { generator.subschema_for::<#field_type>() }
		}

		// Treat the rest as fixed-size arrays.
		field_count => {
			let field_type = fields.fields.iter().map(|x| &x.ty);
			let item_count = u32::try_from(*field_count)
				.map_err(|_| context.spanned_error(fields, format_args!(
					"too many fields in variant: {} > {}",
					fields.fields.len(),
					u32::MAX
				)))
				.unwrap_or(u32::MAX);

			let schemars = &context.schemars;
			quote! {{
				let mut items = ::std::vec::Vec::with_capacity(#field_count);
				#(
					items.push(generator.subschema_for::<#field_type>());
				)*
				#schemars::schema::Schema::Object(
					#schemars::schema::SchemaObject {
						instance_type: ::core::option::Option::Some(
							#schemars::schema::SingleOrVec::Single(
								::std::boxed::Box::new(
									#schemars::schema::InstanceType::Array
								)
							)
						),
						array: ::core::option::Option::Some(::std::boxed::Box::new(
							#schemars::schema::ArrayValidation {
								items: ::core::option::Option::Some(#schemars::schema::SingleOrVec::Vec(items)),
								min_items: ::core::option::Option::Some(#item_count),
								max_items: ::core::option::Option::Some(#item_count),
								..::core::default::Default::default()
							}
						)),
						..::core::default::Default::default()
					}
				)
			}}
		}
	}
}

fn make_where_clause(context: &Context, item: &crate::input::Enum) -> Option<syn::WhereClause> {
	let schemars = &context.schemars;

	let mut predicates = Vec::<syn::WherePredicate>::new();
	for variant in &item.variants {
		for ty in variant.fields.iter_types() {
			let ty = util::strip_type_wrappers(ty);
			if util::type_uses_generic(ty, &item.generics) {
				predicates.push(syn::parse_quote! {
					#ty: #schemars::JsonSchema
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

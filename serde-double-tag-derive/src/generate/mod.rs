use proc_macro2::TokenStream;
use quote::quote;

use crate::Context;

mod deserialize;
pub use deserialize::impl_deserialize_enum;

mod serialize;
pub use serialize::impl_serialize_enum;

#[cfg(feature = "schemars")]
mod json_schema;
#[cfg(feature = "schemars")]
pub use json_schema::impl_json_schema;

/// Compute the name of the tag enum.
fn tag_struct_name(item: &crate::input::Enum) -> String {
	match &item.attr.rename {
		Some(x) => format!("{}Tag", x.value.value()),
		None => format!("{}Tag", item.ident),
	}
}

/// Compute the name of the tag field for an enum.
fn tag_field_name(context: &mut Context, item: &crate::input::Enum) -> String {
	match &item.attr.tag {
		Some(x) => x.value.value(),
		None => {
			context.error(item.ident.span(), "missing required #[serde(tag = \"...\")] attribute");
			"tag".into()
		},
	}
}

/// Compute the tag value for a variant.
fn variant_tag_value(item: &crate::input::Enum, variant: &crate::input::Variant) -> String {
	if let Some(name) = &variant.attr.rename {
		name.value.value()
	} else if let Some(rename_all) = &item.attr.rename_all {
		rename_all.value.rule.apply_to_variant(&variant.ident.to_string())
	} else {
		variant.ident.to_string()
	}
}

/// Compute the serialized name for a field.
#[cfg_attr(not(feature = "schemars"), allow(unused))]
fn field_name(item: &crate::input::Enum, variant: &crate::input::Variant, field: &crate::input::StructField) -> String {
	if let Some(rename) = &field.attrs.rename {
		rename.value.value()
	} else if let Some(rename_all) = &variant.attr.rename_all {
		rename_all.value.rule.apply_to_field(&field.ident.to_string())
	} else if let Some(rename_all_fields) = &item.attr.rename_all_fields {
		rename_all_fields.value.rule.apply_to_field(&field.ident.to_string())
	} else {
		field.ident.to_string()
	}
}

/// Create an expression for capturing and specifying the fields of a variant.
///
/// The name of the variable to capture each field is prefixed with `field_`.
///
/// The returned tokens depend on the type of fields:
/// * For unit fields, this returns an empty TokenStream.
/// * For tuple fields, this returns the tokens: `(field_0, field_1 ...)`.
/// * For struct fields, this returns the tokens: `{ a: field_a, b: field_b, ...}`.
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
		},
	}
}

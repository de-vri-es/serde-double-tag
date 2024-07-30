use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::Context;
use crate::input::attributes::KeyValueArg;


mod deserialize;
pub use deserialize::impl_deserialize_enum;

mod serialize;
pub use serialize::impl_serialize_enum;


fn make_tag_enum(context: &Context, item: &crate::input::Enum) -> TokenStream {
	let serde = &context.serde;
	let serde_str = serde.to_token_stream().to_string();
	let variant_name: Vec<_> = item.variants.iter().map(|x| &x.ident).collect();
	let rename_all = &item.attr.rename_all;
	let rename = tag_struct_name(item);

	quote! {
		#[serde(crate = #serde_str)]
		#[serde(rename = #rename)]
		#rename_all
		enum Tag {
			#(#variant_name,)*
		}
	}
}

fn tag_struct_name(item: &crate::input::Enum) -> String {
	match &item.attr.rename {
		Some(x) => format!("{}Tag", x.value.value()),
		None => format!("{}Tag", item.ident),
	}
}

fn tag_field_name(context: &mut Context, item: &crate::input::Enum) -> String {
	match &item.attr.tag {
		Some(x) => x.value.value(),
		None => {
			context.call_site_error("missing required #[serde(tag = \"...\")] attribute");
			"tag".into()
		},
	}
}

fn variant_tag_value(item: &crate::input::Enum, variant: &crate::input::Variant) -> String {
	if let Some(name) = &variant.attr.rename {
		name.value.value()
	} else if let Some(rename_all) = &item.attr.rename_all {
		rename_all.value.rule.apply_to_variant(&variant.ident.to_string())
	} else {
		variant.ident.to_string()
	}
}

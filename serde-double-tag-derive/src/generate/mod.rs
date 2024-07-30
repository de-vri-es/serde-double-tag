use crate::Context;

mod deserialize;
pub use deserialize::impl_deserialize_enum;

mod serialize;
pub use serialize::impl_serialize_enum;

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
			context.error(item.ident.span(), "missing required #[serde(tag = \"...\")] attribute");
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

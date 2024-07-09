// mod config_enum;
mod deserialize;
mod util;
mod serialize;

/// Get the crate name to use for the main library crate.
fn anchors() -> Anchors {
    Anchors::new(syn::parse_quote!(::serde_exjacent))
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    deserialize::impl_deserialize_enum(&anchors(), tokens.into()).into()
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serialize::impl_serialize_enum(&anchors(), tokens.into()).into()
}

#[proc_macro_derive(JsonSchema)]
pub fn derive_json_schema(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!();
}

struct Anchors {
	pub crate_name: syn::Path,
	pub internal: syn::Path,
	pub serde: syn::Path,
}

impl Anchors {
	fn new(crate_name: syn::Path) -> Self {
		let internal = syn::parse_quote!(#crate_name::internal__);
		let serde = syn::parse_quote!(#internal::serde);
		Self {
			crate_name,
			internal,
			serde,
		}
	}
}

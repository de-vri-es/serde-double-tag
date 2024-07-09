// mod config_enum;
mod util;
mod serialize;

/// Get the crate name to use for the main library crate.
fn crate_name() -> syn::Path {
    syn::parse_quote!(::serde_exjacent)
}

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!();
}

#[proc_macro_derive(DeserializePartial)]
pub fn derive_deserialize_partial(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!();
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serialize::impl_serialize_enum(&crate_name(), tokens.into()).into()
}

#[proc_macro_derive(SerializePartial)]
pub fn derive_serialize_partial(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!();
}

#[proc_macro_derive(JsonSchema)]
pub fn derive_json_schema(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!();
}

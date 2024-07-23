use proc_macro2::TokenStream;
use quote::quote;

mod deserialize_partial;
mod json_schema;
mod serialize_partial;

/// Derive the `ConfigEnum` trait and all super traits.
pub fn derive_config_enum(crate_name: syn::Path, tokens: TokenStream) -> TokenStream {
    let item: syn::ItemEnum = match syn::parse2(tokens) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error(),
    };

    let mut output = TokenStream::new();

    // Check for an enum variant that conflicts with the `type` tag we will add.
    for variant in &item.variants {
        if variant.ident.to_string().eq_ignore_ascii_case("type") {
            let error = syn::Error::new_spanned(&variant.ident, "#[derive(ConfigEnum)]: variant name conflicts with implicitly added enum tag in the serialization");
            output.extend(error.to_compile_error());
        }
    }

    output.extend(deserialize_partial::impl_deserialize_partial_enum(
        &crate_name,
        &item,
    ));
    output.extend(serialize_partial::impl_serialize_partial_enum(
        &crate_name,
        &item,
    ));
    output.extend(json_schema::impl_json_schema_enum(&crate_name, &item));

    let ty = &item.ident;
    output.extend(quote! {
        impl #crate_name::ConfigEnum for #ty {}
    });

    output
}

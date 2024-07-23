use proc_macro2::TokenStream;
use quote::quote;

use crate::util;

/// Generate code that implement the `JsonSchema` trait for an enum.
pub fn impl_json_schema_enum(crate_name: &syn::Path, item: &syn::ItemEnum) -> TokenStream {
    let ty = &item.ident;
    let tag_values: Vec<_> = item
        .variants
        .iter()
        .map(|x| util::to_snake_case(&x.ident.to_string()))
        .collect();
    let variant_count = item.variants.len();
    let tag_name = "type";
    let tag_schema = make_tag_schema(crate_name, &tag_values);
    let subschemas = make_variant_subschemas(crate_name, item, tag_name, &tag_values);

    quote! {
        impl #crate_name::internal__::schemars::JsonSchema for #ty {
            fn schema_name() -> ::std::string::String {
                #crate_name::internal__::string(::core::any::type_name::<Self>())
            }

            fn schema_id() -> ::std::borrow::Cow<'static, ::core::primitive::str> {
                ::std::borrow::Cow::Borrowed(::core::any::type_name::<Self>())
            }

            fn json_schema(generator: &mut #crate_name::internal__::schemars::gen::SchemaGenerator) -> #crate_name::internal__::schemars::schema::Schema {
                let mut type_values = ::std::vec::Vec::with_capacity(#variant_count);
                #(
                    type_values.push(#crate_name::internal__::string(#tag_values));
                )*
                let mut properties = #crate_name::internal__::schemars::Map::with_capacity(1);
                let mut required = ::std::collections::BTreeSet::new();

                properties.insert(#crate_name::internal__::string(#tag_name), #tag_schema);
                required.insert(#crate_name::internal__::string(#tag_name));

                #crate_name::internal__::schemars::schema::Schema::Object(
                    #crate_name::internal__::schemars::schema::SchemaObject {
                        instance_type: ::core::option::Option::Some(
                            #crate_name::internal__::schemars::schema::SingleOrVec::Single(
                                ::std::boxed::Box::new(
                                    #crate_name::internal__::schemars::schema::InstanceType::Object
                                )
                            )
                        ),
                        object: ::core::option::Option::Some(::std::boxed::Box::new(
                            #crate_name::internal__::schemars::schema::ObjectValidation {
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
fn make_tag_schema(crate_name: &syn::Path, tag_values: &[String]) -> TokenStream {
    let count = tag_values.len();
    quote! {{
        #[allow(unused_mut)]
        let mut tag_values = ::std::vec::Vec::with_capacity(#count);
        #(
            tag_values.push(#crate_name::internal__::json_value(#tag_values));
        )*

        #crate_name::internal__::schemars::schema::Schema::Object(
            #crate_name::internal__::schemars::schema::SchemaObject {
                instance_type: ::core::option::Option::Some(
                    #crate_name::internal__::schemars::schema::SingleOrVec::Single(
                        ::std::boxed::Box::new(
                            #crate_name::internal__::schemars::schema::InstanceType::String
                        )
                    )
                ),
                enum_values: ::core::option::Option::Some(tag_values),
                .. ::core::default::Default::default()
            }
        )
    }}
}

/// Generate code that returns the `schemars::schema::SubschemaValidation` for all variants of an enum (wrapped in `Option<Box<T>>`).
///
/// The subschemas are a bunch of `if/then` schemas that match on the enum tag,
/// and extend the object with a required property for the variant.
fn make_variant_subschemas(
    crate_name: &syn::Path,
    item: &syn::ItemEnum,
    tag_name: &str,
    tag_values: &[String],
) -> TokenStream {
    let mut subschemas = Vec::with_capacity(item.variants.len());
    for (variant, tag_value) in item.variants.iter().zip(tag_values) {
        if variant.fields.is_empty() {
            continue;
        }
        let fields_schema = make_schema_for_fields(crate_name, &variant.fields);
        subschemas.push(quote! {{
            let mut if_properties = #crate_name::internal__::schemars::Map::with_capacity(1);
            if_properties.insert(
                #crate_name::internal__::string(#tag_name),
                #crate_name::internal__::schemars::schema::Schema::Object(
                    #crate_name::internal__::schemars::schema::SchemaObject {
                        const_value: ::core::option::Option::Some(
                            #crate_name::internal__::json_value(#tag_value)
                        ),
                        .. ::core::default::Default::default()
                    }
                )
            );

            let mut then_properties = #crate_name::internal__::schemars::Map::with_capacity(1);
            let mut then_required = ::std::collections::BTreeSet::new();
            then_properties.insert(
                #crate_name::internal__::string(#tag_value),
                #fields_schema,
            );
            then_required.insert(#crate_name::internal__::string(#tag_value));


            #crate_name::internal__::schemars::schema::SubschemaValidation {
                if_schema: ::core::option::Option::Some(
                   ::std::boxed::Box::new(
                       #crate_name::internal__::schemars::schema::Schema::Object(
                           #crate_name::internal__::schemars::schema::SchemaObject {
                                object: ::core::option::Option::Some(
                                    std::boxed::Box::new(
                                        #crate_name::internal__::schemars::schema::ObjectValidation {
                                            properties: if_properties,
                                            .. ::core::default::Default::default()
                                        }
                                    )
                                ),
                                .. ::core::default::Default::default()
                           }
                        )
                    )
                ),
                then_schema: ::core::option::Option::Some(
                    ::std::boxed::Box::new(
                        #crate_name::internal__::schemars::schema::Schema::Object(
                            #crate_name::internal__::schemars::schema::SchemaObject {
                                object: ::core::option::Option::Some(
                                    ::std::boxed::Box::new(
                                        #crate_name::internal__::schemars::schema::ObjectValidation {
                                            properties: then_properties,
                                            required: then_required,
                                            .. ::core::default::Default::default()
                                        }
                                    )
                                ),
                                .. ::core::default::Default::default()
                            }
                        )
                    )
                ),
                .. ::core::default::Default::default()
            }
        }});
    }

    match subschemas.len() {
        0 => quote!(::core::option::Option::None),
        1 => {
            let subschema = subschemas.remove(0);
            quote!( ::core::option::Option::Some(::std::boxed::Box::new(#subschema)) )
        }
        count => quote! {
            ::core::option::Option::Some(::std::boxed::Box::new(
                #crate_name::internal__::schemars::schema::SubschemaValidation {
                    all_of: {
                        let mut all_of = ::std::vec::Vec::with_capacity(#count);
                        #(
                            all_of.push(
                                #crate_name::internal__::schemars::schema::Schema::Object(
                                    #crate_name::internal__::schemars::schema::SchemaObject {
                                        subschemas: ::core::option::Option::Some(::std::boxed::Box::new(#subschemas)),
                                        .. ::core::default::Default::default()
                                    }
                                )
                            );
                        )*
                        ::core::option::Option::Some(all_of)
                    },
                    .. ::core::default::Default::default()
                }
            ))
        },
    }
}

/// Generate code that returns a `schemars::schema::Schema` for the given [`syn::Fields`].
fn make_schema_for_fields(crate_name: &syn::Path, fields: &syn::Fields) -> TokenStream {
    match fields {
        syn::Fields::Unit => make_schema_for_unit_value(crate_name),
        syn::Fields::Named(fields) => make_schema_for_named_fields(crate_name, fields),
        syn::Fields::Unnamed(fields) => make_schema_for_unnamed_fields(crate_name, fields),
    }
}

/// Generate code that returns a `schemars::schema::Schema` for a unit value.
fn make_schema_for_unit_value(crate_name: &syn::Path) -> TokenStream {
    quote! {
        #crate_name::internal__::schemars::schema::Schema::Object(
            #crate_name::internal__::schemars::schema::SchemaObject {
                instance_type: ::core::option::Option::Some(
                    #crate_name::internal__::schemars::schema::SingleOrVec::Single(
                        ::std::boxed::Box::new(
                            #crate_name::internal__::schemars::schema::InstanceType::Null
                        )
                    )
                ),
                ..::core::default::Default::default()
            }
        )
    }
}

/// Generate code that returns a `schemars::schema::Schema` for struct fields.
fn make_schema_for_named_fields(crate_name: &syn::Path, fields: &syn::FieldsNamed) -> TokenStream {
    // Treat an empty struct variant as a unit variant.
    if fields.named.is_empty() {
        return make_schema_for_unit_value(crate_name);
    }

    let field_name_str: Vec<_> = fields
        .named
        .iter()
        .map(|x| x.ident.as_ref().unwrap().to_string())
        .collect();
    let field_type: Vec<_> = fields.named.iter().map(|x| &x.ty).collect();
    let field_count = fields.named.len();
    quote! {{
        #[allow(unused_mut)]
        let mut properties = #crate_name::internal__::schemars::Map::with_capacity(#field_count);
        #[allow(unused_mut)]
        let mut required = ::std::collections::BTreeSet::new();
        #(
            properties.insert(#crate_name::internal__::string(#field_name_str), generator.subschema_for::<#field_type>());
            if <#field_type as #crate_name::internal__::schemars::JsonSchema>::_internal__::schemars_private_is_option() == false {
                required.insert(#crate_name::internal__::string(#field_name_str));
            }
        )*
        #crate_name::internal__::schemars::schema::Schema::Object(
            #crate_name::internal__::schemars::schema::SchemaObject {
                instance_type: ::core::option::Option::Some(
                    #crate_name::internal__::schemars::schema::SingleOrVec::Single(
                        ::std::boxed::Box::new(
                            #crate_name::internal__::schemars::schema::InstanceType::Object
                        )
                    )
                ),
                object: ::core::option::Option::Some(::std::boxed::Box::new(
                    #crate_name::internal__::schemars::schema::ObjectValidation {
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
fn make_schema_for_unnamed_fields(
    crate_name: &syn::Path,
    fields: &syn::FieldsUnnamed,
) -> TokenStream {
    match &fields.unnamed.len() {
        // Treat an empty tuple variant as a unit variant.
        0 => make_schema_for_unit_value(crate_name),

        // Treat single-field tuple variants as the inner type.
        1 => {
            let field_type = &fields.unnamed[0].ty;
            quote! { generator.subschema_for::<#field_type>() }
        }

        // Treat the rest as fixed-size arrays.
        field_count => {
            let field_type = fields.unnamed.iter().map(|x| &x.ty);
            let item_bound = u32::try_from(*field_count)
                .map(|count| quote!(::core::option::Option::Some(#count)))
                .unwrap_or_else(|_| {
                    syn::Error::new_spanned(
                        fields,
                        format!(
                            "too many fields in variant: {} > {}",
                            fields.unnamed.len(),
                            u32::MAX
                        ),
                    )
                    .into_compile_error()
                });

            // TODO: Lift this restriction if the UI learns to show tuples in a nice way.
            let error = syn::Error::new_spanned(
                &fields.unnamed[1],
                concat!(
                    "#[derive(ConfigEnum)]: Tuple variants with multiple fields are not allowed.\n",
                    "\n",
                    "This is disallowed because the user interface can not properly handle tuples.\n",
                    "\n",
                    "Consider using a single field tuple variant that holds a struct, or a struct variant with named fields.",
                )
            ).into_compile_error();

            quote! {{
                #error
                let mut items = ::std::vec::Vec::with_capacity(#field_count);
                #(
                    items.push(generator.subschema_for::<#field_type>());
                )*
                #crate_name::internal__::schemars::schema::Schema::Object(
                    #crate_name::internal__::schemars::schema::SchemaObject {
                        instance_type: ::core::option::Option::Some(
                            #crate_name::internal__::schemars::schema::SingleOrVec::Single(
                                ::std::boxed::Box::new(
                                    #crate_name::internal__::schemars::schema::InstanceType::Array
                                )
                            )
                        ),
                        array: ::core::option::Option::Some(::std::boxed::Box::new(
                            #crate_name::internal__::schemars::schema::ArrayValidation {
                                items: ::core::option::Option::Some(#crate_name::internal__::schemars::schema::SingleOrVec::Vec(items)),
                                min_items: #item_bound,
                                max_items: #item_bound,
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

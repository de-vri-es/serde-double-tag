mod util;
mod input;
mod generate;

fn crate_name() -> syn::Path {
	let mut segments = syn::punctuated::Punctuated::new();
	segments.push(syn::PathSegment {
		ident: syn::Ident::new("serde_double_tag", proc_macro2::Span::call_site()),
		arguments: syn::PathArguments::None,
	});
	syn::Path {
		leading_colon: Some(syn::token::PathSep(proc_macro2::Span::call_site())),
		segments,
	}
}

struct Context {
	internal: syn::Path,
	serde: syn::Path,
	errors: Vec<syn::Error>,
}

impl Context {
	fn new(crate_name: syn::Path) -> Self {
		let mut internal = crate_name;
		internal.segments.push(syn::PathSegment {
			ident: syn::Ident::new("internal__", proc_macro2::Span::call_site()),
			arguments: syn::PathArguments::None,
		});
		let mut serde = internal.clone();
		serde.segments.push(syn::PathSegment {
			ident: syn::Ident::new("serde", proc_macro2::Span::call_site()),
			arguments: syn::PathArguments::None,
		});
		Self {
			internal,
			serde,
			errors: Vec::new(),
		}
	}


	fn error(&mut self, span: proc_macro2::Span, message: impl std::fmt::Display) {
		self.errors.push(syn::Error::new(span, format_args!("serde_double_tag: {message}")))
	}

	fn spanned_error<T: quote::ToTokens>(&mut self, object: &T, message: impl std::fmt::Display) {
		self.errors.push(syn::Error::new_spanned(object, format_args!("serde_double_tag: {message}")))
	}

	fn syn_error(&mut self, error: syn::Error) {
		self.error(error.span(), error)
	}

	fn collect_errors(self, mut tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
		for error in self.errors {
			tokens.extend(error.into_compile_error())
		}
		tokens
	}
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut context = Context::new(crate_name());
	let output = match input::Enum::parse2(&mut context, tokens.into()) {
		Ok(input) => generate::impl_deserialize_enum(&mut context, input),
		Err(()) => proc_macro2::TokenStream::new(),
	};
	context.collect_errors(output).into()
}

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut context = Context::new(crate_name());
	let output = match input::Enum::parse2(&mut context, tokens.into()) {
		Ok(input) => generate::impl_serialize_enum(&mut context, input),
		Err(()) => proc_macro2::TokenStream::new(),
	};
	context.collect_errors(output).into()
}

#[proc_macro_derive(JsonSchema, attributes(serde))]
pub fn derive_json_schema(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	todo!();
}

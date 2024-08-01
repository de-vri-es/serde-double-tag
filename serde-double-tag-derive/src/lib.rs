//! This crate contains proc-macros for the [`serde-double-tag`] crate.
//!
//! See the documentation of that crate for more information.

mod generate;
mod input;
mod util;

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
	#[cfg(feature = "schemars")]
	schemars: syn::Path,
	errors: Vec<syn::Error>,
}

impl Context {
	fn new(crate_name: syn::Path) -> Self {
		let internal = extend_path(&crate_name, "internal__");
		let serde = extend_path(&internal, "serde");
		#[cfg(feature = "schemars")]
		let schemars = extend_path(&internal, "schemars");
		Self {
			internal,
			serde,
			#[cfg(feature = "schemars")]
			schemars,
			errors: Vec::new(),
		}
	}

	fn error(&mut self, span: proc_macro2::Span, message: impl std::fmt::Display) {
		self.errors.push(syn::Error::new(span, format_args!("serde_double_tag: {message}")))
	}

	fn spanned_error<T: quote::ToTokens>(&mut self, object: &T, message: impl std::fmt::Display) {
		self.errors.push(syn::Error::new_spanned(
			object,
			format_args!("serde_double_tag: {message}"),
		))
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
#[cfg(feature = "schemars")]
pub fn derive_json_schema(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut context = Context::new(crate_name());
	let output = match input::Enum::parse2(&mut context, tokens.into()) {
		Ok(input) => generate::impl_json_schema(&mut context, input),
		Err(()) => proc_macro2::TokenStream::new(),
	};
	context.collect_errors(output).into()
}

fn extend_path(path: &syn::Path, segment: &str) -> syn::Path {
	let mut output = path.clone();
	output.segments.push(syn::PathSegment {
		ident: syn::Ident::new(segment, proc_macro2::Span::call_site()),
		arguments: Default::default(),
	});
	output
}

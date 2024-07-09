use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;

/// Convert a string to snake case.
///
/// The input must be in snake_case, kebab-case, camelCase or UpperCamerCase.
///
/// Each upper-case letter, dash or underscore will be seen as a word boundary.
/// Dashes themselves are also replaced by underscores.
///
/// As per the Rust conventions, this means that only the first letter of an acronym should be upper case:
/// `"parseHtml"`, not `"parseHTML"`.
pub fn to_snake_case(data: &str) -> String {
	let mut output = String::with_capacity(data.len() + data.len() / 5);
	for c in data.chars() {
		if c == '-' {
			output.push('_')
		} else if c.is_uppercase() {
			if !output.is_empty() {
				output.push('_');
			}
			output.extend(c.to_lowercase());
		} else {
			output.push(c);
		}
	}
	output
}

pub fn parse_enum_item(tokens: TokenStream) -> Result<syn::ItemEnum, syn::Error> {
	let item = syn::parse2(tokens)?;
	let error_span = match item {
		syn::Item::Enum(x) => return Ok(x),
		syn::Item::Const(x) => x.const_token.span,
		syn::Item::ExternCrate(x) => x.extern_token.span,
		syn::Item::Fn(x) => x.sig.fn_token.span,
		syn::Item::ForeignMod(x) => x.abi.extern_token.span,
		syn::Item::Impl(x) => x.impl_token.span,
		syn::Item::Macro(x) => x.mac.path.span(),
		syn::Item::Mod(x) => x.mod_token.span,
		syn::Item::Static(x) => x.static_token.span,
		syn::Item::Struct(x) => x.struct_token.span,
		syn::Item::Trait(x) => x.trait_token.span,
		syn::Item::TraitAlias(x) => x.trait_token.span,
		syn::Item::Type(x) => x.type_token.span,
		syn::Item::Union(x) => x.union_token.span,
		syn::Item::Use(x) => x.use_token.span,
		syn::Item::Verbatim(x) => x.span(),
		_ => item.span(),
	};
	Err(syn::Error::new(error_span, "serde_exjacent: expected an enum"))
}

pub fn add_lifetime(generics: &syn::Generics, hint: &str) -> (syn::Generics, syn::Lifetime, Option<TokenStream>) {
	let lifetime = match allocate_unused_lifetime(generics, hint) {
		Ok(x) => x,
		Err(e) => return (generics.clone(), syn::Lifetime::new("'a", Span::call_site()), Some(e.into_compile_error())),
	};
	let lifetime = syn::Lifetime::new(&format!("'{lifetime}"), Span::call_site());
	let param = syn::LifetimeParam::new(lifetime.clone());

	let mut generics = generics.clone();
	generics.params.push(syn::GenericParam::Lifetime(param));
	(generics, lifetime, None)
}

fn allocate_unused_lifetime(generics: &syn::Generics, hint: &str) -> Result<String, syn::Error> {
	if !has_lifetime(generics, hint) {
		return Ok(hint.into());
	}
	for lifetime in 'a'..='z' {
		let lifetime = lifetime.to_string();
		if !has_lifetime(generics, &lifetime) {
			return Ok(lifetime);
		}
	}
	for i in 0.. {
		let lifetime = format!("_{i}");
		if !has_lifetime(generics, &lifetime) {
			return Ok(lifetime);
		}
	}

	Err(syn::Error::new_spanned(generics, "Failed to allocate unused lifetime"))
}

fn has_lifetime(generics: &syn::Generics, lifetime: &str) -> bool {
	for param in &generics.params {
		if let syn::GenericParam::Lifetime(param) = param {
			if param.lifetime.ident == lifetime {
				return true;
			}
		}
	}
	false
}

/// Check if a type uses any of the given generic arguments.
pub fn type_uses_generic(ty: &syn::Type, generics: &syn::Generics) -> bool {
	struct Visit<'a> {
		generics: &'a syn::Generics,
		found: bool,
	}
	impl syn::visit::Visit<'_> for Visit<'_> {
		fn visit_path(&mut self, item: &syn::Path) {
			for param in &self.generics.params {
				if let syn::GenericParam::Type(param) = param {
					if item.is_ident(&param.ident) {
						self.found = true;
						return;
					}
				}
			}
			syn::visit::visit_path(self, item)
		}
		fn visit_lifetime(&mut self, item: &syn::Lifetime) {
			for param in &self.generics.params {
				if let syn::GenericParam::Lifetime(param) = param {
					if item.ident == param.lifetime.ident {
						self.found = true;
						return;
					}
				}
			}
			syn::visit::visit_lifetime(self, item)

		}
		fn visit_const_param(&mut self, item: &syn::ConstParam) {
			for param in &self.generics.params {
				if let syn::GenericParam::Const(param) = param {
					if item.ident == param.ident {
						self.found = true;
						return;
					}
				}
			}
			syn::visit::visit_const_param(self, item)
		}
	}

	let mut visitor = Visit {
		generics,
		found: false,
	};

	syn::visit::Visit::visit_type(&mut visitor, ty);
	visitor.found
}

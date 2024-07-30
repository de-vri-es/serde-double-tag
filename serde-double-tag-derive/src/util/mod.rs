use proc_macro2::Span;
use syn::spanned::Spanned;

mod rename_rule;
pub use rename_rule::RenameRule;

mod prune_generics;
pub use prune_generics::prune_generics;

use crate::Context;

pub fn item_token_span(item: &syn::Item) -> Span {
	match item {
		syn::Item::Enum(x) => x.enum_token.span,
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
	}
}

pub fn add_lifetime(context: &mut Context, generics: &syn::Generics, hint: &str) -> (syn::Generics, syn::Lifetime) {
	let lifetime = match allocate_unused_lifetime(generics, hint) {
		Ok(x) => x,
		Err(e) => {
			context.syn_error(e);
			return (generics.clone(), syn::Lifetime::new(hint, Span::call_site()));
		}
	};
	let lifetime = syn::Lifetime::new(&format!("'{lifetime}"), Span::call_site());
	let param = syn::LifetimeParam::new(lifetime.clone());

	let mut generics = generics.clone();
	generics.params.push(syn::GenericParam::Lifetime(param));
	(generics, lifetime)
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
		fn visit_attribute(&mut self, _i: &'_ syn::Attribute) {
			// Ignore everything in attributes.
		}

		fn visit_path(&mut self, path: &syn::Path) {
			// Only visit single-identifier paths and path arguments.
			// Do not visit individual identifiers of path segments.
			if let Some(ident) = path.get_ident() {
				self.visit_ident(ident);
			} else {
				for segment in &path.segments {
					self.visit_path_arguments(&segment.arguments);
				}
			}
		}

		fn visit_ident(&mut self, ident: &proc_macro2::Ident) {
			for param in &self.generics.params {
				let param = match param {
					syn::GenericParam::Lifetime(_) => continue,
					syn::GenericParam::Type(param) => &param.ident,
					syn::GenericParam::Const(param) => &param.ident,
				};
				if ident == param {
					self.found = true;
					return;
				}
			}
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

pub fn strip_type_wrappers(ty: &syn::Type) -> &syn::Type {
	let mut ty = ty;
	loop {
		match ty {
			syn::Type::Array(x) => ty = &x.elem,
			syn::Type::Group(x) => ty = &x.elem,
			syn::Type::Paren(x) => ty = &x.elem,
			syn::Type::Reference(x) => ty = &x.elem,
			syn::Type::Slice(x) => ty = &x.elem,
			other => return other,
		}
	}
}

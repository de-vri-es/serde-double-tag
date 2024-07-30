use std::collections::BTreeSet;

/// Create a new [`syn::Generics`] with all parameters removed that are not used by any of the given types.
///
/// Also removes where predicates that reference removed parameters.
pub fn prune_generics<'a>(generics: &syn::Generics, types: impl Iterator<Item = &'a syn::Type>) -> syn::Generics {
	// Analyze all the types to collect all their identifiers and lifetimes.
	let mut used_idents = UsedGenerics::new();
	for ty in types {
		used_idents.analyze_type(ty);
	}

	// Remove unused parameters and where clauses that reference removed parameters.
	let params = used_idents.prune_generic_params(&generics.params);
	let where_clause = generics.where_clause.as_ref().map(|x| used_idents.prune_where_clause(x));
	syn::Generics {
		lt_token: generics.lt_token,
		params,
		gt_token: generics.gt_token,
		where_clause,
	}
}

struct UsedGenerics<'a> {
	// Identifiers used in type definitions (excluding any path components).
	used_idents: BTreeSet<&'a syn::Ident>,

	// Lifetimes used in type definitions.
	used_lifetimes: BTreeSet<&'a syn::Lifetime>,

	// Removed generic type and const parameters.
	removed_idents: BTreeSet<&'a syn::Ident>,

	// Removed generic lifetime parameters.
	removed_lifetimes: BTreeSet<&'a syn::Lifetime>,
}

impl<'a> UsedGenerics<'a> {
	/// Create a new empty list of used/removed generic parameters.
	fn new() -> Self {
		Self {
			used_idents: BTreeSet::new(),
			used_lifetimes: BTreeSet::new(),
			removed_idents: BTreeSet::new(),
			removed_lifetimes: BTreeSet::new(),
		}
	}

	/// Analyze a type, adding all identifiers and lifetimes to [`self.used_idents`] and [`self.used_lifetimes`].
	fn analyze_type(&mut self, ty: &'a syn::Type) {
		struct Visit<'ast, 'b> {
			used: &'b mut UsedGenerics<'ast>,
		}
		impl<'ast, 'b> syn::visit::Visit<'ast> for Visit<'ast, 'b> {
			fn visit_attribute(&mut self, _i: &'ast syn::Attribute) {
				// Ignore attributes as we have no clue what they mean.
			}

			fn visit_path(&mut self, path: &'ast syn::Path) {
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

			fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
				self.used.used_idents.insert(i);
			}

			fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
				self.used.used_lifetimes.insert(i);
			}
		}

		let mut visit = Visit {
			used: self,
		};

		syn::visit::Visit::visit_type(&mut visit, ty);
	}

	/// Create a filtered parameter list where all parameters are removed that are not in [`self.used_idents`] or [`self.used_lifetimes`].
	///
	/// Adds all removed parameters to [`self.removed_idents`] or [`self.removed_lifetimes`].
	fn prune_generic_params(&mut self, params: &'a syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>) -> syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma> {
		params
			.pairs()
			.filter_map(|elem| {
				if !self.is_used(elem.value()) {
					self.mark_removed(elem.value());
					return None;
				}
				let (value, punct) = elem.into_tuple();
				Some(syn::punctuated::Pair::new(value.clone(), punct.cloned()))
			})
		.collect()
	}

	/// Create a filtered where clause where all predicates are removed that used a parameter from [`self.removed_idents`] or [`self.removed_lifetimes`].
	fn prune_where_clause(&self, where_clause: &syn::WhereClause) -> syn::WhereClause {
		syn::WhereClause {
			where_token: where_clause.where_token,
			predicates: self.prune_where_predicates(&where_clause.predicates)
		}
	}

	/// Create a filtered predicate list where all predicates are removed that used a parameter from [`self.removed_idents`] or [`self.removed_lifetimes`].
	fn prune_where_predicates(&self, predicates: &syn::punctuated::Punctuated<syn::WherePredicate, syn::token::Comma>) -> syn::punctuated::Punctuated<syn::WherePredicate, syn::token::Comma> {
		predicates
			.pairs()
			.filter_map(|elem| {
				if self.predicate_uses_removed_ident(elem.value()) {
					return None;
				}
				let (value, punct) = elem.into_tuple();
				Some(syn::punctuated::Pair::new(value.clone(), punct.cloned()))
			})
		.collect()
	}

	/// Check if a generic parameter is in [`self.used_idents`] or [`self.used_lifetimes`].
	fn is_used(&self, generic_param: &syn::GenericParam) -> bool {
		match generic_param {
			syn::GenericParam::Lifetime(param) => self.used_lifetimes.contains(&param.lifetime),
			syn::GenericParam::Type(param) => self.used_idents.contains(&param.ident),
			syn::GenericParam::Const(param) => self.used_idents.contains(&param.ident),
		}
	}

	/// Check if a where predicate uses a parameter from [`self.removed_idents`] or [`self.removed_lifetimes`].
	fn predicate_uses_removed_ident(&self, predicate: &syn::WherePredicate) -> bool {
		struct Visit<'a> {
			used: &'a UsedGenerics<'a>,
			found: bool,
		}

		let mut visit = Visit {
			used: self,
			found: false,
		};

		impl<'ast> syn::visit::Visit<'ast> for Visit<'_> {
			fn visit_attribute(&mut self, _i: &'ast syn::Attribute) {
				// Ignore attributes as we have no clue what they mean.
			}

			fn visit_path(&mut self, path: &'ast syn::Path) {
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

			fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {
				if self.used.removed_lifetimes.contains(i) {
					self.found = true;
				}
			}

			fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
				if !self.used.removed_idents.contains(i) {
					self.found = true;
				}
			}
		}

		syn::visit::Visit::visit_where_predicate(&mut visit, predicate);
		visit.found
	}

	/// Mark a generic parameter as removed.
	///
	/// This adds the parameter to [`self.removed_idents`] or [`self.removed_lifetimes`].
	pub fn mark_removed(&mut self, parameter: &'a syn::GenericParam) {
		match parameter {
			syn::GenericParam::Lifetime(param) => self.removed_lifetimes.insert(&param.lifetime),
			syn::GenericParam::Type(param) => self.removed_idents.insert(&param.ident),
			syn::GenericParam::Const(param) => self.removed_idents.insert(&param.ident),
		};
	}
}

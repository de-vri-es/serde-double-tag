use proc_macro2::{Span, TokenStream};

use crate::Context;

pub struct Attr<Value> {
	pub ident: syn::Ident,
	pub value: Value,
}

pub struct WithSpan<T> {
	pub span: Span,
	pub value: T,
}

impl<T> WithSpan<T> {
	pub fn new(span: Span, value: T) -> Self {
		Self { span, value }
	}
}

#[allow(clippy::enum_variant_names)]
pub enum RenameAll {
	LowerCase,
	UpperCase,
	PascalCase,
	CamelCase,
	SnakeCase,
	ScreamingSnakeCase,
	KebabCase,
	ScreamingKebabCase,
}

impl std::str::FromStr for RenameAll {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"lowercase" => Ok(Self::LowerCase),
			"UPPERCASE" => Ok(Self::UpperCase),
			"PascalCase" => Ok(Self::PascalCase),
			"camelCase" => Ok(Self::CamelCase),
			"snake_case" => Ok(Self::SnakeCase),
			"SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
			"kebab-case" => Ok(Self::KebabCase),
			"SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
			_ => Err(())
		}
	}
}

impl syn::parse::Parse for WithSpan<RenameAll> {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let input: syn::LitStr = input.parse()?;
		let value = input.value()
			.parse()
			.map_err(|()| syn::Error::new_spanned(
				&input,
				"invalid value: expected one of: \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"SCREAMING_SNAKE_CASE\", \"kebab-case\" or \"SCREAMING-KEBAB-CASE\"",
			))?;
		Ok(WithSpan::new(input.span(), value))
	}
}

#[derive(Default)]
pub struct EnumAttributes {
	pub rename: Option<Attr<syn::LitStr>>,
	pub rename_all: Option<Attr<WithSpan<RenameAll>>>,
	pub rename_all_fields: Option<Attr<WithSpan<RenameAll>>>,
	pub deny_unknown_fields: Option<syn::Ident>,
	pub tag: Option<Attr<syn::LitStr>>,
	pub bound: Option<Attr<syn::WhereClause>>,
	pub expecting: Option<Attr<syn::LitStr>>,
}

#[derive(Default)]
pub struct VariantAttributes {
	pub rename: Option<Attr<syn::LitStr>>,
	pub rename_all: Option<Attr<WithSpan<RenameAll>>>,
	pub alias: Vec<Attr<syn::LitStr>>,
	pub bound: Option<Attr<syn::WhereClause>>,
}

impl EnumAttributes {
	pub fn from_syn(context: &mut Context, input: Vec<syn::Attribute>) -> Self {
		let mut output = Self::default();
		for attr in input {
			output.parse_one(context, attr)
		}
		output
	}

	fn parse_one(&mut self, context: &mut Context, attr: syn::Attribute) {
		if attr.path().is_ident("serde") {
			let result = attr.parse_nested_meta(|nested| {
				let ident = match nested.path.get_ident() {
					Some(ident) => ident,
					None => {
						context.spanned_error(&nested.path, "unsupported attribute");
						discard_arg_input(nested.input);
						return Ok(());
					}
				};
				if ident == "rename" {
					set_attr(context, &mut self.rename, ident, nested.input);
				} else if ident == "rename_all" {
					set_attr(context, &mut self.rename_all, ident, nested.input);
				} else if ident == "rename_all_fields" {
					set_attr(context, &mut self.rename_all_fields, ident, nested.input);
				} else if ident == "deny_unknown_fields" {
					set_marker(context, &mut self.deny_unknown_fields, ident, nested.input);
				} else if ident == "tag" {
					set_attr(context, &mut self.tag, ident, nested.input);
				} else if ident == "bound" {
					set_attr(context, &mut self.bound, ident, nested.input);
				} else if ident == "expecting" {
					set_attr(context, &mut self.expecting, ident, nested.input);
				} else {
					context.spanned_error(&nested.path, "unsupported attribute");
					discard_arg_input(nested.input);
				}
				Ok(())
			});
			if let Err(e) = result {
				context.syn_error(e);
			}
		}
	}
}

impl VariantAttributes {
	pub fn from_syn(context: &mut Context, input: Vec<syn::Attribute>) -> Self {
		let mut output = Self::default();
		for attr in input {
			output.parse_one(context, attr)
		}
		output
	}

	fn parse_one(&mut self, context: &mut Context, attr: syn::Attribute) {
		if attr.path().is_ident("serde") {
			let result = attr.parse_nested_meta(|nested| {
				let ident = match nested.path.get_ident() {
					Some(ident) => ident,
					None => {
						context.spanned_error(&nested.path, "unsupported attribute");
						discard_arg_input(nested.input);
						return Ok(());
					}
				};
				if ident == "rename" {
					set_attr(context, &mut self.rename, ident, nested.input);
				} else if ident == "rename_all" {
					set_attr(context, &mut self.rename_all, ident, nested.input);
				} else if ident == "bound" {
					set_attr(context, &mut self.bound, ident, nested.input);
				} else if ident == "alias" {
					append_attr(context, &mut self.alias, ident, nested.input);
				} else {
					context.spanned_error(&nested.path, "unsupported attribute");
					discard_arg_input(nested.input);
				}
				Ok(())
			});
			if let Err(e) = result {
				context.syn_error(e);
			}
		}
	}
}

fn set_attr<T: syn::parse::Parse>(context: &mut Context, output: &mut Option<Attr<T>>, ident: &syn::Ident, value: &syn::parse::ParseBuffer) {
	let _: syn::token::Eq = match value.parse() {
		Ok(x) => x,
		Err(_) => {
			context.spanned_error(ident, "attribute requires a value");
			return;
		}
	};
	let value: T = match value.parse() {
		Ok(x) => x,
		Err(e) => {
			context.syn_error(e);
			return;
		}
	};

	if output.is_some() {
		context.spanned_error(ident, format_args!("attribute `{ident}` already set before"));
	} else {
		*output = Some(Attr {
			ident: ident.clone(),
			value,
		})
	}
}

fn append_attr<T: syn::parse::Parse>(context: &mut Context, output: &mut Vec<Attr<T>>, ident: &syn::Ident, value: &syn::parse::ParseBuffer) {
	let _: syn::token::Eq = match value.parse() {
		Ok(x) => x,
		Err(_) => {
			context.spanned_error(ident, "attribute requires a value");
			return;
		}
	};
	let value: T = match value.parse() {
		Ok(x) => x,
		Err(e) => {
			context.syn_error(e);
			return;
		}
	};

	output.push(Attr {
		ident: ident.clone(),
		value,
	})
}

fn set_marker(context: &mut Context, output: &mut Option<syn::Ident>, ident: &syn::Ident, value: &syn::parse::ParseBuffer) {
	if !value.is_empty() && !value.peek(syn::token::Comma) {
		context.syn_error(value.error("attribute does not take a value"));
	}
	if output.is_none() {
		*output = Some(ident.clone());
	}
}

fn discard_arg_input(input: &syn::parse::ParseBuffer) {
	input.step(|cursor| {
		let mut rest = *cursor;
		while let Some((tree, next)) = rest.token_tree() {
			match tree {
				proc_macro2::TokenTree::Punct(x) if x.as_char() == ',' => return Ok(((), rest)),
				_ => rest = next,
			}
		}
		Ok(((), rest))
	}).unwrap();
}

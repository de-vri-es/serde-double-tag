use proc_macro2::Span;

mod args;
pub use args::{KeyValueArg, KeywordArg};

use crate::Context;

#[derive(Clone, Default)]
pub struct EnumAttributes {
	pub rename: Option<KeyValueArg<keyword::rename, syn::LitStr>>,
	pub rename_all: Option<KeyValueArg<keyword::rename_all, CaseConvert>>,
	pub rename_all_fields: Option<KeyValueArg<keyword::rename_all_fields, CaseConvert>>,
	pub deny_unknown_fields: Option<KeywordArg<keyword::deny_unknown_fields>>,
	pub tag: Option<KeyValueArg<keyword::tag, syn::LitStr>>,
	pub bound: Option<KeyValueArg<keyword::bound, syn::WhereClause>>,
	pub expecting: Option<KeyValueArg<keyword::expecting, syn::LitStr>>,
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
		if let Some(mut parser) = args::AttrParser::new(context, attr, "serde") {
			parser.parse(context, [
				&mut self.rename,
				&mut self.rename_all,
				&mut self.rename_all_fields,
				&mut self.deny_unknown_fields,
				&mut self.tag,
				&mut self.bound,
				&mut self.expecting,
			]);
		}
	}
}

impl quote::ToTokens for EnumAttributes {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			rename,
			rename_all,
			rename_all_fields,
			deny_unknown_fields,
			tag,
			bound,
			expecting,
		} = self;
		rename.to_tokens(tokens);
		rename_all.to_tokens(tokens);
		rename_all_fields.to_tokens(tokens);
		deny_unknown_fields.to_tokens(tokens);
		tag.to_tokens(tokens);
		bound.to_tokens(tokens);
		expecting.to_tokens(tokens);
	}
}

#[derive(Clone, Default)]
pub struct VariantAttributes {
	pub rename: Option<KeyValueArg<keyword::rename, syn::LitStr>>,
	pub rename_all: Option<KeyValueArg<keyword::rename_all, CaseConvert>>,
	pub alias: Vec<KeyValueArg<keyword::alias, syn::LitStr>>,
	pub bound: Option<KeyValueArg<keyword::bound, syn::WhereClause>>,
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
		if let Some(mut parser) = args::AttrParser::new(context, attr, "serde") {
			parser.parse(context, [
				&mut self.rename,
				&mut self.rename_all,
				&mut self.bound,
				&mut self.alias,
			]);
		}
	}
}

impl quote::ToTokens for VariantAttributes {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			rename,
			rename_all,
			bound,
			alias,
		} = self;
		rename.to_tokens(tokens);
		rename_all.to_tokens(tokens);
		bound.to_tokens(tokens);
		for alias in alias {
			alias.to_tokens(tokens);
		}
	}
}

pub mod keyword {
	syn::custom_keyword!(rename);
	syn::custom_keyword!(rename_all);
	syn::custom_keyword!(rename_all_fields);
	syn::custom_keyword!(deny_unknown_fields);
	syn::custom_keyword!(tag);
	syn::custom_keyword!(bound);
	syn::custom_keyword!(expecting);
	syn::custom_keyword!(alias);
}

#[derive(Clone, Copy)]
pub enum CaseConvert {
	Lower(Span),
	Upper(Span),
	Pascal(Span),
	Camel(Span),
	Snake(Span),
	ScreamingSnake(Span),
	Kebab(Span),
	ScreamingKebab(Span),
}

const LOWER_CASE: &str = "lowercase";
const UPPER_CASE: &str = "UPPERCASE";
const PASCAL_CASE: &str = "PascalCase";
const CAMEL_CASE: &str = "camelCase";
const SNAKE_CASE: &str = "snake_case";
const SCREAMING_SNAKE_CASE: &str = "SCREAMING_SNAKE_CASE";
const KEBAB_CASE: &str = "kebab-case";
const SCREAMING_KEBAB_CASE: &str = "SCREAMING-KEBAB-CASE";

impl syn::parse::Parse for CaseConvert {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let input: syn::LitStr = input.parse()?;
		match input.value().as_str() {
			self::LOWER_CASE => Ok(Self::Lower(input.span())),
			self::UPPER_CASE => Ok(Self::Upper(input.span())),
			self::PASCAL_CASE => Ok(Self::Pascal(input.span())),
			self::CAMEL_CASE => Ok(Self::Camel(input.span())),
			self::SNAKE_CASE => Ok(Self::Snake(input.span())),
			self::SCREAMING_SNAKE_CASE => Ok(Self::ScreamingSnake(input.span())),
			self::KEBAB_CASE => Ok(Self::Kebab(input.span())),
			self::SCREAMING_KEBAB_CASE => Ok(Self::ScreamingKebab(input.span())),
			_ => Err(syn::Error::new_spanned(
				&input,
				"invalid value: expected one of: \"lowercase\", \"UPPERCASE\", \"PascalCase\", \"camelCase\", \"snake_case\", \"SCREAMING_SNAKE_CASE\", \"kebab-case\" or \"SCREAMING-KEBAB-CASE\"",
			))
		}
	}
}
impl quote::ToTokens for CaseConvert {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let (value, span) = match *self {
			Self::Lower(span) => (LOWER_CASE, span),
			Self::Upper(span) => (UPPER_CASE, span),
			Self::Pascal(span) => (PASCAL_CASE, span),
			Self::Camel(span) => (CAMEL_CASE, span),
			Self::Snake(span) => (SNAKE_CASE, span),
			Self::ScreamingSnake(span) => (SCREAMING_SNAKE_CASE, span),
			Self::Kebab(span) => (KEBAB_CASE, span),
			Self::ScreamingKebab(span) => (SCREAMING_KEBAB_CASE, span),
		};

		let mut literal = proc_macro2::Literal::string(value);
		literal.set_span(span);
		tokens.extend([proc_macro2::TokenTree::Literal(literal)]);
	}

}

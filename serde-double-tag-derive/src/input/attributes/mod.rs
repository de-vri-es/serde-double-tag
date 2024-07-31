use proc_macro2::Span;

mod args;
pub use args::{KeyValueArg, KeywordArg};

use crate::Context;

#[derive(Clone, Default)]
pub struct EnumAttributes {
	pub rename: Option<KeyValueArg<keyword::rename, syn::LitStr>>,
	pub rename_all: Option<KeyValueArg<keyword::rename_all, RenameRule>>,
	pub rename_all_fields: Option<KeyValueArg<keyword::rename_all_fields, RenameRule>>,
	pub deny_unknown_fields: Option<KeywordArg<keyword::deny_unknown_fields>>,
	pub tag: Option<KeyValueArg<keyword::tag, syn::LitStr>>,
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
		} = self;
		rename.to_tokens(tokens);
		rename_all.to_tokens(tokens);
		rename_all_fields.to_tokens(tokens);
		deny_unknown_fields.to_tokens(tokens);
		tag.to_tokens(tokens);
	}
}

#[derive(Clone, Default)]
pub struct VariantAttributes {
	pub rename: Option<KeyValueArg<keyword::rename, syn::LitStr>>,
	pub rename_all: Option<KeyValueArg<keyword::rename_all, RenameRule>>,
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
			]);
		}
	}
}

impl quote::ToTokens for VariantAttributes {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			rename,
			rename_all,
		} = self;
		rename.to_tokens(tokens);
		rename_all.to_tokens(tokens);
	}
}

#[derive(Clone, Default)]
pub struct FieldAttributes {
	pub rename: Option<KeyValueArg<keyword::rename, syn::LitStr>>,
}

impl FieldAttributes {
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
			]);
		}
	}
}

impl quote::ToTokens for FieldAttributes {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			rename,
		} = self;
		rename.to_tokens(tokens);
	}
}

pub mod keyword {
	syn::custom_keyword!(rename);
	syn::custom_keyword!(rename_all);
	syn::custom_keyword!(rename_all_fields);
	syn::custom_keyword!(deny_unknown_fields);
	syn::custom_keyword!(tag);
}

#[derive(Clone, Copy)]
pub struct RenameRule {
	pub rule: crate::util::RenameRule,
	pub span: Span,
}

impl syn::parse::Parse for RenameRule {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let literal: syn::LitStr = input.parse()?;
		let rule = crate::util::RenameRule::from_str(&literal.value())
			.map_err(|e| syn::Error::new_spanned(&literal, e))?;
		Ok(Self {
			rule,
			span: literal.span(),
		})
	}
}
impl quote::ToTokens for RenameRule {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let value = self.rule.as_str();
		let mut literal = proc_macro2::Literal::string(value);
		literal.set_span(self.span);
		tokens.extend([proc_macro2::TokenTree::Literal(literal)]);
	}
}

use proc_macro2::TokenStream;

use crate::Context;

pub struct AttrParser {
	pound: syn::token::Pound,
	bracket: syn::token::Bracket,
	path: syn::Path,
	delimiter: syn::MacroDelimiter,
	arguments: proc_macro2::TokenStream,
}

impl AttrParser {
	pub fn new(context: &mut Context, attribute: syn::Attribute, ident: &str) -> Option<Self> {
		if !attribute.path().is_ident(ident) {
			return None;
		}
		let meta = match attribute.meta {
			syn::Meta::List(x) => x,
			_ => {
				context.spanned_error(attribute.path(), format_args!("expected #[{ident}(...)]"));
				return None;
			}
		};
		Some(Self {
			pound: attribute.pound_token,
			bracket: attribute.bracket_token,
			path: meta.path,
			delimiter: meta.delimiter,
			arguments: meta.tokens,
		})
	}

	pub fn parse<const N: usize>(&mut self, context: &mut Context, args: [&mut dyn AttributeArg; N]) {
		let mut args = args;
		'outer: while !self.arguments.is_empty() {
			for arg in &mut args {
				if arg.try_parse(context, self) {
					continue 'outer;
				}
			}
			self.parse_unrecognized_argument(context)
		}
	}

	fn parse_unrecognized_argument(&mut self, context: &mut Context) {
		let arguments = std::mem::take(&mut self.arguments);
		match syn::parse::Parser::parse2(parse_unregcognized_argument, arguments) {
			Err(e) => {
				context.syn_error(e);
			}
			Ok((ident, rest)) => {
				self.arguments = rest;
				context.error(ident.span(), "unrecognized attribute argument: {ident}");
			}
		}
	}
}

#[derive(Clone)]
pub struct KeywordArg<K> {
	pub pound: syn::token::Pound,
	pub bracket: syn::token::Bracket,
	pub attr_path: syn::Path,
	pub delimiter: syn::MacroDelimiter,
	pub keyword: K,
}

impl<K: quote::ToTokens> quote::ToTokens for KeywordArg<K> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.pound.to_tokens(tokens);
		self.bracket.surround(tokens, |tokens| {
			self.attr_path.to_tokens(tokens);
			macro_delim_surround(&self.delimiter, tokens, |tokens| {
				self.keyword.to_tokens(tokens);
			})
		})
	}
}

#[derive(Clone)]
pub struct KeyValueArg<K, V> {
	pub pound: syn::token::Pound,
	pub bracket: syn::token::Bracket,
	pub attr_path: syn::Path,
	pub delimiter: syn::MacroDelimiter,
	pub key: K,
	pub eq: syn::token::Eq,
	pub value: V,
}

impl<K, V> KeyValueArg<K, V> {
	pub fn new_call_site(attr_path: &str, value: V) -> Self
	where
		K: Default,
	{
		Self {
			pound: Default::default(),
			bracket: Default::default(),
			attr_path: syn::parse_str(attr_path).unwrap(),
			delimiter: syn::MacroDelimiter::Paren(Default::default()),
			key: K::default(),
			eq: Default::default(),
			value,
		}
	}

	pub fn map_key<F, NewKey>(self, fun: F) -> KeyValueArg<NewKey, V>
	where
		F: FnOnce(K) -> NewKey,
	{
		KeyValueArg {
			pound: self.pound,
			bracket: self.bracket,
			attr_path: self.attr_path,
			delimiter: self.delimiter,
			key: fun(self.key),
			eq: self.eq,
			value: self.value,
		}
	}
}

impl<K: quote::ToTokens, V: quote::ToTokens> quote::ToTokens for KeyValueArg<K, V> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.pound.to_tokens(tokens);
		self.bracket.surround(tokens, |tokens| {
			self.attr_path.to_tokens(tokens);
			macro_delim_surround(&self.delimiter, tokens, |tokens| {
				self.key.to_tokens(tokens);
				self.eq.to_tokens(tokens);
				self.value.to_tokens(tokens);
			})
		})
	}
}

pub trait AttributeArg {
	fn try_parse(&mut self, context: &mut Context, parser: &mut AttrParser) -> bool;
}

impl<K> AttributeArg for Option<KeywordArg<K>>
where
	K: syn::parse::Parse + quote::ToTokens,
{
	fn try_parse(&mut self, context: &mut Context, parser: &mut AttrParser) -> bool {
		let arguments = std::mem::take(&mut parser.arguments);
		let keyword = match syn::parse::Parser::parse2(parse_keyword_attr_arg::<K>, arguments) {
			Ok((keyword, rest)) => {
				parser.arguments = rest;
				match keyword {
					Some(keyword) => keyword,
					None => return false,
				}
			},
			Err(e) => {
				context.syn_error(e);
				return false;
			}
		};

		if self.is_none() {
			*self = Some(KeywordArg {
				pound: parser.pound,
				bracket: parser.bracket,
				attr_path: parser.path.clone(),
				delimiter: parser.delimiter.clone(),
				keyword,
			});
		}
		true
	}
}

impl<K, V> AttributeArg for Option<KeyValueArg<K, V>>
where
	K: syn::parse::Parse + quote::ToTokens,
	V: syn::parse::Parse,
{
	fn try_parse(&mut self, context: &mut Context, parser: &mut AttrParser) -> bool {
		let arguments = std::mem::take(&mut parser.arguments);
		let (key, eq, value) = match syn::parse::Parser::parse2(parse_key_value_attr_arg::<K, V>, arguments) {
			Ok((arg, rest)) => {
				parser.arguments = rest;
				match arg {
					Some(arg) => arg,
					None => return false,
				}
			},
			Err(e) => {
				context.syn_error(e);
				return false;
			}
		};

		if self.is_some() {
			context.spanned_error(&key, format_args!("attribute `{}` already set before", key.to_token_stream()));
			return true;
		}

		*self = Some(KeyValueArg {
			pound: parser.pound,
			bracket: parser.bracket,
			attr_path: parser.path.clone(),
			delimiter: parser.delimiter.clone(),
			key,
			eq,
			value,
		});
		true
	}
}

impl<K, V> AttributeArg for Vec<KeyValueArg<K, V>>
where
	K: syn::parse::Parse + quote::ToTokens,
	V: syn::parse::Parse,
{
	fn try_parse(&mut self, context: &mut Context, parser: &mut AttrParser) -> bool {
		let arguments = std::mem::take(&mut parser.arguments);
		let (key, eq, value) = match syn::parse::Parser::parse2(parse_key_value_attr_arg::<K, V>, arguments) {
			Ok((arg, rest)) => {
				parser.arguments = rest;
				match arg {
					Some(arg) => arg,
					None => return false,
				}
			},
			Err(e) => {
				context.syn_error(e);
				return false;
			}
		};

		self.push(KeyValueArg {
			pound: parser.pound,
			bracket: parser.bracket,
			attr_path: parser.path.clone(),
			delimiter: parser.delimiter.clone(),
			key,
			eq,
			value,
		});
		true
	}
}

fn parse_keyword_attr_arg<K>(input: &syn::parse::ParseBuffer) -> syn::Result<(Option<K>, TokenStream)>
where
	K: syn::parse::Parse,
{
	// Input does NOT begin with the expected keyword.
	if input.fork().parse::<K>().is_err() {
		let rest = input.parse()?;
		return Ok((None, rest));
	}

	// Input DOES begin with the expected keyword.
	let keyword = input.parse()?;

	// If the input is not empty now, we need a comma to separate the next argument.
	if !input.is_empty() {
		let _: syn::token::Comma = input.parse()?;
	}

	// Collect the remainder into a TokenStream again.
	let rest = input.parse()?;

	Ok((Some(keyword), rest))
}

#[allow(clippy::type_complexity)]
fn parse_key_value_attr_arg<K, V>(input: &syn::parse::ParseBuffer) -> syn::Result<(Option<(K, syn::token::Eq, V)>, TokenStream)>
where
	K: syn::parse::Parse,
	V: syn::parse::Parse,
{
	// Input does NOT begin with the expected keyword.
	if input.fork().parse::<K>().is_err() {
		let rest = input.parse()?;
		return Ok((None, rest));
	}

	// Input DOES begin with the expected keyword.
	let key = input.parse()?;
	let eq = input.parse()?;
	let value = input.parse()?;

	// If the input is not empty now, we need a comma to separate the next argument.
	if !input.is_empty() {
		let _: syn::token::Comma = input.parse()?;
	}

	// Collect the remainder into a TokenStream again.
	let rest = input.parse()?;

	Ok((Some((key, eq, value)), rest))
}

fn parse_unregcognized_argument(input: &syn::parse::ParseBuffer) -> syn::Result<(syn::Ident, TokenStream)> {
	// Parse an identifier first.
	let ident: syn::Ident = input.parse()?;

	// Eat all token trees up to the next comma, and the comma itself.
	input.step(|cursor| {
		let mut cursor = *cursor;
		while let Some((next, rest)) = cursor.token_tree() {
			cursor = rest;
			if let proc_macro2::TokenTree::Punct(punct) = next {
				if punct.as_char() == ',' {
					return Ok(((), cursor));
				}
			}
		}
		Ok(((), cursor))
	}).unwrap();

	// Collect the remainder into a TokenStream again.
	let rest = input.parse()?;

	Ok((ident, rest))
}

fn macro_delim_surround<F>(delim: &syn::MacroDelimiter, tokens: &mut TokenStream, content: F)
where
	F: FnOnce(&mut TokenStream),
{
	match delim {
		syn::MacroDelimiter::Paren(x) => x.surround(tokens, content),
		syn::MacroDelimiter::Brace(x) => x.surround(tokens, content),
		syn::MacroDelimiter::Bracket(x) => x.surround(tokens, content),
	}
}

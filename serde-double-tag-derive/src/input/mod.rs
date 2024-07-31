use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use crate::Context;

pub mod attributes;

pub struct Enum {
	pub attr: attributes::EnumAttributes,
	pub ident: syn::Ident,
	pub generics: syn::Generics,
	pub variants: syn::punctuated::Punctuated<Variant, syn::token::Comma>,
}

impl Enum {
	pub fn parse2(context: &mut Context, tokens: TokenStream) -> Result<Self, ()> {
		let item: syn::Item = syn::parse2(tokens).map_err(|e| context.syn_error(e))?;
		match item {
			syn::Item::Enum(item) => Ok(Self::from_syn(context, item)),
			_ => {
				context.error(
					crate::util::item_token_span(&item),
					"this serde representation is only available for enums",
				);
				Err(())
			},
		}
	}

	fn from_syn(context: &mut Context, input: syn::ItemEnum) -> Self {
		Self {
			attr: attributes::EnumAttributes::from_syn(context, input.attrs),
			ident: input.ident,
			generics: input.generics,
			variants: Variant::from_punctuated(context, input.variants),
		}
	}
}

pub struct Variant {
	pub attr: attributes::VariantAttributes,
	pub ident: syn::Ident,
	pub fields: Fields,
}

impl Variant {
	fn from_syn(context: &mut Context, input: syn::Variant) -> Self {
		Self {
			attr: attributes::VariantAttributes::from_syn(context, input.attrs),
			ident: input.ident,
			fields: Fields::from_syn(context, input.fields),
		}
	}

	fn from_punctuated(
		context: &mut Context,
		input: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
	) -> syn::punctuated::Punctuated<Self, syn::token::Comma> {
		input
			.into_pairs()
			.map(|elem| {
				let (variant, punct) = elem.into_tuple();
				let variant = Self::from_syn(context, variant);
				syn::punctuated::Pair::new(variant, punct)
			})
			.collect()
	}

	#[allow(clippy::manual_map)]
	pub fn rename_all_rule(
		&self,
		item: &Enum,
	) -> Option<attributes::KeyValueArg<attributes::keyword::rename_all, attributes::RenameRule>> {
		if let Some(rename_all) = self.attr.rename_all.clone() {
			Some(rename_all)
		} else if let Some(rename_all_fields) = item.attr.rename_all_fields.clone() {
			Some(rename_all_fields.map_key(|key| attributes::keyword::rename_all(key.span)))
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub enum Fields {
	Unit,
	Tuple(TupleFields),
	Struct(StructFields),
}

impl Fields {
	pub fn is_unit(&self) -> bool {
		matches!(self, Self::Unit)
	}
}

impl Fields {
	fn from_syn(context: &mut Context, input: syn::Fields) -> Self {
		match input {
			syn::Fields::Unit => Self::Unit,
			syn::Fields::Named(fields) => Self::Struct(StructFields::from_syn(context, fields)),
			syn::Fields::Unnamed(fields) => Self::Tuple(TupleFields::from_syn(context, fields)),
		}
	}

	pub fn add_lifetime(&self, lifetime: &syn::Lifetime) -> Self {
		match self {
			Self::Unit => Self::Unit,
			Self::Tuple(x) => Self::Tuple(x.add_lifetime(lifetime)),
			Self::Struct(x) => Self::Struct(x.add_lifetime(lifetime)),
		}
	}

	pub fn iter_types(&self) -> FieldTypes<'_> {
		match self {
			Self::Unit => FieldTypes::Unit,
			Self::Tuple(x) => FieldTypes::Tuple(x.fields.iter()),
			Self::Struct(x) => FieldTypes::Struct(x.fields.iter()),
		}
	}
}

impl ToTokens for Fields {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Unit => (),
			Self::Tuple(x) => x.to_tokens(tokens),
			Self::Struct(x) => x.to_tokens(tokens),
		}
	}
}

#[derive(Clone)]
pub struct TupleFields {
	pub paren: syn::token::Paren,
	pub fields: syn::punctuated::Punctuated<TupleField, syn::token::Comma>,
}

impl TupleFields {
	fn from_syn(context: &mut Context, input: syn::FieldsUnnamed) -> Self {
		let fields = input
			.unnamed
			.into_pairs()
			.enumerate()
			.map(|(index, elem)| {
				let (field, punct) = elem.into_tuple();
				let field = TupleField::from_syn(context, index, field);
				syn::punctuated::Pair::new(field, punct)
			})
			.collect();
		Self {
			paren: input.paren_token,
			fields,
		}
	}

	pub fn add_lifetime(&self, lifetime: &syn::Lifetime) -> Self {
		let fields = self
			.fields
			.pairs()
			.map(|elem| {
				let (field, punct) = elem.into_tuple();
				let field = field.add_lifetime(lifetime.clone());
				syn::punctuated::Pair::new(field, punct.cloned())
			})
			.collect();
		Self {
			paren: self.paren,
			fields,
		}
	}
}

impl ToTokens for TupleFields {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { paren, fields } = self;
		paren.surround(tokens, |tokens| {
			fields.to_tokens(tokens);
		})
	}
}

#[derive(Clone)]
pub struct TupleField {
	pub index: usize,
	pub attrs: attributes::FieldAttributes,
	pub vis: syn::Visibility,
	pub ty: syn::Type,
}

impl TupleField {
	fn from_syn(context: &mut Context, index: usize, input: syn::Field) -> Self {
		Self {
			index,
			attrs: attributes::FieldAttributes::from_syn(context, input.attrs),
			vis: input.vis,
			ty: input.ty,
		}
	}

	fn add_lifetime(&self, lifetime: syn::Lifetime) -> Self {
		Self {
			index: self.index,
			attrs: self.attrs.clone(),
			vis: self.vis.clone(),
			ty: syn::Type::Reference(syn::TypeReference {
				and_token: Default::default(),
				lifetime: Some(lifetime.clone()),
				mutability: None,
				elem: Box::new(self.ty.clone()),
			}),
		}
	}
}

impl ToTokens for TupleField {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			index: _,
			attrs,
			vis,
			ty,
		} = self;
		attrs.to_tokens(tokens);
		vis.to_tokens(tokens);
		ty.to_tokens(tokens);
	}
}

#[derive(Clone)]
pub struct StructFields {
	pub brace: syn::token::Brace,
	pub fields: syn::punctuated::Punctuated<StructField, syn::token::Comma>,
}

impl StructFields {
	fn from_syn(context: &mut Context, input: syn::FieldsNamed) -> Self {
		let fields = input
			.named
			.into_pairs()
			.map(|elem| {
				let (field, punct) = elem.into_tuple();
				let field = StructField::from_syn(context, field);
				syn::punctuated::Pair::new(field, punct)
			})
			.collect();
		Self {
			brace: input.brace_token,
			fields,
		}
	}

	pub fn add_lifetime(&self, lifetime: &syn::Lifetime) -> Self {
		let fields = self
			.fields
			.pairs()
			.map(|elem| {
				let (field, punct) = elem.into_tuple();
				let field = field.add_lifetime(lifetime.clone());
				syn::punctuated::Pair::new(field, punct.cloned())
			})
			.collect();
		Self {
			brace: self.brace,
			fields,
		}
	}
}

impl ToTokens for StructFields {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { brace, fields } = self;
		brace.surround(tokens, |tokens| fields.to_tokens(tokens))
	}
}

#[derive(Clone)]
pub struct StructField {
	pub attrs: attributes::FieldAttributes,
	pub vis: syn::Visibility,
	pub ident: syn::Ident,
	pub colon: syn::token::Colon,
	pub ty: syn::Type,
}

impl StructField {
	fn from_syn(context: &mut Context, input: syn::Field) -> Self {
		let ident = match input.ident {
			Some(x) => x,
			None => {
				context.spanned_error(&input.ty, "struct field must have an identifier");
				syn::Ident::new("missing_identifier", Span::call_site())
			},
		};
		let colon = match input.colon_token {
			Some(x) => x,
			None => {
				context.spanned_error(
					&input.ty,
					"struct field must have a colon (`:`) to separate the identifier and the type",
				);
				syn::token::Colon(Span::call_site())
			},
		};
		let attrs = attributes::FieldAttributes::from_syn(context, input.attrs);
		if let Some(rename) = &attrs.rename {
			context.error(rename.key.span, "#[serde(rename)] is not supported for tuple fields");
		}
		Self {
			attrs,
			vis: input.vis,
			ident,
			colon,
			ty: input.ty,
		}
	}

	fn add_lifetime(&self, lifetime: syn::Lifetime) -> Self {
		Self {
			attrs: self.attrs.clone(),
			vis: self.vis.clone(),
			ident: self.ident.clone(),
			colon: self.colon,
			ty: syn::Type::Reference(syn::TypeReference {
				and_token: Default::default(),
				lifetime: Some(lifetime.clone()),
				mutability: None,
				elem: Box::new(self.ty.clone()),
			}),
		}
	}
}

impl ToTokens for StructField {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			attrs,
			vis,
			ident,
			colon,
			ty,
		} = self;
		attrs.to_tokens(tokens);
		vis.to_tokens(tokens);
		ident.to_tokens(tokens);
		colon.to_tokens(tokens);
		ty.to_tokens(tokens);
	}
}

pub enum FieldTypes<'a> {
	Unit,
	Tuple(syn::punctuated::Iter<'a, TupleField>),
	Struct(syn::punctuated::Iter<'a, StructField>),
}

impl<'a> Iterator for FieldTypes<'a> {
	type Item = &'a syn::Type;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Unit => None,
			Self::Tuple(x) => x.next().map(|x| &x.ty),
			Self::Struct(x) => x.next().map(|x| &x.ty),
		}
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		match self {
			Self::Unit => None,
			Self::Tuple(x) => x.nth(n).map(|x| &x.ty),
			Self::Struct(x) => x.nth(n).map(|x| &x.ty),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Unit => (0, Some(0)),
			Self::Tuple(x) => x.size_hint(),
			Self::Struct(x) => x.size_hint(),
		}
	}
}

impl std::iter::ExactSizeIterator for FieldTypes<'_> {
	fn len(&self) -> usize {
		match self {
			Self::Unit => 0,
			Self::Tuple(x) => x.len(),
			Self::Struct(x) => x.len(),
		}
	}
}

impl std::iter::DoubleEndedIterator for FieldTypes<'_> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::Unit => None,
			Self::Tuple(x) => x.next_back().map(|x| &x.ty),
			Self::Struct(x) => x.next_back().map(|x| &x.ty),
		}
	}

	fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
		match self {
			Self::Unit => None,
			Self::Tuple(x) => x.nth_back(n).map(|x| &x.ty),
			Self::Struct(x) => x.nth_back(n).map(|x| &x.ty),
		}
	}
}

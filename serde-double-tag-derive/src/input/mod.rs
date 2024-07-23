mod attrs;
pub use attrs::*;
use proc_macro2::Span;

use crate::Context;

pub struct Enum {
	pub attr: EnumAttributes,
	pub vis: syn::Visibility,
	pub enum_token: syn::token::Enum,
	pub ident: syn::Ident,
	pub generics: syn::Generics,
	pub variants: syn::punctuated::Punctuated<Variant, syn::token::Comma>,
}

impl Enum {
	pub fn parse2(context: &mut Context, tokens: proc_macro2::TokenStream) -> Result<Self, ()> {
		let item: syn::Item = syn::parse2(tokens)
			.map_err(|e| context.syn_error(e))?;
		match item {
			syn::Item::Enum(item) => Ok(Self::from_syn(context, item)),
			_ => {
				context.error(crate::util::item_token_span(&item), "this serde represebntation is only available for enums");
				Err(())
			}
		}
	}

	fn from_syn(context: &mut Context, input: syn::ItemEnum) -> Self {
		Self {
			attr: EnumAttributes::from_syn(context, input.attrs),
			vis: input.vis,
			enum_token: input.enum_token,
			ident: input.ident,
			generics: input.generics,
			variants: Variant::from_punctuated(context, input.variants),
		}
	}
}

pub struct Variant {
	pub attr: VariantAttributes,
	pub ident: syn::Ident,
	pub fields: Fields,
}

impl Variant {
	fn from_syn(context: &mut Context, input: syn::Variant) -> Self {
		Self {
			attr: VariantAttributes::from_syn(context, input.attrs),
			ident: input.ident,
			fields: Fields::from_syn(context, input.fields),
		}
	}

	fn from_punctuated(context: &mut Context, input: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>) -> syn::punctuated::Punctuated<Self, syn::token::Comma> {
		input
			.into_pairs()
			.map(|elem| {
				let (variant, punct) = elem.into_tuple();
				let variant = Self::from_syn(context, variant);
				syn::punctuated::Pair::new(variant, punct)
			})
			.collect()
	}
}

pub enum Fields {
	Unit,
	Tuple(TupleFields),
	Struct(StructFields),
}

impl Fields {
	fn from_syn(context: &mut Context, input: syn::Fields) -> Self {
		match input {
			syn::Fields::Unit => Self::Unit,
			syn::Fields::Named(fields) => Self::Struct(StructFields::from_syn(context, fields)),
			syn::Fields::Unnamed(fields) => Self::Tuple(TupleFields::from_syn(context, fields)),
		}
	}

	pub fn iter_types(&self) -> FieldTypes<'_> {
		match self {
			Self::Unit => FieldTypes::Unit,
			Self::Tuple(x) => FieldTypes::Tuple(x.fields.iter()),
			Self::Struct(x) => FieldTypes::Struct(x.fields.iter()),
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Self::Unit => 0,
			Self::Tuple(x) => x.len(),
			Self::Struct(x) => x.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Unit => true,
			Self::Tuple(x) => x.is_empty(),
			Self::Struct(x) => x.is_empty(),
		}
	}
}

pub struct TupleFields {
	pub paren: syn::token::Paren,
	pub fields: syn::punctuated::Punctuated<TupleField, syn::token::Comma>,
}

impl TupleFields {
	fn from_syn(context: &mut Context, input: syn::FieldsUnnamed) -> Self {
		let fields = input.unnamed
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

	pub fn len(&self) -> usize {
		self.fields.len()
	}

	pub fn is_empty(&self) -> bool {
		self.fields.is_empty()
	}
}

pub struct TupleField {
	pub index: usize,
	pub attrs: Vec<syn::Attribute>,
	pub vis: syn::Visibility,
	pub ty: syn::Type,
}

impl TupleField {
	fn from_syn(_context: &mut Context, index: usize, input: syn::Field) -> Self {
		Self {
			index,
			attrs: input.attrs,
			vis: input.vis,
			ty: input.ty,
		}
	}
}

pub struct StructFields {
	pub brace: syn::token::Brace,
	pub fields: syn::punctuated::Punctuated<StructField, syn::token::Comma>,
}

impl  StructFields {
	fn from_syn(context: &mut Context, input: syn::FieldsNamed) -> Self {
		let fields = input.named
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

	pub fn len(&self) -> usize {
		self.fields.len()
	}

	pub fn is_empty(&self) -> bool {
		self.fields.is_empty()
	}
}

impl quote::ToTokens for StructFields {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			brace,
			fields,
		} = self;
		brace.surround(tokens, |tokens| {
			for field in fields {
				field.to_tokens(tokens);
			}
		})
	}
}

pub struct StructField {
	pub attrs: Vec<syn::Attribute>,
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
				context.spanned_error(&input.ty, "struct field must have a colon (`:`) to separate the identifier and the type");
				syn::token::Colon(Span::call_site())
			},
		};
		Self {
			attrs: input.attrs,
			vis: input.vis,
			ident,
			colon,
			ty: input.ty,
		}
	}
}

impl quote::ToTokens for StructField {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let Self {
			attrs,
			vis,
			ident,
			colon,
			ty,
		} = self;
		for attr in attrs {
			attr.to_tokens(tokens);
		}
		vis.to_tokens(tokens);
		ident.to_tokens(tokens);
		colon.to_tokens(tokens);
		ty.to_tokens(tokens);
	}
}

pub struct Discriminant {
	pub eq: syn::token::Eq,
	pub expr: syn::Expr,
}

impl Discriminant {
	fn from_syn(input: (syn::token::Eq, syn::Expr)) -> syn::Result<Self> {
		let (eq, expr) = input;
		Ok(Self {
			eq,
			expr,
		})
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

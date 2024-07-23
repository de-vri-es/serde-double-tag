mod attrs;
pub use attrs::*;

pub struct Enum {
	pub attr: EnumAttributes,
	pub vis: syn::Visibility,
	pub enum_token: syn::token::Enum,
	pub ident: syn::Ident,
	pub generics: syn::Generics,
	pub variants: syn::punctuated::Punctuated<Variant, syn::token::Comma>,
}

struct Variant {
	pub attr: VariantAttributes,
	pub ident: syn::Ident,
	pub fields: Fields,
	pub discriminant: Option<Discriminant>,
}

pub enum Fields {
	Unit,
	Tuple(TupleFields),
	Struct(StructFields),
}

pub struct TupleFields {
	pub paren: syn::token::Paren,
	pub fields: syn::punctuated::Punctuated<TupleField, syn::token::Comma>,
}

pub struct StructFields {
	pub brace: syn::token::Brace,
	pub fields: syn::punctuated::Punctuated<StructField, syn::token::Comma>,
}

pub struct TupleField {
	pub attrs: Vec<syn::Attribute>,
	pub vis: syn::Visibility,
	pub ty: syn::Type,
}

pub struct StructField {
	pub attrs: Vec<syn::Attribute>,
	pub vis: syn::Visibility,
	pub ident: syn::Ident,
	pub colon: syn::token::Colon,
	pub ty: syn::Type,
}

pub struct Discriminant {
	pub eq: syn::token::Eq,
	pub expr: syn::Expr,
}

impl syn::parse::Parse for Enum {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			attr: input.parse()?,
			vis: input.parse()?,
			enum_token: input.parse()?,
			ident: input.parse()?,
			generics: input.parse()?,
			variants: syn::punctuated::Punctuated::parse_terminated(input)?,
		})
	}
}

impl syn::parse::Parse for Variant {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attr = input.parse()?;
		let ident = input.parse()?;
		let fields = input.parse()?;
		let discriminant = if input.peek(syn::token::Eq) {
			Some(input.parse()?)
		} else {
			None
		};
		Ok(Self {
			attr,
			ident,
			fields,
			discriminant,
		})
	}
}

impl syn::parse::Parse for Fields {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.peek(syn::token::Comma) || input.is_empty() {
			Ok(Self::Unit)
		} else if input.peek(syn::token::Paren) {
			let content;
			Ok(Self::Tuple(TupleFields {
				paren: syn::parenthesized!(content in input),
				fields: syn::punctuated::Punctuated::parse_terminated(&content)?,
			}))
		} else if input.peek(syn::token::Brace) {
			let content;
			Ok(Self::Struct(StructFields {
				brace: syn::braced!(content in input),
				fields: syn::punctuated::Punctuated::parse_terminated(&content)?,
			}))
		} else {
			Err(input.error("expected a unit variant, tuple variant or struct variant"))
		}
	}
}

impl syn::parse::Parse for TupleField {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			attrs: syn::Attribute::parse_outer(input)?,
			vis: input.parse()?,
			ty: input.parse()?,
		})
	}
}

impl syn::parse::Parse for StructField {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			attrs: syn::Attribute::parse_outer(input)?,
			vis: input.parse()?,
			ident: input.parse()?,
			colon: input.parse()?,
			ty: input.parse()?,
		})
	}
}

impl syn::parse::Parse for Discriminant {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(Self {
			eq: input.parse()?,
			expr: input.parse()?,
		})
	}
}

use proc_macro2::Span;

pub mod keywords {
	syn::custom_keyword!(rename);
	syn::custom_keyword!(rename_all);
	syn::custom_keyword!(rename_all_fields);
	syn::custom_keyword!(deny_unknown_fields);
	syn::custom_keyword!(tag);
	syn::custom_keyword!(content);
	syn::custom_keyword!(untagged);
	syn::custom_keyword!(expecting);
	syn::custom_keyword!(alias);
}

struct Attr<Keyword, Value> {
	pub keyword: Keyword,
	pub value: Value,
}

struct AttrOptValue<Keyword, Value> {
	pub keyword: Keyword,
	pub value: Option<Value>,
}

pub struct WithSpan<T> {
	pub span: Span,
	pub value: T,
}

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

pub struct EnumAttributes {
	pub rename: Option<Attr<keywords::rename, syn::LitStr>>,
	pub rename_all: Option<Attr<keywords::rename, WithSpan<RenameAll>>>,
	pub rename_all_fields: Option<Attr<keywords::rename, WithSpan<RenameAll>>>,
	pub deny_unknown_fields: Option<keywords::deny_unknown_fields>,
	pub tag: Option<Attr<keywords::tag, syn::LitStr>>,
	pub bound: Option<Attr<keywords::tag, syn::WhereClause>>,
	pub expecting: Option<Attr<keywords::tag, syn::LitStr>>,
}

pub struct VariantAttributes {
	pub rename: Option<Attr<keywords::rename, syn::LitStr>>,
	pub rename_all: Option<Attr<keywords::rename, WithSpan<RenameAll>>>,
	pub alias: Vec<Attr<keywords::alias, syn::LitStr>>,
	pub expecting: Option<Attr<keywords::tag, syn::LitStr>>,
}

impl syn::parse::Parse for EnumAttributes {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attributes = syn::Attribute::parse_outer(input)?;
		Ok(Self {
			rename: None,
			rename_all: None,
			rename_all_fields: None,
			deny_unknown_fields: None,
			tag: None,
			bound: None,
			expecting: None,
		})
	}
}

impl syn::parse::Parse for VariantAttributes {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let attributes = syn::Attribute::parse_outer(input)?;
		Ok(Self {
			rename: None,
			rename_all: None,
			alias: Vec::new(),
			expecting: None,
		})
	}
}

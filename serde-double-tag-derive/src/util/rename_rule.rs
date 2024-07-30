/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Copy, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RenameRule {
	/// Rename direct children to "lowercase" style.
	LowerCase,
	/// Rename direct children to "UPPERCASE" style.
	UpperCase,
	/// Rename direct children to "PascalCase" style, as typically used for
	/// enum variants.
	PascalCase,
	/// Rename direct children to "camelCase" style.
	CamelCase,
	/// Rename direct children to "snake_case" style, as commonly used for
	/// fields.
	SnakeCase,
	/// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
	/// used for constants.
	ScreamingSnakeCase,
	/// Rename direct children to "kebab-case" style.
	KebabCase,
	/// Rename direct children to "SCREAMING-KEBAB-CASE" style.
	ScreamingKebabCase,
}

const RENAME_RULES: [RenameRule; 8] = [
	RenameRule::LowerCase,
	RenameRule::UpperCase,
	RenameRule::PascalCase,
	RenameRule::CamelCase,
	RenameRule::SnakeCase,
	RenameRule::ScreamingSnakeCase,
	RenameRule::KebabCase,
	RenameRule::ScreamingKebabCase,
];

const LOWER_CASE: &str = "lowercase";
const UPPER_CASE: &str = "UPPERCASE";
const PASCAL_CASE: &str = "PascalCase";
const CAMEL_CASE: &str = "camelCase";
const SNAKE_CASE: &str = "snake_case";
const SCREAMING_SNAKE_CASE: &str = "SCREAMING_SNAKE_CASE";
const KEBAB_CASE: &str = "kebab-case";
const SCREAMING_KEBAB_CASE: &str = "SCREAMING-KEBAB-CASE";

impl RenameRule {
	pub fn from_str(input: &str) -> Result<Self, ParseError> {
		match input {
			self::LOWER_CASE => Ok(Self::LowerCase),
			self::UPPER_CASE => Ok(Self::UpperCase),
			self::PASCAL_CASE => Ok(Self::PascalCase),
			self::CAMEL_CASE => Ok(Self::CamelCase),
			self::SNAKE_CASE => Ok(Self::SnakeCase),
			self::SCREAMING_SNAKE_CASE => Ok(Self::ScreamingSnakeCase),
			self::KEBAB_CASE => Ok(Self::KebabCase),
			self::SCREAMING_KEBAB_CASE => Ok(Self::ScreamingKebabCase),
			unknown => Err(ParseError { unknown }),
		}
	}

	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::LowerCase => LOWER_CASE,
			Self::UpperCase => UPPER_CASE,
			Self::PascalCase => PASCAL_CASE,
			Self::CamelCase => CAMEL_CASE,
			Self::SnakeCase => SNAKE_CASE,
			Self::ScreamingSnakeCase => SCREAMING_SNAKE_CASE,
			Self::KebabCase => KEBAB_CASE,
			Self::ScreamingKebabCase => SCREAMING_KEBAB_CASE,
		}
	}

	/// Apply a renaming rule to an enum variant, returning the version expected in the source.
	pub fn apply_to_variant(self, variant: &str) -> String {
		match self {
			Self::PascalCase => variant.to_owned(),
			Self::LowerCase => variant.to_ascii_lowercase(),
			Self::UpperCase => variant.to_ascii_uppercase(),
			Self::CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
			Self::SnakeCase => {
				let mut snake = String::new();
				for (i, ch) in variant.char_indices() {
					if i > 0 && ch.is_uppercase() {
						snake.push('_');
					}
					snake.push(ch.to_ascii_lowercase());
				}
				snake
			}
			Self::ScreamingSnakeCase => Self::SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
			Self::KebabCase => Self::SnakeCase.apply_to_variant(variant).replace('_', "-"),
			Self::ScreamingKebabCase => Self::ScreamingSnakeCase
				.apply_to_variant(variant)
				.replace('_', "-"),
		}
	}

	/// Apply a renaming rule to a struct field, returning the version expected in the source.
	#[allow(unused)]
	pub fn apply_to_field(self, field: &str) -> String {
		match self {
			Self::LowerCase | Self::SnakeCase => field.to_owned(),
			Self::UpperCase => field.to_ascii_uppercase(),
			Self::PascalCase => {
				let mut pascal = String::new();
				let mut capitalize = true;
				for ch in field.chars() {
					if ch == '_' {
						capitalize = true;
					} else if capitalize {
						pascal.push(ch.to_ascii_uppercase());
						capitalize = false;
					} else {
						pascal.push(ch);
					}
				}
				pascal
			}
			Self::CamelCase => {
				let pascal = Self::PascalCase.apply_to_field(field);
				pascal[..1].to_ascii_lowercase() + &pascal[1..]
			}
			Self::ScreamingSnakeCase => field.to_ascii_uppercase(),
			Self::KebabCase => field.replace('_', "-"),
			Self::ScreamingKebabCase => Self::ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
		}
	}
}

pub struct ParseError<'a> {
	unknown: &'a str,
}

impl<'a> std::fmt::Display for ParseError<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str("unknown rename rule `rename_all = ")?;
		std::fmt::Debug::fmt(self.unknown, f)?;
		f.write_str("`, expected one of ")?;
		for (i, rule) in RENAME_RULES.iter().enumerate() {
			if i > 0 {
				f.write_str(", ")?;
			}
			std::fmt::Debug::fmt(rule.as_str(), f)?;
		}
		Ok(())
	}
}

#[test]
fn rename_variants() {
	for &(original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in &[
		(
			"Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
		),
		(
			"VeryTasty",
			"verytasty",
			"VERYTASTY",
			"veryTasty",
			"very_tasty",
			"VERY_TASTY",
			"very-tasty",
			"VERY-TASTY",
		),
		("A", "a", "A", "a", "a", "A", "a", "A"),
		("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
	] {
		assert_eq!(RenameRule::LowerCase.apply_to_variant(original), lower);
		assert_eq!(RenameRule::UpperCase.apply_to_variant(original), upper);
		assert_eq!(RenameRule::PascalCase.apply_to_variant(original), original);
		assert_eq!(RenameRule::CamelCase.apply_to_variant(original), camel);
		assert_eq!(RenameRule::SnakeCase.apply_to_variant(original), snake);
		assert_eq!(RenameRule::ScreamingSnakeCase.apply_to_variant(original), screaming);
		assert_eq!(RenameRule::KebabCase.apply_to_variant(original), kebab);
		assert_eq!(RenameRule::ScreamingKebabCase.apply_to_variant(original), screaming_kebab);
	}
}

#[test]
fn rename_fields() {
	for &(original, upper, pascal, camel, screaming, kebab, screaming_kebab) in &[
		(
			"outcome", "OUTCOME", "Outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
		),
		(
			"very_tasty",
			"VERY_TASTY",
			"VeryTasty",
			"veryTasty",
			"VERY_TASTY",
			"very-tasty",
			"VERY-TASTY",
		),
		("a", "A", "A", "a", "A", "a", "A"),
		("z42", "Z42", "Z42", "z42", "Z42", "z42", "Z42"),
	] {
		assert_eq!(RenameRule::UpperCase.apply_to_field(original), upper);
		assert_eq!(RenameRule::PascalCase.apply_to_field(original), pascal);
		assert_eq!(RenameRule::CamelCase.apply_to_field(original), camel);
		assert_eq!(RenameRule::SnakeCase.apply_to_field(original), original);
		assert_eq!(RenameRule::ScreamingSnakeCase.apply_to_field(original), screaming);
		assert_eq!(RenameRule::KebabCase.apply_to_field(original), kebab);
		assert_eq!(RenameRule::ScreamingKebabCase.apply_to_field(original), screaming_kebab);
	}
}

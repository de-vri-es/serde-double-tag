use assert2::{assert, let_assert};

#[track_caller]
fn json<T: serde::Serialize>(value: T) -> String {
	let_assert!(Ok(json) = serde_json::to_string(&value));
	json
}

#[test]
fn serialize_enum() {
	#[derive(serde_double_tag::Serialize, serde_double_tag::Deserialize, serde_double_tag::JsonSchema)]
	#[serde(rename_all = "snake_case")]
	#[serde(deny_unknown_fields)]
	#[serde(tag = "type")]
	enum MyEnum {
		Unit,
		EmptyTuple(),
		NewType(String),
		Tuple(u8, String),
		EmptyStruct {},
		Struct { field_a: String, field_b: u8 },
	}
	assert!(json(MyEnum::Unit) == r#"{"type":"unit"}"#);
	assert!(json(MyEnum::EmptyTuple()) == r#"{"type":"empty_tuple","empty_tuple":[]}"#);
	assert!(json(MyEnum::NewType("hello".into())) == r#"{"type":"new_type","new_type":"hello"}"#);
	assert!(json(MyEnum::Tuple(3, "world".into())) == r#"{"type":"tuple","tuple":[3,"world"]}"#);
	assert!(json(MyEnum::EmptyStruct {}) == r#"{"type":"empty_struct","empty_struct":{}}"#);
	assert!(
		json(MyEnum::Struct {
			field_a: "bye".into(),
			field_b: 7
		}) == r#"{"type":"struct","struct":{"field_a":"bye","field_b":7}}"#
	);
}

#[test]
fn serialize_enum_generic() {
	#[derive(serde_double_tag::Serialize, serde_double_tag::Deserialize, serde_double_tag::JsonSchema)]
	#[serde(rename_all = "snake_case")]
	#[serde(tag = "type")]
	enum MyEnum<T> {
		Unit,
		EmptyTuple(),
		NewType(T),
		Tuple(u8, T),
		EmptyStruct {},
		Struct { field_a: T, field_b: u8 },
	}
	assert!(json(MyEnum::Unit::<()>) == r#"{"type":"unit"}"#);
	assert!(json(MyEnum::EmptyTuple::<()>()) == r#"{"type":"empty_tuple","empty_tuple":[]}"#);
	assert!(json(MyEnum::NewType("hello")) == r#"{"type":"new_type","new_type":"hello"}"#);
	assert!(json(MyEnum::Tuple(3, "world")) == r#"{"type":"tuple","tuple":[3,"world"]}"#);
	assert!(json(MyEnum::EmptyStruct::<()> {}) == r#"{"type":"empty_struct","empty_struct":{}}"#);
	assert!(
		json(MyEnum::Struct {
			field_a: "bye",
			field_b: 7
		}) == r#"{"type":"struct","struct":{"field_a":"bye","field_b":7}}"#
	);
}

#[test]
fn serialize_enum_generic_lifetime() {
	#[derive(serde_double_tag::Serialize, serde_double_tag::JsonSchema)]
	#[serde(rename_all = "snake_case")]
	#[serde(tag = "type")]
	enum MyEnum<'a, T> {
		Unit,
		EmptyTuple(),
		NewType(&'a T),
		Tuple(u8, &'a T),
		EmptyStruct {},
		Struct { field_a: &'a T, field_b: u8 },
	}
	assert!(json(MyEnum::Unit::<()>) == r#"{"type":"unit"}"#);
	assert!(json(MyEnum::EmptyTuple::<()>()) == r#"{"type":"empty_tuple","empty_tuple":[]}"#);
	assert!(json(MyEnum::NewType(&"hello")) == r#"{"type":"new_type","new_type":"hello"}"#);
	assert!(json(MyEnum::Tuple(3, &"world")) == r#"{"type":"tuple","tuple":[3,"world"]}"#);
	assert!(json(MyEnum::EmptyStruct::<()> {}) == r#"{"type":"empty_struct","empty_struct":{}}"#);
	assert!(
		json(MyEnum::Struct {
			field_a: &"bye",
			field_b: 7
		}) == r#"{"type":"struct","struct":{"field_a":"bye","field_b":7}}"#
	);
}

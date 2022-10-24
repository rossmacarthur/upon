#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use serde::Serialize;

use upon::{to_value, Value};

#[test]
fn to_value_bool() {
    assert_eq!(to_value(true).unwrap(), Value::Bool(true));
    assert_eq!(to_value(false).unwrap(), Value::Bool(false));
}

#[test]
fn to_value_integer() {
    assert_eq!(to_value(123_i8).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_i16).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_i32).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_i64).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_u8).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_u16).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_u32).unwrap(), Value::Integer(123));
    assert_eq!(to_value(123_u64).unwrap(), Value::Integer(123));
}

#[test]
fn to_value_out_of_range_integral() {
    let err = to_value(u64::MAX).unwrap_err().to_string();
    assert_eq!(
        err,
        "serialize error: out of range integral type conversion attempted"
    );
}

#[test]
fn to_value_char() {
    assert_eq!(to_value('a').unwrap(), Value::String(String::from('a')));
}

#[test]
fn to_value_str() {
    assert_eq!(
        to_value("testing...").unwrap(),
        Value::String(String::from("testing..."))
    );
}

#[test]
fn to_value_bytes() {
    assert_eq!(
        to_value([1u8, 2, 3, 4]).unwrap(),
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4)
        ]),
    )
}

#[test]
fn to_value_none() {
    assert_eq!(to_value(None::<i32>).unwrap(), Value::None);
}

#[test]
fn to_value_some() {
    assert_eq!(
        to_value(Some("testing...")).unwrap(),
        Value::String(String::from("testing..."))
    );
}

#[test]
fn to_value_unit() {
    assert_eq!(to_value(()).unwrap(), Value::None);
}

#[test]
fn to_value_unit_struct() {
    #[derive(Serialize)]
    struct Test;
    assert_eq!(to_value(Test).unwrap(), Value::None);
}

#[test]
fn to_value_unit_variant() {
    #[derive(Serialize)]
    enum Test {
        Variant,
    }
    assert_eq!(
        to_value(Test::Variant).unwrap(),
        Value::String(String::from("Variant"))
    );
}

#[test]
fn to_value_newtype_struct() {
    #[derive(Serialize)]
    struct Test(&'static str);

    assert_eq!(
        to_value(Test("testing...")).unwrap(),
        Value::String(String::from("testing..."))
    );
}

#[test]
fn to_value_newtype_variant() {
    #[derive(Serialize)]
    enum Test {
        Variant(&'static str),
    }
    assert_eq!(
        to_value(Test::Variant("testing...")).unwrap(),
        Value::Map(BTreeMap::from([(
            String::from("Variant"),
            Value::String(String::from("testing..."))
        )]))
    );
}

#[test]
fn to_value_seq() {
    assert_eq!(
        to_value(vec!["a", "b", "c"]).unwrap(),
        Value::List(Vec::from([
            Value::String(String::from("a")),
            Value::String(String::from("b")),
            Value::String(String::from("c")),
        ]))
    );
}

#[test]
fn to_value_tuple() {
    assert_eq!(
        to_value(("a", "b", "c")).unwrap(),
        Value::List(Vec::from([
            Value::String(String::from("a")),
            Value::String(String::from("b")),
            Value::String(String::from("c")),
        ]))
    );
}

#[test]
fn to_value_tuple_struct() {
    #[derive(Serialize)]
    struct Test<'a>(&'a str, &'a str, &'a str);
    assert_eq!(
        to_value(Test("a", "b", "c")).unwrap(),
        Value::List(Vec::from([
            Value::String(String::from("a")),
            Value::String(String::from("b")),
            Value::String(String::from("c")),
        ]))
    );
}

#[test]
fn to_value_tuple_variant() {
    #[derive(Serialize)]
    enum Test<'a> {
        Variant(&'a str, &'a str, &'a str),
    }
    assert_eq!(
        to_value(Test::Variant("a", "b", "c")).unwrap(),
        Value::Map(BTreeMap::from([(
            String::from("Variant"),
            Value::List(Vec::from([
                Value::String(String::from("a")),
                Value::String(String::from("b")),
                Value::String(String::from("c")),
            ]))
        )]))
    );
}

#[test]
fn to_value_map_key_not_string() {
    assert_eq!(
        to_value(BTreeMap::from([(Some("a"), "b"), (Some("c"), "d")]))
            .unwrap_err()
            .to_string(),
        "serialize error: map key must be a string"
    );
}

#[test]
fn to_value_map() {
    assert_eq!(
        to_value(BTreeMap::from([("a", "b"), ("c", "d")])).unwrap(),
        Value::Map(BTreeMap::from([
            (String::from("a"), Value::String(String::from("b"))),
            (String::from("c"), Value::String(String::from("d")))
        ]))
    );
}

#[test]
fn to_value_struct() {
    #[derive(Serialize)]
    struct Test {
        a: String,
        c: String,
    }
    assert_eq!(
        to_value(Test {
            a: "b".into(),
            c: "d".into()
        })
        .unwrap(),
        Value::Map(BTreeMap::from([
            (String::from("a"), Value::String(String::from("b"))),
            (String::from("c"), Value::String(String::from("d")))
        ]))
    );
}

#[test]
fn to_value_struct_variant() {
    #[derive(Serialize)]
    enum Test {
        Variant { a: String, c: String },
    }
    assert_eq!(
        to_value(Test::Variant {
            a: "b".into(),
            c: "d".into()
        })
        .unwrap(),
        Value::Map(BTreeMap::from([(
            String::from("Variant"),
            Value::Map(BTreeMap::from([
                (String::from("a"), Value::String(String::from("b"))),
                (String::from("c"), Value::String(String::from("d")))
            ]))
        )]))
    );
}

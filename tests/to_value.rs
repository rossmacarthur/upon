use std::collections::BTreeMap;

use serde::Serialize;

use upon::value::{to_value, List, Map, Value};

#[test]
fn to_value_unsupported_err() {
    let err = to_value(true).unwrap_err().to_string();
    assert_eq!(err.to_string(), "unsupported type");
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
        Value::Map(Map::from([(
            String::from("Variant"),
            Value::String(String::from("testing..."))
        )]))
    );
}

#[test]
fn to_value_seq() {
    assert_eq!(
        to_value(vec!["a", "b", "c"]).unwrap(),
        Value::List(List::from([
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
        Value::List(List::from([
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
        Value::List(List::from([
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
        Value::Map(Map::from([(
            String::from("Variant"),
            Value::List(List::from([
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
        "map key must be a string"
    );
}

#[test]
fn to_value_map() {
    assert_eq!(
        to_value(BTreeMap::from([("a", "b"), ("c", "d")])).unwrap(),
        Value::Map(Map::from([
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
        Value::Map(Map::from([
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
        Value::Map(Map::from([(
            String::from("Variant"),
            Value::Map(Map::from([
                (String::from("a"), Value::String(String::from("b"))),
                (String::from("c"), Value::String(String::from("d")))
            ]))
        )]))
    );
}

#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use upon::{value, Value};

#[test]
fn value_empty() {
    let v = value! {};
    let exp = Value::Map(Default::default());
    assert_eq!(v, exp);
}

#[test]
fn value_literal() {
    let tests = [
        (value! { f: None }, Value::from([("f", Value::None)])),
        (value! { f: true }, Value::from([("f", true)])),
        (value! { f: false }, Value::from([("f", false)])),
        (value! { f: 123 }, Value::from([("f", 123)])),
        (value! { f: -123 }, Value::from([("f", -123)])),
        (value! { f: 12.3 }, Value::from([("f", 12.3)])),
        (value! { f: -12.3 }, Value::from([("f", -12.3)])),
        (value! { f: "test" }, Value::from([("f", "test")])),
    ];
    for (v, exp) in tests {
        assert_eq!(v, exp);
    }
}

#[test]
fn value_list() {
    // empty list
    let v = value! { field: [] };
    let exp = Value::from([("field", Value::List(vec![]))]);
    assert_eq!(v, exp);

    // empty list, with trailing comma
    let v = value! { field: [,] };
    let exp = Value::from([("field", Value::List(vec![]))]);
    assert_eq!(v, exp);

    // single element list
    let v = value! { field: [true] };
    let exp = Value::from([("field", Value::from(vec![true]))]);
    assert_eq!(v, exp);

    // single element expression list
    let v = value! { field: [ (1+2*3) ] };
    let exp = Value::from([("field", Value::from(vec![7]))]);
    assert_eq!(v, exp);

    // single element list, with trailing comma
    let v = value! { field: [true,] };
    let exp = Value::from([("field", Value::from(vec![true]))]);
    assert_eq!(v, exp);

    // a variety of elements
    let v = value! { all: [None, false, true, 123, 12.3, "testing...", [], {}] };
    let exp = Value::from([(
        "all",
        vec![
            Value::None,
            Value::Bool(false),
            Value::Bool(true),
            Value::Integer(123),
            Value::Float(12.3),
            Value::String("testing...".into()),
            Value::List(Vec::new()),
            Value::Map(BTreeMap::new()),
        ],
    )]);
    assert_eq!(v, exp);
}

#[test]
fn value_map() {
    // empty map
    let v = value! { field: {} };
    let exp = Value::from([("field", Value::Map(Default::default()))]);
    assert_eq!(v, exp);

    // single field map
    let v = value! { field: { x: "hello" } };
    let exp = Value::from([("field", Value::from([("x", "hello")]))]);
    assert_eq!(v, exp);

    // single field expression map
    let v = value! { field: { x: (1+2*3) } };
    let exp = Value::from([("field", Value::from([("x", 7)]))]);
    assert_eq!(v, exp);

    // single field map, with trailing comma
    let v = value! { field: { x: "hello", } };
    let exp = Value::from([("field", Value::from([("x", "hello")]))]);
    assert_eq!(v, exp);

    // a variety of elements
    let v = value! {
        all: {
            none: None,
            no: false,
            yes: true,
            integer: 123,
            float: 12.3,
            string: "testing...",
            list: [],
            map: {}
        }
    };
    let exp = Value::from([(
        "all",
        Value::from([
            ("none", Value::None),
            ("no", Value::Bool(false)),
            ("yes", Value::Bool(true)),
            ("integer", Value::Integer(123)),
            ("float", Value::Float(12.3)),
            ("string", Value::String("testing...".into())),
            ("list", Value::List(Vec::new())),
            ("map", Value::Map(BTreeMap::new())),
        ]),
    )]);
    assert_eq!(v, exp);
}

#[test]
fn value_compile_fail() {
    // let _ = value! { field: {,} };

    // let _ = value! { field: {false} };

    // let _ = value! { field: {None} };

    // let _ = value! { field: {x} };

    // let _ = value! { field: {x:} };

    // let _ = value! { field: { x: y } };

    // let _ = value! { field: { "x": false, } };

    // let _ = value! { field: [x] };

    // let _ = value! { field: { x y } };
}

#![cfg(feature = "builtins")]

mod helpers;

use upon::{value, Engine};

#[test]
fn bool() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", helpers::debug);
    let t = engine.compile("{{ value | bool | debug }}").unwrap();
    macro_rules! render {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ }).to_string().unwrap()
        };
    }
    assert_eq!(render!(None), "false");
    assert_eq!(render!(false), "false");
    assert_eq!(render!(true), "true");
    assert_eq!(render!(0), "false");
    assert_eq!(render!(123), "true");
    assert_eq!(render!(0.0), "false");
    assert_eq!(render!(1.23), "true");
    assert_eq!(render!(""), "false");
    assert_eq!(render!("hello"), "true");
    assert_eq!(render!([]), "false");
    assert_eq!(render!([1, 2, 3]), "true");
    assert_eq!(render!({}), "false");
    assert_eq!(render!({ a: 1 }), "true");
}

#[test]
fn integer() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", helpers::debug);
    let t = engine.compile("{{ value | integer | debug }}").unwrap();
    macro_rules! render {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ }).to_string().unwrap()
        };
    }
    macro_rules! render_err {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ })
                .to_string().unwrap_err()
                .to_string().replace("function error: ", "")
        };
    }
    assert_eq!(render!(false), "0");
    assert_eq!(render!(true), "1");
    assert_eq!(render!(0), "0");
    assert_eq!(render!(123), "123");
    assert_eq!(render!(0.0), "0");
    assert_eq!(render!(1.23), "1");
    assert_eq!(render!("123"), "123");

    assert_eq!(
        render_err!(None),
        "none value cannot be converted to an integer"
    );
    assert_eq!(render_err!("hello"), "invalid digit found in string");
    assert_eq!(
        render_err!([]),
        "list value cannot be converted to an integer"
    );
    assert_eq!(
        render_err!({}),
        "map value cannot be converted to an integer"
    );
}

#[test]
fn float() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", helpers::debug);
    let t = engine.compile("{{ value | float | debug }}").unwrap();
    macro_rules! render {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ }).to_string().unwrap()
        };
    }
    macro_rules! render_err {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ })
                .to_string().unwrap_err()
                .to_string().replace("function error: ", "")
        };
    }
    assert_eq!(render!(0), "0.0");
    assert_eq!(render!(123), "123.0");
    assert_eq!(render!(0.0), "0.0");
    assert_eq!(render!(1.23), "1.23");
    assert_eq!(render!("123"), "123.0");

    assert_eq!(
        render_err!(None),
        "none value cannot be converted to a float"
    );
    assert_eq!(
        render_err!(false),
        "bool value cannot be converted to a float"
    );
    assert_eq!(render_err!("hello"), "invalid float literal");
    assert_eq!(render_err!([]), "list value cannot be converted to a float");
    assert_eq!(render_err!({}), "map value cannot be converted to a float");
}

#[test]
fn string() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", helpers::debug);
    let t = engine.compile("{{ value | string }}").unwrap();
    macro_rules! render {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ }).to_string().unwrap()
        };
    }
    macro_rules! render_err {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ })
                .to_string().unwrap_err()
                .to_string().replace("function error: ", "")
        };
    }
    assert_eq!(render!(None), "");
    assert_eq!(render!(false), "false");
    assert_eq!(render!(true), "true");
    assert_eq!(render!(0), "0");
    assert_eq!(render!(123), "123");
    assert_eq!(render!(0.0), "0");
    assert_eq!(render!(1.23), "1.23");
    assert_eq!(render!("hello"), "hello");

    assert_eq!(
        render_err!([]),
        "expression evaluated to unformattable type list"
    );
    assert_eq!(
        render_err!({}),
        "expression evaluated to unformattable type map"
    );
}

#[test]
fn list() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", helpers::debug);
    let t = engine.compile("{{ value | list | debug }}").unwrap();
    macro_rules! render {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ }).to_string().unwrap()
        };
    }
    macro_rules! render_err {
        ( $($tt:tt)+ ) => {
            t.render(&engine, value! { value: $($tt)+ })
                .to_string().unwrap_err()
                .to_string().replace("function error: ", "")
        };
    }
    assert_eq!(render!("hello"), "[\"h\", \"e\", \"l\", \"l\", \"o\"]");
    assert_eq!(render!([1, 2, 3]), "[1, 2, 3]");
    assert_eq!(render!({ a: 1, b: 2 }), "[[\"a\", 1], [\"b\", 2]]");

    assert_eq!(
        render_err!(None),
        "none value cannot be converted to a list"
    );
    assert_eq!(
        render_err!(false),
        "bool value cannot be converted to a list"
    );
    assert_eq!(
        render_err!(123),
        "integer value cannot be converted to a list"
    );
    assert_eq!(
        render_err!(0.0),
        "float value cannot be converted to a list"
    );
}

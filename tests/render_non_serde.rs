mod helpers;

use upon::{Engine, Value, ValueKey};

use crate::helpers::Writer;

#[test]
fn render_from() {
    let result = Engine::new()
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from(Value::from([(
            "ipsum",
            Value::from([("dolor", Value::String(String::from("test")))]),
        )]))
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
#[allow(clippy::needless_borrow)]
fn render_from_ref() {
    let ctx = Value::from([(
        "ipsum",
        Value::from([("dolor", Value::String(String::from("test")))]),
    )]);
    let result = Engine::new()
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from(&ctx)
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_to_writer_from() {
    let mut w = Writer::new();
    Engine::new()
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_to_writer_from(
            &mut w,
            Value::from([("ipsum", Value::String(String::from("test")))]),
        )
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");
}

#[test]
fn render_with_value_fn() {
    let value_fn = |path: &[ValueKey<'_>]| match path {
        [ValueKey::Map("ipsum"), ValueKey::Map("dolor")] => Ok(Value::String(String::from("test"))),
        _ => Err(String::from("not found")),
    };

    let result = Engine::new()
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_with_value_fn(value_fn)
        .unwrap();
    assert_eq!(result, "lorem test");

    let err = Engine::new()
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_with_value_fn(value_fn)
        .unwrap_err();

    assert_eq!(err.to_string(), "render error: not found");
}

#[test]
fn render_to_writer_with_value_fn() {
    let value_fn = |path: &[ValueKey<'_>]| match path {
        [ValueKey::Map("ipsum"), ValueKey::Map("dolor")] => Ok(Value::String(String::from("test"))),
        _ => Err(String::from("not found")),
    };

    let mut w = Writer::new();
    Engine::new()
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_to_writer_with_value_fn(&mut w, value_fn)
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");

    let mut w = Writer::new();
    let err = Engine::new()
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_to_writer_with_value_fn(&mut w, value_fn)
        .unwrap_err();

    assert_eq!(err.to_string(), "render error: not found");
}

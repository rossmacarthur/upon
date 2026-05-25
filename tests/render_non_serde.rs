mod helpers;

use std::fmt::Write;

use upon::{Engine, Value, ValueAccess, ValueAccessOp, ValueMember};

use crate::helpers::Writer;

#[test]
fn render_from() {
    let engine = Engine::new();
    let result = engine
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from(
            &engine,
            &Value::from([(
                "ipsum",
                Value::from([("dolor", Value::String(String::from("test")))]),
            )]),
        )
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
#[allow(clippy::needless_borrow)]
fn render_from_ref() {
    let engine = Engine::new();
    let ctx = Value::from([(
        "ipsum",
        Value::from([("dolor", Value::String(String::from("test")))]),
    )]);
    let result = engine
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from(&engine, &ctx)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_to_writer_from() {
    let engine = Engine::new();
    let mut w = Writer::new();
    Engine::new()
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_from(
            &engine,
            &Value::from([("ipsum", Value::String(String::from("test")))]),
        )
        .to_writer(&mut w)
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");
}

#[test]
fn render_with_value_fn() {
    let engine = Engine::new();

    let result = engine
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");

    let err = engine
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_string()
        .unwrap_err();

    assert_eq!(err.to_string(), "render error: not found");
}

#[test]
fn render_with_value_fn_optional_access() {
    let engine = Engine::new();

    let result = engine
        .compile(r#"lorem {{ ipsum?.dolor }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");

    let result = Engine::new()
        .compile(r#"lorem {{ ipsum?.sit }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem ");
}

#[test]
fn render_with_value_fn_mut() {
    let engine = Engine::new();

    let mut variables = Vec::new();
    let value_fn_mut = |path: &[ValueMember<'_>]| -> Result<Value, String> {
        let mut s = String::new();
        for member in path {
            match member.op {
                ValueAccessOp::Direct if !s.is_empty() => s.push('.'),
                ValueAccessOp::Optional => s.push_str("?."),
                _ => {}
            }
            match member.access {
                ValueAccess::Key(k) => write!(&mut s, "{k}"),
                ValueAccess::Index(i) => write!(&mut s, "{i}"),
            }
            .map_err(|e| e.to_string())?;
        }
        variables.push(s);
        Ok("test".into())
    };

    let result = engine
        .compile(r#"lorem {{ ?.ipsum.dolor }} {{ ipsum.sit }} {{ ipsum?.amet }}"#)
        .unwrap()
        .render_from_fn(&engine, value_fn_mut)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test test test");
    assert_eq!(variables, ["?.ipsum.dolor", "ipsum.sit", "ipsum?.amet"]);
}

#[test]
fn render_to_writer_with_value_fn() {
    let engine = Engine::new();

    let mut w = Writer::new();
    engine
        .compile(r#"lorem {{ ipsum.dolor }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_writer(&mut w)
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");

    let mut w = Writer::new();
    let err = engine
        .compile(r#"lorem {{ ipsum }}"#)
        .unwrap()
        .render_from_fn(&engine, test_value_fn)
        .to_writer(&mut w)
        .unwrap_err();

    assert_eq!(err.to_string(), "render error: not found");
}

// a test value function that returns "test" for `ipsum.dolor`
fn test_value_fn(path: &[ValueMember<'_>]) -> Result<Value, String> {
    let mut prev_access_op = ValueAccessOp::Direct;
    for (i, s) in ["ipsum", "dolor"].iter().enumerate() {
        let member = match path.get(i) {
            Some(m) => m,
            _ if prev_access_op == ValueAccessOp::Optional => return Ok(Value::None),
            _ => return Err(String::from("not found")),
        };

        match (member.op, member.access) {
            (_, ValueAccess::Key(k)) if k == *s => {}
            (ValueAccessOp::Direct, _) => return Err(String::from("not found")),
            (ValueAccessOp::Optional, _) => return Ok(Value::None),
        }
        prev_access_op = member.op;
    }
    Ok(Value::String(String::from("test")))
}

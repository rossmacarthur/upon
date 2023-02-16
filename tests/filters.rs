#![cfg(feature = "filters")]
#![cfg(feature = "serde")]

use std::collections::BTreeMap;

use upon::{value, Engine, Error, Value};

#[test]
fn render_filter_arity_0() {
    let mut engine = Engine::new();
    engine.add_filter("name", || String::from("John"));
    let result = engine
        .compile("{{ name }}")
        .unwrap()
        .render(value! {})
        .unwrap();
    assert_eq!(result, "John");
}

#[test]
fn render_filter_arity_1() {
    let mut engine = Engine::new();
    engine.add_filter("lower", |v: String| v.to_lowercase());
    let result = engine
        .compile("{{ name | lower }}")
        .unwrap()
        .render(value! { name: "JOHN" })
        .unwrap();
    assert_eq!(result, "john");
}

#[test]
fn render_filter_arity_2() {
    let mut engine = Engine::new();
    engine.add_filter("append", |mut v: String, a: String| {
        v.push_str(&a);
        v
    });
    let result = engine
        .compile(r#"{{ name | append: " Smith" }}"#)
        .unwrap()
        .render(value! { name: "John" })
        .unwrap();
    assert_eq!(result, "John Smith");
}

#[test]
fn render_filter_arity_3() {
    let mut engine = Engine::new();
    engine.add_filter("replace", |v: String, from: String, to: String| {
        v.replace(&from, &to)
    });
    let result = engine
        .compile(r#"{{ name | replace: "Smith", "Newton" }}"#)
        .unwrap()
        .render(value! { name: "John Smith" })
        .unwrap();
    assert_eq!(result, "John Newton");
}

#[test]
fn render_filter_arity_4() {
    let mut engine = Engine::new();
    engine.add_filter(
        "append",
        |mut v: String, a: String, b: String, c: String| {
            v.push_str(&a);
            v.push_str(&b);
            v.push_str(&c);
            v
        },
    );
    let result = engine
        .compile(r#"{{ name | append: " Smith", "!", "!" }}"#)
        .unwrap()
        .render(value! { name: "John" })
        .unwrap();
    assert_eq!(result, "John Smith!!");
}

#[test]
fn render_filter_arity_5() {
    let mut engine = Engine::new();
    engine.add_filter(
        "append",
        |mut v: String, a: String, b: String, c: String, d: String| {
            v.push_str(&a);
            v.push_str(&b);
            v.push_str(&c);
            v.push_str(&d);
            v
        },
    );
    let result = engine
        .compile(r#"{{ name | append: " Smith", "!", "!", "!" }}"#)
        .unwrap()
        .render(value! { name: "John" })
        .unwrap();
    assert_eq!(result, "John Smith!!!");
}

#[test]
fn render_filter_value_types() {
    let mut engine = Engine::new();

    // unit
    engine.add_filter("_", |_: ()| ());

    // bool
    engine.add_filter("_", |_: bool| ());

    // ints
    engine.add_filter("_", |_: u8| ());
    engine.add_filter("_", |_: u16| ());
    engine.add_filter("_", |_: u32| ());
    engine.add_filter("_", |_: u64| ());
    engine.add_filter("_", |_: u128| ());
    engine.add_filter("_", |_: usize| ());
    engine.add_filter("_", |_: i8| ());
    engine.add_filter("_", |_: i16| ());
    engine.add_filter("_", |_: i32| ());
    engine.add_filter("_", |_: i64| ());
    engine.add_filter("_", |_: i128| ());
    engine.add_filter("_", |_: isize| ());

    // floats
    engine.add_filter("_", |_: f32| ());
    engine.add_filter("_", |_: f64| ());

    // strings
    engine.add_filter("_", |_: String| ());
    engine.add_filter("_", |_: &str| ());

    // list
    engine.add_filter("_", |_: Vec<Value>| ());
    engine.add_filter("_", |_: &[Value]| ());

    // map
    engine.add_filter("_", |_: BTreeMap<String, Value>| ());
    engine.add_filter("_", |_: &BTreeMap<String, Value>| ());
}

#[test]
fn render_filter_arg_types() {
    let mut engine = Engine::new();

    // unit
    engine.add_filter("_", |_: Value, _: ()| ());

    // bool
    engine.add_filter("_", |_: Value, _: bool| ());

    // ints
    engine.add_filter("_", |_: Value, _: u8| ());
    engine.add_filter("_", |_: Value, _: u16| ());
    engine.add_filter("_", |_: Value, _: u32| ());
    engine.add_filter("_", |_: Value, _: u64| ());
    engine.add_filter("_", |_: Value, _: u128| ());
    engine.add_filter("_", |_: Value, _: usize| ());
    engine.add_filter("_", |_: Value, _: i8| ());
    engine.add_filter("_", |_: Value, _: i16| ());
    engine.add_filter("_", |_: Value, _: i32| ());
    engine.add_filter("_", |_: Value, _: i64| ());
    engine.add_filter("_", |_: Value, _: i128| ());
    engine.add_filter("_", |_: Value, _: isize| ());

    // floats
    engine.add_filter("_", |_: Value, _: f32| ());
    engine.add_filter("_", |_: Value, _: f64| ());

    // strings
    engine.add_filter("_", |_: Value, _: String| ());
    engine.add_filter("_", |_: Value, _: &str| ());

    // list
    engine.add_filter("_", |_: Value, _: Vec<Value>| ());

    // map
    engine.add_filter("_", |_: Value, _: BTreeMap<String, Value>| ());
}

#[test]
fn render_filter_err_expected_0_args() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: Value| v);
    let err = engine
        .compile("{{ name | test: 123 }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected 0 arguments",
        "
  --> <anonymous>:1:11
   |
 1 | {{ name | test: 123 }}
   |           ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_expected_n_args() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: Value, _: i64, _: i64, _: i64| v);
    let err = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected 3 arguments",
        "
  --> <anonymous>:1:11
   |
 1 | {{ name | test }}
   |           ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_borrowed_value_str() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: &str| v.to_owned());
    let result = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap();
    assert_eq!(result, "John Smith");
}

#[test]
fn render_filter_borrowed_value_list() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: &[Value]| v[0].clone());
    let result = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: ["John", "Smith"] })
        .unwrap();
    assert_eq!(result, "John");
}

#[test]
fn render_filter_borrowed_value_map() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: &BTreeMap<String, Value>| v["john"].to_owned());
    let result = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: { john: "Smith" } })
        .unwrap();
    assert_eq!(result, "Smith");
}

#[test]
fn render_filter_borrowed_value_value() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: &Value| v.clone());
    let result = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap();
    assert_eq!(result, "John Smith");
}

#[test]
fn render_filter_borrowed_arg_str() {
    let mut engine = Engine::new();
    engine.add_filter("concat", |mut name: String, surname: &str| {
        name.push_str(surname);
        name
    });
    let result = engine
        .compile("{{ user.name | concat: user.surname }}")
        .unwrap()
        .render(upon::value! { user: { name: "John", surname: "Smith" }})
        .unwrap();
    assert_eq!(result, "JohnSmith");
}

#[test]
fn render_filter_err_expected_value_type() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: bool| v);
    let err = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected bool value, found string",
        "
  --> <anonymous>:1:11
   |
 1 | {{ name | test }}
   |           ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_expected_arg_type() {
    let mut engine = Engine::new();
    engine.add_filter("test", |v: Value, _: bool| v);
    let err = engine
        .compile("{{ name | test: 123 }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected bool argument, found integer",
        "
  --> <anonymous>:1:17
   |
 1 | {{ name | test: 123 }}
   |                 ^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_expected_value_try_from_int() {
    let mut engine = Engine::new();
    engine.add_filter("add", |a: i8, b: i8| a + b);
    let err = engine
        .compile("{{ age | add: 3 }}")
        .unwrap()
        .render(upon::value! { age: 128 })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected i8 value, but `128` is out of range",
        "
  --> <anonymous>:1:10
   |
 1 | {{ age | add: 3 }}
   |          ^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_expected_arg_reference() {
    let mut engine = Engine::new();
    engine.add_filter("into_owned", |v: Value| v);
    engine.add_filter("prepend", |s1: &str, s2: &str| format!("{s2} {s1}"));
    let err = engine
        .compile(
            "{% for name in names | into_owned %}\n\
             {{ surname | prepend: name }}\n\
             {% endfor %}",
        )
        .unwrap()
        .render(upon::value! {
            names: ["John", "James", "Jimothy"],
            surname: "Smith"
        })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected reference argument but this string can only be passed as owned",
        "
  --> <anonymous>:2:23
   |
 2 | {{ surname | prepend: name }}
   |                       ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_expected_arg_try_from_int() {
    let mut engine = Engine::new();
    engine.add_filter("repeat", |s: &str, i: i8| s.repeat(i as usize));
    let err = engine
        .compile("{{ name | repeat: 128 }}")
        .unwrap()
        .render(upon::value! { name: "John" })
        .unwrap_err();
    assert_err(
        &err,
        "filter expected i8 argument, but `128` is out of range",
        "
  --> <anonymous>:1:19
   |
 1 | {{ name | repeat: 128 }}
   |                   ^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_filter_err_custom() {
    let mut engine = Engine::new();
    engine.add_filter("test", |_: &Value| Err::<bool, _>("test error"));
    let err = engine
        .compile("{{ name | test }}")
        .unwrap()
        .render(upon::value! { name: "John Smith" })
        .unwrap_err();
    assert_filter_err(
        &err,
        "test error",
        "
  --> <anonymous>:1:11
   |
 1 | {{ name | test }}
   |           ^^^^
   |
   = reason: REASON
",
    );
}

#[track_caller]
fn assert_filter_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("filter error: {reason}");
    let display_alt = format!("filter error\n{}", pretty.replace("REASON", reason));
    assert_eq!(err.to_string(), display);
    assert_eq!(format!("{err:#}"), display_alt);
}

#[track_caller]
fn assert_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("render error: {reason}");
    let display_alt = format!("render error\n{}", pretty.replace("REASON", reason));
    assert_eq!(err.to_string(), display);
    assert_eq!(format!("{err:#}"), display_alt);
}

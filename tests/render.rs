#![cfg(feature = "serde")]

mod helpers;

use std::collections::BTreeMap;
use std::error::Error as _;
use std::fmt::Write;
use std::iter::zip;

use upon::fmt;
use upon::{value, Engine, Error, Value};

use crate::helpers::Writer;

#[test]
fn render_comment() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {#- ipsum #} dolor")
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem dolor");
}

#[test]
fn render_inline_expr_primitive() {
    let tests = &[
        (value! { ipsum: false  }, "lorem false"),
        (value! { ipsum: true  }, "lorem true"),
        (value! { ipsum: 123_i32  }, "lorem 123"),
        (value! { ipsum: 123_i64  }, "lorem 123"),
        (value! { ipsum: 123.4_f64  }, "lorem 123.4"),
        (value! { ipsum: "dolor" }, "lorem dolor"),
    ];

    let engine = Engine::new();
    let template = engine.compile("lorem {{ ipsum }}").unwrap();
    for (value, exp) in tests {
        let result = template.render(&engine, value).to_string().unwrap();
        assert_eq!(result, *exp);
    }
}

#[test]
fn render_inline_expr_literal_roundtrip() {
    let tests = &[
        ("true", "true"),
        ("false", "false"),
        ("123", "123"),
        ("+123", "123"),
        ("-123", "-123"),
        ("0x1f", "31"),
        ("0o777", "511"),
        ("0b1010", "10"),
        ("3.", "3"),
        ("+3.", "3"),
        ("-3.", "-3"),
        ("3.14", "3.14"),
        ("+3.14", "3.14"),
        ("-3.14", "-3.14"),
        ("3.14e2", "314"),
        ("+3.14e2", "314"),
        ("-3.14e2", "-314"),
        ("3.14e+2", "314"),
        ("+3.14e+2", "314"),
        ("-3.14e+2", "-314"),
        ("314e-2", "3.14"),
        ("+314e-2", "3.14"),
        ("-314e-2", "-3.14"),
    ];
    let engine = Engine::new();
    for (arg, exp) in tests {
        let result = engine
            .compile(&format!("{{{{ {arg} }}}}"))
            .unwrap()
            .render(&engine, Value::None)
            .to_string()
            .unwrap();
        assert_eq!(result, *exp);
    }
}

#[test]
fn render_inline_expr_literal_string() {
    let engine = Engine::new();
    let result = engine
        .compile(r#"lorem {{ "test" }}"#)
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_inline_expr_literal_string_escaped() {
    let engine = Engine::new();
    let result = engine
        .compile(r#"lorem {{ "escaped \n \r \t \\ \"" }}"#)
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem escaped \n \r \t \\ \"");
}

#[cfg(feature = "filters")]
#[test]
fn render_inline_expr_literal_with_filter() {
    let mut engine = Engine::new();
    engine.add_filter("ipsum", str::to_uppercase);
    let result = engine
        .compile(r#"lorem {{ "test" | ipsum }}"#)
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem TEST");
}

#[test]
fn render_inline_expr_map_key() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "sit"} })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem sit");
}

#[test]
fn render_inline_expr_map_optional_key() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum?.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "sit"} })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem sit");
}

#[test]
fn render_inline_expr_map_optional_key_chain() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum?.dolor.sit }}")
        .unwrap()
        .render(&engine, value! { ipsum: { } })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem ");
}

#[test]
fn render_inline_expr_map_optional_key_chain_long() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum.dolor?.sit.amet }}")
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: {} } })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem ");
}

#[cfg(feature = "unicode")]
#[test]
fn render_inline_expr_map_index_unicode_ident() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum.привіт }}")
        .unwrap()
        .render(&engine, value! { ipsum: { привіт: "sit"} })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem sit");
}

#[test]
fn render_inline_expr_list_index() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {{ ipsum.1 }}")
        .unwrap()
        .render(&engine, value! { ipsum: ["sit", "amet"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem amet");
}

#[test]
fn render_inline_expr_custom_formatter() {
    let mut engine = Engine::new();
    engine.add_formatter("format_list", format_list);
    let result = engine
        .compile("lorem {{ ipsum | format_list }}")
        .unwrap()
        .render(&engine, value! { ipsum: ["sit", "amet"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem sit;amet");
}

#[test]
fn render_inline_expr_default_formatter_err() {
    let mut engine = Engine::new();
    engine.set_default_formatter(&format_list);
    let err = engine
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(&engine, value! { ipsum: { sit: "amet"} })
        .to_string()
        .unwrap_err();
    assert_format_err(
        &err,
        "expected list",
        "
  --> <anonymous>:1:10
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_custom_formatter_err() {
    let mut engine = Engine::new();
    engine.add_formatter("format_list", format_list);
    let err = engine
        .compile("lorem {{ ipsum | format_list }}")
        .unwrap()
        .render(&engine, value! { ipsum: { sit: "amet"} })
        .to_string()
        .unwrap_err();
    assert_format_err(
        &err,
        "expected list",
        "
  --> <anonymous>:1:18
   |
 1 | lorem {{ ipsum | format_list }}
   |                  ^^^^^^^^^^^
   |
   = reason: REASON
",
    );
}

fn format_list(f: &mut fmt::Formatter<'_>, v: &Value) -> fmt::Result {
    match v {
        Value::List(list) => {
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    f.write_char(';')?;
                }
                fmt::default(f, item)?;
            }
            Ok(())
        }
        _ => Err("expected list".to_string())?,
    }
}

#[test]
fn render_inline_expr_err_unknown_filter_or_formatter() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum | unknown }}")
        .unwrap()
        .render(&engine, value! { ipsum: true })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "unknown filter or formatter",
        "
  --> <anonymous>:1:18
   |
 1 | lorem {{ ipsum | unknown }}
   |                  ^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_unknown_filter_found_formatter() {
    let mut engine = Engine::new();
    engine.add_formatter("another", |_, _| Ok(()));
    let err = engine
        .compile("lorem {{ ipsum | another | unknown }}")
        .unwrap()
        .render(&engine, value! { ipsum: true })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "expected filter, found formatter",
        "
  --> <anonymous>:1:18
   |
 1 | lorem {{ ipsum | another | unknown }}
   |                  ^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_unknown_filter() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum | another | unknown }}")
        .unwrap()
        .render(&engine, value! { ipsum: true })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "unknown filter",
        "
  --> <anonymous>:1:18
   |
 1 | lorem {{ ipsum | another | unknown }}
   |                  ^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_unrenderable() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(&engine, value! { ipsum: {} })
        .to_string()
        .unwrap_err();
    assert_format_err(
        &err,
        "expression evaluated to unformattable type map",
        "
  --> <anonymous>:1:10
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_none() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: None })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "none does not support key-based access",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.dolor }}
   |               ^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_string() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: "testing..." })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "string does not support key-based access",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.dolor }}
   |               ^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_list_with_string() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: ["test", "ing..."] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "list does not support key-based access",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.dolor }}
   |               ^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_map_with_integer() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.123 }}")
        .unwrap()
        .render(&engine, value! { ipsum: { test: "ing...", } })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "map does not support integer-based access",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.123 }}
   |               ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_index_out_of_bounds() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.123 }}")
        .unwrap()
        .render(&engine, value! { ipsum: ["test", "ing..."] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "index out of bounds, the length is 2",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.123 }}
   |               ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_not_found_in_map() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum : { } })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in map",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum.dolor }}
   |               ^^^^^^
   |
   = reason: REASON
",
    );
}

fn falsy() -> Vec<Value> {
    vec![
        Value::None,
        Value::Bool(false),
        Value::Integer(0),
        Value::Float(0.0),
        Value::String(String::new()),
        Value::List(vec![]),
        Value::Map(BTreeMap::new()),
    ]
}

fn truthy() -> Vec<Value> {
    vec![
        Value::Bool(true),
        Value::Integer(1337),
        Value::Float(13.37),
        Value::String("testing".into()),
        Value::from([1, 2, 3]),
        Value::from([("a", 1i64), ("b", 2i64)]),
    ]
}

#[test]
fn render_if_statement_cond_true() {
    for value in truthy() {
        let engine = Engine::new();
        let result = engine
            .compile("lorem {% if ipsum %}{{ sit }}{% else %}{{ amet }}{% endif %}")
            .unwrap()
            .render(&engine, value! { ipsum: value.clone(), sit: "consectetur" })
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem consectetur");
    }
}

#[test]
fn render_if_statement_cond_false() {
    for value in falsy() {
        let engine = Engine::new();
        let result = engine
            .compile("lorem {% if ipsum.dolor %}{{ sit }}{% else %}{{ amet }}{% endif %}")
            .unwrap()
            .render(
                &engine,
                value! { ipsum: { dolor: value.clone() }, amet: "consectetur" },
            )
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem consectetur");
    }
}

#[test]
fn render_if_statement_cond_not() {
    for value in falsy() {
        let engine = Engine::new();
        let result = engine
            .compile("lorem {% if not ipsum.dolor %}{{ sit }}{% else %}{{ amet }}{% endif %}")
            .unwrap()
            .render(
                &engine,
                value! { ipsum: {dolor: value.clone()}, sit: "consectetur" },
            )
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem consectetur");
    }
}

#[test]
fn render_if_statement_else_if_cond_false() {
    for value in falsy() {
        let engine = Engine::new();
        let result = engine
            .compile("lorem {% if ipsum %} dolor {% else if sit %} amet {% endif %}, consectetur")
            .unwrap()
            .render(&engine, value! { ipsum: value.clone(), sit: value.clone() })
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem , consectetur");
    }
}

#[test]
fn render_if_statement_else_if_cond_true() {
    for (t, f) in zip(truthy(), falsy()) {
        let engine = Engine::new();
        let result = engine
            .compile("lorem {% if ipsum %} dolor {% else if sit %} amet {% endif %}, consectetur")
            .unwrap()
            .render(&engine, value! { ipsum: f, sit: t })
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem  amet , consectetur");
    }
}

#[test]
fn render_if_statement_else_if_cond_not() {
    for falsy in falsy() {
        let engine = Engine::new();
        let result = engine
            .compile(
                "lorem {% if ipsum %} dolor {% else if not sit %} amet {% endif %}, consectetur",
            )
            .unwrap()
            .render(&engine, value! { ipsum: falsy.clone(), sit: falsy })
            .to_string()
            .unwrap();
        assert_eq!(result, "lorem  amet , consectetur");
    }
}

#[test]
fn render_if_statement_multi() {
    let engine = Engine::new();
    let template = engine
        .compile(
            r#"
{%- if a -%} a
{%- else if b -%} b
{%- else if c -%} c
{%- else if d -%} d
{%- else if e -%} e
{%- else -%} f
{%- endif -%}
"#,
        )
        .unwrap();

    let mut map = BTreeMap::from([
        ("a", false),
        ("b", false),
        ("c", false),
        ("d", false),
        ("e", false),
    ]);
    let result = template.render(&engine, &map).to_string().unwrap();
    assert_eq!(result, "f");
    for var in ["a", "b", "c", "d", "e"] {
        map.insert(var, true);
        let result = template.render(&engine, &map).to_string().unwrap();
        assert_eq!(result, var);
        map.insert(var, false);
    }
}

#[test]
fn render_for_statement_list() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[cfg(feature = "filters")]
#[test]
fn render_for_statement_filtered_list() {
    let mut engine = Engine::new();
    engine.add_filter("pop", |mut list: Vec<Value>| {
        list.pop();
        list
    });
    let result = engine
        .compile("lorem {% for ipsum in dolor | pop %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem tes");
}

#[test]
fn render_for_statement_map() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {% for ipsum, dolor in sit %}{{ ipsum }},{{ dolor.0 }} {% endfor %}")
        .unwrap()
        .render(
            &engine,
            value! { sit: { a: ["t"], b: ["e"], c: ["s"], d: ["t"] } },
        )
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem a,t b,e c,s d,t ");
}

#[test]
fn render_for_statement_loop_fields() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {% for ipsum in dolor %}{{ loop.index }},{{ loop.first }},{{ loop.last }},{{ ipsum }}  {% endfor %}")
        .unwrap()
        .render(&engine, value!{ dolor: ["t", "e", "s", "t"] }).to_string().unwrap();
    assert_eq!(
        result,
        "lorem 0,true,false,t  1,false,false,e  2,false,false,s  3,false,true,t  "
    );
}

#[test]
fn render_for_statement_loop_optional_access() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {% for ipsum in dolor %}{{ loop?.notindex }}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_for_statement_loop_map() {
    let mut engine = Engine::new();
    engine.add_formatter("debug", |f, v| {
        writeln!(f, "{v:?}")?;
        Ok(())
    });
    let result = engine
        .compile("lorem {% for ipsum in dolor %} {{ loop | debug }} {% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(
        result,
        r#"lorem  Map({"first": Bool(true), "index": Integer(0), "last": Bool(false)})
  Map({"first": Bool(false), "index": Integer(1), "last": Bool(false)})
  Map({"first": Bool(false), "index": Integer(2), "last": Bool(false)})
  Map({"first": Bool(false), "index": Integer(3), "last": Bool(true)})
 "#
    );
}

#[test]
fn render_err_contains_template_name() {
    let mut engine = Engine::new();
    engine.add_template("test", "{{ ipsum }}").unwrap();
    let err = engine
        .template("test")
        .render(value! {})
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        "
  --> test:1:4
   |
 1 | {{ ipsum }}
   |    ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_not_found_in_map() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum in dolor %} {{ loop.xxx }} {% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in map",
        "
  --> <anonymous>:1:39
   |
 1 | lorem {% for ipsum in dolor %} {{ loop.xxx }} {% endfor %}
   |                                       ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_cannot_index_into_map() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum in dolor %} {{ loop.123 }} {% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "cannot index into map with integer",
        "
  --> <anonymous>:1:39
   |
 1 | lorem {% for ipsum in dolor %} {{ loop.123 }} {% endfor %}
   |                                       ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_cannot_index_into_string() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum, dolor in sit %} {{ ipsum.xxx }} {% endfor %}")
        .unwrap()
        .render(&engine, value! { sit: {t: "e", s: "t"} })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "cannot index into string",
        "
  --> <anonymous>:1:45
   |
 1 | lorem {% for ipsum, dolor in sit %} {{ ipsum.xxx }} {% endfor %}
   |                                             ^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_cannot_index_into_loop_field() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum in dolor %} {{ loop.first.xxx }} {% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "cannot index into bool",
        "
  --> <anonymous>:1:45
   |
 1 | lorem {% for ipsum in dolor %} {{ loop.first.xxx }} {% endfor %}
   |                                             ^^^^
   |
   = reason: REASON
",
    );
}

#[cfg(feature = "filters")]
#[test]
fn render_for_statement_filtered_map() {
    let mut engine = Engine::new();
    engine.add_filter("rm", |mut map: BTreeMap<String, Value>, key: &str| {
        map.remove(key);
        map
    });
    let result = engine
        .compile(r#"lorem {% for ipsum, dolor in sit | rm: "d" %}{{ ipsum }},{{ dolor.0 }} {% endfor %}"#)
        .unwrap()
        .render(&engine, value!{ sit: { a: ["t"], b: ["e"], c: ["s"], d: ["t"] } }).to_string().unwrap();
    assert_eq!(result, "lorem a,t b,e c,s ");
}

#[test]
fn render_for_statement_nested_borrowed_list() {
    let mut engine = Engine::new();
    engine.add_template("nested", "lorem {{ ipsum }} ").unwrap();
    let result = engine
        .compile(r#"lorem {% for ipsum in dolor %}{% include "nested" %}{% endfor %}"#)
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem lorem t lorem e lorem s lorem t ");
}

#[cfg(feature = "filters")]
#[test]
fn render_for_statement_nested_owned_list() {
    let mut engine = Engine::new();
    engine.add_filter("to_owned", Value::to_owned);
    engine.add_template("nested", "lorem {{ ipsum }} ").unwrap();
    let result = engine
        .compile(r#"lorem {% for ipsum in dolor | to_owned %}{% include "nested" %}{% endfor %}"#)
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem lorem t lorem e lorem s lorem t ");
}

#[test]
fn render_for_statement_nested_borrowed_map() {
    let mut engine = Engine::new();
    engine
        .add_template("nested", "lorem {{ ipsum }} {{ dolor }} ")
        .unwrap();
    let result = engine
        .compile(r#"lorem {% for ipsum, dolor in sit %}{% include "nested" %}{% endfor %}"#)
        .unwrap()
        .render(&engine, value! { sit: { a: "t", b: "e", c: "s", d: "t" } })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem lorem a t lorem b e lorem c s lorem d t ");
}

#[cfg(feature = "filters")]
#[test]
fn render_for_statement_nested_owned_map() {
    let mut engine = Engine::new();
    engine.add_filter("to_owned", Value::to_owned);
    engine
        .add_template("nested", "lorem {{ ipsum }} {{ dolor }} ")
        .unwrap();
    let result = engine
        .compile(
            r#"lorem {% for ipsum, dolor in sit | to_owned %}{% include "nested" %}{% endfor %}"#,
        )
        .unwrap()
        .render(&engine, value! { sit: { a: "t", b: "e", c: "s", d: "t" } })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem lorem a t lorem b e lorem c s lorem d t ");
}

#[test]
fn render_for_statement_nested_loop_fields() {
    let mut engine = Engine::new();
    engine
        .add_template(
            "nested",
            "{{ loop.index }},{{ loop.first }},{{ loop.last }},{{ ipsum }}",
        )
        .unwrap();
    let result = engine
        .compile(r#"lorem {% for ipsum in dolor %}{% include "nested" %} {% endfor %}"#)
        .unwrap()
        .render(&engine, value! { dolor: ["t", "e", "s", "t"] })
        .to_string()
        .unwrap();
    assert_eq!(
        result,
        "lorem 0,true,false,t 1,false,false,e 2,false,false,s 3,false,true,t "
    );
}

#[test]
fn render_for_statement_err_not_iterable() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: true })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "expected iterable, but expression evaluated to bool",
        "
  --> <anonymous>:1:23
   |
 1 | lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}
   |                       ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_list_with_two_vars() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: ["sit", "amet"] })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "cannot unpack list item into two variables",
        "
  --> <anonymous>:1:14
   |
 1 | lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}
   |              ^^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_map_with_one_var() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(&engine, value! { dolor: { sit: "amet" }})
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "cannot unpack map item into one variable",
        "
  --> <anonymous>:1:14
   |
 1 | lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}
   |              ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_err_loop_var_scope() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% for _, ipsum in dolor %}{% endfor %}{{ ipsum }}")
        .unwrap()
        .render(&engine, value! { dolor: { ipsum: false }})
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        "
  --> <anonymous>:1:49
   |
 1 | lorem {% for _, ipsum in dolor %}{% endfor %}{{ ipsum }}
   |                                                 ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_with_statement() {
    let engine = Engine::new();
    let result = engine
        .compile("lorem {% with ipsum as dolor %}{{ dolor }}{% endwith %} sit")
        .unwrap()
        .render(&engine, value! { ipsum: "test", dolor: false })
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test sit")
}

#[test]
fn render_with_statement_err_var_scope() {
    let engine = Engine::new();
    let err = engine
        .compile("lorem {% with ipsum as dolor %}{{ dolor }}{% endwith %}{{ dolor }}")
        .unwrap()
        .render(&engine, value! { ipsum: "test" })
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        "
  --> <anonymous>:1:59
   |
 1 | lorem {% with ipsum as dolor %}{{ dolor }}{% endwith %}{{ dolor }}
   |                                                           ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_include_statement() {
    let mut engine = Engine::new();
    engine.add_template("nested", "{{ ipsum.dolor }}").unwrap();
    let result = engine
        .compile(r#"lorem {% include "nested" %} sit"#)
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "test" }})
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test sit");
}

#[test]
fn render_include_with_statement() {
    let mut engine = Engine::new();
    engine.add_template("nested", "{{ dolor }}").unwrap();
    let result = engine
        .compile(r#"lorem {% include "nested" with ipsum %} sit"#)
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "test" }})
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test sit");
}

#[cfg(feature = "filters")]
#[test]
fn render_include_with_statement_owned() {
    let mut engine = Engine::new();
    engine.add_filter("to_owned", Value::to_owned);
    engine.add_template("nested", "{{ dolor }}").unwrap();
    let result = engine
        .compile(r#"lorem {% include "nested" with ipsum | to_owned %} sit"#)
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "test" }})
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test sit");
}

#[test]
fn render_include_statement_parent_template_scope() {
    let mut engine = Engine::new();
    engine.add_template("nested", "{{ ipsum.dolor }}").unwrap();
    let result = engine
        .compile(r#"lorem {% include "nested" %} sit"#)
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "test" }})
        .to_string()
        .unwrap();
    assert_eq!(result, "lorem test sit");
}

#[test]
fn render_include_statement_err_parent_template_scope() {
    let mut engine = Engine::new();
    engine.add_template("nested", "{{ ipsum.dolor }}").unwrap();
    let err = engine
        .compile(r#"lorem {% include "nested" with ipsum %} sit"#)
        .unwrap()
        .render(&engine, value! { ipsum: { dolor: "test" }})
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        r#"
  --> nested:1:4
   |
 1 | {{ ipsum.dolor }}
   |    ^^^^^
   |
   = reason: REASON
"#,
    );
}

#[test]
fn render_include_statement_err_unknown_template() {
    let engine = Engine::new();
    let err = engine
        .compile(r#"lorem {% include "nested" %} sit"#)
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap_err();
    assert_err(
        &err,
        "unknown template",
        r#"
  --> <anonymous>:1:18
   |
 1 | lorem {% include "nested" %} sit
   |                  ^^^^^^^^
   |
   = reason: REASON
"#,
    );
}

#[test]
fn render_include_statement_err_max_include_depth() {
    let mut engine = Engine::new();
    engine
        .add_template("cycle", r#"{% include "cycle" %}"#)
        .unwrap();
    let err = engine
        .template("cycle")
        .render(Value::None)
        .to_string()
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "render error: reached maximum include depth (64)"
    );
}

#[test]
fn render_include_statement_err_max_include_depth_renderer() {
    let mut engine = Engine::new();
    engine.set_max_include_depth(128);
    engine
        .add_template("cycle", r#"{% include "cycle" %}"#)
        .unwrap();
    let err = engine
        .template("cycle")
        .render(Value::None)
        .with_max_include_depth(4)
        .to_string()
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "render error: reached maximum include depth (4)"
    );
}

#[test]
fn render_include_with_statement_inside_with_statement() {
    let mut engine = Engine::new();
    engine.add_template("nested", "").unwrap();
    engine
        .compile(r#"{% with false as x %} {% include "nested" with false %} {% endwith %}"#)
        .unwrap()
        .render(&engine, Value::None)
        .to_string()
        .unwrap();
}

#[test]
fn render_to_writer() {
    let engine = Engine::new();
    let mut w = Writer::new();
    engine
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(&engine, value! { ipsum : "test" })
        .to_writer(&mut w)
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");
}

#[test]
fn render_to_writer_err_io() {
    let engine = Engine::new();
    let mut w = Writer::with_max(1);
    let err = engine
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(&engine, value! { ipsum : "test" })
        .to_writer(&mut w)
        .unwrap_err();
    assert_eq!(format!("{err:#}"), "io error");
    assert_eq!(format!("{:#}", err.source().unwrap()), "address in use");
}

#[test]
fn render_to_writer_err_not_io() {
    let engine = Engine::new();
    let mut w = Writer::with_max(1);
    let err = engine
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(&engine, value! { dolor : "test" })
        .to_writer(&mut w)
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        "
  --> <anonymous>:1:10
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^
   |
   = reason: REASON
",
    );
}

#[track_caller]
fn assert_format_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("format error: {reason}");
    let display_alt = format!("format error\n{}", pretty.replace("REASON", reason));
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

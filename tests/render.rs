use std::collections::BTreeMap;
use std::error::Error as _;
use std::fmt::Write;
use std::io;

use upon::{value, Engine, Error, Formatter, Result, Value};

#[test]
fn render_comment() {
    let result = Engine::new()
        .compile("lorem {#- ipsum #} dolor")
        .unwrap()
        .render(Value::None)
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
        let result = template.render(value).unwrap();
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
            .compile(&format!("{{{{ {} }}}}", arg))
            .unwrap()
            .render(Value::None)
            .unwrap();
        assert_eq!(result, *exp);
    }
}

#[test]
fn render_inline_expr_literal_string() {
    let result = Engine::new()
        .compile(r#"lorem {{ "test" }}"#)
        .unwrap()
        .render(Value::None)
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_inline_expr_literal_string_escaped() {
    let result = Engine::new()
        .compile(r#"lorem {{ "escaped \n \r \t \\ \"" }}"#)
        .unwrap()
        .render(Value::None)
        .unwrap();
    assert_eq!(result, "lorem escaped \n \r \t \\ \"");
}

#[test]
fn render_inline_expr_literal_with_filter() {
    let mut engine = Engine::new();
    engine.add_filter("ipsum", str::to_uppercase);
    let result = engine
        .compile(r#"lorem {{ "test" | ipsum }}"#)
        .unwrap()
        .render(Value::None)
        .unwrap();
    assert_eq!(result, "lorem TEST");
}

#[test]
fn render_inline_expr_map_index() {
    let result = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: { dolor: "sit"} })
        .unwrap();
    assert_eq!(result, "lorem sit");
}

#[test]
fn render_inline_expr_list_index() {
    let result = Engine::new()
        .compile("lorem {{ ipsum.1 }}")
        .unwrap()
        .render(value! { ipsum: ["sit", "amet"] })
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
        .render(value! { ipsum: ["sit", "amet"] })
        .unwrap();
    assert_eq!(result, "lorem sit;amet");
}

#[test]
fn render_inline_expr_custom_formatter_err() {
    let mut engine = Engine::new();
    engine.add_formatter("format_list", format_list);
    let err = engine
        .compile("lorem {{ ipsum | format_list }}")
        .unwrap()
        .render(value! { ipsum: { sit: "amet"} })
        .unwrap_err();
    assert_err(
        &err,
        "expected list",
        "
  --> <anonymous>:1:10
   |
 1 | lorem {{ ipsum | format_list }}
   |          ^^^^^^^^^^^^^^^^^^^
   |
   = reason: REASON
",
    );
}

fn format_list(f: &mut Formatter<'_>, v: &Value) -> Result<()> {
    match v {
        Value::List(list) => {
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    f.write_char(';')?;
                }
                upon::format(f, item)?;
            }
            Ok(())
        }
        _ => Err("expected list".to_string())?,
    }
}

#[test]
fn render_inline_expr_err_unknown_filter_or_formatter() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | unknown }}")
        .unwrap()
        .render(value! { ipsum: true })
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
        .render(value! { ipsum: true })
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
    let err = Engine::new()
        .compile("lorem {{ ipsum | another | unknown }}")
        .unwrap()
        .render(value! { ipsum: true })
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
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: {} })
        .unwrap_err();
    assert_err(
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
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: None })
        .unwrap_err();
    assert_err(
        &err,
        "cannot index into none",
        "
  --> <anonymous>:1:16
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: "testing..." })
        .unwrap_err();
    assert_err(
        &err,
        "cannot index into string",
        "
  --> <anonymous>:1:16
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_cannot_index_list_with_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: ["test", "ing..."] })
        .unwrap_err();
    assert_err(
        &err,
        "cannot index list with string",
        "
  --> <anonymous>:1:16
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_inline_expr_err_not_found_in_map() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum : { } })
        .unwrap_err();
    assert_err(
        &err,
        "not found in map",
        "
  --> <anonymous>:1:16
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_if_statement_cond_true() {
    let result = Engine::new()
        .compile("lorem {% if ipsum.dolor %}{{ sit }}{% else %}{{ amet }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: true }, sit: "consectetur" })
        .unwrap();
    assert_eq!(result, "lorem consectetur")
}

#[test]
fn render_if_statement_cond_false() {
    let result = Engine::new()
        .compile("lorem {% if ipsum.dolor %}{{ sit }}{% else %}{{ amet }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: false }, amet: "consectetur" })
        .unwrap();
    assert_eq!(result, "lorem consectetur")
}

#[test]
fn render_if_statement_cond_not() {
    let result = Engine::new()
        .compile("lorem {% if not ipsum.dolor %}{{ sit }}{% else %}{{ amet }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: false }, sit: "consectetur" })
        .unwrap();
    assert_eq!(result, "lorem consectetur")
}

#[test]
fn render_if_statement_else_if_cond_false() {
    let result = Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else if sit %} amet {% endif %}, consectetur")
        .unwrap()
        .render(value! { ipsum: false, sit: false })
        .unwrap();
    assert_eq!(result, "lorem , consectetur");
}

#[test]
fn render_if_statement_else_if_cond_true() {
    let result = Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else if sit %} amet {% endif %}, consectetur")
        .unwrap()
        .render(value! { ipsum: false, sit: true })
        .unwrap();
    assert_eq!(result, "lorem  amet , consectetur");
}

#[test]
fn render_if_statement_else_if_cond_not() {
    let result = Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else if not sit %} amet {% endif %}, consectetur")
        .unwrap()
        .render(value! { ipsum: false, sit: false })
        .unwrap();
    assert_eq!(result, "lorem  amet , consectetur");
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
    let result = template.render(&map).unwrap();
    assert_eq!(result, "f");
    for var in ["a", "b", "c", "d", "e"] {
        map.insert(var, true);
        let result = template.render(&map).unwrap();
        assert_eq!(result, var);
        map.insert(var, false);
    }
}

#[test]
fn render_if_statement_err_cond_not_bool() {
    let err = Engine::new()
        .compile("lorem {% if ipsum.dolor %}{{ sit }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: { } } })
        .unwrap_err();
    assert_err(
        &err,
        "expected bool, but expression evaluated to map",
        "
  --> <anonymous>:1:13
   |
 1 | lorem {% if ipsum.dolor %}{{ sit }}{% endif %}
   |             ^^^^^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_if_statement_err_cond_not_not_bool() {
    let err = Engine::new()
        .compile("lorem {% if not ipsum.dolor %}{{ sit }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: { } } })
        .unwrap_err();
    assert_err(
        &err,
        "expected bool, but expression evaluated to map",
        "
  --> <anonymous>:1:17
   |
 1 | lorem {% if not ipsum.dolor %}{{ sit }}{% endif %}
   |                 ^^^^^^^^^^^
   |
   = reason: REASON
",
    );
}

#[test]
fn render_for_statement_list() {
    let result = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: ["t", "e", "s", "t"] })
        .unwrap();
    assert_eq!(result, "lorem test");
}

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
        .render(value! { dolor: ["t", "e", "s", "t"] })
        .unwrap();
    assert_eq!(result, "lorem tes");
}

#[test]
fn render_for_statement_map() {
    let result = Engine::new()
        .compile("lorem {% for ipsum, dolor in sit %}{{ ipsum }},{{ dolor.0 }} {% endfor %}")
        .unwrap()
        .render(value! { sit: { a: ["t"], b: ["e"], c: ["s"], d: ["t"] } })
        .unwrap();
    assert_eq!(result, "lorem a,t b,e c,s d,t ");
}

#[test]
fn render_for_statement_loop_index() {
    let result = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ loop.index }},{{ loop.first }},{{ loop.last }},{{ ipsum }}  {% endfor %}")
        .unwrap()
        .render(value! { dolor: ["t", "e", "s", "t"] })
        .unwrap();
    assert_eq!(
        result,
        "lorem 0,true,false,t  1,false,false,e  2,false,false,s  3,false,true,t  "
    );
}

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
        .render(value! { sit: { a: ["t"], b: ["e"], c: ["s"], d: ["t"] } })
        .unwrap();
    assert_eq!(result, "lorem a,t b,e c,s ");
}

#[test]
fn render_for_statement_err_not_iterable() {
    let err = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: true })
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
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: ["sit", "amet"] })
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
    let err = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: { sit: "amet" }})
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
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %}{% endfor %}{{ ipsum }}")
        .unwrap()
        .render(value! { dolor: { ipsum: false }})
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
    let result = Engine::new()
        .compile("lorem {% with ipsum as dolor %}{{ dolor }}{% endwith %} sit")
        .unwrap()
        .render(value! { ipsum: "test", dolor: false })
        .unwrap();
    assert_eq!(result, "lorem test sit")
}

#[test]
fn render_with_statement_err_var_scope() {
    let err = Engine::new()
        .compile("lorem {% with ipsum as dolor %}{{ dolor }}{% endwith %}{{ dolor }}")
        .unwrap()
        .render(value! { ipsum: "test" })
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
        .render(value! { ipsum: { dolor: "test" }})
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
        .render(value! { ipsum: { dolor: "test" }})
        .unwrap();
    assert_eq!(result, "lorem test sit");
}

#[test]
fn render_include_with_statement_owned() {
    let mut engine = Engine::new();
    engine.add_filter("to_owned", Value::to_owned);
    engine.add_template("nested", "{{ dolor }}").unwrap();
    let result = engine
        .compile(r#"lorem {% include "nested" with ipsum | to_owned %} sit"#)
        .unwrap()
        .render(value! { ipsum: { dolor: "test" }})
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
        .render(value! { ipsum: { dolor: "test" }})
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
        .render(value! { ipsum: { dolor: "test" }})
        .unwrap_err();
    assert_err(
        &err,
        "not found in this scope",
        r#"
  --> <anonymous>:1:4
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
    let err = Engine::new()
        .compile(r#"lorem {% include "nested" %} sit"#)
        .unwrap()
        .render(Value::None)
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
fn render_include_statement_err_maximum_depth() {
    let mut engine = Engine::new();
    engine
        .add_template("cycle", r#"{% include "cycle" %}"#)
        .unwrap();
    let err = engine
        .get_template("cycle")
        .unwrap()
        .render(Value::None)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "render error: reached maximum include depth (64)"
    );
}

#[test]
fn render_include_with_statement_inside_with_statement() {
    let mut engine = Engine::new();
    engine.add_template("nested", "").unwrap();
    engine
        .compile(r#"{% with false as x %} {% include "nested" with false %} {% endwith %}"#)
        .unwrap()
        .render(Value::None)
        .unwrap();
}

#[test]
fn render_to_writer() {
    let mut w = Writer::new();
    Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render_to_writer(&mut w, value! { ipsum : "test" })
        .unwrap();
    assert_eq!(w.into_string(), "lorem test");
}

#[test]
fn render_to_writer_err_io() {
    let mut w = Writer::with_max(1);
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render_to_writer(&mut w, value! { ipsum : "test" })
        .unwrap_err();
    assert_eq!(format!("{:#}", err), "io error");
    assert_eq!(format!("{:#}", err.source().unwrap()), "address in use");
}

#[test]
fn render_to_writer_err_not_io() {
    let mut w = Writer::with_max(1);
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render_to_writer(&mut w, value! { dolor : "test" })
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

#[derive(Default)]
struct Writer {
    buf: Vec<u8>,
    count: usize,
    max: usize,
}

impl Writer {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            count: 0,
            max: !0,
        }
    }

    fn with_max(max: usize) -> Self {
        Self {
            buf: Vec::new(),
            count: 0,
            max,
        }
    }

    #[track_caller]
    fn into_string(self) -> String {
        String::from_utf8(self.buf).unwrap()
    }
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += 1;
        if self.count > self.max {
            return Err(io::Error::from(io::ErrorKind::AddrInUse));
        }
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[track_caller]
fn assert_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("render error: {}", reason);
    let display_alt = format!("render error\n{}", pretty.replace("REASON", reason));
    assert_eq!(err.to_string(), display);
    assert_eq!(format!("{:#}", err), display_alt);
}

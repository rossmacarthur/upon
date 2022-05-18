use upon::{value, Engine, Value};

#[test]
fn render_inline_expr_normal() {
    let result = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: "dolor" })
        .unwrap();
    assert_eq!(result, "lorem dolor");
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
fn render_inline_expr_err_unknown_function() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | unknown }}")
        .unwrap()
        .render(Value::None)
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | unknown }}
   |                  ^^^^^^^ unknown filter function
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_none() {
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(Value::None)
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^ cannot index into none
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(Value::from("test"))
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^ cannot index into string
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_list_with_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(Value::from(["test", "ing..."]))
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^ cannot index list with string
"
    );
}

#[test]
fn render_inline_expr_err_not_found_in_map() {
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { dolor: "testing..." })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^ not found in map
"
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
fn render_if_statement_err_cond_not_bool() {
    let err = Engine::new()
        .compile("lorem {% if ipsum.dolor %}{{ sit }}{% endif %}")
        .unwrap()
        .render(value! { ipsum: { dolor: { } } })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% if ipsum.dolor %}{{ sit }}{% endif %}
   |             ^^^^^^^^^^^ expected bool, but expression evaluated to map
"
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
#[ignore]
fn render_for_statement_map() {
    // FIXME: enable when indexmap
    let result = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: { a: "t", b: "e", c: "s", d: "t" } })
        .unwrap();
    assert_eq!(result, "lorem test");
}

#[test]
fn render_for_statement_err_not_iterable() {
    let err = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: true })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}
   |                       ^^^^^ expected iterable, but expression evaluated to bool
"
    );
}

#[test]
fn render_for_statement_err_list_with_two_vars() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: ["sit", "amet"] })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for _, ipsum in dolor %}{{ ipsum }}{% endfor %}
   |              ^^^^^^^^ cannot unpack list item into two variables
"
    );
}

#[test]
fn render_for_statement_err_map_with_one_var() {
    let err = Engine::new()
        .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
        .unwrap()
        .render(value! { dolor: { sit: "amet" }})
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}
   |              ^^^^^ cannot unpack map item into one variable
"
    );
}

#[test]
fn render_for_statement_err_loop_var_scope() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %}{% endfor %}{{ ipsum }}")
        .unwrap()
        .render(value! { dolor: { ipsum: false }})
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for _, ipsum in dolor %}{% endfor %}{{ ipsum }}
   |                                                 ^^^^^ not found in this scope
"
    );
}

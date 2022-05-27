use upon::{value, Engine};

#[test]
fn render_inline_expr_bool() {
    let result = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: true })
        .unwrap();
    assert_eq!(result, "lorem true");
}

#[test]
fn render_inline_expr_i32() {
    let result = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: 123_i32 })
        .unwrap();
    assert_eq!(result, "lorem 123");
}

#[test]
fn render_inline_expr_i64() {
    let result = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: 123_i64 })
        .unwrap();
    assert_eq!(result, "lorem 123");
}

#[test]
fn render_inline_expr_f64() {
    let result = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: 123.4_f64  })
        .unwrap();
    assert_eq!(result, "lorem 123.4");
}

#[test]
fn render_inline_expr_string() {
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
        .render(value! { ipsum: true })
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
fn render_inline_expr_err_unrenderable() {
    let err = Engine::new()
        .compile("lorem {{ ipsum }}")
        .unwrap()
        .render(value! { ipsum: {} })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum }}
   |          ^^^^^ expected renderable value, but expression evaluated to map
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_none() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: None })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^ cannot index into none
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_into_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: "testing..." })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^ cannot index into string
"
    );
}

#[test]
fn render_inline_expr_err_cannot_index_list_with_string() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum: ["test", "ing..."] })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^ cannot index list with string
"
    );
}

#[test]
fn render_inline_expr_err_not_found_in_map() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor }}")
        .unwrap()
        .render(value! { ipsum : { } })
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor }}
   |                ^^^^^ not found in map
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
fn render_for_statement_map() {
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

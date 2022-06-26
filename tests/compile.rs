use upon::Engine;

#[test]
fn compile_empty() {
    Engine::new().compile("").unwrap();
}

#[test]
fn compile_raw() {
    Engine::new().compile("lorem ipsum dolor sit amet").unwrap();
}

#[test]
fn compile_comment() {
    Engine::new()
        .compile("lorem {# ipsum dolor #} sit amet")
        .unwrap();
}

#[test]
fn compile_inline_expr() {
    Engine::new()
        .compile("lorem {{ ipsum.dolor | fn | another }} sit amet")
        .unwrap();
}

#[test]
fn compile_inline_expr_filter_arg() {
    let tests = [
        "nested.path",
        r#""normal""#,
        r#""escaped \n \r \t \\ \"""#,
        "true",
        "false",
        "123",
        "-123",
        "+123",
        "0x1f",
        "0o777",
        "0b1010",
        "3.14",
        "+3.14",
        "-3.14",
    ];
    let engine = Engine::new();
    for arg in tests {
        engine
            .compile(&format!("{{{{ lorem | ipsum: {} }}}}", arg))
            .unwrap();
    }
}

#[test]
fn compile_inline_expr_filter_args() {
    Engine::new()
        .compile("{{ lorem | ipsum: true, 3.14, -0b1010 }}")
        .unwrap();
}

#[test]
fn compile_if_statement() {
    Engine::new()
        .compile("lorem {% if ipsum %} dolor {% endif %} sit")
        .unwrap();
}

#[test]
fn compile_if_else_statement() {
    Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else %} sit {% endif %} amet")
        .unwrap();
}

#[test]
fn compile_nested_if_else_statement() {
    Engine::new()
        .compile(
            "lorem {% if ipsum %} dolor {% else %} {% if sit %} amet {% endif %}, consectetur {% endif %}",
        )
        .unwrap();
}

#[test]
fn compile_for_statement_item() {
    Engine::new()
        .compile("lorem {% for ipsum in dolor %} {{ sit }} {% endfor %} amet")
        .unwrap();
}

#[test]
fn compile_for_statement_key_value() {
    Engine::new()
        .compile("lorem {% for ipsum, dolor in sit %} {{ amet }} {% endfor %}, consectetur")
        .unwrap();
}

#[test]
fn compile_inline_expr_err_eof() {
    let err = Engine::new().compile("lorem {{ ipsum.dolor |").unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor |
   |                       ^ expected identifier, found EOF
"
    )
}

#[test]
fn compile_inline_expr_err_args_eof() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor:")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | dolor:
   |                        ^ expected argument, found EOF
"
    )
}

#[test]
fn compile_inline_expr_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: for }}")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        r#"
   |
 1 | lorem {{ ipsum | dolor: for }}
   |                         ^^^ unexpected keyword `for`
"#
    )
}

#[test]
fn compile_inline_expr_err_integer_invalid_digit() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: 0b0131 }}")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | dolor: 0b0131 }}
   |                             ^ invalid digit for base 2 literal
"
    )
}

#[test]
fn compile_inline_expr_err_integer_overflow() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: 0xffffffffffffffff }}")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | dolor: 0xffffffffffffffff }}
   |                         ^^^^^^^^^^^^^^^^^^ base 16 literal out of range for 64-bit integer
"
    )
}

#[test]
fn compile_inline_expr_err_unknown_escape_character() {
    let err = Engine::new()
        .compile(r#"lorem {{ ipsum | dolor: "sit \x" }}"#)
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        r#"
   |
 1 | lorem {{ ipsum | dolor: "sit \x" }}
   |                               ^ unknown escape character
"#
    )
}

#[test]
fn compile_inline_expr_err_unexpected_comma_token() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: ,")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | dolor: ,
   |                         ^ expected argument, found comma
"
    )
}

#[test]
fn compile_inline_expr_err_empty() {
    let err = Engine::new()
        .compile("lorem {{ }} ipsum dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ }} ipsum dolor
   |          ^^ expected identifier, found end expression
"
    );
}

#[test]
fn compile_inline_expr_err_unexpected_pipe_token() {
    let err = Engine::new()
        .compile("lorem {{ | }} ipsum dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ | }} ipsum dolor
   |          ^ expected identifier, found pipe
"
    );
}

#[test]
fn compile_inline_expr_err_unexpected_period_token() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | . }} dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum | . }} dolor
   |                  ^ expected identifier, found period
"
    );
}

#[test]
fn compile_inline_expr_err_expected_function() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor | }} sit")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum.dolor | }} sit
   |                        ^^ expected identifier, found end expression
"
    );
}

#[test]
fn compile_inline_expr_err_expected_end_expression() {
    let err = Engine::new()
        .compile("lorem {{ ipsum dolor }} sit")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum dolor }} sit
   |                ^^^^^ expected end expression, found identifier
"
    );
}

#[test]
fn compile_if_statement_err_expected_keyword() {
    let err = Engine::new()
        .compile("lorem {% fi ipsum %} dolor {% endif %} sit")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% fi ipsum %} dolor {% endif %} sit
   |          ^^ expected keyword, found identifier
"
    );
}

#[test]
fn compile_if_statement_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {% in ipsum %} dolor {% endif %} sit")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% in ipsum %} dolor {% endif %} sit
   |          ^^ unexpected keyword `in`
"
    );
}

#[test]
fn compile_if_statement_err_unexpected_endif_block() {
    let err = Engine::new()
        .compile("lorem {% endif %} ipsum")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% endif %} ipsum
   |       ^^^^^^^^^^^ unexpected `endif` block
"
    );
}

#[test]
fn compile_if_statement_err_unexpected_else_block() {
    let err = Engine::new()
        .compile("lorem {% else %} {% endif %} ipsum")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% else %} {% endif %} ipsum
   |       ^^^^^^^^^^ unexpected `else` block
"
    );
}

#[test]
fn compile_if_statement_err_unexpected_endfor_block() {
    let err = Engine::new()
        .compile("lorem {% if ipsum %} {% endfor %} dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% if ipsum %} {% endfor %} dolor
   |                      ^^^^^^^^^^^^ unexpected `endfor` block
"
    );
}

#[test]
fn compile_if_statement_err_unclosed_if_block() {
    let err = Engine::new()
        .compile("lorem {% if ipsum %} dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% if ipsum %} dolor
   |       ^^^^^^^^^^^^^^ unclosed `if` block
"
    );
}

#[test]
fn compile_for_statement_err_trailing_comma() {
    let err = Engine::new()
        .compile("lorem {% for ipsum, in dolor %} sit")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum, in dolor %} sit
   |                     ^^ expected identifier, found keyword
"
    );
}

#[test]
fn compile_for_statement_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {% for ipsum endif %} dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum endif %} dolor
   |                    ^^^^^ expected keyword `in`, found keyword `endif`
"
    );
}

#[test]
fn compile_for_statement_err_missing_iterable() {
    let err = Engine::new()
        .compile("lorem {% for ipsum in %} dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum in %} dolor
   |                       ^^ expected identifier, found end block
"
    );
}

#[test]
fn compile_for_statement_err_unexpected_endfor_block() {
    let err = Engine::new()
        .compile("lorem {% endfor %} ipsum")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% endfor %} ipsum
   |       ^^^^^^^^^^^^ unexpected `endfor` block
"
    );
}

#[test]
fn compile_for_statement_err_unexpected_else_block() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %} {% else %} {% endif %}")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for _, ipsum in dolor %} {% else %} {% endif %}
   |                                   ^^^^^^^^^^ unexpected `else` block
"
    );
}

#[test]
fn compile_for_statement_err_unexpected_endif_block() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %} {% endif %}")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for _, ipsum in dolor %} {% endif %}
   |                                   ^^^^^^^^^^^ unexpected `endif` block
"
    );
}

#[test]
fn compile_for_statement_err_unclosed_for_block() {
    let err = Engine::new()
        .compile("lorem {% for ipsum, dolor in sit %} amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% for ipsum, dolor in sit %} amet
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unclosed `for` block
"
    );
}

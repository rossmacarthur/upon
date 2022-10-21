use upon::{Engine, Error};

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

const BASE_EXPRS: &[&str] = &[
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
    "3.",
    "+3.",
    "-3.",
    "3.14",
    "+3.14",
    "-3.14",
    "3.14e2",
    "+3.14e2",
    "-3.14e2",
    "3.14e+2",
    "+3.14e+2",
    "-3.14e+2",
    "314e-2",
    "+314e-2",
    "-314e-2",
];

#[test]
fn compile_inline_expr_literal() {
    let engine = Engine::new();
    for arg in BASE_EXPRS {
        engine.compile(&format!("{{{{ {} }}}}", arg)).unwrap();
    }
}

#[test]
fn compile_inline_expr_filter_arg() {
    let engine = Engine::new();
    for arg in BASE_EXPRS {
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
fn compile_inline_expr_err_eof() {
    let err = Engine::new().compile("lorem {{ ipsum.dolor |").unwrap_err();
    assert_err(
        &err,
        "expected identifier, found EOF",
        "
   |
 1 | lorem {{ ipsum.dolor |
   |                       ^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_args_eof() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor:")
        .unwrap_err();
    assert_err(
        &err,
        "expected token, found EOF",
        "
   |
 1 | lorem {{ ipsum | dolor:
   |                        ^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: for }}")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected keyword `for`",
        r#"
   |
 1 | lorem {{ ipsum | dolor: for }}
   |                         ^^^ REASON
"#,
    )
}

#[test]
fn compile_inline_expr_err_integer_invalid_digit() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: 0b0131 }}")
        .unwrap_err();
    assert_err(
        &err,
        "invalid digit for base 2 literal",
        "
   |
 1 | lorem {{ ipsum | dolor: 0b0131 }}
   |                             ^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_integer_overflow() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: 0xffffffffffffffff }}")
        .unwrap_err();
    assert_err(
        &err,
        "base 16 literal out of range for 64-bit integer",
        "
   |
 1 | lorem {{ ipsum | dolor: 0xffffffffffffffff }}
   |                         ^^^^^^^^^^^^^^^^^^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_float_invalid() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: +0.23d5 }}")
        .unwrap_err();
    assert_err(
        &err,
        "invalid float literal",
        "
   |
 1 | lorem {{ ipsum | dolor: +0.23d5 }}
   |                         ^^^^^^^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_unknown_escape_character() {
    let err = Engine::new()
        .compile(r#"lorem {{ ipsum | dolor: "sit \x" }}"#)
        .unwrap_err();
    assert_err(
        &err,
        "unknown escape character",
        r#"
   |
 1 | lorem {{ ipsum | dolor: "sit \x" }}
   |                               ^ REASON
"#,
    )
}

#[test]
fn compile_inline_expr_err_unexpected_comma_token() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | dolor: ,")
        .unwrap_err();
    assert_err(
        &err,
        "expected expression, found comma",
        "
   |
 1 | lorem {{ ipsum | dolor: ,
   |                         ^ REASON
",
    )
}

#[test]
fn compile_inline_expr_err_empty() {
    let err = Engine::new()
        .compile("lorem {{ }} ipsum dolor")
        .unwrap_err();
    assert_err(
        &err,
        "expected expression, found end expression",
        "
   |
 1 | lorem {{ }} ipsum dolor
   |          ^^ REASON
",
    );
}

#[test]
fn compile_inline_expr_err_unexpected_pipe_token() {
    let err = Engine::new()
        .compile("lorem {{ | }} ipsum dolor")
        .unwrap_err();
    assert_err(
        &err,
        "expected expression, found pipe",
        "
   |
 1 | lorem {{ | }} ipsum dolor
   |          ^ REASON
",
    );
}

#[test]
fn compile_inline_expr_err_unexpected_period_token() {
    let err = Engine::new()
        .compile("lorem {{ ipsum | . }} dolor")
        .unwrap_err();
    assert_err(
        &err,
        "expected identifier, found period",
        "
   |
 1 | lorem {{ ipsum | . }} dolor
   |                  ^ REASON
",
    );
}

#[test]
fn compile_inline_expr_err_expected_function() {
    let err = Engine::new()
        .compile("lorem {{ ipsum.dolor | }} sit")
        .unwrap_err();
    assert_err(
        &err,
        "expected identifier, found end expression",
        "
   |
 1 | lorem {{ ipsum.dolor | }} sit
   |                        ^^ REASON
",
    );
}

#[test]
fn compile_inline_expr_err_expected_end_expression() {
    let err = Engine::new()
        .compile("lorem {{ ipsum dolor }} sit")
        .unwrap_err();
    assert_err(
        &err,
        "expected end expression, found identifier",
        "
   |
 1 | lorem {{ ipsum dolor }} sit
   |                ^^^^^ REASON
",
    );
}

#[test]
fn compile_if_statement() {
    Engine::new()
        .compile("lorem {% if ipsum %} dolor {% endif %} sit")
        .unwrap();
}

#[test]
fn compile_if_else_if_statement() {
    Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else if sit %} amet {% endif %}, consectetur")
        .unwrap();
}

#[test]
fn compile_if_else_if_else_statement() {
    Engine::new()
        .compile(
            "lorem {% if ipsum %} dolor {% else if sit %} amet {% else %}, consectetur {% endif %}",
        )
        .unwrap();
}

#[test]
fn compile_if_else_statement() {
    Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else %} sit {% endif %} amet")
        .unwrap();
}

#[test]
fn compile_if_statement_nested() {
    Engine::new()
        .compile(
            "lorem {% if ipsum %} dolor {% else %} {% if sit %} amet {% endif %}, consectetur {% endif %}",
        )
        .unwrap();
}

#[test]
fn compile_if_statement_err_expected_keyword() {
    let err = Engine::new()
        .compile("lorem {% fi ipsum %} dolor {% endif %} sit")
        .unwrap_err();
    assert_err(
        &err,
        "expected keyword, found identifier",
        "
   |
 1 | lorem {% fi ipsum %} dolor {% endif %} sit
   |          ^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {% in ipsum %} dolor {% endif %} sit")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected keyword `in`",
        "
   |
 1 | lorem {% in ipsum %} dolor {% endif %} sit
   |          ^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unexpected_endif_block() {
    let err = Engine::new()
        .compile("lorem {% endif %} ipsum")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `endif` block",
        "
   |
 1 | lorem {% endif %} ipsum
   |       ^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unexpected_else_if_block() {
    let err = Engine::new()
        .compile("lorem {% else if cond %} {% endif %} ipsum")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `else if` block",
        "
   |
 1 | lorem {% else if cond %} {% endif %} ipsum
   |       ^^^^^^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unexpected_else_block() {
    let err = Engine::new()
        .compile("lorem {% else %} {% endif %} ipsum")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `else` block",
        "
   |
 1 | lorem {% else %} {% endif %} ipsum
   |       ^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unexpected_endfor_block() {
    let err = Engine::new()
        .compile("lorem {% if ipsum %} {% endfor %} dolor")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `endfor` block",
        "
   |
 1 | lorem {% if ipsum %} {% endfor %} dolor
   |                      ^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_if_else_statement_err_unclosed_if_block() {
    let err = Engine::new()
        .compile("lorem {% if ipsum %} dolor {% else if sit %}")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed `if` block",
        "
   |
 1 | lorem {% if ipsum %} dolor {% else if sit %}
   |       ^^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_if_statement_err_unclosed_if_block() {
    let err = Engine::new()
        .compile("lorem {% if ipsum %} dolor")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed `if` block",
        "
   |
 1 | lorem {% if ipsum %} dolor
   |       ^^^^^^^^^^^^^^ REASON
",
    );
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
fn compile_for_statement_err_trailing_comma() {
    let err = Engine::new()
        .compile("lorem {% for ipsum, in dolor %} sit")
        .unwrap_err();
    assert_err(
        &err,
        "expected identifier, found keyword",
        "
   |
 1 | lorem {% for ipsum, in dolor %} sit
   |                     ^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unexpected_keyword() {
    let err = Engine::new()
        .compile("lorem {% for ipsum endif %} dolor")
        .unwrap_err();
    assert_err(
        &err,
        "expected keyword `in`, found keyword `endif`",
        "
   |
 1 | lorem {% for ipsum endif %} dolor
   |                    ^^^^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_missing_iterable() {
    let err = Engine::new()
        .compile("lorem {% for ipsum in %} dolor")
        .unwrap_err();
    assert_err(
        &err,
        "expected expression, found end block",
        "
   |
 1 | lorem {% for ipsum in %} dolor
   |                       ^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unexpected_endfor_block() {
    let err = Engine::new()
        .compile("lorem {% endfor %} ipsum")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `endfor` block",
        "
   |
 1 | lorem {% endfor %} ipsum
   |       ^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unexpected_else_block() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %} {% else %} {% endif %}")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `else` block",
        "
   |
 1 | lorem {% for _, ipsum in dolor %} {% else %} {% endif %}
   |                                   ^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unexpected_else_if_block() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %} {% else if cond %}")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `else if` block",
        "
   |
 1 | lorem {% for _, ipsum in dolor %} {% else if cond %}
   |                                   ^^^^^^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unexpected_endif_block() {
    let err = Engine::new()
        .compile("lorem {% for _, ipsum in dolor %} {% endif %}")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `endif` block",
        "
   |
 1 | lorem {% for _, ipsum in dolor %} {% endif %}
   |                                   ^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_for_statement_err_unclosed_for_block() {
    let err = Engine::new()
        .compile("lorem {% for ipsum, dolor in sit %} amet")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed `for` block",
        "
   |
 1 | lorem {% for ipsum, dolor in sit %} amet
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_with_statement() {
    Engine::new()
        .compile("lorem {% with ipsum as dolor %} {{ dolor }} {% endwith %} sit")
        .unwrap();
}

#[test]
fn compile_with_statement_err_unclosed_with_block() {
    let err = Engine::new()
        .compile("lorem {% with ipsum as dolor %} sit")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed `with` block",
        "
   |
 1 | lorem {% with ipsum as dolor %} sit
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_with_statement_err_unexpected_endwith_block() {
    let err = Engine::new()
        .compile("lorem {% with ipsum as dolor %} sit {% else %} {% endif %}")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `else` block",
        "
   |
 1 | lorem {% with ipsum as dolor %} sit {% else %} {% endif %}
   |                                     ^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_with_statement_err_unexpected_else_block() {
    let err = Engine::new()
        .compile("lorem {% endwith %} ipsum")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected `endwith` block",
        "
   |
 1 | lorem {% endwith %} ipsum
   |       ^^^^^^^^^^^^^ REASON
",
    );
}

#[test]
fn compile_include_statement() {
    Engine::new()
        .compile(r#"lorem {% include "ipsum" %} dolor"#)
        .unwrap();
}

#[test]
fn compile_include_with_statement() {
    Engine::new()
        .compile(r#"lorem {% include "ipsum" with dolor %} sit"#)
        .unwrap();
}

#[test]
fn compile_include_with_statement_filters() {
    Engine::new()
        .compile(r#"lorem {% include "ipsum" with dolor.sit | amet: 1337 %}"#)
        .unwrap();
}

#[track_caller]
fn assert_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("invalid syntax: {}", reason);
    let display_alt = format!("invalid syntax{}", pretty.replace("REASON", reason));
    assert_eq!(err.to_string(), display);
    assert_eq!(format!("{:#}", err), display_alt);
}

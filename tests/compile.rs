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
fn compile_inline_expr() {
    Engine::new()
        .compile("lorem {{ ipsum.dolor | fn | another }} sit amet")
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
        .compile("lorem {% else %} {% else %} {% endif %} ipsum")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% else %} {% else %} {% endif %} ipsum
   |       ^^^^^^^^^^ unexpected `else` block
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
        .compile("lorem {% else %} {% endfor %} ipsum")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {% else %} {% endfor %} ipsum
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

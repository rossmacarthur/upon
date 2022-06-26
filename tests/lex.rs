use upon::{Engine, Syntax};

#[test]
fn lex_while_eof() {
    let err = Engine::new().compile("lorem {{ ipsum").unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem {{ ipsum
   |               ^ expected end expression, found EOF
"
    );
}

#[test]
fn lex_syntax_overlapping() {
    let syntax = Syntax::builder().expr("{", "}").block("{{", "}}").build();
    Engine::with_syntax(syntax)
        .compile("lorem { ipsum } {{ if dolor }} {{ endif }} sit amet")
        .unwrap();
}

#[test]
fn lex_syntax_overlapping_flipped() {
    let syntax = Syntax::builder().expr("{{", "}}").block("{", "}").build();
    Engine::with_syntax(syntax)
        .compile("lorem {{ ipsum }} { if dolor } { endif } sit amet")
        .unwrap();
}

#[test]
fn lex_syntax_whitespace_trimming() {
    Engine::new()
        .compile("lorem {{- ipsum -}} {%- if dolor -%} {% endif %} sit amet")
        .unwrap();
}

#[test]
fn lex_syntax_precedence() {
    let syntax = Syntax::builder().expr("{|", "|}").block("{", "}").build();
    Engine::with_syntax(syntax)
        .compile("lorem {| ipsum | dolor |} sit")
        .unwrap();
}

#[test]
fn lex_err_unexpected_end_expr() {
    let err = Engine::new()
        .compile("lorem ipsum }} dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum }} dolor sit amet
   |             ^^ unexpected end expression
"
    );
}

#[test]
fn lex_err_unexpected_end_block() {
    let err = Engine::new()
        .compile("lorem ipsum %} dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum %} dolor sit amet
   |             ^^ unexpected end block
"
    );
}

#[test]
fn lex_err_unclosed_begin_expr() {
    let err = Engine::new()
        .compile("lorem ipsum {{ {{ dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {{ {{ dolor sit amet
   |             ^^ unclosed begin expression
"
    );
}

#[test]
fn lex_err_unclosed_begin_block() {
    let err = Engine::new()
        .compile("lorem ipsum {% {{ dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {% {{ dolor sit amet
   |             ^^ unclosed begin block
"
    );
}

#[test]
fn lex_err_unexpected_end_tag_after_begin_block() {
    let err = Engine::new()
        .compile("lorem ipsum {{ %} dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {{ %} dolor sit amet
   |                ^^ unexpected end block
"
    );
}

#[test]
fn lex_err_unexpected_character() {
    let err = Engine::new()
        .compile("lorem ipsum {{ ✨ }} dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {{ ✨ }} dolor sit amet
   |                ^^ unexpected character
"
    );
}

#[test]
fn lex_err_unclosed_begin_comment() {
    let err = Engine::new()
        .compile("lorem ipsum {# {{ dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {# {{ dolor sit amet
   |             ^^ unclosed begin comment
"
    );
}

#[test]
fn lex_err_unexpected_end_tag_after_begin_comment() {
    let err = Engine::new()
        .compile("lorem ipsum {# %} dolor sit amet")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        "
   |
 1 | lorem ipsum {# %} dolor sit amet
   |                ^^ unexpected end block
"
    );
}

#[test]
fn lex_err_undelimited_string_eof() {
    let err = Engine::new().compile("lorem {% \"ipsum").unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        r#"
   |
 1 | lorem {% "ipsum
   |          ^^^^^^ undelimited string
"#
    );
}

#[test]
fn lex_err_undelimited_string_newline() {
    let err = Engine::new()
        .compile("lorem {% \"ipsum\n dolor")
        .unwrap_err();
    assert_eq!(
        format!("{:#}", err),
        r#"
   |
 1 | lorem {% "ipsum
   |          ^^^^^^ undelimited string
"#
    );
}

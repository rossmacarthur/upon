use upon::{Engine, Error, Syntax};

#[test]
fn lex_while_eof() {
    let err = Engine::new().compile("lorem {{ ipsum").unwrap_err();
    assert_err(
        &err,
        "expected end expression, found EOF",
        "
  --> <anonymous>:1:15
   |
 1 | lorem {{ ipsum
   |               ^--
   |
   = reason: REASON
",
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
    assert_err(
        &err,
        "unexpected end expression",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum }} dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unexpected_end_block() {
    let err = Engine::new()
        .compile("lorem ipsum %} dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected end block",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum %} dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unexpected_end_comment() {
    let err = Engine::new()
        .compile("lorem ipsum #} dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected end comment",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum #} dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unclosed_begin_expr() {
    let err = Engine::new()
        .compile("lorem ipsum {{ {{ dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed begin expression",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum {{ {{ dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unclosed_begin_block() {
    let err = Engine::new()
        .compile("lorem ipsum {% {{ dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed begin block",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum {% {{ dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unexpected_end_tag_after_begin_block() {
    let err = Engine::new()
        .compile("lorem ipsum {{ %} dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected end block",
        "
  --> <anonymous>:1:16
   |
 1 | lorem ipsum {{ %} dolor sit amet
   |                ^^-
   |
   = reason: REASON
",
    );
}

#[cfg(feature = "unicode")]
#[test]
fn lex_err_unexpected_character() {
    let err = Engine::new()
        .compile("lorem ipsum {{ ✨ }} dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected character",
        "
  --> <anonymous>:1:16
   |
 1 | lorem ipsum {{ ✨ }} dolor sit amet
   |                ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unclosed_begin_comment() {
    let err = Engine::new()
        .compile("lorem ipsum {# {{ dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unclosed begin comment",
        "
  --> <anonymous>:1:13
   |
 1 | lorem ipsum {# {{ dolor sit amet
   |             ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_unexpected_end_tag_after_begin_comment() {
    let err = Engine::new()
        .compile("lorem ipsum {# %} dolor sit amet")
        .unwrap_err();
    assert_err(
        &err,
        "unexpected end block",
        "
  --> <anonymous>:1:16
   |
 1 | lorem ipsum {# %} dolor sit amet
   |                ^^-
   |
   = reason: REASON
",
    );
}

#[test]
fn lex_err_undelimited_string_eof() {
    let err = Engine::new().compile("lorem {% \"ipsum").unwrap_err();
    assert_err(
        &err,
        "undelimited string",
        r#"
  --> <anonymous>:1:10
   |
 1 | lorem {% "ipsum
   |          ^^^^^^
   |
   = reason: REASON
"#,
    );
}

#[test]
fn lex_err_undelimited_string_newline() {
    let err = Engine::new()
        .compile("lorem {% \"ipsum\n dolor")
        .unwrap_err();
    assert_err(
        &err,
        "undelimited string",
        r#"
  --> <anonymous>:1:10
   |
 1 | lorem {% "ipsum
   |          ^^^^^^
   |
   = reason: REASON
"#,
    );
}

#[track_caller]
fn assert_err(err: &Error, reason: &str, pretty: &str) {
    let display = format!("invalid syntax: {reason}");
    let display_alt = format!("invalid syntax\n{}", pretty.replace("REASON", reason));
    assert_eq!(err.to_string(), display);
    assert_eq!(format!("{err:#}"), display_alt);
}

use crate::ast;
use crate::lex::{Lexer, Token};
use crate::{Delimiters, Error, Result, Span};

pub fn template<'t>(source: &'t str, delims: &Delimiters<'_>) -> Result<ast::Template<'t>> {
    Parser::new(source, delims).expect_template()
}

pub struct Parser<'e, 't> {
    tokens: Lexer<'e, 't>,
    peeked: Option<Option<(Token, Span)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keyword {
    If,
    Else,
    EndIf,
    For,
    In,
    EndFor,
}

enum State<'t> {
    If(ast::Expr<'t>, Span),
    Else(Span),
    For(ast::LoopVars<'t>, ast::Expr<'t>, Span),
}

enum Block<'t> {
    If(ast::Expr<'t>),
    Else,
    EndIf,
    For(ast::LoopVars<'t>, ast::Expr<'t>),
    EndFor,
}

impl<'e, 't> Parser<'e, 't> {
    fn new(source: &'t str, delims: &'e Delimiters<'e>) -> Self {
        Self {
            tokens: Lexer::new(source, delims),
            peeked: None,
        }
    }

    fn expect_template(mut self) -> Result<ast::Template<'t>> {
        let mut blocks: Vec<State<'t>> = vec![];
        let mut scopes: Vec<ast::Scope<'t>> = vec![ast::Scope::new()];

        while let Some(next) = self.next()? {
            let stmt = match next {
                (Token::Raw, span) => ast::Stmt::Raw(&self.source()[span]),

                (Token::BeginExpr, begin_tag) => {
                    let expr = self.expect_expr()?;
                    let end_tag = self.expect(Token::EndExpr)?;
                    let span = begin_tag.combine(end_tag);
                    ast::Stmt::InlineExpr(ast::InlineExpr { expr, span })
                }

                (Token::BeginBlock, begin_tag) => {
                    let block = self.expect_block()?;
                    let end_tag = self.expect(Token::EndBlock)?;
                    let span = begin_tag.combine(end_tag);
                    match block {
                        Block::If(cond) => {
                            blocks.push(State::If(cond, span));
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        Block::Else => {
                            blocks.push(State::Else(span));
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        Block::EndIf => {
                            let err = || Error::span("unexpected `endif`", self.source(), span);

                            let mut last = blocks.pop().ok_or_else(err)?;
                            let mut else_branch = None;
                            if let State::Else(_) = last {
                                last = blocks.pop().ok_or_else(err)?;
                                else_branch = Some(scopes.pop().unwrap());
                            };

                            let if_else = match last {
                                State::If(cond, _) => {
                                    let then_branch = scopes.pop().unwrap();
                                    ast::IfElse {
                                        cond,
                                        then_branch,
                                        else_branch,
                                    }
                                }
                                State::Else(span) => {
                                    return Err(Error::span(
                                        "expected `if`, found `else`",
                                        self.source(),
                                        span,
                                    ));
                                }
                                State::For(_, _, span) => {
                                    return Err(Error::span(
                                        "expected `if`, found `for`",
                                        self.source(),
                                        span,
                                    ));
                                }
                            };

                            ast::Stmt::IfElse(if_else)
                        }

                        Block::For(vars, iterable) => {
                            blocks.push(State::For(vars, iterable, span));
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        Block::EndFor => {
                            let last = blocks.pop().ok_or_else(|| {
                                Error::span("unexpected `endfor`", self.source(), span)
                            })?;
                            let for_loop = match last {
                                State::For(vars, iterable, _) => {
                                    let body = scopes.pop().unwrap();
                                    ast::ForLoop {
                                        vars,
                                        iterable,
                                        body,
                                    }
                                }
                                State::If(_, span) => {
                                    return Err(Error::span(
                                        "expected `for`, found `if`",
                                        self.source(),
                                        span,
                                    ));
                                }
                                State::Else(span) => {
                                    return Err(Error::span(
                                        "expected `for`, found `else`",
                                        self.source(),
                                        span,
                                    ));
                                }
                            };

                            ast::Stmt::ForLoop(for_loop)
                        }
                    }
                }
                (tk, span) => {
                    panic!("lexer bug, got token `{:?}` at {:?}", tk, span);
                }
            };
            scopes.last_mut().unwrap().stmts.push(stmt);
        }

        if let Some(block) = blocks.first() {
            let (msg, span) = match block {
                State::If(_, sp) => ("unclosed `if`", sp),
                State::Else(sp) => ("unexpected `else`", sp),
                State::For(_, _, sp) => ("unclosed `for`", sp),
            };
            return Err(Error::span(msg, self.source(), *span));
        }

        assert!(blocks.is_empty());
        assert_eq!(scopes.len(), 1);

        Ok(ast::Template {
            source: self.source(),
            scope: scopes.remove(0),
        })
    }

    fn expect_block(&mut self) -> Result<Block<'t>> {
        let (kw, span) = self.expect_keyword()?;
        match kw {
            Keyword::If => {
                let expr = self.expect_expr()?;
                Ok(Block::If(expr))
            }
            Keyword::Else => Ok(Block::Else),
            Keyword::EndIf => Ok(Block::EndIf),
            Keyword::For => {
                let vars = self.expect_loop_vars()?;
                self.expect_keyword_exact(Keyword::In)?;
                let iterable = self.expect_expr()?;
                Ok(Block::For(vars, iterable))
            }
            Keyword::EndFor => Ok(Block::EndFor),
            _ => Err(Error::span(
                format!("unexpected keyword `{}`", kw.human()),
                self.source(),
                span,
            )),
        }
    }

    fn expect_loop_vars(&mut self) -> Result<ast::LoopVars<'t>> {
        let key = self.expect_ident()?;
        if !self.is_next(Token::Comma)? {
            return Ok(ast::LoopVars::Item(key));
        }
        self.expect(Token::Comma)?;
        let value = self.expect_ident()?;
        let span = key.span.combine(value.span);
        Ok(ast::LoopVars::KeyValue(ast::KeyValue { key, value, span }))
    }

    fn expect_expr(&mut self) -> Result<ast::Expr<'t>> {
        let mut expr = ast::Expr::Var(self.expect_var()?);
        while self.is_next(Token::Pipe)? {
            self.expect(Token::Pipe)?;
            let name = self.expect_ident()?;
            let span = name.span.combine(expr.span());
            expr = ast::Expr::Call(ast::Call {
                name,
                receiver: Box::new(expr),
                span,
            });
        }
        Ok(expr)
    }

    fn expect_var(&mut self) -> Result<ast::Var<'t>> {
        let mut path = Vec::new();
        loop {
            path.push(self.expect_ident()?);
            if self.is_next(Token::Period)? {
                self.expect(Token::Period)?;
                continue;
            }
            break;
        }
        let span = path[0].span.combine(path[path.len() - 1].span);
        Ok(ast::Var { path, span })
    }

    fn expect_keyword_exact(&mut self, exp: Keyword) -> Result<Span> {
        let (kw, span) = self.expect_keyword()?;
        if kw != exp {
            return Err(Error::span(
                format!(
                    "expected keyword `{}`, found keyword `{}`",
                    exp.human(),
                    kw.human()
                ),
                self.source(),
                span,
            ));
        }
        Ok(span)
    }

    fn expect_keyword(&mut self) -> Result<(Keyword, Span)> {
        let span = self.expect(Token::Keyword)?;
        let kw = &self.source()[span];
        match Keyword::from_str(kw) {
            Some(kw) => Ok((kw, span)),
            None => panic!("bug in lexer, got keyword `{}`", kw),
        }
    }

    fn expect_ident(&mut self) -> Result<ast::Ident<'t>> {
        let span = self.expect(Token::Ident)?;
        let value = &self.source()[span];
        Ok(ast::Ident { value, span })
    }

    fn expect(&mut self, exp: Token) -> Result<Span> {
        match self.next()? {
            Some((tk, sp)) if tk == exp => Ok(sp),
            Some((tk, sp)) => Err(Error::span(
                format!("expected {}, found {}", exp.human(), tk.human()),
                self.source(),
                sp,
            )),
            None => {
                let n = self.source().len();
                Err(Error::span(
                    format!("expected {}, found EOF", exp.human()),
                    self.source(),
                    n..n,
                ))
            }
        }
    }

    fn is_next(&mut self, token: Token) -> Result<bool> {
        Ok(self.peek()?.map(|(tk, _)| tk == token).unwrap_or(false))
    }

    fn peek(&mut self) -> Result<Option<(Token, Span)>> {
        if let o @ None = &mut self.peeked {
            *o = Some(self.tokens.next()?);
        }
        Ok(self.peeked.unwrap())
    }

    fn next(&mut self) -> Result<Option<(Token, Span)>> {
        match self.peeked.take() {
            Some(v) => Ok(v),
            None => self.tokens.next(),
        }
    }

    fn source(&self) -> &'t str {
        self.tokens.source
    }
}

impl Keyword {
    pub(crate) fn all() -> &'static [&'static str] {
        &["if", "else", "endif", "for", "in", "endfor"]
    }

    fn human(&self) -> &'static str {
        match *self {
            Self::If => "if",
            Self::Else => "else",
            Self::EndIf => "endif",
            Self::For => "for",
            Self::In => "in",
            Self::EndFor => "endfor",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        let kw = match s {
            "if" => Self::If,
            "else" => Self::Else,
            "endif" => Self::EndIf,
            "for" => Self::For,
            "in" => Self::In,
            "endfor" => Self::EndFor,
            _ => return None,
        };
        Some(kw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn compile_template_empty() {
        let ast = parse("").unwrap();
        goldie::assert!(format!("{:#?}", ast));
    }

    #[test]
    fn compile_template_raw() {
        let ast = parse("lorem ipsum dolor sit amet").unwrap();
        goldie::assert!(format!("{:#?}", ast));
    }

    #[test]
    fn compile_template_inline_expr() {
        let tokens = parse("lorem {{ ipsum.dolor | fn | another }} sit amet").unwrap();
        goldie::assert!(format!("{:#?}", tokens));
    }

    #[test]
    fn compile_template_inline_expr_eof() {
        let err = parse("lorem {{ ipsum.dolor |").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {{ ipsum.dolor |
   |                       ^ expected identifier, found EOF
"
        );
    }

    #[test]
    fn compile_template_inline_expr_empty() {
        let err = parse("lorem {{ }} ipsum dolor").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {{ }} ipsum dolor
   |          ^^ expected identifier, found end tag
"
        );
    }

    #[test]
    fn compile_template_inline_expr_unexpected_pipe_token() {
        let err = parse("lorem {{ | }} ipsum dolor").unwrap_err();
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
    fn compile_template_inline_expr_unexpected_period_token() {
        let err = parse("lorem {{ var | . }} ipsum dolor").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {{ var | . }} ipsum dolor
   |                ^ expected identifier, found period
"
        );
    }

    #[test]
    fn compile_template_inline_expr_expected_a_function() {
        let err = parse("lorem {{ ipsum.dolor | }} sit").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {{ ipsum.dolor | }} sit
   |                        ^^ expected identifier, found end tag
"
        );
    }

    #[test]
    fn compile_template_inline_expr_expected_end_tag() {
        let err = parse("lorem {{ var another }} ipsum dolor").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {{ var another }} ipsum dolor
   |              ^^^^^^^ expected end tag, found identifier
"
        );
    }

    #[test]
    fn compile_template_if_then_block() {
        let tokens = parse("lorem {% if another %} ipsum {% endif %} dolor").unwrap();
        goldie::assert!(format!("{:#?}", tokens));
    }

    #[test]
    fn compile_template_if_then_else_block() {
        let tokens =
            parse("lorem {% if another %} ipsum {% else %} dolor {% endif %} sit").unwrap();
        goldie::assert!(format!("{:#?}", tokens));
    }

    #[test]
    fn compile_template_expected_keyword() {
        let err = parse("lorem {% fi another %} ipsum {% endif %} dolor").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% fi another %} ipsum {% endif %} dolor
   |          ^^ expected keyword, found identifier
"
        );
    }

    #[test]
    fn compile_template_unexpected_keyword() {
        let err = parse("lorem {% in another %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% in another %} ipsum
   |          ^^ unexpected keyword `in`
"
        );
    }

    #[test]
    fn compile_template_unexpected_endif_block() {
        let err = parse("lorem {% endif %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% endif %} ipsum
   |       ^^^^^^^^^^^ unexpected `endif`
"
        );
    }

    #[test]
    fn compile_template_expected_if_block() {
        let err = parse("lorem {% else %} {% else %} {% endif %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% else %} {% else %} {% endif %} ipsum
   |       ^^^^^^^^^^ expected `if`, found `else`
"
        );
    }

    #[test]
    fn compile_template_unexpected_else_block() {
        let err = parse("lorem {% else %} {% else %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% else %} {% else %} ipsum
   |       ^^^^^^^^^^ unexpected `else`
"
        );
    }

    #[test]
    fn compile_template_unclosed_if_statement() {
        let err = parse("lorem {% if cond %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% if cond %} ipsum
   |       ^^^^^^^^^^^^^ unclosed `if`
"
        );
    }

    #[test]
    fn compile_template_for_loop() {
        let tokens =
            parse("lorem {% for k, v in iter %} ipsum {{ k }} dolor {% endfor %} sit").unwrap();
        goldie::assert!(format!("{:#?}", tokens));
    }

    #[test]
    fn compile_template_for_loop_trailing_comma() {
        let err = parse("lorem {% for k, in iter %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for k, in iter %} ipsum
   |                 ^^ expected identifier, found keyword
"
        );
    }

    #[test]
    fn compile_template_for_loop_missing_iterable() {
        let err = parse("lorem {% for kv in %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for kv in %} ipsum
   |                    ^^ expected identifier, found end tag
"
        );
    }

    #[test]
    fn compile_template_unexpected_endfor_block() {
        let err = parse("lorem {% endfor %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% endfor %} ipsum
   |       ^^^^^^^^^^^^ unexpected `endfor`
"
        );
    }

    #[test]
    fn compile_template_endfor_expected_for_block() {
        let err = parse("lorem {% else %} {% endfor %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% else %} {% endfor %} ipsum
   |       ^^^^^^^^^^ expected `for`, found `else`
"
        );
    }

    #[test]
    fn compile_template_for_loop_unexpected_else_block() {
        let err = parse("lorem {% if cond %} {% endfor %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% if cond %} {% endfor %} ipsum
   |       ^^^^^^^^^^^^^ expected `for`, found `if`
"
        );
    }

    #[test]
    fn compile_template_for_loop_unclosed() {
        let err = parse("lorem {% for k, v in iter %} ipsum").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for k, v in iter %} ipsum
   |       ^^^^^^^^^^^^^^^^^^^^^^ unclosed `for`
"
        );
    }

    fn parse(source: &str) -> Result<ast::Template<'_>> {
        template(source, &Delimiters::default())
    }
}

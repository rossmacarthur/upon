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

enum State<'t> {
    If(ast::Expr<'t>, Span),
    Else(Span),
}

enum Block<'t> {
    If(ast::Expr<'t>),
    Else,
    EndIf,
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
                    let end_tag = self.expect(Token::EndExpr, None)?;
                    let span = begin_tag.combine(end_tag);
                    ast::Stmt::InlineExpr(ast::InlineExpr { expr, span })
                }

                (Token::BeginBlock, begin_tag) => {
                    let block = self.expect_block()?;
                    let end_tag = self.expect(Token::EndBlock, None)?;
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
                            };

                            ast::Stmt::IfElse(if_else)
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
        let id = self.expect_ident(None)?;
        match id.ident {
            "if" => {
                let expr = self.expect_expr()?;
                Ok(Block::If(expr))
            }
            "else" => Ok(Block::Else),
            "endif" => Ok(Block::EndIf),
            _ => Err(Error::span(
                "expected keyword `if`, `else`, or `endif`, found identifier",
                self.source(),
                id.span,
            )),
        }
    }

    fn expect_expr(&mut self) -> Result<ast::Expr<'t>> {
        let mut expr = ast::Expr::Var(self.expect_var(Some("an expression"))?);
        while self.is_next(Token::Pipe)? {
            self.expect(Token::Pipe, None)?;
            let name = self.expect_ident(Some("a function"))?;
            let span = name.span.combine(expr.span());
            expr = ast::Expr::Call(ast::Call {
                name,
                receiver: Box::new(expr),
                span,
            });
        }
        Ok(expr)
    }

    fn expect_var(&mut self, exp: Option<&'static str>) -> Result<ast::Var<'t>> {
        let mut path = Vec::new();
        loop {
            path.push(self.expect_ident(exp)?);
            if self.is_next(Token::Period)? {
                self.expect(Token::Period, None)?;
                continue;
            }
            break;
        }
        let span = path[0].span.combine(path[path.len() - 1].span);
        Ok(ast::Var { path, span })
    }

    fn expect_ident(&mut self, exp: Option<&'static str>) -> Result<ast::Ident<'t>> {
        let span = self.expect(Token::Ident, exp)?;
        let ident = &self.source()[span];
        Ok(ast::Ident { ident, span })
    }

    fn expect(&mut self, token: Token, exp: Option<&'static str>) -> Result<Span> {
        match self.next()? {
            Some((tk, sp)) if tk == token => Ok(sp),
            Some((tk, sp)) => Err(Error::span(
                format!(
                    "expected {}, found {}",
                    exp.unwrap_or_else(|| token.human()),
                    tk.human()
                ),
                self.source(),
                sp,
            )),
            None => {
                let n = self.source().len();
                Err(Error::span(
                    format!(
                        "expected {}, found EOF",
                        exp.unwrap_or_else(|| token.human())
                    ),
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
   |                       ^ expected a function, found EOF
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
   |          ^^ expected an expression, found end tag
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
   |          ^ expected an expression, found a pipe
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
   |                ^ expected a function, found a period
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
   |                        ^^ expected a function, found end tag
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
   |              ^^^^^^^ expected end tag, found an identifier
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
   |          ^^ expected keyword `if`, `else`, or `endif`, found identifier
"
        );
    }

    #[test]
    fn compile_template_unexpected_end_block() {
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

    fn parse(source: &str) -> Result<ast::Template<'_>> {
        template(source, &Delimiters::default())
    }
}

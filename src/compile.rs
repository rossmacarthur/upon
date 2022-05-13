use crate::ast;
use crate::lex::{Lexer, Token};
use crate::{Delimiters, Error, Result, Span};

pub(crate) fn template<'t>(source: &'t str, delims: &Delimiters<'_>) -> Result<ast::Template<'t>> {
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
                            let err =
                                || Error::span("unexpected `endif` block", self.source(), span);

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
                                state => {
                                    return Err(Error::span(
                                        format!("unexpected `{}` block", state.human()),
                                        self.source(),
                                        state.span(),
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
                                Error::span("unexpected `endfor` block", self.source(), span)
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
                                state => {
                                    return Err(Error::span(
                                        format!("unexpected `{}` block", state.human()),
                                        self.source(),
                                        state.span(),
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
                State::If(_, sp) => ("unclosed `if` block", sp),
                State::Else(sp) => ("unexpected `else` block", sp),
                State::For(_, _, sp) => ("unclosed `for` block", sp),
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
        match self {
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

impl State<'_> {
    fn human(&self) -> &'static str {
        match self {
            State::If(_, _) => "if",
            State::Else(_) => "else",
            State::For(_, _, _) => "for",
        }
    }

    fn span(&self) -> Span {
        match self {
            State::If(_, span) => *span,
            State::Else(span) => *span,
            State::For(_, _, span) => *span,
        }
    }
}

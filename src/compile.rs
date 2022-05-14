use crate::ast;
use crate::lex::{Lexer, Token};
use crate::{Engine, Error, Result, Span};

pub(crate) fn template<'engine, 'source>(
    engine: &'engine Engine<'engine>,
    source: &'source str,
) -> Result<ast::Template<'source>> {
    Parser::new(engine, source).parse_template()
}

struct Parser<'engine, 'source> {
    tokens: Lexer<'engine, 'source>,
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

enum State<'source> {
    If(ast::Expr<'source>, Span),
    Else(Span),
    For(ast::LoopVars<'source>, ast::Expr<'source>, Span),
}

enum Block<'source> {
    If(ast::Expr<'source>),
    Else,
    EndIf,
    For(ast::LoopVars<'source>, ast::Expr<'source>),
    EndFor,
}

impl<'engine, 'source> Parser<'engine, 'source> {
    fn new(engine: &'engine Engine<'engine>, source: &'source str) -> Self {
        Self {
            tokens: Lexer::new(engine, source),
            peeked: None,
        }
    }

    fn parse_template(mut self) -> Result<ast::Template<'source>> {
        let mut blocks: Vec<State<'_>> = vec![];
        let mut scopes: Vec<ast::Scope<'_>> = vec![ast::Scope::new()];

        while let Some(next) = self.next()? {
            let stmt = match next {
                (Token::Raw, span) => ast::Stmt::Raw(&self.source()[span]),

                (Token::BeginExpr, begin) => {
                    let expr = self.parse_expr()?;
                    let end = self.parse(Token::EndExpr)?;
                    let span = begin.combine(end);
                    ast::Stmt::InlineExpr(ast::InlineExpr { expr, span })
                }

                (Token::BeginBlock, begin) => {
                    let block = self.parse_block()?;
                    let end = self.parse(Token::EndBlock)?;
                    let span = begin.combine(end);
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
                                || Error::new("unexpected `endif` block", self.source(), span);

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
                                    return Err(Error::new(
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
                                Error::new("unexpected `endfor` block", self.source(), span)
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
                                    return Err(Error::new(
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
            return Err(Error::new(msg, self.source(), *span));
        }

        assert!(blocks.is_empty());
        assert_eq!(scopes.len(), 1);

        Ok(ast::Template {
            source: self.source(),
            scope: scopes.remove(0),
        })
    }

    fn parse_block(&mut self) -> Result<Block<'source>> {
        let (kw, span) = self.parse_keyword()?;
        match kw {
            Keyword::If => {
                let expr = self.parse_expr()?;
                Ok(Block::If(expr))
            }
            Keyword::Else => Ok(Block::Else),
            Keyword::EndIf => Ok(Block::EndIf),
            Keyword::For => {
                let vars = self.parse_loop_vars()?;
                self.parse_keyword_exact(Keyword::In)?;
                let iterable = self.parse_expr()?;
                Ok(Block::For(vars, iterable))
            }
            Keyword::EndFor => Ok(Block::EndFor),
            _ => Err(Error::new(
                format!("unexpected keyword `{}`", kw.human()),
                self.source(),
                span,
            )),
        }
    }

    fn parse_loop_vars(&mut self) -> Result<ast::LoopVars<'source>> {
        let key = self.parse_ident()?;
        if !self.is_next(Token::Comma)? {
            return Ok(ast::LoopVars::Item(key));
        }
        self.parse(Token::Comma)?;
        let value = self.parse_ident()?;
        let span = key.span.combine(value.span);
        Ok(ast::LoopVars::KeyValue(ast::KeyValue { key, value, span }))
    }

    fn parse_expr(&mut self) -> Result<ast::Expr<'source>> {
        let mut expr = ast::Expr::Var(self.parse_var()?);
        while self.is_next(Token::Pipe)? {
            self.parse(Token::Pipe)?;
            let name = self.parse_ident()?;
            let span = name.span.combine(expr.span());
            expr = ast::Expr::Call(ast::Call {
                name,
                receiver: Box::new(expr),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_var(&mut self) -> Result<ast::Var<'source>> {
        let mut path = Vec::new();
        loop {
            path.push(self.parse_ident()?);
            if self.is_next(Token::Period)? {
                self.parse(Token::Period)?;
                continue;
            }
            break;
        }
        let span = path[0].span.combine(path[path.len() - 1].span);
        Ok(ast::Var { path, span })
    }

    fn parse_keyword_exact(&mut self, exp: Keyword) -> Result<Span> {
        let (kw, span) = self.parse_keyword()?;
        if kw != exp {
            return Err(Error::new(
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

    fn parse_keyword(&mut self) -> Result<(Keyword, Span)> {
        let span = self.parse(Token::Keyword)?;
        let kw = &self.source()[span];
        match Keyword::from_str(kw) {
            Some(kw) => Ok((kw, span)),
            None => panic!("bug in lexer, got keyword `{}`", kw),
        }
    }

    fn parse_ident(&mut self) -> Result<ast::Ident<'source>> {
        let span = self.parse(Token::Ident)?;
        let value = &self.source()[span];
        Ok(ast::Ident { value, span })
    }

    fn parse(&mut self, exp: Token) -> Result<Span> {
        match self.next()? {
            Some((tk, sp)) if tk == exp => Ok(sp),
            Some((tk, sp)) => Err(Error::new(
                format!("expected {}, found {}", exp.human(), tk.human()),
                self.source(),
                sp,
            )),
            None => {
                let n = self.source().len();
                Err(Error::new(
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

    fn source(&self) -> &'source str {
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

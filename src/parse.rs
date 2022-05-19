use crate::ast;
use crate::lex::{Lexer, Token};
use crate::{Engine, Error, Result, Span};

/// A parser that constructs an AST from a token stream.
///
/// The parser is implemented as a simple hand written parser with no recursion.
/// It sometimes needs to peek at the next token to know how to proceed and uses
/// the `peeked` buffer to do this.
pub struct Parser<'engine, 'source> {
    /// A lexer that tokenizes the template source.
    tokens: Lexer<'engine, 'source>,

    /// Remember a peeked value, even if it was `None`
    peeked: Option<Option<(Token, Span)>>,
}

/// Stores the state of a statement during parsing.
enum State<'source> {
    /// A partial `if` statement.
    If {
        /// The condition in the `if` block.
        cond: ast::Expr<'source>,
        /// The span of the `if` block.
        span: Span,
        /// Whether or not this `if` statement has an `else` clause.
        has_else: bool,
    },

    /// A partial `for` statement.
    For {
        /// The loop variables.
        vars: ast::LoopVars<'source>,
        /// The value we are iterating over.
        iterable: ast::Expr<'source>,
        /// The span of the `for` block.
        span: Span,
    },
}

/// A parsed block definition.
enum Block<'source> {
    If(ast::Expr<'source>),
    Else,
    EndIf,
    For(ast::LoopVars<'source>, ast::Expr<'source>),
    EndFor,
}

/// A keyword in the template syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keyword {
    If,
    Else,
    EndIf,
    For,
    In,
    EndFor,
}

impl<'engine, 'source> Parser<'engine, 'source> {
    /// Construct a new parser.
    pub fn new(engine: &'engine Engine<'engine>, source: &'source str) -> Self {
        Self {
            tokens: Lexer::new(engine, source),
            peeked: None,
        }
    }

    /// Parses a template.
    ///
    /// This function works using two stacks:
    /// - A stack of blocks e.g. `{% if cond %} ... {% else %}`.
    /// - A stack of scopes which collect each parsed statement.
    pub fn parse_template(mut self) -> Result<ast::Template<'source>> {
        let mut blocks = vec![];
        let mut scopes = vec![ast::Scope::new()];

        while let Some(next) = self.next()? {
            let stmt = match next {
                // Simply raw template, emit a single statement for it.
                (Token::Raw, span) => ast::Stmt::Raw(&self.source()[span]),

                // The start of an expression, e.g. `{{ user.name }}`
                (Token::BeginExpr, begin) => {
                    let expr = self.parse_expr()?;
                    let end = self.expect(Token::EndExpr)?;
                    let span = begin.combine(end);
                    ast::Stmt::InlineExpr(ast::InlineExpr { expr, span })
                }

                // The start of a block, e.g. `{% if cond %}`
                (Token::BeginBlock, begin) => {
                    let block = self.parse_block()?;
                    let end = self.expect(Token::EndBlock)?;
                    let span = begin.combine(end);

                    match block {
                        // The start of an `if` statement. For example:
                        //
                        //   {% if cond %}
                        //
                        // We must push a block to the block stack and a scope
                        // to the scope stack because an if statement starts a
                        // new scope.
                        Block::If(cond) => {
                            blocks.push(State::If {
                                cond,
                                span,
                                has_else: false,
                            });
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        // The `else` clause of an `if` statement. For example:
                        //
                        //   {% else %}
                        //
                        // We expect that the previous block was an `if` block
                        // and update it accordingly. We must also push to the
                        // scope stack since an `else` clause starts a new
                        // scope.
                        Block::Else => {
                            let err = || Error::new("unexpected `else` block", self.source(), span);
                            match blocks.last_mut().ok_or_else(err)? {
                                State::If {
                                    has_else: has_else @ false,
                                    ..
                                } => {
                                    *has_else = true;
                                }
                                _ => return Err(err()),
                            }
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        // The end of an `if` statement. For example:
                        //
                        //   {% endif %}
                        //
                        // We expect that the previous block was an `if` block.
                        // Making sure to pop an extra scope if it has an `else`
                        // clause.
                        Block::EndIf => {
                            let err =
                                || Error::new("unexpected `endif` block", self.source(), span);

                            let if_else = match blocks.pop().ok_or_else(err)? {
                                State::If { cond, has_else, .. } => {
                                    let else_branch = has_else.then(|| scopes.pop().unwrap());
                                    let then_branch = scopes.pop().unwrap();
                                    ast::IfElse {
                                        cond,
                                        then_branch,
                                        else_branch,
                                    }
                                }
                                _ => return Err(err()),
                            };
                            ast::Stmt::IfElse(if_else)
                        }

                        // The start of a `for` statement. For example:
                        //
                        //   {% for var in iterable %}
                        //
                        // We must push a block to the block stack and a scope
                        // to the scope stack because a for statement starts a
                        // new scope.
                        Block::For(vars, iterable) => {
                            blocks.push(State::For {
                                vars,
                                iterable,
                                span,
                            });
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        // The end of a `for` statement. For example:
                        //
                        //   {% endfor %}
                        //
                        // We expect that the previous block was an `for` block.
                        Block::EndFor => {
                            let err =
                                || Error::new("unexpected `endfor` block", self.source(), span);

                            let for_loop = match blocks.pop().ok_or_else(err)? {
                                State::For { vars, iterable, .. } => {
                                    let body = scopes.pop().unwrap();
                                    ast::ForLoop {
                                        vars,
                                        iterable,
                                        body,
                                    }
                                }
                                _ => return Err(err()),
                            };
                            ast::Stmt::ForLoop(for_loop)
                        }
                    }
                }
                (tk, span) => {
                    panic!("lexer bug: received token `{:?}` at {:?}", tk, span);
                }
            };
            scopes.last_mut().unwrap().stmts.push(stmt);
        }

        if let Some(block) = blocks.first() {
            let (msg, span) = match block {
                State::If { span, .. } => ("unclosed `if` block", span),
                State::For { span, .. } => ("unclosed `for` block", span),
            };
            return Err(Error::new(msg, self.source(), *span));
        }

        assert!(
            scopes.len() == 1,
            "parser bug: we should end with a single scope"
        );

        Ok(ast::Template {
            source: self.source(),
            scope: scopes.remove(0),
        })
    }

    /// Parses a single block. All of the following are valid blocks.
    ///
    ///   if user.is_enabled
    ///
    ///   else
    ///
    ///   endif
    ///
    ///   for uid, user in group.user_map | filter_enabled
    ///
    ///   endfor
    ///
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
                self.expect_keyword(Keyword::In)?;
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

    /// Parses an expression.
    ///
    /// This is a variable with zero or more function calls. For example:
    ///
    ///   user.name | lower | reverse
    ///
    fn parse_expr(&mut self) -> Result<ast::Expr<'source>> {
        let mut expr = ast::Expr::Var(self.parse_var()?);
        while self.is_next(Token::Pipe)? {
            self.expect(Token::Pipe)?;
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

    /// Parses loop variable(s).
    ///
    /// This is either a single identifier or two comma separated identifiers.
    /// Both of the following are valid:
    ///
    ///   item
    ///
    ///   key, value
    ///
    fn parse_loop_vars(&mut self) -> Result<ast::LoopVars<'source>> {
        let key = self.parse_ident()?;
        if !self.is_next(Token::Comma)? {
            return Ok(ast::LoopVars::Item(key));
        }
        self.expect(Token::Comma)?;
        let value = self.parse_ident()?;
        let span = key.span.combine(value.span);
        Ok(ast::LoopVars::KeyValue(ast::KeyValue { key, value, span }))
    }

    /// Parses a variable.
    ///
    /// This is one or more identifiers separated by a period. For example:
    ///
    ///   user.name
    ///
    fn parse_var(&mut self) -> Result<ast::Var<'source>> {
        let mut path = Vec::new();
        loop {
            path.push(self.parse_ident()?);
            if !self.is_next(Token::Period)? {
                break;
            }
            self.expect(Token::Period)?;
        }
        let span = match path.len() {
            1 => path[0].span,
            n => path[0].span.combine(path[n - 1].span),
        };
        Ok(ast::Var { path, span })
    }

    /// Expects the given keyword.
    fn expect_keyword(&mut self, exp: Keyword) -> Result<Span> {
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

    /// Parses a keyword.
    fn parse_keyword(&mut self) -> Result<(Keyword, Span)> {
        let span = self.expect(Token::Keyword)?;
        let kw = &self.source()[span];
        match Keyword::from_str(kw) {
            Some(kw) => Ok((kw, span)),
            None => unreachable!(),
        }
    }

    /// Parses an identifier.
    fn parse_ident(&mut self) -> Result<ast::Ident<'source>> {
        let span = self.expect(Token::Ident)?;
        let value = &self.source()[span];
        Ok(ast::Ident { raw: value, span })
    }

    /// Parses the specified token and returns its span.
    fn expect(&mut self, exp: Token) -> Result<Span> {
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

    /// Returns `true` if the next token is equal to the provided one.
    fn is_next(&mut self, token: Token) -> Result<bool> {
        Ok(self.peek()?.map(|(tk, _)| tk == token).unwrap_or(false))
    }

    /// Returns a copy of the next token without affecting the result of the
    /// following `.next()` call.
    fn peek(&mut self) -> Result<Option<(Token, Span)>> {
        if let o @ None = &mut self.peeked {
            *o = Some(self.tokens.next()?);
        }
        Ok(self.peeked.unwrap())
    }

    /// Returns the next token and span in the stream.
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

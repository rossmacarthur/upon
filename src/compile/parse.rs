use std::fmt::Display;

use crate::compile::lex::{Lexer, Token};
use crate::types::ast;
use crate::types::span::Span;
use crate::{Engine, Error, Result, Value};

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
        /// Whether this is an an `if not` or a `if` statement.
        not: bool,
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
    If(bool, ast::Expr<'source>),
    Else,
    EndIf,
    For(ast::LoopVars<'source>, ast::Expr<'source>),
    EndFor,
}

/// A keyword in the template syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keyword {
    If,
    Not,
    Else,
    EndIf,
    For,
    In,
    EndFor,
    True,
    False,
}

enum Sign {
    Neg,
    Pos,
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

                // The start of a comment, e.g. `{# ... #}`
                (Token::BeginComment, _) => {
                    self.expect(Token::Raw)?;
                    self.expect(Token::EndComment)?;
                    continue;
                }

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
                        Block::If(not, cond) => {
                            blocks.push(State::If {
                                not,
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
                                State::If {
                                    not,
                                    cond,
                                    has_else,
                                    ..
                                } => {
                                    let else_branch = has_else.then(|| scopes.pop().unwrap());
                                    let then_branch = scopes.pop().unwrap();
                                    ast::IfElse {
                                        not,
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
                let (not, expr) = self.parse_if_cond()?;
                Ok(Block::If(not, expr))
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
            kw => Err(self.err_unexpected_keyword(kw.human(), span)),
        }
    }

    /// Parses an if condition.
    ///
    /// This is an expression with an optional `not`.
    ///
    ///   not user.is_enabled
    ///
    fn parse_if_cond(&mut self) -> Result<(bool, ast::Expr<'source>)> {
        match self.peek()? {
            Some((Token::Keyword, span)) if Keyword::from_str(&self.source()[span]).is_not() => {
                self.expect_keyword(Keyword::Not)?;
                let expr = self.parse_expr()?;
                Ok((true, expr))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok((false, expr))
            }
        }
    }

    /// Parses an expression.
    ///
    /// This is a variable with zero or more function calls. For example:
    ///
    ///   user.name | lower | prefix("Mr. ")
    ///
    fn parse_expr(&mut self) -> Result<ast::Expr<'source>> {
        let mut expr = ast::Expr::Var(self.parse_var()?);
        while self.is_next(Token::Pipe)? {
            self.expect(Token::Pipe)?;
            let name = self.parse_ident()?;
            let span = name.span.combine(expr.span());
            let args = if self.is_next(Token::Colon)? {
                let span = self.expect(Token::Colon)?;
                Some(self.parse_args(span)?)
            } else {
                None
            };
            expr = ast::Expr::Call(ast::Call {
                name,
                args,
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

    /// Parses comma separate arguments. All of the following are valid
    /// arguments.
    ///
    ///   user.name
    ///
    ///   "a string"
    ///
    ///   true
    ///
    ///   -13.37
    ///
    ///   0xff
    ///
    ///
    fn parse_args(&mut self, span: Span) -> Result<ast::Args<'source>> {
        let mut values = Vec::new();
        loop {
            let arg = match self.peek()? {
                Some((Token::Ident, _)) => ast::Arg::Var(self.parse_var()?),
                Some(_) => ast::Arg::Literal(self.parse_literal()?),
                None => {
                    return Err(self.err_unexpected_eof("argument"));
                }
            };
            values.push(arg);
            if !self.is_next(Token::Comma)? {
                break;
            }
            self.expect(Token::Comma)?;
        }
        let span = span.combine(values.last().unwrap().span());
        Ok(ast::Args { values, span })
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
            path.push(self.parse_ident_or_index()?);
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

    /// Parses an identifier or index.
    fn parse_ident_or_index(&mut self) -> Result<ast::Ident<'source>> {
        let span = match self.parse()? {
            (Token::Number, span) if is_base10_integer(&self.source()[span]) => span,
            (Token::Ident, span) => span,
            (tk, span) => {
                return Err(self.err_unexpected_token("identifier", tk, span));
            }
        };
        let raw = &self.source()[span];
        Ok(ast::Ident { raw, span })
    }

    /// Parses a literal.
    fn parse_literal(&mut self) -> Result<ast::Literal> {
        match self.parse()? {
            (Token::Keyword, span) => self.parse_bool(span),
            (Token::Minus, sign) => {
                let span = self.expect(Token::Number)?;
                self.parse_number(&self.source()[span], sign.combine(span), Sign::Neg)
            }
            (Token::Plus, sign) => {
                let span = self.expect(Token::Number)?;
                self.parse_number(&self.source()[span], sign.combine(span), Sign::Pos)
            }
            (Token::Number, span) => self.parse_number(&self.source()[span], span, Sign::Pos),
            (Token::String, span) => self.parse_string(span),
            (tk, span) => Err(self.err_unexpected_token("argument", tk, span)),
        }
    }

    /// Parses a boolean argument.
    fn parse_bool(&mut self, span: Span) -> Result<ast::Literal> {
        let bool = match &self.source()[span] {
            "false" => false,
            "true" => true,
            kw => {
                return Err(self.err_unexpected_keyword(kw, span));
            }
        };
        let value = Value::Bool(bool);
        Ok(ast::Literal { value, span })
    }

    /// Parses an integer or a float.
    fn parse_number(&self, raw: &'source str, span: Span, sign: Sign) -> Result<ast::Literal> {
        if raw.contains('.') {
            let float: f64 = raw
                .parse::<f64>()
                .map_err(|_| Error::new("invalid float literal", self.source(), span))?;
            let value = Value::Float(float);
            Ok(ast::Literal { value, span })
        } else {
            self.parse_integer(raw, span, sign)
        }
    }

    /// Parse an integer.
    fn parse_integer(&self, raw: &str, span: Span, sign: Sign) -> Result<ast::Literal> {
        let digits = raw.as_bytes();
        let (i, radix) = match digits {
            [b'0', b'b', ..] => (2, 2),
            [b'0', b'o', ..] => (2, 8),
            [b'0', b'x', ..] => (2, 16),
            _ => (0, 10),
        };
        let int = digits[i..]
            .iter()
            .enumerate()
            .filter(|(_, &d)| d != b'_')
            .try_fold(0i64, |acc, (j, &d)| {
                let x = (d as char).to_digit(radix).ok_or_else(|| {
                    let m = span.m + i + j;
                    Error::new(
                        format!("invalid digit for base {} literal", radix),
                        self.source(),
                        m..m + 1,
                    )
                })?;
                let err = || {
                    Error::new(
                        format!("base {} literal out of range for 64-bit integer", radix),
                        self.source(),
                        span,
                    )
                };
                let value = acc.checked_mul(radix.into()).ok_or_else(err)?;
                match sign {
                    Sign::Pos => value.checked_add(x.into()),
                    Sign::Neg => value.checked_sub(x.into()),
                }
                .ok_or_else(err)
            })?;
        let value = Value::Integer(int);
        Ok(ast::Literal { value, span })
    }

    /// Parses a string and handles escape characters.
    fn parse_string(&self, span: Span) -> Result<ast::Literal> {
        let raw = &self.source()[span];
        let value = if raw.contains('\\') {
            let mut iter = raw.char_indices().map(|(i, c)| (span.m + i, c));
            let mut value = String::new();
            while let Some((_, c)) = iter.next() {
                match c {
                    '"' => continue,
                    '\\' => {
                        let (i, esc) = iter.next().unwrap();
                        let c = match esc {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            _ => {
                                let j = iter.next().unwrap().0;
                                return Err(Error::new(
                                    "unknown escape character",
                                    self.source(),
                                    i..j,
                                ));
                            }
                        };
                        value.push(c);
                    }
                    c => value.push(c),
                }
            }
            Value::String(value)
        } else {
            Value::String(raw[1..raw.len() - 1].to_owned())
        };
        Ok(ast::Literal { value, span })
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
        Ok((Keyword::from_str(kw), span))
    }

    /// Parses an identifier.
    fn parse_ident(&mut self) -> Result<ast::Ident<'source>> {
        let span = self.expect(Token::Ident)?;
        let raw = &self.source()[span];
        Ok(ast::Ident { raw, span })
    }

    /// Parses any token.
    fn parse(&mut self) -> Result<(Token, Span)> {
        match self.next()? {
            Some((tk, sp)) => Ok((tk, sp)),
            None => Err(self.err_unexpected_eof("token")),
        }
    }

    /// Parses the specified token and returns its span.
    fn expect(&mut self, exp: Token) -> Result<Span> {
        match self.next()? {
            Some((tk, span)) if tk == exp => Ok(span),
            Some((tk, span)) => Err(self.err_unexpected_token(exp.human(), tk, span)),
            None => Err(self.err_unexpected_eof(exp.human())),
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

    fn err_unexpected_eof(&self, exp: impl Display) -> Error {
        let n = self.source().len();
        Error::new(format!("expected {}, found EOF", exp), self.source(), n..n)
    }

    fn err_unexpected_token(&self, exp: impl Display, got: Token, span: Span) -> Error {
        Error::new(
            format!("expected {}, found {}", exp, got.human()),
            self.source(),
            span,
        )
    }

    fn err_unexpected_keyword(&self, kw: impl Display, span: Span) -> Error {
        Error::new(format!("unexpected keyword `{}`", kw), self.source(), span)
    }
}

impl Keyword {
    pub(crate) const fn all() -> &'static [&'static str] {
        &[
            "if", "not", "else", "endif", "for", "in", "endfor", "true", "false",
        ]
    }

    const fn human(&self) -> &'static str {
        match self {
            Self::If => "if",
            Self::Not => "not",
            Self::Else => "else",
            Self::EndIf => "endif",
            Self::For => "for",
            Self::In => "in",
            Self::EndFor => "endfor",
            Self::True | Self::False => "bool",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "if" => Self::If,
            "not" => Self::Not,
            "else" => Self::Else,
            "endif" => Self::EndIf,
            "for" => Self::For,
            "in" => Self::In,
            "endfor" => Self::EndFor,
            "true" => Self::True,
            "false" => Self::False,
            _ => unreachable!(),
        }
    }

    fn is_not(&self) -> bool {
        matches!(self, Self::Not)
    }
}

fn is_base10_integer(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

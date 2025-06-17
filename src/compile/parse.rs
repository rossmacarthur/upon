use std::fmt::Display;

use crate::compile::lex::{Lexer, Token};
use crate::types::ast;
use crate::types::span::Span;
use crate::{Engine, Error, Result, Value};

/// A parser that constructs an AST from a token stream.
///
/// The parser is implemented as a simple hand written parser. It sometimes
/// needs to peek at the next token to know how to proceed and uses the `peeked`
/// buffer to do this.
pub struct Parser<'engine, 'source> {
    /// A lexer that tokenizes the template source.
    tokens: Lexer<'engine, 'source>,

    /// Remember a peeked value, even if it was `None`
    peeked: Option<Option<(Token, Span)>>,
}

/// Stores the state of a statement during parsing.
enum State {
    /// A partial `if` statement.
    If {
        /// Whether or not this `if` statement is an `else if` clause.
        is_else_if: bool,
        /// Whether this is an an `if not` or a `if` statement.
        not: bool,
        /// The condition in the `if` block.
        cond: ast::Expr,
        /// The span of the `if` block.
        span: Span,
        /// Whether or not this `if` statement has an `else` clause.
        has_else: bool,
    },

    /// A partial `for` statement.
    For {
        /// The loop variables.
        vars: ast::LoopVars,
        /// The value we are iterating over.
        iterable: ast::Expr,
        /// The span of the `for` block.
        span: Span,
    },

    /// A partial `with` statement.
    With {
        /// The expression to shadow.
        expr: ast::Expr,
        /// The name to assign to this expression.
        name: ast::Ident,
        /// The span of the `with` block.
        span: Span,
    },
}

/// A parsed block definition.
enum Block {
    If(bool, ast::Expr),
    Else,
    ElseIf(bool, ast::Expr),
    EndIf,
    For(ast::LoopVars, ast::Expr),
    EndFor,
    With(ast::Expr, ast::Ident),
    EndWith,
    Include(ast::String, Option<ast::Expr>),
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
    With,
    As,
    EndWith,
    Include,
    True,
    False,
}

#[derive(Clone, Copy)]
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
    pub fn parse_template(mut self) -> Result<ast::Template> {
        let mut blocks = vec![];
        let mut scopes = vec![ast::Scope::new()];

        while let Some(next) = self.next()? {
            let stmt = match next {
                // Simply raw template, emit a single statement for it.
                (Token::Raw, span) => ast::Stmt::Raw(span),

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
                    let _span = begin.combine(end);
                    ast::Stmt::InlineExpr(ast::InlineExpr { expr, _span })
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
                                is_else_if: false,
                                not,
                                cond,
                                span,
                                has_else: false,
                            });
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        // An `else if` clause. For example:
                        //
                        //   {% else if cond %}
                        //
                        // We expect that the previous block was an `if` block
                        // and update it accordingly. We must also push two
                        // scopes to the scope stack, one for the `else` and one
                        // for the `if`.
                        Block::ElseIf(not, cond) => {
                            let err =
                                || Error::syntax("unexpected `else if` block", self.source(), span);
                            match blocks.last_mut().ok_or_else(err)? {
                                State::If {
                                    has_else: has_else @ false,
                                    ..
                                } => {
                                    *has_else = true;
                                }
                                _ => return Err(err()),
                            }
                            blocks.push(State::If {
                                is_else_if: true,
                                not,
                                cond,
                                span,
                                has_else: false,
                            });
                            scopes.push(ast::Scope::new());
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
                            let err =
                                || Error::syntax("unexpected `else` block", self.source(), span);
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
                        // We have to make sure to pop back the scopes until we
                        // get to the original `if`. Any `else if` blocks along
                        // the way are desugared into an `if` statement.
                        Block::EndIf => {
                            let err =
                                || Error::syntax("unexpected `endif` block", self.source(), span);

                            loop {
                                match blocks.pop().ok_or_else(err)? {
                                    State::If {
                                        is_else_if,
                                        not,
                                        cond,
                                        has_else,
                                        ..
                                    } => {
                                        let else_branch = has_else.then(|| scopes.pop().unwrap());
                                        let then_branch = scopes.pop().unwrap();
                                        let stmt = ast::Stmt::IfElse(ast::IfElse {
                                            not,
                                            cond,
                                            then_branch,
                                            else_branch,
                                        });
                                        if !is_else_if {
                                            break stmt;
                                        }
                                        scopes.last_mut().unwrap().stmts.push(stmt);
                                    }
                                    _ => return Err(err()),
                                };
                            }
                        }

                        // The start of a `for` statement. For example:
                        //
                        //   {% for vars in iterable %}
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
                        // We expect that the previous block was a `for` block.
                        Block::EndFor => {
                            let err =
                                || Error::syntax("unexpected `endfor` block", self.source(), span);

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

                        // The start of a `with` statement. For example:
                        //
                        //   {% with expr as name %}
                        //
                        // We must push a block to the block stack and a scope
                        // to the scope stack because a with statement starts a
                        // new scope.
                        Block::With(expr, name) => {
                            blocks.push(State::With { expr, name, span });
                            scopes.push(ast::Scope::new());
                            continue;
                        }

                        // The end of a `with` statement. For example:
                        //
                        //   {% endwith %}
                        //
                        // We expect that the previous block was a `with` block.
                        Block::EndWith => {
                            let err =
                                || Error::syntax("unexpected `endwith` block", self.source(), span);

                            let with = match blocks.pop().ok_or_else(err)? {
                                State::With { expr, name, .. } => {
                                    let body = scopes.pop().unwrap();
                                    ast::With { expr, name, body }
                                }
                                _ => return Err(err()),
                            };
                            ast::Stmt::With(with)
                        }

                        // An `include` statement. For example:
                        //
                        //   {% include name with expr %}
                        //
                        Block::Include(name, globals) => {
                            ast::Stmt::Include(ast::Include { name, globals })
                        }
                    }
                }
                (tk, span) => {
                    panic!("lexer bug: received token `{tk:?}` at {span:?}");
                }
            };
            scopes.last_mut().unwrap().stmts.push(stmt);
        }

        if let Some(block) = blocks.first() {
            let (msg, span) = match block {
                State::If { span, .. } => ("unclosed `if` block", span),
                State::For { span, .. } => ("unclosed `for` block", span),
                State::With { span, .. } => ("unclosed `with` block", span),
            };
            return Err(Error::syntax(msg, self.source(), *span));
        }

        assert!(
            scopes.len() == 1,
            "parser bug: we should end with a single scope"
        );

        Ok(ast::Template {
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
    ///   with loop.index | is_even as even
    ///
    fn parse_block(&mut self) -> Result<Block> {
        let (kw, span) = self.parse_keyword()?;
        match kw {
            Keyword::If => {
                let (not, expr) = self.parse_if_cond()?;
                Ok(Block::If(not, expr))
            }
            Keyword::Else => {
                if self.is_next_keyword(Keyword::If)? {
                    self.expect_keyword(Keyword::If)?;
                    let (not, expr) = self.parse_if_cond()?;
                    Ok(Block::ElseIf(not, expr))
                } else {
                    Ok(Block::Else)
                }
            }
            Keyword::EndIf => Ok(Block::EndIf),
            Keyword::For => {
                let vars = self.parse_loop_vars()?;
                self.expect_keyword(Keyword::In)?;
                let iterable = self.parse_expr()?;
                Ok(Block::For(vars, iterable))
            }
            Keyword::EndFor => Ok(Block::EndFor),
            Keyword::With => {
                let expr = self.parse_expr()?;
                self.expect_keyword(Keyword::As)?;
                let name = self.parse_ident()?;
                Ok(Block::With(expr, name))
            }
            Keyword::EndWith => Ok(Block::EndWith),
            Keyword::Include => {
                let name = self.parse_string()?;
                let globals = if self.is_next_keyword(Keyword::With)? {
                    self.expect_keyword(Keyword::With)?;
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                Ok(Block::Include(name, globals))
            }
            kw => Err(self.err_unexpected_keyword(kw.human(), span)),
        }
    }

    /// Parses an if condition.
    ///
    /// This is an expression with an optional `not`.
    ///
    ///   not user.is_enabled
    ///
    fn parse_if_cond(&mut self) -> Result<(bool, ast::Expr)> {
        if self.is_next_keyword(Keyword::Not)? {
            self.expect_keyword(Keyword::Not)?;
            let expr = self.parse_expr()?;
            Ok((true, expr))
        } else {
            let expr = self.parse_expr()?;
            Ok((false, expr))
        }
    }

    /// Parses an expression.
    ///
    /// This is a base expression with zero or more function calls. For example:
    ///
    ///   user.name | lower | prefix: "Mr. "
    ///
    fn parse_expr(&mut self) -> Result<ast::Expr> {
        let mut expr = ast::Expr::Base(self.parse_base_expr()?);
        while self.is_next(Token::Pipe)? {
            self.expect(Token::Pipe)?;
            let name = self.parse_ident()?;
            let (args, span) = if self.is_next(Token::Colon)? {
                let span = self.expect(Token::Colon)?;
                let args = self.parse_args(span)?;
                let span = expr.span().combine(args.span);
                (Some(args), span)
            } else {
                (None, expr.span().combine(name.span))
            };
            let receiver = Box::new(expr);
            expr = ast::Expr::Filter(ast::Filter {
                name,
                args,
                receiver,
                span,
            });
        }
        Ok(expr)
    }

    /// Parses a variable or literal.
    ///
    /// This is either a variable like
    ///
    ///   users.2.name
    ///
    /// Or a literal like
    ///
    ///   "John Smith"
    ///
    ///    0x150
    ///
    fn parse_base_expr(&mut self) -> Result<ast::BaseExpr> {
        let expr = match self.parse()? {
            (Token::Keyword, span) => {
                let lit = self.parse_literal_bool(span)?;
                ast::BaseExpr::Literal(lit)
            }

            (Token::Minus, sign) => {
                let span = self.expect(Token::Number)?;
                let raw = &self.source()[span];
                let lit = self.parse_literal_number(Sign::Neg, sign.combine(span), raw)?;
                ast::BaseExpr::Literal(lit)
            }

            (Token::Plus, sign) => {
                let span = self.expect(Token::Number)?;
                let raw = &self.source()[span];
                let lit = self.parse_literal_number(Sign::Pos, sign.combine(span), raw)?;
                ast::BaseExpr::Literal(lit)
            }

            (Token::Number, span) => {
                let raw = &self.source()[span];
                let lit = self.parse_literal_number(Sign::Pos, span, raw)?;
                ast::BaseExpr::Literal(lit)
            }

            (Token::String, span) => {
                let lit = self.parse_literal_string(span)?;
                ast::BaseExpr::Literal(lit)
            }

            (Token::Dot, span) => {
                let access = self.parse_access()?;
                let first = ast::Member {
                    op: ast::AccessOp::Direct,
                    access,
                    span: span.combine(access.span()),
                };
                let var = self.parse_var(first)?;
                ast::BaseExpr::Var(var)
            }

            (Token::QuestionDot, span) => {
                let access = self.parse_access()?;
                let first = ast::Member {
                    op: ast::AccessOp::Optional,
                    access,
                    span: span.combine(access.span()),
                };
                let var = self.parse_var(first)?;
                ast::BaseExpr::Var(var)
            }

            (Token::Ident, span) => {
                if let Some((Token::OpenParen, _)) = self.peek()? {
                    let call = self.parse_call(span)?;
                    ast::BaseExpr::Call(call)
                } else {
                    let first = ast::Member {
                        op: ast::AccessOp::Direct,
                        access: ast::Access::Key(ast::Ident { span }),
                        span,
                    };
                    let var = self.parse_var(first)?;
                    ast::BaseExpr::Var(var)
                }
            }

            (Token::OpenBracket, span) => {
                let list = self.parse_list(span)?;
                ast::BaseExpr::List(list)
            }

            (Token::OpenBrace, span) => {
                let map = self.parse_map(span)?;
                ast::BaseExpr::Map(map)
            }

            (Token::OpenParen, span) => {
                let expr = self.parse_expr()?;
                let end = self.expect(Token::CloseParen)?;
                let span = span.combine(end);
                ast::BaseExpr::Paren(ast::Paren {
                    expr: Box::new(expr),
                    span,
                })
            }

            (tk, span) => {
                return Err(self.err_unexpected_token("expression", tk, span));
            }
        };
        Ok(expr)
    }

    /// Parses a function call.
    ///
    ///    name()
    ///
    ///    name("nested", arg, user?.age)
    ///
    fn parse_call(&mut self, span: Span) -> Result<ast::Call> {
        self.expect(Token::OpenParen)?;
        let name = ast::Ident { span };
        let args = if self.is_next(Token::CloseParen)? {
            None
        } else {
            Some(self.parse_args(span)?)
        };
        let end = self.expect(Token::CloseParen)?;
        let span = span.combine(end);
        Ok(ast::Call { name, args, span })
    }

    /// Parses a variable specification.
    ///
    ///    user
    ///
    ///    user.names.0
    ///
    ///    user?.age
    ///
    fn parse_var(&mut self, first: ast::Member) -> Result<ast::Var> {
        let mut path = vec![first];
        loop {
            match self.peek()? {
                Some((Token::Dot, sp)) => {
                    self.expect(Token::Dot)?;
                    let access = self.parse_access()?;
                    path.push(ast::Member {
                        op: ast::AccessOp::Direct,
                        access,
                        span: sp.combine(access.span()),
                    });
                }
                Some((Token::QuestionDot, sp)) => {
                    self.expect(Token::QuestionDot)?;
                    let access = self.parse_access()?;
                    path.push(ast::Member {
                        op: ast::AccessOp::Optional,
                        access,
                        span: sp.combine(access.span()),
                    });
                }
                _ => break,
            }
        }

        Ok(ast::Var { path })
    }

    /// Parses a type of member access.
    ///
    /// This is a path member which is either an index or an identifier.
    ///
    ///   users
    ///
    ///   2
    ///
    ///   name
    ///
    fn parse_access(&mut self) -> Result<ast::Access> {
        match self.parse()? {
            (Token::Index, span) => {
                let value = match self.source()[span].parse() {
                    Ok(value) => value,
                    Err(_) => {
                        return Err(Error::syntax(
                            format!(
                                "base 10 literal out of range for unsigned {}-bit integer",
                                usize::BITS
                            ),
                            self.source(),
                            span,
                        ));
                    }
                };
                Ok(ast::Access::Index(ast::Index { value, span }))
            }
            (Token::Ident, span) => Ok(ast::Access::Key(ast::Ident { span })),
            (tk, span) => Err(self.err_unexpected_token("identifier or index", tk, span)),
        }
    }

    /// Parses function arguments.
    ///
    /// This is just a comma separate list of base expressions. For example
    ///
    ///   user.name, "a string", true
    ///
    fn parse_args(&mut self, span: Span) -> Result<ast::Args> {
        let mut values = Vec::new();
        loop {
            values.push(self.parse_base_expr()?);
            if !self.is_next(Token::Comma)? {
                break;
            }
            self.expect(Token::Comma)?;
        }
        let span = span.combine(values.last().unwrap().span());
        Ok(ast::Args { values, span })
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
    fn parse_loop_vars(&mut self) -> Result<ast::LoopVars> {
        let key = self.parse_ident()?;
        if !self.is_next(Token::Comma)? {
            return Ok(ast::LoopVars::Item(key));
        }
        self.expect(Token::Comma)?;
        let value = self.parse_ident()?;
        let span = key.span.combine(value.span);
        Ok(ast::LoopVars::KeyValue(ast::KeyValue { key, value, span }))
    }

    /// Parses a boolean argument.
    fn parse_literal_bool(&mut self, span: Span) -> Result<ast::Literal> {
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
    fn parse_literal_number(&self, sign: Sign, span: Span, raw: &str) -> Result<ast::Literal> {
        match self.parse_literal_integer(sign, span, raw) {
            Ok(lit) => Ok(lit),
            Err(err) => match self.parse_literal_float(sign, span, raw) {
                Ok(lit) => Ok(lit),
                Err(err2) => {
                    if raw.contains(['.', '-', '+']) {
                        Err(err2)
                    } else {
                        Err(err)
                    }
                }
            },
        }
    }

    /// Parse a literal integer.
    fn parse_literal_integer(&self, sign: Sign, span: Span, raw: &str) -> Result<ast::Literal> {
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
                    Error::syntax(
                        format!("invalid digit for base {radix} literal"),
                        self.source(),
                        m..m + 1,
                    )
                })?;
                let err = || {
                    Error::syntax(
                        format!("base {radix} literal out of range for 64-bit integer"),
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

    /// Parses a literal float.
    fn parse_literal_float(&self, sign: Sign, span: Span, raw: &str) -> Result<ast::Literal> {
        let float: f64 = raw
            .parse()
            .map_err(|_| Error::syntax("invalid float literal", self.source(), span))?;
        let value = match sign {
            Sign::Neg => Value::Float(-float),
            Sign::Pos => Value::Float(float),
        };
        Ok(ast::Literal { value, span })
    }

    /// Parses a literal string.
    fn parse_literal_string(&self, span: Span) -> Result<ast::Literal> {
        let value = Value::String(self.parse_quoted_string(span)?);
        Ok(ast::Literal { value, span })
    }

    /// Parses a string.
    fn parse_string(&mut self) -> Result<ast::String> {
        let span = self.expect(Token::String)?;
        let value = self.parse_quoted_string(span)?;
        Ok(ast::String { value, span })
    }

    /// Parses a quoted string and handles escape characters.
    fn parse_quoted_string(&self, span: Span) -> Result<String> {
        let raw = &self.source()[span];
        let string = if raw.contains('\\') {
            let mut iter = raw.char_indices().map(|(i, c)| (span.m + i, c));
            let mut string = String::new();
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
                                return Err(Error::syntax(
                                    "unknown escape character",
                                    self.source(),
                                    i..j,
                                ));
                            }
                        };
                        string.push(c);
                    }
                    c => string.push(c),
                }
            }
            string
        } else {
            String::from(&raw[1..raw.len() - 1])
        };
        Ok(string)
    }

    /// Parses a list.
    fn parse_list(&mut self, span: Span) -> Result<ast::List> {
        let mut items = Vec::new();
        loop {
            if self.is_next(Token::CloseBracket)? {
                break;
            }
            let item = self.parse_base_expr()?;
            items.push(item);
            if !self.is_next(Token::Comma)? {
                break;
            }
            self.expect(Token::Comma)?;
        }
        let span = span.combine(self.expect(Token::CloseBracket)?);
        Ok(ast::List { items, span })
    }

    /// Parses a map.
    fn parse_map(&mut self, span: Span) -> Result<ast::Map> {
        let mut items = Vec::new();
        loop {
            if self.is_next(Token::CloseBrace)? {
                break;
            }
            let key = self.parse_string()?;
            self.expect(Token::Colon)?;
            let value = self.parse_base_expr()?;
            items.push((key, value));
            if !self.is_next(Token::Comma)? {
                break;
            }
            self.expect(Token::Comma)?;
        }
        let span = span.combine(self.expect(Token::CloseBrace)?);
        Ok(ast::Map { items, span })
    }

    /// Expects the given keyword.
    fn expect_keyword(&mut self, exp: Keyword) -> Result<Span> {
        let (kw, span) = self.parse_keyword()?;
        if kw != exp {
            let exp = exp.human();
            let kw = kw.human();
            return Err(Error::syntax(
                format!("expected keyword `{exp}`, found keyword `{kw}`"),
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
    fn parse_ident(&mut self) -> Result<ast::Ident> {
        let span = self.expect(Token::Ident)?;
        Ok(ast::Ident { span })
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

    /// Returns `true` if the next token is a keyword equal to the provided one.
    fn is_next_keyword(&mut self, exp: Keyword) -> Result<bool> {
        Ok(self
            .peek()?
            .map(|(tk, sp)| tk == Token::Keyword && Keyword::from_str(&self.source()[sp]) == exp)
            .unwrap_or(false))
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

    fn source(&self) -> &str {
        self.tokens.source
    }

    fn err_unexpected_eof(&self, exp: impl Display) -> Error {
        let n = self.source().len();
        Error::syntax(format!("expected {exp}, found EOF"), self.source(), n..n)
    }

    fn err_unexpected_token(&self, exp: impl Display, got: Token, span: Span) -> Error {
        let got = got.human();
        Error::syntax(format!("expected {exp}, found {got}"), self.source(), span)
    }

    fn err_unexpected_keyword(&self, kw: impl Display, span: Span) -> Error {
        Error::syntax(format!("unexpected keyword `{kw}`"), self.source(), span)
    }
}

impl Keyword {
    pub(crate) const fn all() -> &'static [&'static str] {
        &[
            "if", "not", "else", "endif", "for", "in", "endfor", "with", "as", "endwith",
            "include", "true", "false",
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
            Self::With => "with",
            Self::As => "as",
            Self::EndWith => "endwith",
            Self::Include => "include",
            Self::True => "true",
            Self::False => "false",
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
            "with" => Self::With,
            "as" => Self::As,
            "endwith" => Self::EndWith,
            "include" => Self::Include,
            "true" => Self::True,
            "false" => Self::False,
            _ => unreachable!(),
        }
    }
}

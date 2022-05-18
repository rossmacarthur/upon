//! Renders a compiled template.

use std::fmt::Write;
use std::slice::Iter;

use crate::ast;
use crate::{Engine, Error, Result, Value};

/// A renderer that can render a compiled template as a string.
pub struct Renderer<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: &'source ast::Template<'source>,
}

enum State<'source> {
    Scope {
        stmts: Iter<'source, ast::Stmt<'source>>,
    },
    ForLoop {
        vars: &'source ast::LoopVars<'source>,
        iter: IntoIter,
        body: &'source ast::Scope<'source>,
    },
}

impl<'engine, 'source> Renderer<'engine, 'source> {
    pub fn new(
        engine: &'engine Engine<'engine>,
        template: &'source ast::Template<'source>,
    ) -> Self {
        Self { engine, template }
    }

    /// Renders a template using the provided data.
    ///
    /// This function works using two stacks:
    /// - A stack of blocks containing the state of a scope or for loop.
    /// - A stack of variables for each state.
    pub fn render(&self, globals: Value) -> Result<String> {
        let mut buf = String::new();

        let mut blocks = vec![State::scope(&self.template.scope)];
        let mut locals = vec![globals];

        'blocks: while let Some(state) = blocks.last_mut() {
            match state {
                // Currently iterating over a scope. Advance to the next
                // statement and optionally start a new state.
                State::Scope { stmts } => {
                    for stmt in stmts {
                        match stmt {
                            // Raw template, simply write it to the buffer.
                            ast::Stmt::Raw(raw) => {
                                buf.push_str(raw);
                            }

                            // An inline expression, simply evaluate it and
                            // write the rendered value to the buffer.
                            ast::Stmt::InlineExpr(ast::InlineExpr { expr, .. }) => {
                                match self.eval_expr(&locals, expr)? {
                                    Value::None => {}
                                    Value::Bool(b) => write!(buf, "{}", b).unwrap(),
                                    Value::Integer(n) => write!(buf, "{}", n).unwrap(),
                                    Value::Float(n) => write!(buf, "{}", n).unwrap(),
                                    Value::String(s) => write!(buf, "{}", s).unwrap(),
                                    val => {
                                        return Err(Error::new(
                                            format!(
                                            "expected renderable value, but expression evaluated to {}",
                                            val.human()
                                        ),
                                            self.source(),
                                            expr.span(),
                                        ));
                                    }
                                }
                            }

                            // An `if` statement. We need to evaluate the
                            // condition first and push the correct branch scope
                            // to the block scope.
                            ast::Stmt::IfElse(ast::IfElse {
                                cond,
                                then_branch,
                                else_branch,
                            }) => {
                                let cond = match self.eval_expr(&locals, cond)? {
                                    Value::Bool(cond) => cond,
                                    value => {
                                        return Err(Error::new(
                                            format!(
                                                "expected bool, but expression evaluated to {}",
                                                value.human()
                                            ),
                                            self.source(),
                                            cond.span(),
                                        ));
                                    }
                                };
                                if cond {
                                    blocks.push(State::scope(then_branch));
                                    continue 'blocks;
                                } else if let Some(else_branch) = &else_branch {
                                    blocks.push(State::scope(else_branch));
                                    continue 'blocks;
                                }
                            }

                            // A `for` statement. We need to evaluate the
                            // iterable and create an iterator.
                            ast::Stmt::ForLoop(ast::ForLoop {
                                vars,
                                iterable,
                                body,
                            }) => {
                                let (vars, iter) = match self.eval_expr(&locals, iterable)? {
                                    Value::List(list) => (vars, IntoIter::List(list.into_iter())),
                                    Value::Map(map) => (vars, IntoIter::Map(map.into_iter())),
                                    value => {
                                        return Err(Error::new(
                                            format!(
                                                "expected iterable, but expression evaluated to {}",
                                                value.human()
                                            ),
                                            self.source(),
                                            iterable.span(),
                                        ));
                                    }
                                };
                                // Push a dummy variable, that will be replaced
                                // by the first loop variable.
                                locals.push(Value::None);
                                blocks.push(State::ForLoop { vars, iter, body });
                                continue 'blocks;
                            }
                        }
                    }
                }

                // Currently iterating over a `for` statement. Advance by one
                // element and push the body on to the stack.
                State::ForLoop { vars, iter, body } => {
                    match iter.next(&self.template, vars)? {
                        Some(next_locals) => {
                            *locals.last_mut().unwrap() = next_locals;
                            let body: &ast::Scope<'_> = *body; // ¯\_(ツ)_/¯
                            blocks.push(State::scope(body));
                            continue 'blocks;
                        }
                        None => {
                            locals.pop().unwrap();
                        }
                    }
                }
            }

            blocks.pop().unwrap();
        }

        Ok(buf)
    }

    /// Recursively evaluates an expression.
    fn eval_expr(&self, locals: &[Value], expr: &ast::Expr<'_>) -> Result<Value> {
        match expr {
            ast::Expr::Var(ast::Var { path, .. }) => self.resolve_value(locals, path),
            ast::Expr::Call(ast::Call { name, receiver, .. }) => {
                let func = self.engine.filters.get(name.raw).ok_or_else(|| {
                    Error::new("unknown filter function", self.source(), name.span)
                })?;
                Ok((func)(self.eval_expr(locals, receiver)?))
            }
        }
    }

    /// Resolves a path to a value in the given locals stack.
    fn resolve_value(&self, locals: &[Value], path: &[ast::Ident]) -> Result<Value> {
        'outer: for vars in locals.iter().rev() {
            let mut result = vars;
            for (i, segment) in path.iter().enumerate() {
                result = match self.lookup_value(result, segment) {
                    Ok(d) => d,
                    Err(err) => {
                        // If it is the first segment of the path then we can
                        // try another locals.
                        if i == 0 {
                            continue 'outer;
                        }
                        return Err(err);
                    }
                };
            }
            return Ok(result.clone());
        }
        Err(Error::new(
            "not found in this scope",
            self.source(),
            path[0].span,
        ))
    }

    // Lookup an index in a value.
    fn lookup_value<'render>(
        &self,
        value: &'render Value,
        idx: &ast::Ident<'_>,
    ) -> Result<&'render Value> {
        let ast::Ident { raw, span } = idx;
        match value {
            Value::List(list) => match raw.parse::<usize>() {
                Ok(i) => Ok(&list[i]),
                Err(_) => Err(Error::new(
                    "cannot index list with string",
                    self.source(),
                    *span,
                )),
            },
            Value::Map(map) => match map.get(*raw) {
                Some(value) => Ok(value),
                None => Err(Error::new("not found in map", self.source(), *span)),
            },
            val => Err(Error::new(
                format!("cannot index into {}", val.human()),
                self.source(),
                *span,
            )),
        }
    }

    fn source(&self) -> &'source str {
        self.template.source
    }
}

impl<'source> State<'source> {
    fn scope(scope: &'source ast::Scope<'source>) -> Self {
        Self::Scope {
            stmts: scope.stmts.iter(),
        }
    }
}

impl Value {
    fn human(&self) -> &'static str {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
    }
}

/// Wrapper for an owned [`Value`] iterator.
enum IntoIter {
    List(crate::value::ListIntoIter),
    Map(crate::value::MapIntoIter),
}

impl IntoIter {
    /// Returns the next value in the iterator, or `None` if exhausted.
    fn next(
        &mut self,
        template: &ast::Template<'_>,
        vars: &ast::LoopVars<'_>,
    ) -> Result<Option<Value>> {
        match self {
            IntoIter::List(list) => {
                let v = match list.next() {
                    Some(v) => v,
                    None => return Ok(None),
                };
                match vars {
                    ast::LoopVars::Item(item) => Ok(Some(Value::from([(item.raw, v)]))),
                    ast::LoopVars::KeyValue(kv) => Err(Error::new(
                        "cannot unpack list item into two variables",
                        template.source,
                        kv.span,
                    )),
                }
            }
            IntoIter::Map(map) => {
                let (vk, vv) = match map.next() {
                    Some(v) => v,
                    None => return Ok(None),
                };
                match vars {
                    ast::LoopVars::Item(item) => Err(Error::new(
                        "cannot unpack map item into one variable",
                        template.source,
                        item.span,
                    )),
                    ast::LoopVars::KeyValue(kv) => Ok(Some(Value::from([
                        (kv.key.raw, Value::from(vk)),
                        (kv.value.raw, vv),
                    ]))),
                }
            }
        }
    }
}

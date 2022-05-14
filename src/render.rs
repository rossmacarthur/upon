//! Renders a compiled template.

use std::fmt::Write;
use std::slice::Iter;

use crate::ast;
use crate::{Engine, Error, Result, Value};

enum State<'source> {
    Block {
        stmts: Iter<'source, ast::Stmt<'source>>,
    },
    Loop {
        vars: LoopVars<'source>,
        iter: IntoIter,
        body: &'source ast::Scope<'source>,
    },
}

impl<'source> State<'source> {
    fn new(scope: &'source ast::Scope<'source>) -> Self {
        Self::Block {
            stmts: scope.stmts.iter(),
        }
    }
}

pub fn template<'engine, 'source>(
    engine: &Engine<'engine>,
    template: &ast::Template<'source>,
    globals: Value,
) -> Result<String> {
    let mut buf = String::new();

    let mut stack = vec![State::new(&template.scope)];
    let mut locals = vec![globals];

    'outer: while let Some(state) = stack.last_mut() {
        match state {
            State::Block { stmts } => {
                for stmt in stmts {
                    match stmt {
                        ast::Stmt::Raw(raw) => {
                            buf.push_str(raw);
                            continue;
                        }

                        ast::Stmt::InlineExpr(ast::InlineExpr { expr, .. }) => {
                            let value = eval(engine, template.source, &locals, expr)?;
                            write!(buf, "{}", value).unwrap();
                            continue;
                        }

                        ast::Stmt::IfElse(ast::IfElse {
                            cond,
                            then_branch,
                            else_branch,
                        }) => {
                            let cond = match eval(engine, template.source, &locals, cond)? {
                                Value::Bool(cond) => cond,
                                val => {
                                    return Err(Error::new(
                                        format!(
                                            "expected bool, but expression evaluated to {}",
                                            val.human()
                                        ),
                                        template.source,
                                        cond.span(),
                                    ));
                                }
                            };
                            if cond {
                                stack.push(State::new(then_branch));
                                continue 'outer;
                            } else if let Some(else_branch) = &else_branch {
                                stack.push(State::new(else_branch));
                                continue 'outer;
                            } else {
                                continue;
                            }
                        }

                        ast::Stmt::ForLoop(ast::ForLoop {
                            vars,
                            iterable,
                            body,
                        }) => {
                            let (vars, iter) =
                                match eval(engine, template.source, &locals, iterable)? {
                                    Value::List(list) => match vars {
                                        ast::LoopVars::Item(item) => {
                                            let vars = LoopVars::Item(item.value);
                                            let iter = IntoIter::List(list.into_iter());
                                            (vars, iter)
                                        }
                                        ast::LoopVars::KeyValue(kv) => {
                                            return Err(Error::new(
                                                "cannot unpack list item into two variables",
                                                template.source,
                                                kv.span,
                                            ));
                                        }
                                    },
                                    Value::Map(map) => match vars {
                                        ast::LoopVars::Item(item) => {
                                            return Err(Error::new(
                                                "cannot unpack map item into one variable",
                                                template.source,
                                                item.span,
                                            ));
                                        }
                                        ast::LoopVars::KeyValue(kv) => {
                                            let iter = IntoIter::Map(map.into_iter());
                                            let vars =
                                                LoopVars::KeyValue(kv.key.value, kv.value.value);
                                            (vars, iter)
                                        }
                                    },
                                    val => {
                                        return Err(Error::new(
                                            format!(
                                                "expected iterable, but expression evaluated to {}",
                                                val.human()
                                            ),
                                            template.source,
                                            iterable.span(),
                                        ));
                                    }
                                };
                            locals.push(Value::None); // dummy
                            stack.push(State::Loop { vars, iter, body });
                            continue 'outer;
                        }
                    }
                }
            }

            State::Loop { vars, iter, body } => {
                let body: &ast::Scope<'_> = *body; // needed for some reason

                if let Some(next_locals) = iter.next(vars) {
                    *locals.last_mut().unwrap() = next_locals;
                    stack.push(State::new(body));
                    continue 'outer;
                }
            }
        }

        stack.pop().unwrap();
    }

    Ok(buf)
}

fn eval(
    engine: &Engine<'_>,
    source: &str,
    locals: &[Value],
    expr: &ast::Expr<'_>,
) -> Result<Value> {
    match expr {
        ast::Expr::Var(ast::Var { path, .. }) => resolve(locals, source, path),
        ast::Expr::Call(ast::Call { name, receiver, .. }) => {
            let func = engine
                .filters
                .get(name.value)
                .ok_or_else(|| Error::new("unknown filter function", source, name.span))?;
            Ok((func)(eval(engine, source, locals, receiver)?))
        }
    }
}

fn resolve(locals: &[Value], source: &str, path: &[ast::Ident]) -> Result<Value> {
    'outer: for (i, vars) in locals.iter().enumerate().rev() {
        let mut result = vars;
        for (j, segment) in path.iter().enumerate() {
            result = match lookup(source, result, segment) {
                Ok(d) => d,
                Err(err) => {
                    // If it is the first segment of the path then we can try
                    // another locals. If we are on the last locals then error
                    // anyway.
                    if j == 0 && i != 0 {
                        continue 'outer;
                    }
                    return Err(err);
                }
            };
        }
        return Ok(result.clone());
    }
    Err(Error::new("not found in map", source, path[0].span))
}

fn lookup<'render>(
    source: &str,
    data: &'render Value,
    idx: &ast::Ident<'_>,
) -> Result<&'render Value> {
    let ast::Ident { value: idx, span } = idx;
    match data {
        Value::List(list) => match idx.parse::<usize>() {
            Ok(i) => Ok(&list[i]),
            Err(_) => Err(Error::new("cannot index list with string", source, *span)),
        },
        Value::Map(map) => match map.get(*idx) {
            Some(value) => Ok(value),
            None => Err(Error::new("not found in map", source, *span)),
        },
        val => Err(Error::new(
            format!("cannot index into {}", val.human()),
            source,
            *span,
        )),
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

enum LoopVars<'source> {
    Item(&'source str),
    KeyValue(&'source str, &'source str),
}

enum IntoIter {
    List(crate::value::ListIntoIter),
    Map(crate::value::MapIntoIter),
}

impl IntoIter {
    fn next(&mut self, vars: &LoopVars<'_>) -> Option<Value> {
        match self {
            IntoIter::List(list) => {
                let item = list.next()?;
                match vars {
                    LoopVars::Item(var) => Some(Value::from([(*var, item)])),
                    LoopVars::KeyValue(_, _) => unreachable!(),
                }
            }
            IntoIter::Map(map) => {
                let (left, right) = map.next()?;
                match vars {
                    LoopVars::Item(_) => unreachable!(),
                    LoopVars::KeyValue(k, v) => {
                        Some(Value::from([(*k, Value::from(left)), (*v, right)]))
                    }
                }
            }
        }
    }
}

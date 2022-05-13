//! Renders a compiled template.

use std::fmt::Write;
use std::slice::Iter;

use crate::ast;
use crate::{Engine, Error, Result, Value};

enum State<'t> {
    Block {
        stmts: Iter<'t, ast::Stmt<'t>>,
    },
    Loop {
        vars: LoopVars<'t>,
        iter: IntoIter,
        body: &'t ast::Scope<'t>,
    },
}

impl<'t> State<'t> {
    fn new(scope: &'t ast::Scope<'t>) -> Self {
        Self::Block {
            stmts: scope.stmts.iter(),
        }
    }
}

pub fn template<'t>(engine: &Engine<'_>, t: &ast::Template<'_>, globals: Value) -> Result<String> {
    let mut buf = String::new();

    let mut stack = vec![State::new(&t.scope)];
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
                            let value = eval(engine, t.source, &locals, expr)?;
                            write!(buf, "{}", value).unwrap();
                            continue;
                        }

                        ast::Stmt::IfElse(ast::IfElse {
                            cond,
                            then_branch,
                            else_branch,
                        }) => {
                            let cond = match eval(engine, t.source, &locals, cond)? {
                                Value::Bool(cond) => cond,
                                val => {
                                    return Err(Error::span(
                                        format!(
                                            "expected bool, but expression evaluated to {}",
                                            val.human()
                                        ),
                                        t.source,
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
                            let (vars, iter) = match eval(engine, t.source, &locals, iterable)? {
                                Value::List(list) => match vars {
                                    ast::LoopVars::Item(item) => {
                                        let vars = LoopVars::Item(item.value);
                                        let iter = IntoIter::List(list.into_iter());
                                        (vars, iter)
                                    }
                                    ast::LoopVars::KeyValue(kv) => {
                                        return Err(Error::span(
                                            "cannot unpack list item into two variables",
                                            t.source,
                                            kv.span,
                                        ));
                                    }
                                },
                                Value::Map(map) => match vars {
                                    ast::LoopVars::Item(item) => {
                                        return Err(Error::span(
                                            "cannot unpack map item into one variable",
                                            t.source,
                                            item.span,
                                        ));
                                    }
                                    ast::LoopVars::KeyValue(kv) => {
                                        let iter = IntoIter::Map(map.into_iter());
                                        let vars = LoopVars::KeyValue(kv.key.value, kv.value.value);
                                        (vars, iter)
                                    }
                                },
                                val => {
                                    return Err(Error::span(
                                        format!(
                                            "expected iterable, but expression evaluated to {}",
                                            val.human()
                                        ),
                                        t.source,
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

fn eval<'v>(
    engine: &Engine<'_>,
    source: &str,
    locals: &[Value],
    expr: &ast::Expr<'_>,
) -> Result<Value> {
    match expr {
        ast::Expr::Var(ast::Var { path, .. }) => lookup(locals, source, path),
        ast::Expr::Call(ast::Call { name, receiver, .. }) => {
            let func = engine
                .filters
                .get(name.value)
                .ok_or_else(|| Error::span("unknown filter function", source, name.span))?;
            Ok((func)(eval(engine, source, locals, receiver)?))
        }
    }
}

fn lookup(locals: &[Value], source: &str, path: &[ast::Ident]) -> Result<Value> {
    'outer: for mut data in locals.iter().rev() {
        for (i, ast::Ident { span, value: p }) in path.iter().enumerate() {
            data = match data {
                Value::List(list) => match p.parse::<usize>() {
                    Ok(i) => &list[i],
                    Err(_) => {
                        if i == 0 {
                            continue 'outer;
                        }
                        return Err(Error::span("cannot index list with string", source, *span));
                    }
                },

                Value::Map(map) => match map.get(*p) {
                    Some(value) => value,
                    None => {
                        if i == 0 {
                            continue 'outer;
                        }
                        return Err(Error::span("not found in map", source, *span));
                    }
                },

                val => {
                    if i == 0 {
                        continue 'outer;
                    }
                    return Err(Error::span(
                        format!("cannot index into {}", val.human()),
                        source,
                        *span,
                    ));
                }
            }
        }
        return Ok(data.clone());
    }
    return Err(Error::span("not found in map", source, path[0].span));
}

enum LoopVars<'t> {
    Item(&'t str),
    KeyValue(&'t str, &'t str),
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
                    LoopVars::KeyValue(_, _) => unimplemented!(),
                }
            }
            IntoIter::Map(map) => {
                let (left, right) = map.next()?;
                match vars {
                    LoopVars::Item(var) => unimplemented!(),
                    LoopVars::KeyValue(k, v) => {
                        Some(Value::from([(*k, Value::from(left)), (*v, right)]))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn template_inline_expr() {
        let engine = Engine::new();
        let t = engine.compile("lorem {{ ipsum.dolor }}").unwrap();
        let globals = crate::data! { ipsum: { dolor: "testing..." } };
        let result = template(&engine, &t.template, globals).unwrap();
        assert_eq!(result, "lorem testing...");
    }

    #[test]
    fn template_if_else_cond_true() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% if ipsum.dolor %} {{ sit }} {% else %} {{ amet }} {% endif %}")
            .unwrap();
        let globals = crate::data! {
            ipsum: { dolor: true },
            sit: "testing..."
        };
        let result = template(&engine, &t.template, globals).unwrap();
        assert_eq!(result, "lorem  testing... ");
    }

    #[test]
    fn template_if_else_cond_false() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% if ipsum.dolor %} {{ sit }} {% else %} {{ amet }} {% endif %}")
            .unwrap();
        let globals = crate::data! {
            ipsum: { dolor: false },
            amet: "testing..."
        };
        let result = template(&engine, &t.template, globals).unwrap();
        assert_eq!(result, "lorem  testing... ");
    }

    #[test]
    fn template_if_cond_not_bool() {
        let engine = Engine::new();
        let t = engine.compile("{% if cond %}test{% endif %}").unwrap();
        let globals = Value::from([("cond", "not bool")]);
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {% if cond %}test{% endif %}
   |       ^^^^ expected bool, but expression evaluated to string
"
        );
    }

    #[test]
    fn template_if_cond_expr_not_bool() {
        let mut engine = Engine::new();
        engine.add_filter("fn", |_| Value::None);
        let t = engine.compile("{% if cond | fn %}test{% endif %}").unwrap();
        let globals = Value::from([("cond", true)]);
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {% if cond | fn %}test{% endif %}
   |       ^^^^^^^^^ expected bool, but expression evaluated to none
"
        );
    }

    #[test]
    fn template_for_loop_over_list() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% for ipsum in dolor %}{{ ipsum }}{% endfor %}")
            .unwrap();
        let globals = crate::data! {
            dolor: ["t", "e", "s", "t"]
        };
        let result = template(&engine, &t.template, globals).unwrap();
        assert_eq!(result, "lorem test");
    }

    #[test]
    fn template_for_loop_over_map() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% for k, v in dolor %}{{ v }}{% endfor %}")
            .unwrap();
        let globals = crate::data! { dolor: { a: "t" } };
        let result = template(&engine, &t.template, globals).unwrap();
        assert_eq!(result, "lorem t");
    }

    #[test]
    fn template_for_loop_not_iterable() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% for item in dolor %}{{ item }}{% endfor %}")
            .unwrap();
        let globals = crate::data! { dolor: true };
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for item in dolor %}{{ item }}{% endfor %}
   |                      ^^^^^ expected iterable, but expression evaluated to bool
"
        );
    }

    #[test]
    fn template_for_loop_list_with_two_vars() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% for k, v in dolor %}{{ item }}{% endfor %}")
            .unwrap();
        let globals = crate::data! { dolor: [] };
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for k, v in dolor %}{{ item }}{% endfor %}
   |              ^^^^ cannot unpack list item into two variables
"
        );
    }

    #[test]
    fn template_for_loop_map_with_one_var() {
        let engine = Engine::new();
        let t = engine
            .compile("lorem {% for item in dolor %}{{ item }}{% endfor %}")
            .unwrap();
        let globals = crate::data! { dolor: {} };
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | lorem {% for item in dolor %}{{ item }}{% endfor %}
   |              ^^^^ cannot unpack map item into one variable
"
        );
    }

    #[test]
    fn template_unknown_filter_function() {
        let engine = Engine::new();
        let t = engine.compile("{{ cond | fn }}").unwrap();
        let globals = Value::from([("cond", true)]);
        let err = template(&engine, &t.template, globals).unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ cond | fn }}
   |           ^^ unknown filter function
"
        );
    }
}

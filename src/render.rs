//! Renders a compiled template.

use std::fmt::Write;
use std::slice::Iter;

use crate::ast;
use crate::{Engine, Error, Result, Value};

struct State<'t> {
    stmts: Iter<'t, ast::Stmt<'t>>,
}

impl<'t> State<'t> {
    fn new(scope: &'t ast::Scope<'t>) -> Self {
        Self {
            stmts: scope.stmts.iter(),
        }
    }
}

pub fn template(engine: &Engine<'_>, t: &ast::Template<'_>, globals: Value) -> Result<String> {
    let mut buf = String::new();

    let ast::Template { scope, source } = t;

    let mut stack = vec![State {
        stmts: scope.stmts.iter(),
    }];

    'outer: while let Some(State { stmts }) = stack.last_mut() {
        for stmt in stmts.by_ref() {
            match stmt {
                ast::Stmt::Raw(raw) => {
                    buf.push_str(raw);
                }

                ast::Stmt::InlineExpr(ast::InlineExpr { expr, .. }) => {
                    let value = eval(engine, t.source, &globals, expr)?;
                    write!(buf, "{}", value).unwrap();
                }

                ast::Stmt::IfElse(ast::IfElse {
                    cond,
                    then_branch,
                    else_branch,
                }) => {
                    let cond = match eval(engine, t.source, &globals, cond)? {
                        Value::Bool(cond) => cond,
                        val => {
                            return Err(Error::span(
                                format!(
                                    "expected bool, but expression evaluated to {}",
                                    val.human()
                                ),
                                source,
                                cond.span(),
                            ))
                        }
                    };
                    if cond {
                        stack.push(State::new(then_branch))
                    } else if let Some(else_branch) = &else_branch {
                        stack.push(State::new(else_branch))
                    }
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
    globals: &'v Value,
    expr: &ast::Expr<'_>,
) -> Result<Value> {
    match expr {
        ast::Expr::Var(ast::Var { path, .. }) => globals.lookup(source, path).cloned(),
        ast::Expr::Call(ast::Call { name, receiver, .. }) => {
            let func = engine
                .filters
                .get(name.ident)
                .ok_or_else(|| Error::span("unknown filter function", source, name.span))?;
            Ok((func)(eval(engine, source, globals, receiver)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

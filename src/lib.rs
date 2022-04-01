mod ast;
mod engine;
mod macros;
mod result;
pub mod value;

use crate::ast::{Expr, Span};
pub use crate::engine::Engine;
pub use crate::result::{Error, Result};
pub use crate::value::Value;

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template<'e> {
    engine: &'e Engine<'e>,
    tmpl: &'e str,
    subs: Vec<Sub<'e>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sub<'e> {
    span: Span,
    expr: Expr<'e>,
}

impl<'e> Template<'e> {
    fn with_env(tmpl: &'e str, env: &'e Engine<'e>) -> Result<Self> {
        let mut cursor = 0;
        let mut subs = Vec::new();

        loop {
            let (i, m) = match tmpl[cursor..].find(env.begin_tag) {
                Some(m) => (m, m + env.begin_tag.len()),
                None => {
                    if let Some(n) = tmpl[cursor..].find(env.end_tag) {
                        let span = Span::new(n, n + env.end_tag.len());
                        return Err(Error::new("unexpected end tag", tmpl, span));
                    }
                    return Ok(Template {
                        engine: env,
                        tmpl,
                        subs,
                    });
                }
            };

            let (j, n) = match tmpl[m..].find(env.end_tag) {
                Some(n) => (m + n + env.end_tag.len(), m + n),
                None => {
                    let span = Span::new(i, m);
                    return Err(Error::new("unclosed tag", tmpl, span));
                }
            };

            let outer = Span::new(i, j);
            let inner = Span::new(m, n);
            let expr = ast::parse_expr(tmpl, inner)?;
            subs.push(Sub { span: outer, expr });

            cursor = j;
        }
    }

    /// Render the template to a string using the provided data.
    pub fn render<V>(&self, data: V) -> Result<String>
    where
        V: Into<Value>,
    {
        let data = data.into();
        let mut s = String::new();
        let mut i = 0;
        for Sub { span, expr } in &self.subs {
            s.push_str(&self.tmpl[i..span.m]);
            i = span.n;
            let value = render_expr(self.tmpl, self.engine, &data, expr)?;
            s.push_str(&value.to_string());
        }
        s.push_str(&self.tmpl[i..]);

        Ok(s)
    }
}

fn render_expr(tmpl: &str, env: &Engine<'_>, data: &Value, expr: &Expr<'_>) -> Result<Value> {
    match expr {
        Expr::Value(ast::Value { path, .. }) => data.lookup(tmpl, path).map(|v| v.clone()),
        Expr::Call(ast::Call { name, receiver, .. }) => match env.filters.get(name.ident) {
            Some(f) => render_expr(tmpl, env, data, receiver).map(&**f),
            None => {
                return Err(Error::new(
                    format!("function not found `{}`", name.ident),
                    tmpl,
                    name.span,
                ))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn env_compile_no_tags() {
        let env = Engine::new();

        let t = env.compile("").unwrap();
        assert_eq!(t.subs, Vec::new());

        let t = env.compile("just testing").unwrap();
        assert_eq!(t.subs, Vec::new());
    }

    #[test]
    fn env_compile_default_tags() {
        let env = Engine::new();
        let t = env.compile("test {{ basic }} test").unwrap();
        let subs = vec![Sub {
            span: Span::new(5, 16),
            expr: Expr::Value(ast::Value {
                span: Span::new(8, 13),
                path: vec![ast::Ident {
                    span: Span::new(8, 13),
                    ident: "basic",
                }],
            }),
        }];
        assert_eq!(t.subs, subs);
    }

    #[test]
    fn env_compile_custom_tags() {
        let env = Engine::with_tags("<", "/>");
        let t = env.compile("test <basic/> test").unwrap();
        let subs = vec![Sub {
            span: Span::new(5, 13),
            expr: Expr::Value(ast::Value {
                span: Span::new(6, 11),
                path: vec![ast::Ident {
                    span: Span::new(6, 11),
                    ident: "basic",
                }],
            }),
        }];
        assert_eq!(t.subs, subs);
    }

    #[test]
    fn env_compile_unclosed_tag() {
        let env = Engine::new();
        let err = env.compile("test {{ test").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | test {{ test
   |      ^^ unclosed tag
"
        )
    }

    #[test]
    fn env_compile_unexpected_end_tag() {
        let env = Engine::new();
        let err = env.compile("test }} test").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | test }} test
   |      ^^ unexpected end tag
"
        )
    }

    #[test]
    fn template_render_basic() {
        let env = Engine::new();
        let t = env.compile("basic {{ here }}ment").unwrap();
        let s = t.render(data!({ here: "replace" })).unwrap();
        assert_eq!(s, "basic replacement");
    }

    #[test]
    fn template_render_nested() {
        let env = Engine::new();
        let t = env.compile("basic {{ here.nested }}ment").unwrap();
        let s = t.render(data!({ here: { nested: "replace" }})).unwrap();
        assert_eq!(s, "basic replacement");
    }

    #[test]
    fn template_render_nested_filter() {
        let mut env = Engine::new();
        env.add_filter("lower", |v| match v {
            Value::String(s) => Value::String(s.to_lowercase()),
            v => v,
        });
        let t = env.compile("basic {{ here.nested | lower }}ment").unwrap();
        let s = t.render(data!({ here: { nested: "RePlAcE" }})).unwrap();
        assert_eq!(s, "basic replacement");
    }
}

//! A tiny, configurable find-and-replace template engine.
//!
//! # Features
//!
//! - Rendering values, e.g. `{{ path.to.value }}`
//! - Configurable template tags, e.g. `<? value ?>`
//! - Arbitrary filter functions, e.g. `{{ value | filter }}`
//!
//! # Examples
//!
//! Render data constructed using the macro.
//!
//! ```
//! use upon::data;
//!
//! let result = upon::render("Hello {{ value }}", data! { value: "World!" })?;
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Render using structured data.
//!
//! ```
//! #[derive(serde::Serialize)]
//! struct Data {
//!     value: String
//! }
//!
//! let result = upon::render("Hello {{ value }}", Data { value: "World!".into() })?;
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Render a template using custom tags.
//!
//! ```
//! use upon::{data, Engine};
//!
//! let engine = Engine::with_tags("<?", "?>");
//! let result = engine.render("Hello <? value ?>", data! { value: "World!" })?;
//! assert_eq!(result, "Hello World!");
//! # Ok::<(), upon::Error>(())
//! ```
//!
//! Transform data using filters.
//!
//! ```
//! use upon::{data, Engine, Value};
//!
//! let mut engine = Engine::new();
//! engine.add_filter("lower", |mut v| {
//!     if let Value::String(s) = &mut v {
//!         *s = s.to_lowercase();
//!     }
//!     v
//! });
//!
//! let result = engine.render("Hello {{ value | lower }}", data! { value: "WORLD!" })?;
//! assert_eq!(result, "Hello world!");
//!
//! # Ok::<(), upon::Error>(())
//! ```

mod ast;
mod engine;
mod macros;
mod result;
pub mod value;

use crate::ast::{Expr, Span};
pub use crate::engine::Engine;
pub use crate::result::{Error, Result};
pub use crate::value::{to_value, Value};

/// Render the template to a string using the provided data.
pub fn render<S>(source: &str, data: S) -> Result<String>
where
    S: serde::Serialize,
{
    Engine::new().compile(source)?.render(data)
}

/// A compiled template.
#[derive(Debug, Clone)]
pub struct Template<'e> {
    engine: &'e Engine<'e>,
    source: &'e str,
    subs: Vec<Sub<'e>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sub<'e> {
    span: Span,
    expr: Expr<'e>,
}

impl<'e> Template<'e> {
    pub fn source(&self) -> &'e str {
        self.source
    }

    fn with_engine(source: &'e str, engine: &'e Engine<'e>) -> Result<Self> {
        let mut cursor = 0;
        let mut subs = Vec::new();

        loop {
            let (i, m) = match source[cursor..].find(engine.begin_tag) {
                Some(m) => (m, m + engine.begin_tag.len()),
                None => {
                    if let Some(n) = source[cursor..].find(engine.end_tag) {
                        let span = Span::new(n, n + engine.end_tag.len());
                        return Err(Error::span("unexpected end tag", source, span));
                    }
                    return Ok(Template {
                        engine,
                        source,
                        subs,
                    });
                }
            };

            let (j, n) = match source[m..].find(engine.end_tag) {
                Some(n) => (m + n + engine.end_tag.len(), m + n),
                None => {
                    let span = Span::new(i, m);
                    return Err(Error::span("unclosed tag", source, span));
                }
            };

            let outer = Span::new(i, j);
            let inner = Span::new(m, n);
            let expr = ast::parse_expr(source, inner)?;
            subs.push(Sub { span: outer, expr });

            cursor = j;
        }
    }

    /// Render the template to a string using the provided data.
    pub fn render<S>(&self, data: S) -> Result<String>
    where
        S: serde::Serialize,
    {
        let data = to_value(data).unwrap();
        self._render(data)
    }

    fn _render(&self, data: Value) -> Result<String> {
        let mut s = String::new();
        let mut i = 0;
        for Sub { span, expr } in &self.subs {
            s.push_str(&self.source[i..span.m]);
            i = span.n;
            let value = render_expr(self.source, self.engine, &data, expr)?;
            s.push_str(&value.to_string());
        }
        s.push_str(&self.source[i..]);

        Ok(s)
    }
}

fn render_expr(source: &str, engine: &Engine<'_>, data: &Value, expr: &Expr<'_>) -> Result<Value> {
    match expr {
        Expr::Value(ast::Value { path, .. }) => data.lookup(source, path).map(|v| v.clone()),
        Expr::Call(ast::Call { name, receiver, .. }) => match engine.filters.get(name.ident) {
            Some(f) => render_expr(source, engine, data, receiver).map(&**f),
            None => {
                return Err(Error::span(
                    format!("function not found `{}`", name.ident),
                    source,
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
    fn engine_compile_no_tags() {
        let eng = Engine::new();

        let t = eng.compile("").unwrap();
        assert_eq!(t.subs, Vec::new());

        let t = eng.compile("just testing").unwrap();
        assert_eq!(t.subs, Vec::new());
    }

    #[test]
    fn engine_compile_default_tags() {
        let eng = Engine::new();
        let t = eng.compile("test {{ basic }} test").unwrap();
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
    fn engine_compile_custom_tags() {
        let eng = Engine::with_tags("<", "/>");
        let t = eng.compile("test <basic/> test").unwrap();
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
    fn engine_compile_unclosed_tag() {
        let eng = Engine::new();
        let err = eng.compile("test {{ test").unwrap_err();
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
    fn engine_compile_unexpected_end_tag() {
        let eng = Engine::new();
        let err = eng.compile("test }} test").unwrap_err();
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
        let eng = Engine::new();
        let t = eng.compile("basic {{ here }}ment").unwrap();
        let s = t.render(data! { here: "replace" }).unwrap();
        assert_eq!(s, "basic replacement");
    }

    #[test]
    fn template_render_nested() {
        let eng = Engine::new();
        let t = eng.compile("basic {{ here.nested }}ment").unwrap();
        let s = t.render(data! { here: { nested: "replace" }}).unwrap();
        assert_eq!(s, "basic replacement");
    }

    #[test]
    fn template_render_nested_filter() {
        let mut eng = Engine::new();
        eng.add_filter("lower", |v| match v {
            Value::String(s) => Value::String(s.to_lowercase()),
            v => v,
        });
        let t = eng.compile("basic {{ here.nested | lower }}ment").unwrap();
        let s = t.render(data! { here: { nested: "RePlAcE" }}).unwrap();
        assert_eq!(s, "basic replacement");
    }
}

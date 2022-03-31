mod ast;
mod data;
mod env;
mod macros;
mod result;

use crate::ast::{Expr, Span};
pub use crate::data::{List, Map, Value};
pub use crate::env::Env;
pub use crate::result::{Error, Result};

#[derive(Debug, Clone)]
pub struct Template<'env> {
    env: &'env Env<'env>,
    tmpl: &'env str,
    subs: Vec<Sub<'env>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sub<'env> {
    span: Span,
    expr: Expr<'env>,
}

impl<'env> Template<'env> {
    fn with_env(tmpl: &'env str, env: &'env Env<'env>) -> Result<Self> {
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
                    return Ok(Template { env, tmpl, subs });
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn template_new_no_tags() {
        let env = Env::new();

        let t = env.compile("").unwrap();
        assert_eq!(t.subs, Vec::new());

        let t = env.compile("just testing").unwrap();
        assert_eq!(t.subs, Vec::new());
    }

    #[test]
    fn template_default_tags() {
        let env = Env::new();
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
    fn template_custom_tags() {
        let env = Env::with_tags("<", "/>");
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
    fn template_unclosed_tag() {
        let env = Env::new();
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
    fn template_unexpected_end_tag() {
        let env = Env::new();
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
}

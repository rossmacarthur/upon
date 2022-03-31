mod ast;
mod data;
mod macros;
mod result;

use crate::ast::{Expr, Span};
pub use crate::data::{List, Map, Value};
pub use crate::result::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template<'t> {
    tmpl: &'t str,
    subs: Vec<Sub<'t>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sub<'t> {
    span: Span,
    expr: Expr<'t>,
}

pub struct Options<'a> {
    begin_tag: &'a str,
    end_tag: &'a str,
}

/// Compile a new template.
pub fn template(tmpl: &str) -> Result<Template<'_>> {
    Template::new(tmpl)
}

impl<'t> Template<'t> {
    pub fn new(tmpl: &'t str) -> Result<Self> {
        Template::with_options(tmpl, Options::default())
    }

    pub fn with_options(tmpl: &'t str, opts: Options<'_>) -> Result<Self> {
        let mut cursor = 0;
        let mut subs = Vec::new();

        loop {
            let (i, m) = match tmpl[cursor..].find(opts.begin_tag) {
                Some(m) => (m, m + opts.begin_tag.len()),
                None => {
                    if let Some(n) = tmpl[cursor..].find(opts.end_tag) {
                        let span = Span::new(n, n + opts.end_tag.len());
                        return Err(Error::new("unexpected end tag", tmpl, span));
                    }
                    return Ok(Template { tmpl, subs });
                }
            };

            let (j, n) = match tmpl[m..].find(opts.end_tag) {
                Some(n) => (m + n + opts.end_tag.len(), m + n),
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

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            begin_tag: "{{",
            end_tag: "}}",
        }
    }
}

impl<'a> Options<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tags(mut self, begin: &'a str, end: &'a str) -> Self {
        self.begin_tag = begin;
        self.end_tag = end;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn template_new_no_tags() {
        let t = Template::new("").unwrap();
        let exp = Template {
            tmpl: "",
            subs: Vec::new(),
        };
        assert_eq!(t, exp);

        let t = Template::new("just testing").unwrap();
        let exp = Template {
            tmpl: "just testing",
            subs: Vec::new(),
        };
        assert_eq!(t, exp);
    }

    #[test]
    fn template_default_tags() {
        let t = Template::new("test {{ basic }} test").unwrap();
        let exp = Template {
            tmpl: "test {{ basic }} test",
            subs: vec![Sub {
                span: Span::new(5, 16),
                expr: Expr::Value(ast::Value {
                    span: Span::new(8, 13),
                    path: vec![ast::Ident {
                        span: Span::new(8, 13),
                        ident: "basic",
                    }],
                }),
            }],
        };
        assert_eq!(t, exp);
    }

    #[test]
    fn template_custom_tags() {
        let t =
            Template::with_options("test <basic/> test", Options::new().tags("<", "/>")).unwrap();
        let exp = Template {
            tmpl: "test <basic/> test",
            subs: vec![Sub {
                span: Span::new(5, 13),
                expr: Expr::Value(ast::Value {
                    span: Span::new(6, 11),
                    path: vec![ast::Ident {
                        span: Span::new(6, 11),
                        ident: "basic",
                    }],
                }),
            }],
        };
        assert_eq!(t, exp);
    }

    #[test]
    fn template_unclosed_tag() {
        let err = template("test {{ test").unwrap_err();
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
        let err = template("test }} test").unwrap_err();
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

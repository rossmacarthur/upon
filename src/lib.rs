mod ast;
mod data;
mod macros;
mod result;

use crate::ast::{Expr, Span};
pub use crate::data::{List, Map, Value};
pub use crate::result::{Error, Result};

const BEGIN_BLOCK: &str = "{{";
const END_BLOCK: &str = "}}";

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

/// Compile a new template.
pub fn template(tmpl: &str) -> Result<Template<'_>> {
    Template::new(tmpl)
}

impl<'t> Template<'t> {
    fn new(tmpl: &'t str) -> Result<Self> {
        let mut cursor = 0;
        let mut subs = Vec::new();

        loop {
            let (i, m) = match tmpl[cursor..].find(BEGIN_BLOCK) {
                Some(m) => (m, m + BEGIN_BLOCK.len()),
                None => {
                    if let Some(n) = tmpl[cursor..].find(END_BLOCK) {
                        let span = Span::new(n, n + END_BLOCK.len());
                        return Err(Error::new("unexpected end block", tmpl, span));
                    }
                    return Ok(Template { tmpl, subs });
                }
            };

            let (j, n) = match tmpl[m..].find(END_BLOCK) {
                Some(n) => (m + n + BEGIN_BLOCK.len(), m + n),
                None => {
                    let span = Span::new(i, m);
                    return Err(Error::new("unclosed block", tmpl, span));
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
    fn template_no_tags() {
        let t = template("").unwrap();
        let exp = Template {
            tmpl: "",
            subs: Vec::new(),
        };
        assert_eq!(t, exp);

        let t = template("just testing").unwrap();
        let exp = Template {
            tmpl: "just testing",
            subs: Vec::new(),
        };
        assert_eq!(t, exp);
    }

    #[test]
    fn template_unclosed_block() {
        let err = template("test {{ test").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | test {{ test
   |      ^^ unclosed block
"
        )
    }

    #[test]
    fn template_unexpected_end_block() {
        let err = template("test }} test").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | test }} test
   |      ^^ unexpected end block
"
        )
    }
}

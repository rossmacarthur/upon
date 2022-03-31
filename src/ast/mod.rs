//! A simple AST for an expression within a template start and end block.

mod span;

#[cfg(test)]
use serde::Serialize;

pub use crate::ast::span::Span;
pub use crate::result::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Expr<'t> {
    Value(Value<'t>),
    Call(Call<'t>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Value<'t> {
    pub span: Span,
    pub path: Vec<Ident<'t>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Call<'t> {
    pub span: Span,
    pub name: Ident<'t>,
    pub receiver: Box<Expr<'t>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
pub struct Ident<'t> {
    pub span: Span,
    pub ident: &'t str,
}

/// Parses an expression from a location in a template.
pub fn parse_expr(tmpl: &str, span: Span) -> Result<Expr<'_>> {
    let s = &tmpl[span];

    // Tokenizes the input into words.
    let mut it = split(s, span, char::is_whitespace).filter(|(_, s)| !s.is_empty());

    let mut expr = match it.next() {
        None => return Err(Error::new("expected value", tmpl, span)),
        Some((span, token)) => Expr::Value(parse_value(tmpl, span, token)?),
    };

    loop {
        match it.next() {
            Some((pspan, "|")) => match it.next() {
                Some((span, ident)) => {
                    let name = Ident { span, ident };
                    expr = Expr::Call(Call {
                        span: pspan.union(span),
                        name,
                        receiver: Box::new(expr),
                    })
                }
                None => return Err(Error::new("expected expression after pipe", tmpl, pspan)),
            },
            Some((span, _)) => return Err(Error::new("unexpected token", tmpl, span)),
            None => break Ok(expr),
        }
    }
}

/// Parses a value from a token in a template.
///
/// - `tmpl` is the entire template
/// - `span` is the span of the given token.
/// - `s` the token to parse
fn parse_value<'t>(tmpl: &'t str, span: Span, token: &'t str) -> Result<Value<'t>> {
    let path: Vec<_> = split(token, span, |c| c == '.')
        .map(|(span, ident)| {
            if ident.is_empty() {
                return Err(Error::new("invalid identifier", tmpl, span));
            }
            Ok(Ident { span, ident })
        })
        .collect::<Result<_>>()?;
    let span = path[0].span.union(path[path.len() - 1].span);
    Ok(Value { span, path })
}

fn split<F>(s: &str, span: Span, f: F) -> impl Iterator<Item = (Span, &str)> + Clone
where
    F: Fn(char) -> bool + Clone,
{
    s.split(f).map(move |w| {
        let addr = |s: &str| s.as_ptr() as usize;
        let m = span.m + (addr(w) - addr(s));
        let n = m + w.len();
        (Span::new(m, n), w)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[track_caller]
    fn parse_template(tmpl: &str) -> Result<Expr<'_>> {
        let m = tmpl.find("{{").unwrap() + 2;
        let n = tmpl.find("}}").unwrap();
        parse_expr(tmpl, Span::new(m, n))
    }

    #[test]
    fn parse_expr_value() {
        let expr = parse_template("{{ basic }}").unwrap();
        goldie::assert_json!(&expr);
    }

    #[test]
    fn parse_expr_value_dotted() {
        let expr = parse_template("{{ path.to.value }}").unwrap();
        goldie::assert_json!(&expr);
    }

    #[test]
    fn parse_expr_call() {
        let expr = parse_template("{{ basic | fn }}").unwrap();
        goldie::assert_json!(&expr);
    }

    #[test]
    fn parse_expr_value_dotted_call() {
        let expr = parse_template("{{ basic.path.segment | fn }}").unwrap();
        goldie::assert_json!(&expr);
    }

    #[test]
    fn parse_expr_double_call() {
        let expr = parse_template("{{ basic | fn | fn }}").unwrap();
        goldie::assert_json!(&expr);
    }

    #[test]
    fn parse_expr_expected_value() {
        let err = parse_template("{{  }}").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{  }}
   |   ^^ expected value
"
        );
    }

    #[test]
    fn parse_expr_expected_identifier() {
        let err = parse_template("{{ basic | }}").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ basic | }}
   |          ^ expected expression after pipe
"
        );
    }

    #[test]
    fn parse_value_invalid_identifier() {
        let err = parse_template("{{ . }}").unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ . }}
   |    ^ invalid identifier
"
        );
    }
}

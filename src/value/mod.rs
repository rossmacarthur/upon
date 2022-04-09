mod from;
mod ser;

pub use std::collections::HashMap as Map;
use std::fmt;
use std::mem;
pub use std::vec::Vec as List;

use crate::ast::Ident;
use crate::result::{Error, Result};
pub use crate::value::ser::to_value;

/// Data to be rendered represented as a recursive enum.
#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(List<Value>),
    Map(Map<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(s), Self::Bool(o)) => s == o,
            (Self::Integer(s), Self::Integer(o)) => s == o,
            (Self::Float(s), Self::Float(o)) => s == o,
            (Self::String(s), Self::String(o)) => s == o,
            (Self::List(s), Self::List(o)) => s == o,
            (Self::Map(s), Self::Map(o)) => s == o,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => fmt::Display::fmt("", f),
            Value::Bool(b) => fmt::Display::fmt(b, f),
            Value::Integer(n) => fmt::Display::fmt(n, f),
            Value::Float(n) => fmt::Display::fmt(n, f),
            Value::String(s) => fmt::Display::fmt(s, f),
            Value::List(list) => {
                f.write_str("[")?;
                for (i, entry) in list.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}", entry)?;
                }
                f.write_str("]")?;
                Ok(())
            }
            Value::Map(map) => {
                f.write_str("{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                f.write_str("}")?;
                Ok(())
            }
        }
    }
}

impl Value {
    pub(crate) fn lookup<'a>(&'a self, source: &str, path: &[Ident]) -> Result<&'a Value> {
        let mut data = self;

        for Ident { span, ident: p } in path {
            data = match data {
                Value::None => {
                    return Err(Error::span(
                        format!("cannot index none with `{}`", p),
                        source,
                        *span,
                    ))
                }

                Value::Bool(_) => {
                    return Err(Error::span(
                        format!("cannot index bool with `{}`", p),
                        source,
                        *span,
                    ))
                }

                Value::Integer(_) => {
                    return Err(Error::span(
                        format!("cannot index integer with `{}`", p),
                        source,
                        *span,
                    ))
                }

                Value::Float(_) => {
                    return Err(Error::span(
                        format!("cannot index float with `{}`", p),
                        source,
                        *span,
                    ))
                }

                Value::String(_) => {
                    return Err(Error::span(
                        format!("cannot index string with `{}`", p),
                        source,
                        *span,
                    ))
                }

                Value::List(list) => match p.parse::<usize>() {
                    Ok(i) => &list[i],
                    Err(_) => {
                        return Err(Error::span(
                            format!("cannot index list with `{}`", p),
                            source,
                            *span,
                        ))
                    }
                },

                Value::Map(map) => match map.get(*p) {
                    Some(value) => value,
                    None => {
                        return Err(Error::span(
                            format!("key `{}` not found in map", p),
                            source,
                            *span,
                        ))
                    }
                },
            }
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{data, Span};

    use pretty_assertions::assert_eq;

    fn id(ident: &str) -> Ident<'_> {
        Ident {
            span: Span::new(0, 0),
            ident,
        }
    }

    #[test]
    fn lookup_single() {
        let data = data! { hello: "world" };
        let exp = data.lookup("", &[id("hello")]).unwrap();
        assert_eq!(&Value::from("world"), exp);
    }

    #[test]
    fn lookup_nested() {
        let data = data! { hello: { world: "testing..." } };
        let exp = data.lookup("", &[id("hello"), id("world")]).unwrap();
        assert_eq!(&Value::from("testing..."), exp);
    }

    #[test]
    fn lookup_list_index() {
        let data = data! { hello: ["world"] };
        let exp = data.lookup("", &[id("hello"), id("0")]).unwrap();
        assert_eq!(&Value::from("world"), exp);
    }

    #[test]
    fn lookup_cannot_index_none() {
        let data = Value::None;
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    ident: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index none with `hello`
"
        );
    }

    #[test]
    fn lookup_cannot_index_string() {
        let data = Value::from("testing...");
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    ident: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index string with `hello`
"
        );
    }

    #[test]
    fn lookup_cannot_index_list() {
        let data = Value::from(["te", "ting..."]);
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    ident: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index list with `hello`
"
        );
    }

    #[test]
    fn lookup_key_not_found() {
        let data = data! { te: "ting..." };
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    ident: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ key `hello` not found in map
"
        );
    }
}

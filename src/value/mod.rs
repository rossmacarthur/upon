mod from;

pub use std::collections::HashMap as Map;
use std::fmt;
pub use std::vec::Vec as List;

use crate::ast::Ident;
use crate::result::{Error, Result};

/// Data to be rendered represented as a recursive enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    None,
    String(String),
    List(List<Value>),
    Map(Map<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => fmt::Display::fmt("", f),
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
    pub(crate) fn lookup<'a>(&'a self, tmpl: &str, path: &[Ident]) -> Result<&'a Value> {
        let mut data = self;

        for Ident { span, ident: p } in path {
            data = match data {
                Value::None => {
                    return Err(Error::new(
                        format!("cannot index None with `{}`", p),
                        tmpl,
                        *span,
                    ))
                }

                Value::String(_) => {
                    return Err(Error::new(
                        format!("cannot index String with `{}`", p),
                        tmpl,
                        *span,
                    ))
                }

                Value::List(list) => match p.parse::<usize>() {
                    Ok(i) => &list[i],
                    Err(_) => {
                        return Err(Error::new(
                            format!("cannot index List with `{}`", p),
                            tmpl,
                            *span,
                        ))
                    }
                },

                Value::Map(map) => match map.get(*p) {
                    Some(value) => value,
                    None => {
                        return Err(Error::new(
                            format!("key `{}` not found in Map", p),
                            tmpl,
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
        let data = data!({ hello: "world" });
        let exp = data.lookup("", &[id("hello")]).unwrap();
        assert_eq!(&Value::from("world"), exp);
    }

    #[test]
    fn lookup_nested() {
        let data = data!({ hello: { world: "testing..." } });
        let exp = data.lookup("", &[id("hello"), id("world")]).unwrap();
        assert_eq!(&Value::from("testing..."), exp);
    }

    #[test]
    fn lookup_list_index() {
        let data = data!({ hello: ["world"] });
        let exp = data.lookup("", &[id("hello"), id("0")]).unwrap();
        assert_eq!(&Value::from("world"), exp);
    }

    #[test]
    fn lookup_cannot_index_none() {
        let data = data!(None);
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
   |    ^^^^^ cannot index None with `hello`
"
        );
    }

    #[test]
    fn lookup_cannot_index_string() {
        let data = data!("testing...");
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
   |    ^^^^^ cannot index String with `hello`
"
        );
    }

    #[test]
    fn lookup_cannot_index_list() {
        let data = data!(["te", "ting..."]);
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
   |    ^^^^^ cannot index List with `hello`
"
        );
    }

    #[test]
    fn lookup_key_not_found() {
        let data = data!({ te: "ting..." });
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
   |    ^^^^^ key `hello` not found in Map
"
        );
    }
}

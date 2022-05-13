mod from;
mod ser;

pub use std::collections::hash_map;
pub use std::collections::HashMap as Map;
use std::fmt;
use std::mem;
use std::vec;
pub use std::vec::Vec as List;

use crate::ast::Ident;
use crate::error::{Error, Result};
pub use crate::value::ser::to_value;

pub type MapIntoIter = hash_map::IntoIter<String, Value>;
pub type ListIntoIter = vec::IntoIter<Value>;

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
    pub(crate) fn human(&self) -> &'static str {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
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
            value: ident,
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
                    value: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index into none
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
                    value: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index into string
"
        );
    }

    #[test]
    fn lookup_cannot_index_list() {
        let data = Value::from(["tes", "ting..."]);
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    value: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ cannot index list with string
"
        );
    }

    #[test]
    fn lookup_key_not_found() {
        let data = data! { tes: "ting..." };
        let err = data
            .lookup(
                "{{ hello }}",
                &[Ident {
                    value: "hello",
                    span: Span::new(3, 8),
                }],
            )
            .unwrap_err();
        assert_eq!(
            format!("{:#}", err),
            "
   |
 1 | {{ hello }}
   |    ^^^^^ not found in map
"
        );
    }
}

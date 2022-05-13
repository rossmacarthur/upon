mod from;
mod ser;

pub use std::collections::hash_map;
pub use std::collections::HashMap as Map;
use std::fmt;
use std::mem;
use std::vec;
pub use std::vec::Vec as List;

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

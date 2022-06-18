//! Defines the [`Value`] enum, representing any valid renderable data.

mod from;
mod ser;

use std::mem;

pub use crate::value::ser::to_value;
#[doc(hidden)]
pub use std::collections::BTreeMap as Map;
#[doc(hidden)]
pub use std::vec::Vec as List;

pub(crate) use std::collections::btree_map as map;
pub(crate) use std::vec as list;

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

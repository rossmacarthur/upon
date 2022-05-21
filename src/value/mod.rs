//! Defines the [`Value`] enum, representing any valid renderable data.

mod from;
mod ser;

pub use std::collections::btree_map;
pub use std::collections::BTreeMap as Map;
use std::mem;
use std::vec;
pub use std::vec::Vec as List;

pub use crate::value::ser::to_value;

pub(crate) type MapIntoIter = btree_map::IntoIter<String, Value>;
pub(crate) type ListIntoIter = vec::IntoIter<Value>;

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

//! Defines the [`Value`] enum, representing any valid renderable data.

mod cow;
mod from;
mod ser;

use std::collections::BTreeMap;
use std::mem;

pub(crate) use crate::value::cow::ValueCow;
pub use crate::value::ser::to_value;

/// Data to be rendered represented as a recursive enum.
#[derive(Debug, Clone)]
pub enum Value {
    None,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl Default for Value {
    fn default() -> Self {
        Self::None
    }
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

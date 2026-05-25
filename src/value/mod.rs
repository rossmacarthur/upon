//! Defines the [`Value`] enum, representing any valid renderable data.

mod cow;
mod from;
#[cfg(feature = "serde")]
mod ser;

use std::collections::BTreeMap;

pub(crate) use crate::value::cow::ValueCow;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use crate::value::ser::to_value;

/// Data to be rendered represented as a recursive enum.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value {
    #[default]
    None,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Value {
        self
    }
}

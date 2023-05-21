//! Defines the [`Value`] enum, representing any valid renderable data.

mod cow;
mod from;
#[cfg(feature = "serde")]
mod ser;

use std::borrow::Cow;
use std::collections::BTreeMap;

pub(crate) use crate::value::cow::ValueCow;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use crate::value::ser::to_value;

/// Data to be rendered represented as a recursive enum.
#[derive(Debug, Clone, PartialEq)]
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

pub trait IntoValue<'a> {
    fn into_value(self) -> Cow<'a, Value>;
}

impl<'a, T: Into<Value> + 'a> IntoValue<'a> for T {
    fn into_value(self) -> Cow<'a, Value> {
        Cow::Owned(self.into())
    }
}

impl<'a> IntoValue<'a> for &'a Value {
    fn into_value(self) -> Cow<'a, Value> {
        Cow::Borrowed(self)
    }
}

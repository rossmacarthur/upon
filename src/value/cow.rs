//! Defines a clone-on-write [`Value`].

use std::ops::Deref;

use crate::Value;

#[cfg_attr(internal_debug, derive(Debug))]
pub enum ValueCow<'a> {
    Borrowed(&'a Value),
    Owned(Value),
}

impl Deref for ValueCow<'_> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(v) => v,
        }
    }
}

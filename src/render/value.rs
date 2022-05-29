//! Defines a clone-on-write [`Value`].

use std::ops::Deref;

use crate::types::ast;
use crate::types::span::Span;
use crate::{Error, Result, Value};

pub enum ValueCow<'a> {
    Borrowed(&'a Value),
    Owned(Value),
}

impl Deref for ValueCow<'_> {
    type Target = Value;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(v) => &*v,
        }
    }
}

impl ValueCow<'_> {
    pub fn as_bool(&self, source: &str, span: Span) -> Result<bool> {
        match &**self {
            Value::Bool(cond) => Ok(*cond),
            value => Err(Error::new(
                format!(
                    "expected bool, but expression evaluated to {}",
                    value.human()
                ),
                source,
                span,
            )),
        }
    }

    pub fn apply<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Value),
    {
        match self {
            ValueCow::Borrowed(b) => {
                let mut o = b.clone();
                f(&mut o);
                *self = ValueCow::Owned(o);
            }
            ValueCow::Owned(ref mut o) => f(o),
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

/// Index the value with the given path segment.
pub fn index<'render>(
    source: &str,
    value: &'render Value,
    p: &ast::Ident<'_>,
) -> Result<&'render Value> {
    let ast::Ident { raw, span } = p;
    match value {
        Value::List(list) => match raw.parse::<usize>() {
            Ok(i) => Ok(&list[i]),
            Err(_) => Err(Error::new("cannot index list with string", source, *span)),
        },
        Value::Map(map) => match map.get(*raw) {
            Some(value) => Ok(value),
            None => Err(Error::new("not found in map", source, *span)),
        },
        value => Err(Error::new(
            format!("cannot index into {}", value.human()),
            source,
            *span,
        )),
    }
}

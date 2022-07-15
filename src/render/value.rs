use crate::types::ast;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

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

/// Index the value with the given path.
pub fn lookup_path<'a>(
    source: &str,
    value: &ValueCow<'a>,
    path: &[ast::Ident<'_>],
) -> Result<ValueCow<'a>> {
    match value {
        &ValueCow::Borrowed(mut v) => {
            for p in path {
                v = lookup(source, v, p)?;
            }
            Ok(ValueCow::Borrowed(v))
        }
        ValueCow::Owned(v) => {
            let mut v = v;
            for p in path {
                v = lookup(source, v, p)?;
            }
            Ok(ValueCow::Owned(v.clone()))
        }
    }
}

/// Index the value with the given path.
pub fn lookup_path_maybe<'a>(
    source: &str,
    value: &ValueCow<'a>,
    path: &[ast::Ident<'_>],
) -> Result<Option<ValueCow<'a>>> {
    let value = match value {
        // If the scope is borrowed we can lookup the value and return a
        // reference with lifetime 'render
        &ValueCow::Borrowed(mut v) => {
            for (i, p) in path.iter().enumerate() {
                v = match lookup(source, v, p) {
                    Ok(v) => v,
                    Err(err) => {
                        if i == 0 {
                            return Ok(None);
                        }
                        return Err(err);
                    }
                }
            }
            ValueCow::Borrowed(v)
        }
        // If the scope is owned then make sure to only clone the edge value
        // that we lookup.
        ValueCow::Owned(v) => {
            let mut v: &Value = v;
            for (i, p) in path.iter().enumerate() {
                v = match lookup(source, v, p) {
                    Ok(v) => v,
                    Err(err) => {
                        if i == 0 {
                            return Ok(None);
                        }
                        return Err(err);
                    }
                }
            }
            ValueCow::Owned(v.clone())
        }
    };
    Ok(Some(value))
}

/// Index into the value with the given path segment.
pub fn lookup<'a>(source: &str, value: &'a Value, p: &ast::Ident<'_>) -> Result<&'a Value> {
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

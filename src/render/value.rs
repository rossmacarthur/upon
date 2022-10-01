use crate::types::ast;
use crate::types::span::{index, Span};
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
    path: &[ast::Ident],
) -> Result<ValueCow<'a>> {
    match value {
        &ValueCow::Borrowed(v) => {
            let v = path.iter().try_fold(v, |v, p| lookup(source, v, p))?;
            Ok(ValueCow::Borrowed(v))
        }
        ValueCow::Owned(v) => {
            let v = path.iter().try_fold(v, |v, p| lookup(source, v, p))?;
            Ok(ValueCow::Owned(v.clone()))
        }
    }
}

/// Index the value with the given path.
pub fn lookup_path_maybe<'a>(
    source: &str,
    value: &ValueCow<'a>,
    path: &[ast::Ident],
) -> Result<Option<ValueCow<'a>>> {
    let value = match value {
        // If the value is borrowed we can lookup the value and return a
        // reference with lifetime a
        &ValueCow::Borrowed(v) => {
            let v = match lookup(source, v, &path[0]) {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let v = path[1..].iter().try_fold(v, |v, p| lookup(source, v, p))?;
            ValueCow::Borrowed(v)
        }
        // If the value is owned then make sure to only clone the edge value
        // that we lookup.
        ValueCow::Owned(v) => {
            let v = match lookup(source, v, &path[0]) {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let v = path[1..].iter().try_fold(v, |v, p| lookup(source, v, p))?;
            ValueCow::Owned(v.clone())
        }
    };
    Ok(Some(value))
}

/// Index into the value with the given path segment.
pub fn lookup<'a>(source: &str, value: &'a Value, p: &ast::Ident) -> Result<&'a Value> {
    let ast::Ident { span } = p;
    let raw = unsafe { index(source, *span) };
    match value {
        Value::List(list) => match raw.parse::<usize>() {
            Ok(i) => Ok(&list[i]),
            Err(_) => Err(Error::new("cannot index list with string", source, *span)),
        },
        Value::Map(map) => match map.get(raw) {
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

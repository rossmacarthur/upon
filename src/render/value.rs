use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

impl ValueCow<'_> {
    pub fn as_bool(&self) -> bool {
        match &**self {
            Value::None | Value::Bool(false) | Value::Integer(0) => false,
            Value::Float(n) if *n == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(l) if l.is_empty() => false,
            Value::Map(m) if m.is_empty() => false,
            _ => true,
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
    path: &[ast::Key],
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
    var: &ast::Var,
) -> Result<Option<ValueCow<'a>>> {
    let value = match value {
        // If the value is borrowed we can lookup the value and return a
        // reference with lifetime a
        &ValueCow::Borrowed(v) => {
            let v = match lookup(source, v, var.first()) {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let v = var.rest().iter().try_fold(v, |v, p| lookup(source, v, p))?;
            ValueCow::Borrowed(v)
        }
        // If the value is owned then make sure to only clone the edge value
        // that we lookup.
        ValueCow::Owned(v) => {
            let v = match lookup(source, v, var.first()) {
                Ok(v) => v,
                Err(_) => return Ok(None),
            };
            let v = var.rest().iter().try_fold(v, |v, p| lookup(source, v, p))?;
            ValueCow::Owned(v.clone())
        }
    };
    Ok(Some(value))
}

/// Index into the value with the given path segment.
pub fn lookup<'a>(source: &str, value: &'a Value, key: &ast::Key) -> Result<&'a Value> {
    match value {
        Value::List(list) => {
            let i = match key {
                ast::Key::List(ast::Index { value, .. }) => value,
                _ => {
                    return Err(Error::render(
                        "cannot index list with string",
                        source,
                        key.span(),
                    ));
                }
            };
            list.get(*i).ok_or_else(|| {
                let len = list.len();
                Error::render(
                    format!("index out of bounds, the length is {len}"),
                    source,
                    key.span(),
                )
            })
        }
        Value::Map(map) => {
            let raw = match key {
                ast::Key::Map(ast::Ident { span }) => &source[*span],
                _ => {
                    return Err(Error::render(
                        "cannot index map with integer",
                        source,
                        key.span(),
                    ));
                }
            };
            match map.get(raw) {
                Some(value) => Ok(value),
                None => Err(Error::render("not found in map", source, key.span())),
            }
        }
        value => Err(Error::render(
            format!("cannot index into {}", value.human()),
            source,
            key.span(),
        )),
    }
}

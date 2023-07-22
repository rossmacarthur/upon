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

/// Lookup the given path.
pub fn lookup_path<'a>(
    source: &str,
    value: &ValueCow<'a>,
    path: &[ast::Member],
) -> Result<ValueCow<'a>> {
    match value {
        ValueCow::Borrowed(mut value) => {
            for p in path {
                match lookup(source, value, p)? {
                    Some(v) => value = v,
                    None => return Ok(ValueCow::Borrowed(&Value::None)),
                }
            }
            Ok(ValueCow::Borrowed(value))
        }
        ValueCow::Owned(value) => {
            let mut value: &Value = value;
            for p in path {
                match lookup(source, value, p)? {
                    Some(v) => value = v,
                    None => return Ok(ValueCow::Borrowed(&Value::None)),
                }
            }
            Ok(ValueCow::Owned(value.clone()))
        }
    }
}

/// Lookup the given path, return None if the first segment is not found.
pub fn lookup_path_maybe<'a>(
    source: &str,
    value: &ValueCow<'a>,
    path: &[ast::Member],
) -> Result<Option<ValueCow<'a>>> {
    match value {
        ValueCow::Borrowed(mut value) => {
            for (i, p) in path.iter().enumerate() {
                match lookup(source, value, p) {
                    Ok(Some(v)) => value = v,
                    Ok(None) | Err(_) if i == 0 => return Ok(None),
                    Ok(None) => return Ok(Some(ValueCow::Borrowed(&Value::None))),
                    Err(err) => return Err(err),
                };
            }
            Ok(Some(ValueCow::Borrowed(value)))
        }
        ValueCow::Owned(value) => {
            let mut value: &Value = value;
            for (i, p) in path.iter().enumerate() {
                match lookup(source, value, p) {
                    Ok(Some(v)) => value = v,
                    Ok(None) | Err(_) if i == 0 => return Ok(None),
                    Ok(None) => return Ok(Some(ValueCow::Borrowed(&Value::None))),
                    Err(err) => return Err(err),
                };
            }
            Ok(Some(ValueCow::Owned(value.clone())))
        }
    }
}

/// Access the given member from the value.
pub fn lookup<'a>(
    source: &str,
    value: &'a Value,
    member: &ast::Member,
) -> Result<Option<&'a Value>> {
    match (value, &member.access) {
        (Value::List(list), ast::Access::Index(index)) => {
            let ast::Index { value: i, .. } = index;
            match (&member.op, list.get(*i)) {
                (_, Some(value)) => Ok(Some(value)),
                (ast::AccessOp::Optional, _) => Ok(None),
                (ast::AccessOp::Direct, _) => {
                    let len = list.len();
                    Err(Error::render(
                        format!("index out of bounds, the length is {len}"),
                        source,
                        member.span,
                    ))
                }
            }
        }
        (Value::Map(map), ast::Access::Key(ident)) => {
            let ast::Ident { span } = ident;
            match (&member.op, map.get(&source[*span])) {
                (_, Some(value)) => Ok(Some(value)),
                (ast::AccessOp::Optional, _) => Ok(None),
                (ast::AccessOp::Direct, _) => {
                    Err(Error::render("not found in map", source, member.span))
                }
            }
        }
        (value, ast::Access::Index(_)) => Err(Error::render(
            format!("{} does not support integer-based access", value.human()),
            source,
            member.span,
        )),
        (value, ast::Access::Key(_)) => Err(Error::render(
            format!("{} does not support key-based access", value.human()),
            source,
            member.span,
        )),
    }
}

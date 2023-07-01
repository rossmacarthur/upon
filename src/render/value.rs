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
    path: &[ast::Member],
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

/// Access the given member from the value.
pub fn lookup<'a>(source: &str, value: &'a Value, member: &ast::Member) -> Result<&'a Value> {
    match (value, &member.access) {
        (Value::List(list), ast::Access::Index(index)) => {
            let ast::Index { value: i, .. } = index;
            match (&member.op, list.get(*i)) {
                (_, Some(value)) => Ok(value),
                (ast::AccessOp::Optional, _) => Ok(&Value::None),
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
                (_, Some(value)) => Ok(value),
                (ast::AccessOp::Optional, _) => Ok(&Value::None),
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

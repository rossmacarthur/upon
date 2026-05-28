use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::types::ast;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

impl ValueCow<'_> {
    pub fn as_bool(&self) -> bool {
        match &**self {
            Value::None => false,
            Value::Bool(false) => false,
            Value::Integer(0) => false,
            Value::Float(n) if *n == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(l) if l.is_empty() => false,
            Value::Map(m) if m.is_empty() => false,
            _ => true,
        }
    }

    pub fn cmp_op(&self, op: ast::Op, other: &Self) -> bool {
        match op {
            ast::Op::Eq => **self == **other,
            ast::Op::Ne => **self != **other,
            ast::Op::Lt => matches!(partial_cmp_value(self, other), Some(Ordering::Less)),
            ast::Op::Le => matches!(
                partial_cmp_value(self, other),
                Some(Ordering::Less | Ordering::Equal)
            ),
            ast::Op::Gt => matches!(partial_cmp_value(self, other), Some(Ordering::Greater)),
            ast::Op::Ge => matches!(
                partial_cmp_value(self, other),
                Some(Ordering::Greater | Ordering::Equal)
            ),
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

    pub(crate) fn new_map() -> Self {
        Self::Map(BTreeMap::new())
    }

    pub(crate) fn new_list() -> Self {
        Self::List(Vec::new())
    }
}

fn partial_cmp_value(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Integer(left), Value::Integer(right)) => Some(left.cmp(right)),
        (Value::Integer(left), Value::Float(right)) => cmp_i64_f64(*left, *right),
        (Value::Float(left), Value::Integer(right)) => {
            cmp_i64_f64(*right, *left).map(reverse_ordering)
        }
        (Value::Float(left), Value::Float(right)) => left.partial_cmp(right),
        (Value::String(left), Value::String(right)) => Some(left.cmp(right)),
        _ => None,
    }
}

fn cmp_i64_f64(left: i64, right: f64) -> Option<Ordering> {
    if right.is_nan() {
        return None;
    }
    if right == f64::INFINITY {
        return Some(Ordering::Less);
    }
    if right == f64::NEG_INFINITY {
        return Some(Ordering::Greater);
    }
    if right < i64::MIN as f64 {
        return Some(Ordering::Greater);
    }
    if right >= i64::MAX as f64 {
        return Some(Ordering::Less);
    }

    let truncated = right.trunc() as i64;
    match left.cmp(&truncated) {
        Ordering::Equal => {
            let truncated = truncated as f64;
            if right > truncated {
                Some(Ordering::Less)
            } else if right < truncated {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
        ordering => Some(ordering),
    }
}

fn reverse_ordering(ordering: Ordering) -> Ordering {
    match ordering {
        Ordering::Less => Ordering::Greater,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Less,
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

/// Lookup the given path, return None if the first member is not found.
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
                    Err(_) if i == 0 => return Ok(None),
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
                    Err(_) if i == 0 => return Ok(None),
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

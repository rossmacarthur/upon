//! Builtin filters.

use std::collections::BTreeMap;

use crate::Value;

/// Returns the lowercase equivalent of this string slice.
///
/// See [`str::to_lowercase`].
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// Returns the uppercase equivalent of this string slice.
///
/// See [`str::to_uppercase`].
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

/// Replaces all matches of a substring with another substring in a string.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn replace(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Returns the first element in a list.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn first(list: &[Value]) -> Option<Value> {
    list.first().cloned()
}

/// Returns the last element in a list.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn last(list: &[Value]) -> Option<Value> {
    list.last().cloned()
}

/// Returns the map keys as a list.
///
/// See [`BTreeMap::keys()`].
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn keys(map: &BTreeMap<String, Value>) -> Vec<String> {
    map.keys().cloned().collect()
}

/// Returns the map values as a list.
///
/// See [`BTreeMap::values()`].
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn values(map: &BTreeMap<String, Value>) -> Vec<Value> {
    map.values().cloned().collect()
}

/// Returns the number of elements in the list or map.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn len(value: &Value) -> Result<i64, String> {
    match value {
        Value::List(l) => Ok(l.len() as i64),
        Value::Map(m) => Ok(m.len() as i64),
        value => Err(format!("unsupported value `{}`", value.human())),
    }
}

/// Reverses a list or string.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn reverse(value: Value) -> Result<Value, String> {
    match value {
        Value::String(string) => Ok(Value::String(string.chars().rev().collect())),
        Value::List(list) => Ok(Value::List(list.into_iter().rev().collect())),
        _ => Err(format!("unsupported value `{}`", value.human())),
    }
}

/// If the value is `None` returns the given default instead, otherwise returns
/// the value.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn default(value: Value, default: Value) -> Value {
    match value {
        Value::None => default,
        value => value,
    }
}

/// Looks up an element in a list or value in a map.
///
/// This filter also maps `Value::None` to `Value::None` so it can be chained.
/// For example:
///
/// ```
/// {{ team | get: "coach" | get: "name" }}
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn get(value: &Value, key: Value) -> Result<Value, String> {
    match value {
        Value::None => Ok(Value::None),
        Value::List(list) => match key {
            Value::Integer(i) => Ok(list.get(i as usize).cloned().unwrap_or(Value::None)),
            k => Err(format!("cannot index into list with {}", k.human())),
        },
        Value::Map(map) => match key {
            Value::String(s) => Ok(map.get(&s).cloned().unwrap_or(Value::None)),
            k => Err(format!("cannot index into map with {}", k.human())),
        },
        value => Err(format!("cannot index into {}", value.human())),
    }
}

//! Builtin filters.

use std::collections::BTreeMap;

use crate::Value;

pub(crate) fn add_functions(engine: &mut crate::Engine) {
    // conversions
    engine.add_function("bool", bool);
    engine.add_function("integer", integer);
    engine.add_function("float", float);
    engine.add_function("string", string);
    engine.add_function("list", list);

    // strings
    engine.add_function("lower", lower);
    engine.add_function("upper", upper);
    engine.add_function("replace", replace);

    // lists
    engine.add_function("first", first);
    engine.add_function("last", last);

    // maps
    engine.add_function("keys", keys);
    engine.add_function("values", values);

    // general
    engine.add_function("range", range);
    engine.add_function("len", len);
    engine.add_function("rev", rev);
    engine.add_function("default", default);
}

// /////////////////////////////////////////////////////////////////////////////
// Conversions
// /////////////////////////////////////////////////////////////////////////////

/// Converts a value to a boolean.
///
/// Returns `true` if the value is "truthy", otherwise `false`. A value is
/// considered "truthy" if it is not `None`, `false`, `0`, `0.0`, an empty
/// string, an empty list, or an empty map.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn bool(v: &Value) -> bool {
    match v {
        &Value::Bool(true) => true,
        &Value::Integer(i) if i != 0 => true,
        &Value::Float(n) if n != 0.0 => true,
        Value::String(s) if !s.is_empty() => true,
        Value::List(l) if !l.is_empty() => true,
        Value::Map(m) if !m.is_empty() => true,
        _ => false,
    }
}

/// Converts a value to an integer.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn integer(v: &Value) -> Result<i64, String> {
    match v {
        &Value::Bool(false) => Ok(0),
        &Value::Bool(true) => Ok(1),
        &Value::Integer(i) => Ok(i),
        &Value::Float(f) => Ok(f as i64),
        Value::String(s) => i64::from_str_radix(s, 10).map_err(|e| e.to_string()),
        _ => Err(format!(
            "{} value cannot be converted to an integer",
            v.human()
        )),
    }
}

/// Converts a value to a float.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn float(v: &Value) -> Result<f64, String> {
    match v {
        &Value::Integer(i) => Ok(i as f64),
        &Value::Float(f) => Ok(f),
        Value::String(s) => s.parse::<f64>().map_err(|e| e.to_string()),
        _ => Err(format!(
            "{} value cannot be converted to a float",
            v.human()
        )),
    }
}

/// Converts a value to a string.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn string(v: &Value) -> Result<String, String> {
    let mut s = String::new();
    let mut f = crate::fmt::Formatter::with_string(&mut s);
    crate::fmt::default(&mut f, v).map_err(|e| e.to_string())?;
    Ok(s)
}

/// Converts a value to a list.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn list(v: &Value) -> Result<Vec<Value>, String> {
    match v {
        Value::String(s) => Ok(s.chars().map(Value::from).collect()),
        Value::List(l) => Ok(l.clone()),
        Value::Map(m) => Ok(m
            .iter()
            .map(|(k, v)| Value::from([Value::String(k.clone()), v.clone()]))
            .collect()),
        _ => Err(format!("{} value cannot be converted to a list", v.human())),
    }
}

// /////////////////////////////////////////////////////////////////////////////
// Strings
// /////////////////////////////////////////////////////////////////////////////

/// Returns the lowercase equivalent of this string slice.
///
/// See [`str::to_lowercase`].
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
#[inline]
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

// /////////////////////////////////////////////////////////////////////////////
// Lists
// /////////////////////////////////////////////////////////////////////////////

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

// /////////////////////////////////////////////////////////////////////////////
// Maps
// /////////////////////////////////////////////////////////////////////////////

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

// /////////////////////////////////////////////////////////////////////////////
// General
// /////////////////////////////////////////////////////////////////////////////

/// Generates a range of integers.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn range(lo: i64, hi: i64) -> Vec<i64> {
    Vec::from_iter(lo..hi)
}

/// Returns the length of the string, list or map.
///
/// - For a string, it returns the number of `char`s.
/// - For a list, it returns the number of elements.
/// - For a map, it returns the number of key-value pairs.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn len(value: &Value) -> Result<i64, String> {
    match value {
        Value::String(s) => Ok(s.chars().count() as i64),
        Value::List(l) => Ok(l.len() as i64),
        Value::Map(m) => Ok(m.len() as i64),
        value => Err(format!("{} value has no length", value.human())),
    }
}

/// Reverses a string or list.
///
/// - For a string, it reverses the characters.
/// - For a list, it reverses the elements.
#[cfg_attr(docsrs, doc(cfg(feature = "builtins")))]
pub fn rev(value: Value) -> Result<Value, String> {
    match value {
        Value::String(string) => Ok(Value::String(string.chars().rev().collect())),
        Value::List(list) => Ok(Value::List(list.into_iter().rev().collect())),
        _ => Err(format!("{} value cannot be reversed", value.human())),
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

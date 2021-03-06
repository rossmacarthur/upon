use std::collections::BTreeMap;

use crate::filters::FilterArg;
use crate::value::ValueCow;
use crate::Value;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    /// When there is a type mismatch.
    Type(
        /// Expected
        &'static str,
        /// Got
        &'static str,
    ),
    /// When the value is owned but the filter expects an owned type.
    Reference(
        /// Expected
        &'static str,
    ),
}

impl<'a> FilterArg<'a> for bool {
    type Output = bool;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::Bool(b) => Ok(*b),
            v => Err(Error::Type("bool", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        Self::from_value_ref(&*v)
    }
}

impl<'a> FilterArg<'a> for i64 {
    type Output = i64;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::Integer(i) => Ok(*i),
            v => Err(Error::Type("integer", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        Self::from_value_ref(&*v)
    }
}

impl<'a> FilterArg<'a> for f64 {
    type Output = f64;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::Float(f) => Ok(*f),
            v => Err(Error::Type("float", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        Self::from_value_ref(&*v)
    }
}

impl<'a> FilterArg<'a> for String {
    type Output = String;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::String(s) => Ok(s.to_owned()),
            v => Err(Error::Type("string", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        match v.take() {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("string", v.human())),
        }
    }
}

pub struct Str;

impl<'a> FilterArg<'a> for Str {
    type Output = &'a str;

    fn from_value(v: Value) -> Result<Self::Output> {
        match v {
            Value::String(_) => Err(Error::Reference("string")),
            v => Err(Error::Type("string", v.human())),
        }
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("string", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        let v: &'a Value = &*v;
        match v {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("string", v.human())),
        }
    }
}

impl<'a> FilterArg<'a> for Vec<Value> {
    type Output = Vec<Value>;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::List(l) => Ok(l.clone()),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        match v.take() {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }
}

pub struct ListRef;

impl<'a> FilterArg<'a> for ListRef {
    type Output = &'a [Value];

    fn from_value(v: Value) -> Result<Self::Output> {
        match v {
            Value::List(_) => Err(Error::Reference("list")),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        let v: &'a Value = &*v;
        match v {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }
}

impl<'a> FilterArg<'a> for BTreeMap<String, Value> {
    type Output = BTreeMap<String, Value>;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::Map(m) => Ok(m.clone()),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        match v.take() {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }
}

pub struct MapRef;

impl<'a> FilterArg<'a> for MapRef {
    type Output = &'a BTreeMap<String, Value>;

    fn from_value(v: Value) -> Result<Self::Output> {
        match v {
            Value::Map(_) => Err(Error::Reference("map")),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        match v {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        let v: &'a Value = &*v;
        match v {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }
}

impl<'a> FilterArg<'a> for Value {
    type Output = Value;

    fn from_value(v: Value) -> Result<Self::Output> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        Ok(v.to_owned())
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        Ok(v.take())
    }
}

pub struct ValueRef;

impl<'a> FilterArg<'a> for ValueRef {
    type Output = &'a Value;

    fn from_value(_: Value) -> Result<Self::Output> {
        Err(Error::Reference("value"))
    }

    fn from_value_ref(v: &'a Value) -> Result<Self::Output> {
        Ok(v)
    }

    fn from_cow_mut(v: &'a mut ValueCow<'a>) -> Result<Self::Output> {
        Ok(&*v)
    }
}

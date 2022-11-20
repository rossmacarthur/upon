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
    /// Failed to convert from i64 to the integer type.
    TryFromInt(
        /// Type
        &'static str,
        /// Value
        i64,
    ),
}

impl FilterArg for () {
    type Output<'a> = ();

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::None => Ok(()),
            v => Err(Error::Type("()", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&*v)
    }
}

impl FilterArg for bool {
    type Output<'a> = bool;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::Bool(b) => Ok(*b),
            v => Err(Error::Type("bool", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&*v)
    }
}

macro_rules! impl_for_int {
    ($($ty:ty)+) => {
        $(
            impl FilterArg for $ty {
                type Output<'a> =$ty;

                fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
                    Self::from_value_ref(&v)
                }

                fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
                    match v {
                        Value::Integer(i) => (*i).try_into().map_err(|_| {
                            Error::TryFromInt(stringify!($ty), *i)
                        }),
                        v => Err(Error::Type(stringify!($ty), v.human())),
                    }
                }

                fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
                    Self::from_value_ref(&*v)
                }
            }
        )+
    };
}

impl_for_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 isize i128 }

macro_rules! impl_for_float {
    ($($ty:ty)+) => {
        $(
            impl FilterArg for $ty {
                type Output<'a> =$ty;

                fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
                    Self::from_value_ref(&v)
                }

                fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
                    match v {
                        Value::Float(f) => Ok(*f as $ty),
                        v => Err(Error::Type(stringify!($ty), v.human())),
                    }
                }

                fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
                    Self::from_value_ref(&*v)
                }
            }
        )+
    }
}

impl_for_float! { f32 f64 }

impl FilterArg for String {
    type Output<'a> = String;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::String(s) => Ok(s.to_owned()),
            v => Err(Error::Type("string", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        match v.take() {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("string", v.human())),
        }
    }
}

pub struct Str;

impl FilterArg for Str {
    type Output<'a> = &'a str;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        match v {
            Value::String(_) => Err(Error::Reference("string")),
            v => Err(Error::Type("&str", v.human())),
        }
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("&str", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        let v: &'a Value = &*v;
        match v {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("&str", v.human())),
        }
    }
}

impl FilterArg for Vec<Value> {
    type Output<'a> = Vec<Value>;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::List(l) => Ok(l.clone()),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        match v.take() {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }
}

pub struct ListRef;

impl FilterArg for ListRef {
    type Output<'a> = &'a [Value];

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        match v {
            Value::List(_) => Err(Error::Reference("list")),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        let v: &'a Value = &*v;
        match v {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }
}

impl FilterArg for BTreeMap<String, Value> {
    type Output<'a> = BTreeMap<String, Value>;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::Map(m) => Ok(m.clone()),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        match v.take() {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }
}

pub struct MapRef;

impl FilterArg for MapRef {
    type Output<'a> = &'a BTreeMap<String, Value>;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        match v {
            Value::Map(_) => Err(Error::Reference("map")),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        match v {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        let v: &'a Value = &*v;
        match v {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }
}

impl FilterArg for Value {
    type Output<'a> = Value;

    fn from_value<'a>(v: Value) -> Result<Self::Output<'a>> {
        Self::from_value_ref(&v)
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        Ok(v.to_owned())
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        Ok(v.take())
    }
}

pub struct ValueRef;

impl FilterArg for ValueRef {
    type Output<'a> = &'a Value;

    fn from_value<'a>(_: Value) -> Result<Self::Output<'a>> {
        Err(Error::Reference("value"))
    }

    fn from_value_ref(v: &Value) -> Result<Self::Output<'_>> {
        Ok(v)
    }

    fn from_cow_mut<'a>(v: &'a mut ValueCow<'a>) -> Result<Self::Output<'a>> {
        Ok(&*v)
    }
}

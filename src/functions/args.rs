use std::collections::BTreeMap;
use std::mem;

use crate::functions::FunctionArg;
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
    /// Failed to convert from i64 to the integer type.
    TryFromInt(
        /// Type
        &'static str,
        /// Value
        i64,
    ),
}

impl FunctionArg for () {
    type Output<'arg> = ();

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match &**v {
            Value::None => Ok(()),
            v => Err(Error::Type("()", v.human())),
        }
    }
}

impl FunctionArg for bool {
    type Output<'arg> = bool;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match &**v {
            &Value::Bool(b) => Ok(b),
            v => Err(Error::Type("bool", v.human())),
        }
    }
}

macro_rules! impl_for_int {
    ($($ty:ty)+) => {
        $(
            impl FunctionArg for $ty {
                type Output<'arg> = $ty;

                fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
                where
                    'stack: 'arg,
                {
                    match &**v {
                        &Value::Integer(i) => {
                            i.try_into().map_err(|_| Error::TryFromInt(stringify!($ty), i))
                        },
                        v => Err(Error::Type(stringify!($ty), v.human())),
                    }
                }
            }
        )+
    };
}

impl_for_int! { u8 u16 u32 u64 u128 usize i8 i16 i32 i64 isize i128 }

macro_rules! impl_for_float {
    ($($ty:ty)+) => {
        $(
            impl FunctionArg for $ty {
                type Output<'arg> = $ty;

                fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
                where
                    'stack: 'arg,
                {
                    match &**v {
                        &Value::Float(f) => Ok(f as $ty),
                        v => Err(Error::Type(stringify!($ty), v.human())),
                    }
                }
            }
        )+
    }
}

impl_for_float! { f32 f64 }

impl FunctionArg for String {
    type Output<'arg> = String;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match v {
            ValueCow::Borrowed(v) => match v {
                Value::String(s) => Ok(s.to_owned()),
                v => Err(Error::Type("string", v.human())),
            },
            ValueCow::Owned(v) => match mem::take(v) {
                Value::String(s) => Ok(s),
                _ => Err(Error::Type("string", v.human())),
            },
        }
    }
}

pub struct StringRef;

impl FunctionArg for StringRef {
    type Output<'arg> = &'arg str;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match &**v {
            Value::String(s) => Ok(s),
            v => Err(Error::Type("&str", v.human())),
        }
    }
}

impl FunctionArg for Vec<Value> {
    type Output<'arg> = Vec<Value>;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match v {
            ValueCow::Borrowed(v) => match v {
                Value::List(l) => Ok(l.to_owned()),
                v => Err(Error::Type("list", v.human())),
            },
            ValueCow::Owned(v) => match mem::take(v) {
                Value::List(l) => Ok(l),
                _ => Err(Error::Type("list", v.human())),
            },
        }
    }
}

pub struct ListRef;

impl FunctionArg for ListRef {
    type Output<'arg> = &'arg [Value];

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match &**v {
            Value::List(l) => Ok(l),
            v => Err(Error::Type("list", v.human())),
        }
    }
}

impl FunctionArg for BTreeMap<String, Value> {
    type Output<'arg> = BTreeMap<String, Value>;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match v {
            ValueCow::Borrowed(v) => match v {
                Value::Map(m) => Ok(m.to_owned()),
                v => Err(Error::Type("map", v.human())),
            },
            ValueCow::Owned(v) => match mem::take(v) {
                Value::Map(m) => Ok(m),
                _ => Err(Error::Type("map", v.human())),
            },
        }
    }
}

pub struct MapRef;

impl FunctionArg for MapRef {
    type Output<'arg> = &'arg BTreeMap<String, Value>;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match &**v {
            Value::Map(m) => Ok(m),
            v => Err(Error::Type("map", v.human())),
        }
    }
}

impl FunctionArg for Value {
    type Output<'arg> = Value;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        match v {
            ValueCow::Borrowed(v) => Ok(v.clone()),
            ValueCow::Owned(v) => Ok(mem::take(v)),
        }
    }
}

pub struct ValueRef;

impl FunctionArg for ValueRef {
    type Output<'arg> = &'arg Value;

    fn from_value<'stack, 'arg>(v: &'arg mut ValueCow<'stack>) -> Result<Self::Output<'arg>>
    where
        'stack: 'arg,
    {
        Ok(&**v)
    }
}

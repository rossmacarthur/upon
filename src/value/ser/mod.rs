mod list;
mod map;
mod variants;

use std::collections::BTreeMap;

use serde::ser::{Error as _, Serialize};

use crate::value::ser::list::SerializeList;
use crate::value::ser::map::SerializeMap;
use crate::value::ser::variants::{SerializeStructVariant, SerializeTupleVariant};
use crate::{Error, Result, Value};

/// Convert a `T` to a `Value`.
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::None => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Integer(i) => serializer.serialize_i64(*i),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(string) => serializer.serialize_str(string),
            Value::List(list) => list.serialize(serializer),
            Value::Map(map) => {
                use serde::ser::SerializeMap;
                let mut m = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    m.serialize_entry(k, v)?;
                }
                m.end()
            }
        }
    }
}

/// Serializer whose output is a `Value`.
///
/// This serializer serializes a `T: Serialize` to a `Value`.
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeList;
    type SerializeTuple = SerializeList;
    type SerializeTupleStruct = SerializeList;

    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;

    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(Value::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(Value::Integer(i64::try_from(v).map_err(|_| {
            Error::custom("out of range integral type conversion attempted")
        })?))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(Value::Float(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Ok(Value::String(String::from(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Value::String(String::from(v)))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(Value::List(
            v.iter()
                .copied()
                .map(i64::from)
                .map(Value::Integer)
                .collect(),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Value::None)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut map = BTreeMap::new();
        map.insert(String::from(variant), to_value(value)?);
        Ok(Value::Map(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeList::with_capacity(len.unwrap_or(0)))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: variant.to_owned(),
            list: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap::new())
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant.to_owned(),
            map: BTreeMap::new(),
        })
    }
}

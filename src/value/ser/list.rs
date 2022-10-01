use serde::ser::Serialize;

use crate::{to_value, Error, Result, Value};

#[derive(Default)]
#[cfg_attr(internal_debug, derive(Debug))]
pub struct SerializeList {
    list: Vec<Value>,
}

impl SerializeList {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            list: Vec::with_capacity(len),
        }
    }
}

impl serde::ser::SerializeSeq for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::List(self.list))
    }
}

impl serde::ser::SerializeTuple for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

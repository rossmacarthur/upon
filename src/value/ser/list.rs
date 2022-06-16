use serde::ser::Serialize;

use crate::value::ser::to_value;
use crate::value::{List, Value};
use crate::{Error, Result};

#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct SerializeList {
    list: List<Value>,
}

impl SerializeList {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            list: List::with_capacity(len),
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
        self.list.push(to_value(&value)?);
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

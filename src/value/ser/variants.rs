use serde::ser::Serialize;

use crate::value::ser::to_value;
use crate::value::{List, Map, Value};
use crate::{Error, Result};

pub struct SerializeTupleVariant {
    pub name: String,
    pub list: List<Value>,
}

pub struct SerializeStructVariant {
    pub name: String,
    pub map: Map<String, Value>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut map = Map::new();
        map.insert(self.name, Value::List(self.list));
        Ok(Value::Map(map))
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.map.insert(key.into(), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut map = Map::new();
        map.insert(self.name, Value::Map(self.map));
        Ok(Value::Map(map))
    }
}

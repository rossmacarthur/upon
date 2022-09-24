use std::collections::BTreeMap;

use serde::ser::Serialize;

use crate::{to_value, Error, Result, Value};

pub struct SerializeTupleVariant {
    pub name: String,
    pub list: Vec<Value>,
}

pub struct SerializeStructVariant {
    pub name: String,
    pub map: BTreeMap<String, Value>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut map = BTreeMap::new();
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
        let mut map = BTreeMap::new();
        map.insert(self.name, Value::Map(self.map));
        Ok(Value::Map(map))
    }
}

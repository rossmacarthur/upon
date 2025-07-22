#![allow(dead_code)]

use std::fmt::Write;

use upon::fmt;
use upon::Value;

pub fn debug(f: &mut fmt::Formatter<'_>, v: &Value) -> fmt::Result {
    match v {
        Value::None => write!(f, "None")?,
        Value::Bool(b) => write!(f, "{b:?}")?,
        Value::Integer(n) => write!(f, "{n:?}")?,
        Value::Float(n) => write!(f, "{n:?}")?,
        Value::String(s) => write!(f, "{s:?}")?,
        Value::List(list) => {
            f.write_char('[')?;
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    f.write_str(", ")?;
                }
                debug(f, item)?;
            }
            f.write_char(']')?;
        }
        Value::Map(map) => {
            f.write_char('{')?;
            for (i, (key, value)) in map.iter().enumerate() {
                if i != 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{key}: ")?;
                debug(f, value)?;
            }
            f.write_char('}')?;
        }
    }
    Ok(())
}

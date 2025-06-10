#![allow(dead_code)]

use std::fmt::Write;

use upon::fmt;
use upon::Value;

pub fn debug(f: &mut fmt::Formatter<'_>, v: &Value) -> fmt::Result {
    match v {
        Value::List(list) => {
            f.write_char('[')?;
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    f.write_str(", ")?;
                }
                debug(f, item)?;
            }
            f.write_char(']')?;
            Ok(())
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
            Ok(())
        }
        _ => fmt::default(f, v),
    }
}

pub use std::collections::HashMap as Map;
pub use std::vec::Vec as List;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    None,
    String(String),
    List(List<Value>),
    Map(Map<String, Value>),
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(String::from(s))
    }
}

impl From<List<Value>> for Value {
    fn from(list: List<Value>) -> Self {
        Self::List(list)
    }
}

impl<const N: usize> From<[Value; N]> for Value {
    fn from(list: [Value; N]) -> Self {
        Self::List(List::from(list))
    }
}

impl From<Map<String, Value>> for Value {
    fn from(map: Map<String, Value>) -> Self {
        Self::Map(map)
    }
}

impl<const N: usize> From<[(String, Value); N]> for Value {
    fn from(arr: [(String, Value); N]) -> Self {
        Self::Map(Map::from(arr))
    }
}

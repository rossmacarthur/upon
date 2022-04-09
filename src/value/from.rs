use std::borrow::Cow;

use super::*;

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

macro_rules! impl_from_int {
    ($($ty:ty)+) => {
        $(
            impl From<$ty> for Value {
                fn from(i: $ty) -> Self {
                    Self::Integer(i64::from(i))
                }
            }
        )+
    };
}

impl_from_int! { u8 u16 u32 i8 i16 i32 i64 }

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Self::Float(f64::from(f))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Float(f64::from(f))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(s: &'a str) -> Self {
        Self::String(String::from(s))
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    fn from(s: Cow<'a, str>) -> Self {
        Self::String(s.into_owned())
    }
}

impl<V> From<List<V>> for Value
where
    V: Into<Value>,
{
    fn from(list: List<V>) -> Self {
        Self::List(list.into_iter().map(Into::into).collect())
    }
}

impl<V, const N: usize> From<[V; N]> for Value
where
    V: Into<Value>,
{
    fn from(list: [V; N]) -> Self {
        Self::List(list.into_iter().map(Into::into).collect())
    }
}

impl<K, V> From<Map<K, V>> for Value
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from(map: Map<K, V>) -> Self {
        Self::Map(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for Value
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from(map: [(K, V); N]) -> Self {
        Self::Map(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

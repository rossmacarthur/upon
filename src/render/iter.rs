use std::collections::btree_map as map;
use std::slice;
use std::vec as list;

use crate::render::value::index;
use crate::types::ast;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

/// The state of a loop iteration.
#[cfg_attr(test, derive(Debug))]
pub enum LoopState<'source, 'render> {
    /// An iterator over a borrowed list and the last item yielded
    ListBorrowed {
        item: &'source ast::Ident<'source>,
        iter: slice::Iter<'render, Value>,
        value: Option<&'render Value>,
    },

    /// An iterator over an owned list and the last item yielded
    ListOwned {
        item: &'source ast::Ident<'source>,
        iter: list::IntoIter<Value>,
        value: Option<Value>,
    },

    /// An iterator over a borrowed map and the last key and value yielded
    MapBorrowed {
        kv: &'source ast::KeyValue<'source>,
        iter: map::Iter<'render, String, Value>,
        value: Option<(&'render String, &'render Value)>,
    },

    /// An iterator over an owned map and the last key and value yielded
    MapOwned {
        kv: &'source ast::KeyValue<'source>,
        iter: map::IntoIter<String, Value>,
        value: Option<(String, Value)>,
    },
}

impl<'source, 'render> LoopState<'source, 'render> {
    /// Constructs the initial loop state.
    pub fn new(
        source: &'source str,
        vars: &'source ast::LoopVars<'source>,
        iterable: ValueCow<'render>,
        span: Span,
    ) -> Result<Self> {
        let human = iterable.human();
        let err = || {
            Error::new(
                format!("expected iterable, but expression evaluated to {}", human),
                source,
                span,
            )
        };

        let unpack_list_item = |vars: &'source ast::LoopVars<'source>| match vars {
            ast::LoopVars::Item(item) => Ok(item),
            ast::LoopVars::KeyValue(kv) => Err(Error::new(
                "cannot unpack list item into two variables",
                source,
                kv.span,
            )),
        };

        let unpack_map_item = |vars: &'source ast::LoopVars<'source>| match vars {
            ast::LoopVars::Item(item) => Err(Error::new(
                "cannot unpack map item into one variable",
                source,
                item.span,
            )),
            ast::LoopVars::KeyValue(kv) => Ok(kv),
        };

        match iterable {
            ValueCow::Borrowed(v) => match v {
                Value::List(list) => {
                    let item = unpack_list_item(vars)?;
                    Ok(Self::ListBorrowed {
                        item,
                        iter: list.iter(),
                        value: None,
                    })
                }

                Value::Map(map) => {
                    let kv = unpack_map_item(vars)?;
                    Ok(Self::MapBorrowed {
                        kv,
                        iter: map.iter(),
                        value: None,
                    })
                }
                _ => Err(err()),
            },

            ValueCow::Owned(v) => match v {
                Value::List(list) => {
                    let item = unpack_list_item(vars)?;
                    Ok(Self::ListOwned {
                        item,
                        iter: list.into_iter(),
                        value: None,
                    })
                }

                Value::Map(map) => {
                    let kv = unpack_map_item(vars)?;
                    Ok(Self::MapOwned {
                        kv,
                        iter: map.into_iter(),
                        value: None,
                    })
                }
                _ => Err(err()),
            },
        }
    }

    pub fn iterate(&mut self) -> Option<()> {
        match self {
            Self::ListBorrowed { iter, value, .. } => {
                *value = Some(iter.next()?);
            }
            Self::ListOwned { iter, value, .. } => {
                *value = Some(iter.next()?);
            }
            Self::MapBorrowed { iter, value, .. } => {
                *value = Some(iter.next()?);
            }
            Self::MapOwned { iter, value, .. } => {
                *value = Some(iter.next()?);
            }
        }
        Some(())
    }

    pub fn resolve_path(
        &self,
        source: &str,
        path: &[ast::Ident<'source>],
    ) -> Result<Option<ValueCow<'render>>> {
        let name = path[0].raw;

        macro_rules! resolve {
            ($v:expr) => {{
                let mut v = $v;
                for p in &path[1..] {
                    v = index(source, v, p)?;
                }
                v
            }};
        }

        let err = |span| Error::new("cannot index into string", source, span);

        match self {
            Self::ListBorrowed {
                item,
                value: Some(value),
                ..
            } if item.raw == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::ListOwned {
                item,
                value: Some(value),
                ..
            } if item.raw == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            Self::MapBorrowed {
                kv,
                value: Some((string, _)),
                ..
            } if kv.key.raw == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String((*string).clone()))))
            }

            Self::MapOwned {
                kv,
                value: Some((string, _)),
                ..
            } if kv.key.raw == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String(string.clone()))))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, value)),
                ..
            } if kv.value.raw == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::MapOwned {
                kv,
                value: Some((_, value)),
                ..
            } if kv.value.raw == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            _ => Ok(None),
        }
    }
}

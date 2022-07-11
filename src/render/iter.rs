use std::collections::btree_map as map;
use std::iter::Enumerate;
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
        iter: Enumerate<slice::Iter<'render, Value>>,
        value: Option<(usize, &'render Value)>,
    },

    /// An iterator over an owned list and the last item yielded
    ListOwned {
        item: &'source ast::Ident<'source>,
        iter: Enumerate<list::IntoIter<Value>>,
        value: Option<(usize, Value)>,
    },

    /// An iterator over a borrowed map and the last key and value yielded
    MapBorrowed {
        kv: &'source ast::KeyValue<'source>,
        iter: Enumerate<map::Iter<'render, String, Value>>,
        value: Option<(usize, (&'render String, &'render Value))>,
    },

    /// An iterator over an owned map and the last key and value yielded
    MapOwned {
        kv: &'source ast::KeyValue<'source>,
        iter: Enumerate<map::IntoIter<String, Value>>,
        value: Option<(usize, (String, Value))>,
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
                        iter: list.iter().enumerate(),
                        value: None,
                    })
                }

                Value::Map(map) => {
                    let kv = unpack_map_item(vars)?;
                    Ok(Self::MapBorrowed {
                        kv,
                        iter: map.iter().enumerate(),
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
                        iter: list.into_iter().enumerate(),
                        value: None,
                    })
                }

                Value::Map(map) => {
                    let kv = unpack_map_item(vars)?;
                    Ok(Self::MapOwned {
                        kv,
                        iter: map.into_iter().enumerate(),
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

        if name == "loop" {
            return self.resolve_loop(source, path);
        }

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
                value: Some((_, value)),
                ..
            } if item.raw == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::ListOwned {
                item,
                value: Some((_, value)),
                ..
            } if item.raw == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (string, _))),
                ..
            } if kv.key.raw == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String((*string).clone()))))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (string, _))),
                ..
            } if kv.key.raw == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String(string.clone()))))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (_, value))),
                ..
            } if kv.value.raw == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (_, value))),
                ..
            } if kv.value.raw == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            _ => Ok(None),
        }
    }

    pub fn resolve_loop(
        &self,
        source: &str,
        path: &[ast::Ident<'source>],
    ) -> Result<Option<ValueCow<'render>>> {
        let (i, rem) = match self.current_index_and_rem() {
            Some((i, rem)) => (i, rem),
            None => return Ok(None),
        };

        if path.len() == 1 {
            return Ok(Some(ValueCow::Owned(crate::value! {
                index: i,
                first: i == 0,
                last: rem == 0,
            })));
        }

        let v = match path[1].raw {
            "index" => Value::Integer(i as i64),
            "first" => Value::Bool(i == 0),
            "last" => Value::Bool(rem == 0),
            _ => return Err(Error::new("not found in map", source, path[1].span)),
        };

        if !path[2..].is_empty() {
            return Err(Error::new(
                format!("cannot index into {}", v.human()),
                source,
                path[2].span,
            ));
        }

        Ok(Some(ValueCow::Owned(v)))
    }

    fn current_index_and_rem(&self) -> Option<(usize, usize)> {
        match self {
            LoopState::ListBorrowed {
                iter,
                value: Some((i, _)),
                ..
            } => Some((*i, iter.len())),
            LoopState::ListOwned {
                iter,
                value: Some((i, _)),
                ..
            } => Some((*i, iter.len())),
            LoopState::MapBorrowed {
                iter,
                value: Some((i, _)),
                ..
            } => Some((*i, iter.len())),
            LoopState::MapOwned {
                iter,
                value: Some((i, _)),
                ..
            } => Some((*i, iter.len())),
            _ => None,
        }
    }
}

use std::collections::btree_map as map;
use std::iter::Enumerate;
use std::slice;
use std::vec as list;

use crate::render::value::lookup;
use crate::types::ast;
use crate::types::span::index;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

/// The state of a loop iteration.
#[cfg_attr(test, derive(Debug))]
pub enum LoopState<'template, 'render> {
    /// An iterator over a borrowed list and the last item yielded
    ListBorrowed {
        item: &'template ast::Ident,
        iter: Enumerate<slice::Iter<'render, Value>>,
        value: Option<(usize, &'render Value)>,
    },

    /// An iterator over an owned list and the last item yielded
    ListOwned {
        item: &'template ast::Ident,
        iter: Enumerate<list::IntoIter<Value>>,
        value: Option<(usize, Value)>,
    },

    /// An iterator over a borrowed map and the last key and value yielded
    MapBorrowed {
        kv: &'template ast::KeyValue,
        iter: Enumerate<map::Iter<'render, String, Value>>,
        value: Option<(usize, (&'render String, &'render Value))>,
    },

    /// An iterator over an owned map and the last key and value yielded
    MapOwned {
        kv: &'template ast::KeyValue,
        iter: Enumerate<map::IntoIter<String, Value>>,
        value: Option<(usize, (String, Value))>,
    },
}

impl<'template, 'render> LoopState<'template, 'render> {
    /// Constructs the initial loop state.
    pub fn new(
        source: &str,
        vars: &'template ast::LoopVars,
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

        let unpack_list_item = |vars: &'template ast::LoopVars| match vars {
            ast::LoopVars::Item(item) => Ok(item),
            ast::LoopVars::KeyValue(kv) => Err(Error::new(
                "cannot unpack list item into two variables",
                source,
                kv.span,
            )),
        };

        let unpack_map_item = |vars: &'template ast::LoopVars| match vars {
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

    pub fn lookup_path(
        &self,
        source: &str,
        path: &[ast::Ident],
    ) -> Result<Option<ValueCow<'render>>> {
        let name = unsafe { index(source, path[0].span) };

        if name == "loop" {
            return self.lookup_loop(source, path);
        }

        macro_rules! resolve {
            ($v:expr) => {{
                let mut v = $v;
                for p in &path[1..] {
                    v = lookup(source, v, p)?;
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
            } if unsafe { index(source, item.span) } == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::ListOwned {
                item,
                value: Some((_, value)),
                ..
            } if unsafe { index(source, item.span) } == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (string, _))),
                ..
            } if unsafe { index(source, kv.key.span) } == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String((*string).clone()))))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (string, _))),
                ..
            } if unsafe { index(source, kv.key.span) } == name => {
                if let [p, ..] = &path[1..] {
                    return Err(err(p.span));
                }
                Ok(Some(ValueCow::Owned(Value::String(string.clone()))))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (_, value))),
                ..
            } if unsafe { index(source, kv.value.span) } == name => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (_, value))),
                ..
            } if unsafe { index(source, kv.value.span) } == name => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            _ => Ok(None),
        }
    }

    pub fn lookup_loop(
        &self,
        source: &str,
        path: &[ast::Ident],
    ) -> Result<Option<ValueCow<'render>>> {
        let (i, rem) = match self.current_index_and_rem() {
            Some((i, rem)) => (i, rem),
            None => return Ok(None),
        };

        if path.len() == 1 {
            return Ok(Some(ValueCow::Owned(Value::from([
                ("index", Value::Integer(i as i64)),
                ("first", Value::Bool(i == 0)),
                ("last", Value::Bool(rem == 0)),
            ]))));
        }

        let v = match unsafe { index(source, path[1].span) } {
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

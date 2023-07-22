use std::collections::btree_map as map;
use std::iter::Enumerate;
use std::slice;
use std::vec as list;

use crate::render::value::lookup;
use crate::types::ast;
use crate::types::span::Span;
use crate::value::ValueCow;
use crate::{Error, Result, Value};

/// The state of a loop iteration.
#[cfg_attr(internal_debug, derive(Debug))]
pub enum LoopState<'a> {
    /// An iterator over a borrowed list and the last item yielded
    ListBorrowed {
        item: &'a ast::Ident,
        iter: Enumerate<slice::Iter<'a, Value>>,
        value: Option<(usize, &'a Value)>,
    },

    /// An iterator over an owned list and the last item yielded
    ListOwned {
        item: &'a ast::Ident,
        iter: Enumerate<list::IntoIter<Value>>,
        value: Option<(usize, Value)>,
    },

    /// An iterator over a borrowed map and the last key and value yielded
    MapBorrowed {
        kv: &'a ast::KeyValue,
        iter: Enumerate<map::Iter<'a, String, Value>>,
        value: Option<(usize, (&'a String, &'a Value))>,
    },

    /// An iterator over an owned map and the last key and value yielded
    MapOwned {
        kv: &'a ast::KeyValue,
        iter: Enumerate<map::IntoIter<String, Value>>,
        value: Option<(usize, (String, Value))>,
    },
}

impl<'a> LoopState<'a> {
    /// Constructs the initial loop state.
    pub fn new(
        source: &str,
        vars: &'a ast::LoopVars,
        iterable: ValueCow<'a>,
        span: Span,
    ) -> Result<Self> {
        let human = iterable.human();
        let err = || {
            Error::render(
                format!("expected iterable, but expression evaluated to {human}"),
                source,
                span,
            )
        };

        let unpack_list_item = |vars: &'a ast::LoopVars| match vars {
            ast::LoopVars::Item(item) => Ok(item),
            ast::LoopVars::KeyValue(kv) => Err(Error::render(
                "cannot unpack list item into two variables",
                source,
                kv.span,
            )),
        };

        let unpack_map_item = |vars: &'a ast::LoopVars| match vars {
            ast::LoopVars::Item(item) => Err(Error::render(
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

    pub fn lookup_var(&self, source: &str, var: &ast::Var) -> Result<Option<ValueCow<'a>>> {
        let name = match var.first().access {
            ast::Access::Index(_) => return Ok(None),
            ast::Access::Key(ast::Ident { span }) => &source[span],
        };

        if name == "loop" {
            return self.lookup_loop(source, &var.path);
        }

        macro_rules! resolve {
            ($v:expr) => {{
                let mut v = $v;
                for m in var.rest() {
                    v = match lookup(source, v, m)? {
                        Some(v) => v,
                        None => return Ok(None),
                    };
                }
                v
            }};
        }

        let err = |span| Error::render("cannot index into string", source, span);

        match self {
            Self::ListBorrowed {
                item,
                value: Some((_, value)),
                ..
            } if name == &source[item.span] => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::ListOwned {
                item,
                value: Some((_, value)),
                ..
            } if name == &source[item.span] => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (string, _))),
                ..
            } if name == &source[kv.key.span] => {
                if let [m, ..] = var.rest() {
                    return Err(err(m.span));
                }
                Ok(Some(ValueCow::Owned(Value::String((*string).clone()))))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (string, _))),
                ..
            } if name == &source[kv.key.span] => {
                if let [m, ..] = var.rest() {
                    return Err(err(m.span));
                }
                Ok(Some(ValueCow::Owned(Value::String(string.clone()))))
            }

            Self::MapBorrowed {
                kv,
                value: Some((_, (_, value))),
                ..
            } if name == &source[kv.value.span] => {
                let v = resolve!(*value);
                Ok(Some(ValueCow::Borrowed(v)))
            }

            Self::MapOwned {
                kv,
                value: Some((_, (_, value))),
                ..
            } if name == &source[kv.value.span] => {
                let v = resolve!(value);
                Ok(Some(ValueCow::Owned(v.clone())))
            }

            _ => Ok(None),
        }
    }

    pub fn lookup_loop(&self, source: &str, path: &[ast::Member]) -> Result<Option<ValueCow<'a>>> {
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

        let member = &path[1];

        let name = match member.access {
            ast::Access::Index(_) => {
                return Err(Error::render(
                    "cannot index into map with integer",
                    source,
                    member.span,
                ))
            }
            ast::Access::Key(ast::Ident { span }) => &source[span],
        };

        let v = match (&member.op, name) {
            (_, "index") => Value::Integer(i as i64),
            (_, "first") => Value::Bool(i == 0),
            (_, "last") => Value::Bool(rem == 0),
            (ast::AccessOp::Optional, _) => Value::None,
            (ast::AccessOp::Direct, _) => {
                return Err(Error::render("not found in map", source, member.span))
            }
        };

        if !path[2..].is_empty() {
            return Err(Error::render(
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

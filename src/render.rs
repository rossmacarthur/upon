use std::fmt::Write;

use crate::instr::{Instr, Template};
use crate::span::Span;
use crate::{ast, Engine, Error, Result, Value};

/// A renderer that can render a compiled template as a string.
pub struct Renderer<'engine, 'source> {
    engine: &'engine Engine<'engine>,
    template: &'source Template<'source>,
}

enum LoopState<'source> {
    List {
        item: &'source ast::Ident<'source>,
        iter: crate::value::ListIntoIter,
    },
    Map {
        kv: &'source ast::KeyValue<'source>,
        iter: crate::value::MapIntoIter,
    },
}

impl<'engine, 'source> Renderer<'engine, 'source> {
    pub fn new(engine: &'engine Engine<'engine>, template: &'source Template<'source>) -> Self {
        Self { engine, template }
    }

    pub fn render(&self, globals: Value) -> Result<String> {
        let mut buf = String::with_capacity(self.source().len());

        let mut pc = 0;

        let mut scopes = vec![globals];
        let mut loops = vec![];
        let mut values = vec![];

        while let Some(instr) = self.template.instrs.get(pc) {
            match instr {
                Instr::EmitRaw(raw) => {
                    buf.push_str(raw);
                }

                Instr::StartLoop(j, vars, span) => {
                    let iterable = values.pop().unwrap();
                    let mut loop_state = self.loop_state(vars, iterable, *span)?;
                    match self.iterate(&mut loop_state) {
                        Some(scope) => {
                            loops.push(loop_state);
                            scopes.push(scope);
                        }
                        None => {
                            // nothing to loop, jump to end
                            pc = *j;
                            continue;
                        }
                    }
                }

                Instr::Iterate(j) => {
                    match self.iterate(loops.last_mut().unwrap()) {
                        Some(scope) => {
                            *scopes.last_mut().unwrap() = scope;
                            pc = *j;
                            continue;
                        }
                        None => {
                            scopes.pop().unwrap();
                        }
                    };
                }

                Instr::JumpIfTrue(j, span) => {
                    if self.if_cond(values.last().unwrap(), *span)? {
                        pc = *j;
                        continue;
                    }
                }

                Instr::JumpIfFalse(j, span) => {
                    if !self.if_cond(values.last().unwrap(), *span)? {
                        pc = *j;
                        continue;
                    }
                }

                Instr::Push(path) => {
                    let value = self.resolve_path(&scopes, path)?;
                    values.push(value);
                }

                Instr::PopEmit(span) => {
                    let value = values.pop().unwrap();
                    self.render_value(&mut buf, &value, *span)?;
                }

                Instr::Call(name) => {
                    let func = self.engine.filters.get(name.raw).ok_or_else(|| {
                        Error::new("unknown filter function", self.source(), name.span)
                    })?;
                    let value = values.pop().unwrap();
                    values.push(func(value));
                }
            }
            pc += 1;
        }

        assert!(pc == self.template.instrs.len());

        Ok(buf)
    }

    fn render_value(&self, buf: &mut String, value: &Value, span: Span) -> Result<()> {
        match value {
            Value::None => {}
            Value::Bool(b) => write!(buf, "{}", b).unwrap(),
            Value::Integer(n) => write!(buf, "{}", n).unwrap(),
            Value::Float(n) => write!(buf, "{}", n).unwrap(),
            Value::String(s) => write!(buf, "{}", s).unwrap(),
            val => {
                return Err(Error::new(
                    format!(
                        "expected renderable value, but expression evaluated to {}",
                        val.human()
                    ),
                    self.source(),
                    span,
                ));
            }
        }
        Ok(())
    }

    fn if_cond(&self, value: &Value, span: Span) -> Result<bool> {
        match value {
            Value::Bool(cond) => Ok(*cond),
            value => {
                return Err(Error::new(
                    format!(
                        "expected bool, but expression evaluated to {}",
                        value.human()
                    ),
                    self.source(),
                    span,
                ));
            }
        }
    }

    fn loop_state(
        &self,
        vars: &'source ast::LoopVars,
        value: Value,
        span: Span,
    ) -> Result<LoopState<'source>> {
        match value {
            Value::List(list) => {
                let item = match vars {
                    ast::LoopVars::Item(item) => item,
                    ast::LoopVars::KeyValue(kv) => {
                        return Err(Error::new(
                            "cannot unpack list item into two variables",
                            self.source(),
                            kv.span,
                        ));
                    }
                };
                Ok(LoopState::List {
                    item,
                    iter: list.into_iter(),
                })
            }

            Value::Map(map) => {
                let kv = match vars {
                    ast::LoopVars::Item(item) => {
                        return Err(Error::new(
                            "cannot unpack map item into one variable",
                            self.source(),
                            item.span,
                        ));
                    }
                    ast::LoopVars::KeyValue(kv) => kv,
                };
                Ok(LoopState::Map {
                    kv,
                    iter: map.into_iter(),
                })
            }

            value => {
                return Err(Error::new(
                    format!(
                        "expected iterable, but expression evaluated to {}",
                        value.human()
                    ),
                    self.source(),
                    span,
                ));
            }
        }
    }

    fn iterate(&self, loop_state: &mut LoopState<'source>) -> Option<Value> {
        match loop_state {
            LoopState::List { item, iter } => {
                let v = iter.next()?;
                Some(Value::from([(item.raw, v)]))
            }
            LoopState::Map {
                kv: ast::KeyValue { key, value, .. },
                iter,
            } => {
                let (k, v) = iter.next()?;
                Some(Value::from([(key.raw, Value::from(k)), (value.raw, v)]))
            }
        }
    }

    /// Resolves a path to a value in the given stack.
    fn resolve_path(&self, scopes: &[Value], path: &ast::Path<'source>) -> Result<Value> {
        'outer: for value in scopes.iter().rev() {
            let mut result = value;
            for (i, segment) in path.iter().enumerate() {
                result = match self.lookup_value(result, segment) {
                    Ok(d) => d,
                    Err(err) => {
                        // If it is the first segment of the path then we can
                        // try another locals.
                        if i == 0 {
                            continue 'outer;
                        }
                        return Err(err);
                    }
                };
            }
            return Ok(result.clone());
        }
        Err(Error::new(
            "not found in this scope",
            self.source(),
            path[0].span,
        ))
    }

    // Lookup an index in a value.
    fn lookup_value<'v>(&self, value: &'v Value, idx: &ast::Ident<'_>) -> Result<&'v Value> {
        let ast::Ident { raw, span } = idx;
        match value {
            Value::List(list) => match raw.parse::<usize>() {
                Ok(i) => Ok(&list[i]),
                Err(_) => Err(Error::new(
                    "cannot index list with string",
                    self.source(),
                    *span,
                )),
            },
            Value::Map(map) => match map.get(*raw) {
                Some(value) => Ok(value),
                None => Err(Error::new("not found in map", self.source(), *span)),
            },
            val => Err(Error::new(
                format!("cannot index into {}", val.human()),
                self.source(),
                *span,
            )),
        }
    }

    fn source(&self) -> &'source str {
        self.template.source
    }
}

impl Value {
    fn human(&self) -> &'static str {
        match self {
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
    }
}
